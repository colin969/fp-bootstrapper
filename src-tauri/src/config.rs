use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub name: String,
    pub windows: Option<OsConfig>,
    pub linux: Option<OsConfig>,
    pub macos: Option<OsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct OsConfig {
    pub default_path: String,
    pub relative_executable: String,
    pub channels: HashMap<String, String>,
    pub default_channel: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut default_channels = HashMap::new();
        default_channels.insert("Stable".to_owned(), "https://nexus-dev.unstable.life/repository/components-test/components.xml".to_owned());

        Self {
            name: "Flashpoint Launcher".to_owned(),
            windows: Some(OsConfig {
                default_path: "C:/Flashpoint".to_owned(),
                relative_executable: "./Launcher/Flashpoint.exe".to_owned(),
                default_channel: "Stable".to_owned(),
                channels: default_channels,
            }),
            linux: None,
            macos: None,
        }
    }
}

pub fn load_config() -> Result<Option<AppConfig>, crate::Error> {
    match std::fs::read_to_string("./bootstrapper.toml") {
        Ok(data) => {
            Ok(Some(toml::from_str(&data)?))
        },
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                // File exists, must be another error!
                Err(e.into())
            } else {
                Ok(None)
            }
        }
    }
}
