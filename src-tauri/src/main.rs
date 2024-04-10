// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, PoisonError};

use serde::{Deserialize, Serialize};
use state::AppState;
use tauri::{async_runtime::Mutex, Manager, State, Window};

mod config;
mod state;

#[derive(Clone, Serialize, Deserialize)]
enum OperatingSystem {
    LINUX,
    WINDOWS,
    MACOS
}

#[derive(Clone, Serialize, Deserialize)]
pub enum View {
    SETUP,
    SETUPSELECT,
}

// Create a custom Error that we can return in Results
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Implement std::io::Error for our Error enum
    #[error(transparent)]
    Io(#[from] std::io::Error),
    // Add a PoisonError, but we implement it manually later
    #[error("the mutex was poisoned")]
    PoisonError(String),
    #[error("error reading config file: {0}")]
    ReadConfigError(String),
    #[error("{0}")]
    GeneralError(String),
}

// Implement Serialize for the error
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Implement From<PoisonError> for Error to convert it to something we have set up serialization for
impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        // We "just" convert the error to a string here
        Error::PoisonError(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        // We "just" convert the error to a string here
        Error::ReadConfigError(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetInstallationPath {
    pub installation_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeView {
    pub view: View,
}

#[tauri::command]
async fn set_installation_path(window: Window, app_state: State<'_, Arc<Mutex<AppState>>>, path: String) -> Result<(), Error> {
    let mut state = app_state.lock().await;
    state.installation_path = path;
    sync_state(&window, state.clone()).unwrap();
    Ok(())
}

#[tauri::command]
async fn change_view(window: Window, app_state: State<'_, Arc<Mutex<AppState>>>, view: View) -> Result<(), Error> {
    let mut state = app_state.lock().await;
    state.change_view(view).await?;
    sync_state(&window, state.clone()).unwrap();
    Ok(())
}

#[tauri::command]
async fn find_component_dependants(app_state: State<'_, Arc<Mutex<AppState>>>, id: String) -> Result<Vec<String>, Error>{
    let state = app_state.lock().await;
    Ok(state.components.find_dependants(&id))
}

#[tauri::command]
async fn find_component_dependencies(app_state: State<'_, Arc<Mutex<AppState>>>, id: String) -> Result<Vec<String>, Error>{
    let state = app_state.lock().await;
    Ok(state.components.find_dependencies(&id))
}

#[tauri::command]
async fn select_component(window: Window, app_state: State<'_, Arc<Mutex<AppState>>>, id: String) -> Result<(), Error> {
    let mut state = app_state.lock().await;
    state.components.select(&id);
    sync_selected(&window, state.components.selected.clone()).unwrap();
    Ok(())
}

#[tauri::command]
async fn unselect_component(window: Window, app_state: State<'_, Arc<Mutex<AppState>>>, id: String) -> Result<(), Error> {
    let mut state = app_state.lock().await;
    state.components.unselect(&id);
    sync_selected(&window, state.components.selected.clone()).unwrap();
    Ok(())
}

#[tauri::command]
async fn init_process(window: Window, app_state: State<'_, Arc<Mutex<AppState>>>) -> Result<AppState, Error> {
    let state = app_state.lock().await;
    if let Some(fe) = state.fatal_error.as_ref() {
        fatal_error(&window, &fe).unwrap();
    }
    Ok(state.clone())
}

fn sync_selected(window: &Window, selected: Vec<String>) -> Result<(), tauri::Error> {
    window.emit("sync_selected", selected)
}

fn sync_state(window: &Window, state: AppState) -> Result<(), tauri::Error> {
    window.emit("sync", state)
}

fn fatal_error(window: &Window, message: &str) -> Result<(), tauri::Error> {
    window.emit("fatal_error", message)
}

fn main() {
    // Initialize the app state, wrap in mutex w/ reference counter for safe sharing
    let mut state = AppState::default();

    // Load config into state
    match config::load_config() {
        Ok(data) => {
            if let Some(config) = data {
                state.config = config;
                state.adjust_installation_target();
            } else {
                state.adjust_installation_target();
            }
        },
        Err(e) => {
            // Store error for GUI to show later
            state.fatal_error = Some(Error::GeneralError(e.to_string()).to_string());
        }
    }

    let app_state = Arc::new(Mutex::new(state));

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            #[cfg(debug_assertions)] // only include this code on debug builds
            {
              window.open_devtools();
              window.close_devtools();
            }


            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_process,
            set_installation_path,
            change_view,
            find_component_dependants,
            find_component_dependencies,
            select_component,
            unselect_component,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
