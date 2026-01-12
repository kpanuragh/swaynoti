use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::Config;

pub struct ConfigLoader;

impl ConfigLoader {
    /// Get the configuration directory path
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("org", "swaynoti", "swaynoti").map(|dirs| dirs.config_dir().to_path_buf())
    }

    /// Get the default config file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Get the data directory path (for history, etc.)
    pub fn data_dir() -> Option<PathBuf> {
        ProjectDirs::from("org", "swaynoti", "swaynoti").map(|dirs| dirs.data_dir().to_path_buf())
    }

    /// Load configuration from the default path or create default
    pub fn load() -> Result<Config> {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                return Self::load_from_path(&path);
            } else {
                debug!("Config file not found at {:?}, using defaults", path);
            }
        }
        Ok(Config::default())
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Config> {
        info!("Loading configuration from {:?}", path);
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(config: &Config) -> Result<()> {
        if let Some(path) = Self::config_path() {
            Self::save_to_path(config, &path)
        } else {
            anyhow::bail!("Could not determine config directory")
        }
    }

    /// Save configuration to a specific path
    pub fn save_to_path(config: &Config, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        info!("Configuration saved to {:?}", path);
        Ok(())
    }

    /// Create default configuration file if it doesn't exist
    pub fn ensure_config_exists() -> Result<PathBuf> {
        let path = Self::config_path().context("Could not determine config directory")?;

        if !path.exists() {
            warn!("Creating default configuration at {:?}", path);
            Self::save(&Config::default())?;
        }

        Ok(path)
    }
}
