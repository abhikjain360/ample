use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::RwLock,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub(crate) libraries: VecDeque<PathBuf>,
    #[serde(skip)]
    pub(crate) path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsInitError {
    #[error("unable to locate home directory")]
    NoHomeDirectory,
    #[error("unable to create settings cache directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("unable to create settings cache file: {0}")]
    CreateFile(std::io::Error),
    #[error("unable to read settings cache file: {0}")]
    ReadFile(std::io::Error),
    #[error("unable to parse settings cache file: {0}")]
    ParseFile(String),
}

impl Settings {
    pub fn load_or_create(settings_path: Option<PathBuf>) -> Result<Self, SettingsInitError> {
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
            std::fs::create_dir_all(&settings_dir).map_err(SettingsInitError::CreateDirectory)?;
            let mut settings = Self::default();
            settings.path = Some(settings_path.clone());
            settings.store(&settings_path)?;
            return Ok(settings);
        }

        Self::from_settings_path(settings_path)
    }

    fn from_settings_path(settings_path: PathBuf) -> Result<Self, SettingsInitError> {
        if settings_path.exists() {
            let contents =
                std::fs::read_to_string(&settings_path).map_err(SettingsInitError::ReadFile)?;
            let mut settings: Self = serde_json::from_str(&contents)
                .map_err(|e| SettingsInitError::ParseFile(e.to_string()))?;
            settings.path = Some(settings_path);
            Ok(settings)
        } else {
            let mut settings = Self::default();
            settings.path = Some(settings_path.clone());
            settings.store(&settings_path)?;
            Ok(settings)
        }
    }

    pub(crate) fn store(&self, path: impl AsRef<Path>) -> Result<(), SettingsInitError> {
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap())
            .map_err(SettingsInitError::CreateFile)
    }

    pub(crate) fn save(&self) -> Result<(), SettingsInitError> {
        if let Some(path) = &self.path {
            self.store(path)
        } else {
            Ok(())
        }
    }
}

pub type SettingsState<'a> = tauri::State<'a, RwLock<Settings>>;

#[tauri::command]
pub fn settings_list_libraries(settings: SettingsState<'_>) -> Vec<String> {
    settings
        .read()
        .unwrap()
        .libraries
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}

#[tauri::command]
pub fn settings_save(settings: SettingsState<'_>) -> crate::Result<()> {
    settings.read().unwrap().save()?;
    Ok(())
}

#[tauri::command]
pub fn settings_remove_library(path: String, settings: SettingsState<'_>) -> crate::Result<()> {
    let path_buf = PathBuf::from(&path);
    let mut settings = settings.write().unwrap();
    settings.libraries.retain(|p| {
        p.to_string_lossy().to_ascii_lowercase() != path_buf.to_string_lossy().to_ascii_lowercase()
    });
    settings.save()?;
    Ok(())
}
