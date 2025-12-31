use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigLoadError {
    #[error("unable to locate config file")]
    Locate,
    #[error("unable to read config file")]
    Read(#[from] std::io::Error),
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
                    fs::create_dir_all(&config_dir)?;
                }

                config_dir.push("config.toml");
                config_dir
            }
        };

        if !config_path.exists() || !config_path.is_file() {
            return Err(ConfigLoadError::Locate);
        }

        let config_str = fs::read_to_string(config_path)?;
        serde_json::from_str(&config_str).map_err(From::from)
    }
}
