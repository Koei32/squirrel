use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = include_str!("../../default.toml");

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    /// App theme. Valid values are "dark", "light" and "system".
    pub theme: String,

    /// Number of days after which a clipboard history item is automatically deleted (unless pinned).
    pub history_ttl: u32,

    /// Whether the clipboard history persists between app launches.
    pub persistence: bool,

    /// Max image size (in bytes) Squirrel should capture.
    pub max_image_size: u32,

    /// Ignore clipboard events when any of the specified programs are focused.
    pub ignore: Vec<String>,

    /// Which clipboard item types to capture.
    pub types: Types,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Types {
    pub text: bool,
    pub images: bool,
    pub files: bool,
}

impl Config {
    /// Loads the config at given path, creating one if it doesn't exist.
    pub fn load(path: &PathBuf) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(config) => {
                Ok(toml::from_str(&config).with_context(|| "Invalid configuration file")?)
            }
            Err(_) => {
                std::fs::write(path, DEFAULT_CONFIG)?;
                Ok(Self::default())
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }
}
