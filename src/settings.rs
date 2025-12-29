use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) libraries: VecDeque<PathBuf>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum SettingsInitError {
    #[error("unable to locate home directory")]
    NoHomeDirectory,
    #[error("unable to create settings cache directory: {0}")]
    CreateDirectory(crate::IoError),
    #[error("unable to create settings cache file: {0}")]
    CreateFile(crate::IoError),
    #[error("unable to read settings cache file: {0}")]
    ReadFile(crate::IoError),
    #[error("unable to parse settings cache file: {0}")]
    ParseFile(String),
}

impl Settings {
    pub(crate) fn load_or_create(
        settings_path: Option<PathBuf>,
    ) -> Result<Self, SettingsInitError> {
        if let Some(settings_path) = settings_path {
            return Self::from_settings_path(settings_path);
        };

        let settings_dir = match dirs::cache_dir() {
            Some(cache_dir) => cache_dir.join("ample"),
            None => dirs::home_dir()
                .ok_or(SettingsInitError::NoHomeDirectory)?
                .join(".cache/ample"),
        };
        let settings_path = settings_dir.join("settings.json");

        if !settings_dir.exists() {
            std::fs::create_dir_all(&settings_dir)
                .map_err(From::from)
                .map_err(SettingsInitError::CreateDirectory)?;
            let settings = Self::default();
            settings.store(settings_path)?;
            return Ok(settings);
        }

        Self::from_settings_path(settings_path)
    }

    fn from_settings_path(settings_path: PathBuf) -> Result<Self, SettingsInitError> {
        if settings_path.exists() {
            let settings = std::fs::read_to_string(settings_path)
                .map_err(From::from)
                .map_err(SettingsInitError::ReadFile)?;
            serde_json::from_str(&settings).map_err(|e| SettingsInitError::ParseFile(e.to_string()))
        } else {
            let settings = Self::default();
            settings.store(settings_path)?;
            Ok(settings)
        }
    }

    pub(crate) fn store(&self, path: impl AsRef<Path>) -> Result<(), SettingsInitError> {
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap())
            .map_err(From::from)
            .map_err(SettingsInitError::CreateFile)
    }
}
