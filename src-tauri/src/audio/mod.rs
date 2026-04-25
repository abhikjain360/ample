use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use libmpv2::Mpv;

use crate::library::LibraryState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Audio(String),
}

pub struct Engine {
    mpv: Mpv,
    generation: u64,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("generation", &self.generation)
            .finish()
    }
}

impl Engine {
    pub fn init() -> Result<Self, Error> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_option("vo", "null")?;
            init.set_option("video", "no")?;
            init.set_option("keep-open", "no")?;
            init.set_option("idle", "yes")?;

            // High-quality audio resampling
            // SoX resampler (soxr) is the gold standard for audio resampling.
            // If soxr is not available in the build, this option is silently ignored.
            init.set_option("audio-swresample-o", "resampler=soxr").ok();
            init.set_option("audio-resample-filter-size", "32")?;
            init.set_option("audio-resample-linear", "no")?;
            init.set_option("gapless-audio", "yes")?;

            Ok(())
        })
        .map_err(|e| Error::Audio(e.to_string()))?;

        Ok(Self { mpv, generation: 0 })
    }

    pub fn start(&mut self, path: PathBuf) -> Result<(), Error> {
        let path_str = path.to_string_lossy().to_string();
        self.mpv
            .command("loadfile", &[&path_str, "replace"])
            .map_err(|e| Error::Audio(e.to_string()))?;
        self.generation = self.generation.wrapping_add(1);
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), Error> {
        self.mpv
            .set_property("pause", false)
            .map_err(|e| Error::Audio(e.to_string()))
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        self.mpv
            .set_property("pause", true)
            .map_err(|e| Error::Audio(e.to_string()))
    }

    pub fn seek_forward(&mut self, seconds: f32) -> Result<(), Error> {
        let current: f64 = self
            .mpv
            .get_property("time-pos")
            .map_err(|e| Error::Audio(e.to_string()))?;
        let duration: f64 = self
            .mpv
            .get_property("duration")
            .map_err(|e| Error::Audio(e.to_string()))?;

        let new_pos = (current + seconds as f64).clamp(0.0, duration - 0.1);

        self.mpv
            .command("seek", &[&format!("{:.6}", new_pos), "absolute"])
            .map_err(|e| Error::Audio(e.to_string()))
    }

    pub fn seek_backward(&mut self, seconds: f32) -> Result<(), Error> {
        let current: f64 = self
            .mpv
            .get_property("time-pos")
            .map_err(|e| Error::Audio(e.to_string()))?;
        let duration: f64 = self
            .mpv
            .get_property("duration")
            .map_err(|e| Error::Audio(e.to_string()))?;

        let new_pos = (current - seconds as f64).clamp(0.0, duration - 0.1);

        self.mpv
            .command("seek", &[&format!("{:.6}", new_pos), "absolute"])
            .map_err(|e| Error::Audio(e.to_string()))
    }

    pub fn get_status(&self) -> (PlaybackPayload, u64) {
        let progress: f64 = self.mpv.get_property("time-pos").unwrap_or(0.0);
        let total: f64 = self.mpv.get_property("duration").unwrap_or(0.0);
        let eof: String = self.mpv.get_property("eof-reached").unwrap_or_default();
        let idle: String = self.mpv.get_property("idle-active").unwrap_or_default();

        let payload = PlaybackPayload {
            progress_frames: (progress * 1000.0) as u64,
            total_frames: (total * 1000.0) as u64,
            is_finished: eof == "yes" || idle == "yes",
        };
        (payload, self.generation)
    }
}

// -- Tauri state & commands --

pub type EngineState<'a> = tauri::State<'a, Arc<RwLock<Engine>>>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaybackPayload {
    pub progress_frames: u64,
    pub total_frames: u64,
    pub is_finished: bool,
}

#[tauri::command]
pub async fn song_start(
    id: usize,
    library: LibraryState<'_>,
    engine: EngineState<'_>,
    on_event: tauri::ipc::Channel<PlaybackPayload>,
) -> crate::Result<()> {
    let lock = library.read().unwrap();
    let Some(library) = lock.as_ref() else {
        log::error!("trying to play when library does not exist");
        return Err(Error::Audio("library not loaded".into()))?;
    };
    let Some(file) = library.files.get(id).map(|file| file.path.clone()) else {
        log::error!("invalid id to play");
        return Err(Error::Audio("invalid song id".into()))?;
    };
    drop(lock);

    let mut engine_guard = engine.write().unwrap();
    engine_guard.start(file)?;
    let generation = engine_guard.generation;
    drop(engine_guard);

    let engine = Arc::clone(&engine);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let guard = engine.read().unwrap();
            let (payload, new_generation) = guard.get_status();
            drop(guard);
            if generation == new_generation {
                if on_event.send(payload).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn song_pause(engine: EngineState<'_>) -> crate::Result<()> {
    let mut engine = engine.write().unwrap();
    engine.pause().map_err(Into::into)
}

#[tauri::command]
pub fn song_play(engine: EngineState<'_>) -> crate::Result<()> {
    let mut engine = engine.write().unwrap();
    engine.play().map_err(Into::into)
}

#[tauri::command]
pub fn song_seek_forward(engine: EngineState<'_>, seconds: f32) -> crate::Result<()> {
    let mut engine = engine.write().unwrap();
    engine.seek_forward(seconds).map_err(Into::into)
}

#[tauri::command]
pub fn song_seek_backward(engine: EngineState<'_>, seconds: f32) -> crate::Result<()> {
    let mut engine = engine.write().unwrap();
    engine.seek_backward(seconds).map_err(Into::into)
}
