use std::{collections::HashSet, path::Path, sync::{Arc, Mutex}, time::{Duration, Instant}};

use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::{spawn, JoinHandle}, Window};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{config::AppConfig, Error, OperatingSystem, View};

// Store operating system name
#[cfg(target_os = "windows")]
const OPERATING_SYSTEM: OperatingSystem = OperatingSystem::WINDOWS;
#[cfg(target_os = "macos")]
const OPERATING_SYSTEM: OperatingSystem = OperatingSystem::MACOS;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const OPERATING_SYSTEM: OperatingSystem = OperatingSystem::LINUX;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct AppState {
    pub fatal_error: Option<String>,
    pub view: View,
    pub operating_system: OperatingSystem,
    pub installation_target: OperatingSystem,
    pub installation_path: String,
    pub installation_channel: String,
    pub components: ComponentList,
    pub config: AppConfig,
    #[serde(skip)]
    pub task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        return AppState {
            fatal_error: None,
            view: View::SETUP,
            operating_system: OPERATING_SYSTEM,
            installation_target: OPERATING_SYSTEM,
            installation_path: "./Flashpoint".to_owned(),
            installation_channel: "Stable".to_owned(),
            components: ComponentList::default(),
            config: AppConfig::default(),
            task_handle: Arc::new(Mutex::new(None)),
        };
    }
}

fn remove_readonly_attr(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        let metadata = fs::metadata(path)?;
        if metadata.file_attributes() & 0x1 != 0 { // FILE_ATTRIBUTE_READONLY
            let mut options = OpenOptions::new();
            options.write(true).custom_flags(0x80000000); // FILE_ATTRIBUTE_NORMAL
            let _file = options.open(path)?;
            // Opening the file with FILE_ATTRIBUTE_NORMAL is enough to clear the readonly attribute.
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(path, permissions).unwrap();
    }
    Ok(())
}

pub async fn install_component(comp: &Component, base_url: &str, base_dir: &str, window: &Window, state: &mut DownloadState) -> Result<(), Box<dyn std::error::Error>> {
    let mut temp_str = base_dir.to_owned() + "/Temp/";
    let temp_str_cpy = temp_str.clone();
    let temp_dir_path = Path::new(&temp_str_cpy);
    if let Some(path) = &comp.path {
        temp_str.push_str(path);
    }
    let base_dir_temp = Path::new(&temp_str);
        
    // Ensure base_dir temp exists
    std::fs::create_dir_all(&base_dir_temp)?;

    // Download each component zip to a file, then extract
    let url = base_url.to_owned() + &comp.id + ".zip";
    let file = download_file_tmp(&url, &comp.hash.to_uppercase(), window, state).await?;

    // Extract to temp folder
    let mut archive = ZipArchive::new(file)?;

    state.stage = "Extracting".to_owned();
    window.emit("download_state", state.clone()).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => base_dir_temp.join(path),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            // Create a directory if the file is a directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // Ensure the file's parent directory exists
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }

            // Write the file content
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // Move files from Temp to main dir

    for entry in WalkDir::new(temp_dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(temp_dir_path)?;
        let dest_path = Path::new(base_dir).join(relative_path);

        if path.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else if path.is_file() {
            let _ = remove_readonly_attr(&dest_path); // If it fails, the copy error will present itself soon and bubble up later
            std::fs::copy(path, &dest_path)?;
        }
    }

    std::fs::remove_dir_all(temp_dir_path)?;
    std::fs::create_dir_all(temp_dir_path)?;
    
    Ok(())
}

pub async fn download_file_tmp(url: &str, crc32_hash: &str, window: &Window, state: &mut DownloadState) -> Result<std::fs::File, Box<dyn std::error::Error>> {
    let mut tmp_file = tokio::fs::File::from(tempfile::tempfile()?);
    let mut byte_stream = reqwest::get(url).await?.bytes_stream();
    let mut hasher = Hasher::new();
    let mut last_call = Instant::now();

    while let Some(item) = byte_stream.next().await {
        let chunk = item?;
        state.total_downloaded += chunk.len() as u64;
        if last_call.elapsed() >= Duration::from_millis(200) {
            window.emit("download_state", state.clone()).unwrap();
            last_call = Instant::now();
        }
        
        hasher.update(&chunk);
        tmp_file.write_all(&chunk).await?;
    }
    tmp_file.flush().await?;

    if crc32_hash != "00000000" {
        let calculated_hash = hasher.finalize();
        let calculated_hash_str = format!("{:08x}", calculated_hash);
        if calculated_hash_str.to_uppercase() != crc32_hash {
            let msg = format!("Download failed, hash mismatch: Got {:?} expected {:?} - URL: {:?}", calculated_hash_str.to_uppercase(), crc32_hash, url);
            return Err(Box::new(crate::Error::GeneralError(msg)));
        }
    }

    let t = tmp_file.try_into_std();
    Ok(t.map_err(|_| crate::Error::GeneralError("Unknown".to_owned()))?)
}

impl AppState {
    pub fn adjust_installation_target(&mut self) {
        match self.installation_target {
            OperatingSystem::LINUX => {
                if let Some(linux) = self.config.linux.as_ref() {
                    self.installation_path = linux.default_path.clone();
                } else {
                    self.installation_target = OperatingSystem::WINDOWS;
                }
            }
            OperatingSystem::MACOS => {
                if let Some(macos) = self.config.macos.as_ref() {
                    self.installation_path = macos.default_path.clone();
                } else {
                    self.installation_target = OperatingSystem::WINDOWS;
                }
            }
            _ => {}
        }
    }

    pub async fn start_downloader(&mut self, window: tauri::Window) {
        let components_ref = get_all_components(&self.components);
        let components: Vec<Component> = components_ref.iter().map(|c| (*c).clone()).collect();
        let base_url = self.components.url.clone();
        let base_dir = self.installation_path.clone();

        let mut handle = self.task_handle.lock().unwrap(); // Lock the handle in state

        *handle = Some(spawn(async move {
            let mut download_state = DownloadState::default();

            download_state.total_components = components.len();
            download_state.component_number = 0;
            download_state.total_size = components.iter().map(|c| c.download_size).sum();

            for comp in components {
                download_state.component_number += 1;
                download_state.current = Some(comp.clone());
                download_state.stage = "Downloading".to_owned();
                window.emit("download_state", download_state.clone()).unwrap();
                match install_component(&comp, &base_url, &base_dir, &window, &mut download_state).await {
                    Ok(_) => (),
                    Err(e) => {
                        window.emit("fatal_error", format!("During Install of {:?} - {:?}", comp.id, e.to_string())).unwrap();
                        return;
                    },
                }
            }
            window.emit("installation_finished", 0).unwrap();
        }));

        drop(handle); // Drop the lock
    }

    pub async fn change_view(&mut self, view: View, window: tauri::Window) -> Result<(), Error> {
        match self.view {
            View::SETUP => {
                match view {
                    View::SETUPSELECT => {
                        // Validate path
                        println!("Checking {:?}", &self.installation_path);
                        let is_empty = installation_path_is_safe(&self.installation_path)?;
                        if !is_empty {
                            // Try appending Flashpoint as a subdirectory
                            let new_path =
                                Path::join(Path::new(&self.installation_path), "Flashpoint");
                            let is_new_empty =
                                installation_path_is_safe(&new_path.to_string_lossy())?;
                            if !is_new_empty {
                                return Err(Error::GeneralError("Installation path already contains files or a Flashpoint directory".to_owned()));
                            } else {
                                // Save valid modified path to state
                                self.installation_path = new_path.to_string_lossy().to_string();
                            }
                        }

                        // Find the correct source url
                        let os_config_opt = match self.installation_target {
                            OperatingSystem::LINUX => self.config.linux.clone(),
                            OperatingSystem::WINDOWS => self.config.windows.clone(),
                            OperatingSystem::MACOS => self.config.macos.clone(),
                        };

                        if os_config_opt.is_none() {
                            return Err(Error::GeneralError(
                                "Selected platform does not have an installation candidate".to_owned(),
                            ));
                        }
                        let os_config = os_config_opt.unwrap();

                        let xml_url = os_config.channels.get(&self.installation_channel)
                            .cloned()
                            .unwrap_or_default();

                        // If no source url found, channel does not exist
                        if xml_url.is_empty() {
                            return Err(Error::GeneralError(
                                "Selected channel does not exist".to_owned(),
                            ));
                        } else {
                            let data = download_text_file(&xml_url)
                                .await
                                .map_err(|e| Error::GeneralError(e.to_string()))?;
                            let mut comp: ComponentList = serde_xml_rs::from_str(&data)
                                .map_err(|e| Error::GeneralError(e.to_string()))?;
                            // Calculate required values and mark as selected
                            comp.setup();
                            self.components = comp;
                        }
                    }
                    _ => {
                        return Err(crate::Error::GeneralError("Invalid view transition".to_owned()));
                    },
                }
            },
            View::SETUPSELECT => {
                match view {
                    View::SETUP => (),
                    View::INSTALLATION => {
                        self.start_downloader(window).await;
                    },
                    _ => {
                        return Err(crate::Error::GeneralError("Invalid view transition".to_owned()));
                    }
                }
            },
            View::INSTALLATION => {
                match view {
                    View::FINISHED => (),
                    _ => {
                        return Err(crate::Error::GeneralError("Invalid view transition".to_owned()));
                    }
                }
            }
            _ => (),
        }

        // No error returned, so assume it's safe to change view
        self.view = view;
        Ok(())
    }
}

async fn download_text_file(url: &str) -> Result<String, reqwest::Error> {
    let resp = reqwest::get(url).await?;
    Ok(resp.text().await?)
}

fn installation_path_is_safe(dir: &str) -> std::io::Result<bool> {
    match std::fs::read_dir(dir) {
        Ok(mut entries) => Ok(!entries.next().is_some()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                // Bad installation path, alert and return early
                Ok(true)
            } else {
                Err(e)
            }
        }
    }
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Ok(false),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentList {
    #[serde(rename = "url", default)]
    url: String,
    #[serde(rename = "categories", alias = "category", default)]
    categories: Vec<Category>,
    #[serde(default)]
    pub selected: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
}

impl Default for ComponentList {
    fn default() -> Self {
        ComponentList {
            url: "Example Component List".to_owned(),
            categories: vec![],
            selected: vec![],
            required: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Category {
    id: String,
    #[serde(alias = "title")]
    name: String,
    description: String,
    // This field can either be a nested category or a component. Depending on your XML structure and needs, you might need to adjust the handling.
    #[serde(alias = "category", default)]
    subcategories: Vec<Category>,
    #[serde(alias = "component")]
    components: Vec<Component>,
    #[serde(default)]
    required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
    id: String,
    #[serde(alias = "title")]
    name: String,
    description: String,
    #[serde(alias = "date-modified")]
    date_modified: String,
    #[serde(alias = "download-size")]
    download_size: u64,
    #[serde(alias = "install-size")]
    install_size: u64,
    path: Option<String>,
    hash: String,
    depends: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool")]
    required: bool,
    #[serde(default)]
    installed: bool,
}

fn update_ids_in_category(
    category: &mut Category,
    working_id: &str,
) {
    // Update own ID
    let mut new_working_id = working_id.to_owned() + "-" + &category.id;
    if new_working_id.starts_with("-") {
        new_working_id.remove(0);
    }
    category.id = new_working_id.clone();

    for subcat in &mut category.subcategories {
        update_ids_in_category(subcat, &new_working_id);
    }

    // Update all components
    for comp in &mut category.components {
        comp.id = new_working_id.clone() + "-" + &comp.id;
    }
}

impl ComponentList {
    pub fn setup(&mut self) {
        let mut required = vec![];
        for category in &mut self.categories {
            // Update all IDs to be correct inside category
            update_ids_in_category(category, ""); 
        }
        for category in self.categories.iter() {
            // Add to the list of required components and categories
            self.find_required(category, &mut required);
        }
        required.sort();
        required.dedup();
        self.required = required.clone();
    }

    fn find_required(
        &self,
        category: &Category,
        list: &mut Vec<String>
    ) -> bool {
        let mut is_required = true;
        // Find non-required comps
        for component in category.components.iter() {
            if component.required {
                // Add all dependants
                let mut dependencies = self.find_dependencies(&component.id);
                list.append(&mut dependencies);
            } else {
                is_required = false;
            }
        }
        for subcat in category.subcategories.iter() {
            if subcat.required {
                // If required itself, add all child components
                let mut subcat_comps = vec![];
                collect_category_components(subcat, &mut subcat_comps);
                for comp in subcat_comps {
                    list.push(comp.id.clone());
                }
                list.push(subcat.id.clone());
            } else {
                // If the category isn't stated required, then we'll decide by checking if all children are required instead
                let required = self.find_required(subcat, list);
                if !required {
                    is_required = false;
                }
            }
     
        }
        // Finally can add to list
        if is_required {
            list.push(category.id.clone());
        }
        is_required
    }    

    // pub fn mark_required(&mut self, id: &str) {
    //     let mut required = vec![];
    //     let components = get_all_components(&self);

    //     if components.iter().find(|&c| c.id == id).is_some() {
    //         // Is a component
    //         self.find_dependants_recursive(id, &components, &mut required);
    //         required.push(id.to_owned());
    //     } else if let Some(category) = self.find_category_by_id(id) {
    //         // Is a category
    //         let mut category_components = vec![];
    //         collect_category_components(category, &mut category_components);
    //         for comp in category_components.iter() {
    //             self.find_dependants_recursive(&comp.id, &components, &mut required);
    //             required.push(comp.id.clone());
    //         }
    //     }

    //     self.required.append(&mut required);
    //     self.required.sort();
    //     self.required.dedup();
    // }

    pub fn select(&mut self, id: &str) {
        let mut dependencies = self.find_dependencies(id);

        self.selected.append(&mut dependencies);
        self.selected.sort();
        self.selected.dedup();
    }

    pub fn unselect(&mut self, id: &str) {
        let dependants = self.find_dependants(id);

        // Should be faster?
        let dependants_set: HashSet<String> = dependants.into_iter().collect();

        self.selected.retain(|e| !dependants_set.contains(e)); // Do not remove required component

    }

    pub fn find_dependants(&self, id: &str) -> Vec<String> {
        let mut dependants: Vec<String> = Vec::new();
        let components = get_all_components(&self);

        if components.iter().find(|&c| c.id == id).is_some() {
            // Is a component
            self.find_dependants_recursive(id, &components, &mut dependants);
            dependants.push(id.to_owned());
        } else if let Some(category) = self.find_category_by_id(id) {
            // Is a category
            let mut category_components = vec![];
            collect_category_components(category, &mut category_components);
            for comp in category_components.iter() {
                self.find_dependants_recursive(&comp.id, &components, &mut dependants);
                dependants.push(comp.id.clone());
            }
        }

        // Since recursion can add duplicates, ensure unique elements
        dependants.retain(|e| !self.required.contains(e)); // Remove all required components from dependants list
        dependants.sort();
        dependants.dedup();

        dependants
    }

    fn find_dependants_recursive<'a>(
        &self,
        id: &str,
        components: &Vec<&'a Component>,
        dependants: &mut Vec<String>,
    ) {
        for component in components.iter() {
            if let Some(depends) = &component.depends {
                let dependencies: Vec<&str> = depends.split_whitespace().collect();

                if dependencies.contains(&id) {
                    // If not already included, add to dependants and search for its dependants
                    if !dependants.contains(&component.id) {
                        dependants.push(component.id.clone());
                        self.find_dependants_recursive(&component.id, components, dependants);
                    }
                }
            }
        }
    }

    pub fn find_dependencies(&self, id: &str) -> Vec<String> {
        let mut dependencies: Vec<String> = Vec::new();
        let components = get_all_components(&self);

        if let Some(component) = components.iter().find(|&c| c.id == id) {
            // Is a component
            self.find_dependencies_recursive(component, &components, &mut dependencies);
            // Always select self as well as dependencies
            dependencies.push(id.to_owned());
        } else if let Some(category) = self.find_category_by_id(id) {
            // Is a category
            let mut category_components = vec![];
            collect_category_components(category, &mut category_components);
            for comp in category_components.iter() {
                self.find_dependencies_recursive(comp, &components, &mut dependencies);
                dependencies.push(comp.id.clone());
            }
        }
        // Since recursion might add duplicates, ensure unique elements
        dependencies.sort();
        dependencies.dedup();

        dependencies
    }

    fn find_dependencies_recursive<'a>(
        &self,
        component: &'a Component,
        components: &Vec<&'a Component>,
        dependencies: &mut Vec<String>,
    ) {
        if let Some(depends) = &component.depends {
            let direct_dependencies: Vec<&str> = depends.split_whitespace().collect();

            for dep_id in direct_dependencies.iter() {
                // Avoid adding duplicate dependencies
                if !dependencies.contains(&dep_id.to_string()) {
                    dependencies.push(dep_id.to_string());

                    // Find the component that matches this dependency ID and recursively find its dependencies
                    if let Some(dep_component) = components.iter().find(|&c| &c.id == *dep_id) {
                        self.find_dependencies_recursive(dep_component, components, dependencies);
                    }
                }
            }
        }
    }

    // Function to find a category by ID and return a reference to it
    fn find_category_by_id(&self, id: &str) -> Option<&Category> {
        self.find_category_recursive(&self.categories, id)
    }

    fn find_category_recursive<'a>(&'a self, categories: &'a [Category], id: &str) -> Option<&'a Category> {
        for category in categories {
            if category.id == id {
                return Some(category);
            }
            if let Some(subcategory) = self.find_category_recursive(&category.subcategories, id) {
                return Some(subcategory);
            }
        }
        None
    }
}

fn collect_category_components<'a>(category: &'a Category, components: &mut Vec<&'a Component>) {
    for subcat in category.subcategories.iter() {
        collect_category_components(subcat, components);
    }
    for component in &category.components {
        components.push(component);
    }
}

fn collect_components<'a>(categories: &'a [Category], components: &mut Vec<&'a Component>) {
    for category in categories {
        // Add all components in the current category to the components vector
        collect_category_components(&category, components);
    }
}

// Utility function to initiate the collection process and return the result
fn get_all_components<'a>(list: &'a ComponentList) -> Vec<&'a Component> {
    let mut components = Vec::new();
    collect_components(&list.categories, &mut components);
    components
}


#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadState {
    pub total_size: u64,
    pub total_downloaded: u64,
    pub total_components: usize,
    pub component_number: i32,
    pub current: Option<Component>,
    pub stage: String,
}

impl Default for DownloadState {
    fn default() -> Self {
        DownloadState {
            total_size: 0,
            total_downloaded: 0,
            total_components: 0,
            component_number: 0,
            current: None,
            stage: String::new(),
        }
    }
}