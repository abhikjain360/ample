use std::sync::{Arc, RwLock};

use config::Config;
use error::Result;
use library::Library;
use miniaudio::Engine;

pub mod cli;
pub mod config;
pub mod error;
pub mod library;
pub mod miniaudio;
pub mod settings;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli::Opts {
        settings_path,
        config_path,
    } = argh::from_env();

    let config = Config::load(config_path).unwrap_or_default();

    let settings = settings::Settings::load_or_create(settings_path)
        .expect("error when loading or creating settings");

    let library = Option::<Library>::None;

    let engine = Engine::init().expect("error when initializing miniaudio engine");

    tauri::Builder::default()
        .manage(config)
        .manage(RwLock::new(settings))
        .manage(Arc::new(RwLock::new(engine)))
        .manage(RwLock::new(library))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .filter(|metadata| metadata.target().starts_with("ample"))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            settings::settings_list_libraries,
            settings::settings_save,
            settings::settings_remove_library,
            library::library_new,
            library::library_open,
            library::library_list_songs,
            miniaudio::song_start,
            miniaudio::song_play,
            miniaudio::song_pause,
            miniaudio::song_seek_forward,
            miniaudio::song_seek_backward,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
