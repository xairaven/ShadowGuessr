use crate::errors::ClientError;
use crate::logs::LogLevel;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum_macros::{Display, EnumIter};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub log_level: LogLevel,
    pub theme: Theme,
    pub map_api_key: String,
    pub interface: String,
    pub keylog_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LogLevel::default(),
            theme: Theme::Dark,
            map_api_key: String::new(),
            interface: "any".to_string(),
            // Default ProtonGE/Wine SSL keylog path for Steam games on Linux
            keylog_path: "/tmp/shadow_keys.log".to_string(),
        }
    }
}

impl Config {
    const FILENAME: &str = "config.toml";

    fn path() -> Result<PathBuf, ClientError> {
        let mut current_dir = std::env::current_exe().map_err(ConfigError::IO)?;
        current_dir.pop(); // Remove executable name

        std::fs::create_dir_all(&current_dir).map_err(ConfigError::IO)?;

        Ok(current_dir.join(Self::FILENAME))
    }

    pub fn from_file() -> Result<Self, ClientError> {
        match Self::path() {
            Ok(path) => {
                let text = std::fs::read_to_string(&path);
                match text {
                    Ok(text) => {
                        let config: Config = toml::from_str(&text)
                            .map_err(ConfigError::Deserialization)?;
                        Ok(config)
                    },
                    Err(_) => {
                        let config = Config::default();
                        config.save_to_file()?;
                        Ok(config)
                    },
                }
            },
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save_to_file(&self) -> Result<(), ClientError> {
        let data = toml::to_string(&self).map_err(ConfigError::Serialization)?;
        let path = Self::path()?;

        std::fs::write(path, data).map_err(ConfigError::IO)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to serialize. {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Failed to deserialize. {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("I/O. {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Copy, Clone, EnumIter, PartialEq, Display, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl From<&Theme> for egui::Theme {
    fn from(value: &Theme) -> Self {
        match value {
            Theme::Dark => Self::Dark,
            Theme::Light => Self::Light,
            Theme::System => Self::Light,
        }
    }
}
