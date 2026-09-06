use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

const DEFAULT_CONFIG: &str = include_str!("../../default.toml");

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub app: App,
    pub history: History,
    pub capture: Capture,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct App {
    /// App theme. Valid values are "dark", "light" and "system".
    pub theme: String,

    /// Ignore clipboard events when any of the specified programs are focused.
    pub ignore: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct History {
    /// Number of days after which a clipboard history item is automatically deleted (unless pinned).
    pub ttl: i64,

    /// Whether the clipboard history persists between app launches.
    pub persistence: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Capture {
    /// Whether to capture text copy events.
    pub text: bool,

    /// Whether to capture image copy events.
    pub images: bool,

    /// Whether to capture file copy events.
    pub files: bool,

    /// Max image size (in bytes) Squirrel should capture.
    pub max_image_size: u32,
}

impl Config {
    /// Loads the config at given path, creating one if it doesn't exist.
    pub fn load(path: &PathBuf) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(config) => {
                Ok(toml::from_str(&config).with_context(|| "Invalid configuration file")?)
            }
            Err(_) => {
                info!("No config found, creating default config file.");
                std::fs::write(path, DEFAULT_CONFIG)?;
                Ok(Self::default())
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("Default config should always be valid TOML")
    }
}
