// ABOUTME: Configuration management for sticky-situation
// ABOUTME: Loads config from XDG directories with sane defaults

use crate::{Result, StickyError};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub log_conflicts: bool,
    pub conflict_log_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("", "", "sticky-situation")
            .expect("Could not determine project directories");

        let data_dir = proj_dirs.data_dir();

        Self {
            database_path: data_dir.join("stickies.db"),
            log_conflicts: true,
            conflict_log_path: data_dir.join("conflicts.log"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("", "", "sticky-situation")
            .ok_or_else(|| StickyError::Config("Could not determine config dir".into()))?;

        let config_path = proj_dirs.config_dir().join("config.toml");

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config =
            toml::from_str(&contents).map_err(|e| StickyError::Config(e.to_string()))?;

        Ok(config)
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.conflict_log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}
