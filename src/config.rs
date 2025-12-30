use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::IoError;

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct Config {
    pub(crate) refresh_rate: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { refresh_rate: 60 }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigLoadError {
    #[error("unable to locate config file")]
    Locate,
    #[error("unable to read config file")]
    Read(#[from] IoError),
    #[error("unable to parse config file")]
    Parse(#[from] serde_json::Error),
}

impl Config {
    pub(crate) fn load(config_path: Option<PathBuf>) -> Result<Self, ConfigLoadError> {
        let config_path = match config_path {
            Some(path) => path,
            None => {
                let mut config_dir = dirs::config_dir().ok_or(ConfigLoadError::Locate)?;
                config_dir.push("ample");

                if !config_dir.exists() {
                    fs::create_dir_all(&config_dir).map_err(IoError::from)?;
                }

                config_dir.push("config.toml");
                config_dir
            }
        };

        if !config_path.exists() || !config_path.is_file() {
            return Err(ConfigLoadError::Locate);
        }

        let config_str = fs::read_to_string(config_path).map_err(IoError::from)?;
        serde_json::from_str(&config_str).map_err(From::from)
    }
}
