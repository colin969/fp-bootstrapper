use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};

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
        };
    }
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

    pub async fn change_view(&mut self, view: View) -> Result<(), Error> {
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
                        let xml_url = match self.installation_target {
                            OperatingSystem::LINUX => self
                                .config
                                .linux
                                .as_ref()
                                .and_then(|linux| linux.channels.get(&self.installation_channel))
                                .cloned()
                                .unwrap_or_default(),

                            OperatingSystem::WINDOWS => self
                                .config
                                .windows
                                .as_ref()
                                .and_then(|windows| {
                                    windows.channels.get(&self.installation_channel)
                                })
                                .cloned()
                                .unwrap_or_default(),

                            OperatingSystem::MACOS => self
                                .config
                                .macos
                                .as_ref()
                                .and_then(|macos| macos.channels.get(&self.installation_channel))
                                .cloned()
                                .unwrap_or_default(),
                        };

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
                            comp.fix_ids();
                            comp.mark_required();
                            self.components = comp;
                        }
                    }
                    _ => (),
                }
            }
            View::SETUPSELECT => todo!(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentList {
    #[serde(rename = "url", default)]
    url: String,
    #[serde(rename = "categories", alias = "category", default)]
    categories: Vec<Category>,
    #[serde(default)]
    selected: Vec<String>,
}

impl Default for ComponentList {
    fn default() -> Self {
        ComponentList {
            url: "Example Component List".to_owned(),
            categories: vec![],
            selected: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Category {
    id: String,
    #[serde(default)]
    real_id: String,
    title: String,
    description: String,
    // This field can either be a nested category or a component. Depending on your XML structure and needs, you might need to adjust the handling.
    #[serde(alias = "category", default)]
    subcategories: Vec<Category>,
    #[serde(alias = "component")]
    components: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Component {
    id: String,
    #[serde(default)]
    real_id: String,
    title: String,
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
    #[serde(default)]
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
    category.real_id = new_working_id.clone();

    for subcat in &mut category.subcategories {
        update_ids_in_category(subcat, &new_working_id);
    }

    // Update all components
    for comp in &mut category.components {
        comp.real_id = new_working_id.clone() + "-" + &comp.id;
    }
}


impl ComponentList {
    pub fn fix_ids(&mut self) {
        for cat in &mut self.categories {
            update_ids_in_category(cat, "");
        }
    }


    pub fn mark_required(&mut self) {
        let _ = get_all_components(&self);
    
    }

    pub fn select(&mut self, id: &str) {
        let mut dependencies: Vec<String> = Vec::new();
        let components = get_all_components(&self);

        if let Some(component) = components.iter().find(|&c| c.id == id) {
            self.find_dependencies_recursive(component, &components, &mut dependencies);
        }
        dependencies.push(id.to_owned());

        self.selected.append(&mut dependencies);
        self.selected.sort();
        self.selected.dedup();
    }

    pub fn unselect(&mut self, id: &str) {
        let mut dependants: Vec<String> = Vec::new();
        let components = get_all_components(&self);

        self.find_dependants_recursive(id, &components, &mut dependants);
        dependants.push(id.to_owned());
        // Should be faster?
        let dependants_set: HashSet<String> = dependants.into_iter().collect();

        self.selected.retain(|e| !dependants_set.contains(e));

    }

    pub fn find_dependants(&self, id: &str) -> Vec<String> {
        let mut dependants: Vec<String> = Vec::new();
        let components = get_all_components(&self);

        // Use a helper function to recursively find dependants
        self.find_dependants_recursive(id, &components, &mut dependants);

        // Since recursion can add duplicates, ensure unique elements
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

        // Initial search for the component by id and if found, start finding its dependencies
        if let Some(component) = components.iter().find(|&c| c.id == id) {
            self.find_dependencies_recursive(component, &components, &mut dependencies);
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
}

fn collect_components<'a>(categories: &'a [Category], components: &mut Vec<&'a Component>) {
    for category in categories {
        // Add all components in the current category to the components vector
        for component in &category.components {
            components.push(component);
        }

        // Recursively process subcategories
        collect_components(&category.subcategories, components);
    }
}

// Utility function to initiate the collection process and return the result
fn get_all_components<'a>(list: &'a ComponentList) -> Vec<&'a Component> {
    let mut components = Vec::new();
    collect_components(&list.categories, &mut components);
    components
}
