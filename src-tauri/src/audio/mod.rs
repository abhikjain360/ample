use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::library::LibraryState;

mod decode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Audio(String),
    #[error("no audio output device found")]
    NoDevice,
    #[error("invalid operation")]
    InvalidOperation,
}

pub struct Engine {
    device: cpal::Device,
    supported_config: cpal::SupportedStreamConfig,
    session: Option<PlaybackSession>,
    generation: u64,
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("generation", &self.generation)
            .finish()
    }
}

struct SharedState {
    decode_finished: AtomicBool,
    playback_finished: AtomicBool,
}

struct PlaybackSession {
    stream: cpal::Stream,
    decode_handle: Option<std::thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
    shared: Arc<SharedState>,
    path: PathBuf,
    start_position_secs: f64,
    total_duration_secs: f64,
    play_start_instant: Instant,
    accumulated_pause_duration: Duration,
    pause_start_instant: Option<Instant>,
}

impl Drop for PlaybackSession {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(handle) = self.decode_handle.take() {
            let _ = handle.join();
        }
    }
}

impl PlaybackSession {
    fn current_time_secs(&self) -> f64 {
        let elapsed = self.play_start_instant.elapsed();
        let pause_duration = self.accumulated_pause_duration
            + self
                .pause_start_instant
                .map(|i| i.elapsed())
                .unwrap_or(Duration::ZERO);
        let play_duration = elapsed.saturating_sub(pause_duration);
        self.start_position_secs + play_duration.as_secs_f64()
    }
}

/// Size of the ring buffer in seconds of audio.
const RING_BUFFER_SECONDS: f64 = 1.0;

impl Engine {
    pub fn init() -> Result<Self, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let supported_config = device
            .default_output_config()
            .map_err(|e| Error::Audio(e.to_string()))?;

        log::info!(
            "audio device: sample_rate={}, channels={}, format={:?}",
            supported_config.sample_rate(),
            supported_config.channels(),
            supported_config.sample_format(),
        );

        Ok(Self {
            device,
            supported_config,
            session: None,
            generation: 0,
        })
    }

    pub fn start(&mut self, path: PathBuf) -> Result<(), Error> {
        // Drop old session first
        self.session = None;

        let (total_duration_secs, _source_sr) =
            decode::probe_file(&path).map_err(|e| Error::Audio(e.to_string()))?;

        self.create_session(path, 0.0, total_duration_secs)?;
        self.generation = self.generation.wrapping_add(1);
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), Error> {
        if let Some(ref mut session) = self.session {
            session
                .stream
                .play()
                .map_err(|e| Error::Audio(e.to_string()))?;
            if let Some(pause_start) = session.pause_start_instant.take() {
                session.accumulated_pause_duration += pause_start.elapsed();
            }
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        if let Some(ref mut session) = self.session {
            session
                .stream
                .pause()
                .map_err(|e| Error::Audio(e.to_string()))?;
            session.pause_start_instant = Some(Instant::now());
        }
        Ok(())
    }

    pub fn seek_forward(&mut self, seconds: f32) -> Result<(), Error> {
        let Some(ref session) = self.session else {
            return Err(Error::InvalidOperation);
        };
        let current = session.current_time_secs();
        let total = session.total_duration_secs;
        let new_pos = (current + seconds as f64).clamp(0.0, total - 0.1);
        let path = session.path.clone();
        let total_duration_secs = session.total_duration_secs;

        self.session = None;
        self.create_session(path, new_pos, total_duration_secs)
    }

    pub fn seek_backward(&mut self, seconds: f32) -> Result<(), Error> {
        let Some(ref session) = self.session else {
            return Err(Error::InvalidOperation);
        };
        let current = session.current_time_secs();
        let total = session.total_duration_secs;
        let new_pos = (current - seconds as f64).clamp(0.0, total - 0.1);
        let path = session.path.clone();
        let total_duration_secs = session.total_duration_secs;

        self.session = None;
        self.create_session(path, new_pos, total_duration_secs)
    }

    pub fn get_status(&self) -> (PlaybackPayload, u64) {
        if let Some(ref session) = self.session {
            let current_secs = session.current_time_secs().min(session.total_duration_secs);
            let total_secs = session.total_duration_secs;
            let is_finished = session.shared.playback_finished.load(Ordering::Relaxed);

            // Use milliseconds as pseudo-frames for the frontend ratio calculation.
            let payload = PlaybackPayload {
                progress_frames: (current_secs * 1000.0) as u64,
                total_frames: (total_secs * 1000.0) as u64,
                is_finished,
            };
            (payload, self.generation)
        } else {
            (
                PlaybackPayload {
                    progress_frames: 0,
                    total_frames: 0,
                    is_finished: false,
                },
                self.generation,
            )
        }
    }

    fn create_session(
        &mut self,
        path: PathBuf,
        start_secs: f64,
        total_duration_secs: f64,
    ) -> Result<(), Error> {
        let device_sample_rate = self.supported_config.sample_rate();
        let device_channels = self.supported_config.channels();
        let sample_format = self.supported_config.sample_format();

        let ring_size = (device_sample_rate as f64 * RING_BUFFER_SECONDS) as usize
            * device_channels as usize;
        let (producer, consumer) = rtrb::RingBuffer::new(ring_size);

        let config = self.supported_config.config();

        let stream = build_output_stream(&self.device, &config, sample_format, consumer)?;
        stream
            .play()
            .map_err(|e| Error::Audio(e.to_string()))?;

        let shared = Arc::new(SharedState {
            decode_finished: AtomicBool::new(false),
            playback_finished: AtomicBool::new(false),
        });
        let stop_signal = Arc::new(AtomicBool::new(false));

        let decode_path = path.clone();
        let decode_shared = Arc::clone(&shared);
        let decode_stop = Arc::clone(&stop_signal);

        let decode_handle = std::thread::Builder::new()
            .name("audio-decode".into())
            .spawn(move || {
                if let Err(e) = decode::run(
                    decode_path,
                    start_secs,
                    producer,
                    ring_size,
                    decode_stop,
                    decode_shared,
                    device_sample_rate,
                    device_channels,
                ) {
                    log::error!("decode thread error: {e}");
                }
            })
            .map_err(|e| Error::Audio(e.to_string()))?;

        self.session = Some(PlaybackSession {
            stream,
            decode_handle: Some(decode_handle),
            stop_signal,
            shared,
            path,
            start_position_secs: start_secs,
            total_duration_secs,
            play_start_instant: Instant::now(),
            accumulated_pause_duration: Duration::ZERO,
            pause_start_instant: None,
        });

        Ok(())
    }
}

fn build_output_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
    consumer: rtrb::Consumer<f32>,
) -> Result<cpal::Stream, Error> {
    match sample_format {
        cpal::SampleFormat::F32 => build_stream_typed::<f32>(device, config, consumer),
        cpal::SampleFormat::I16 => build_stream_typed::<i16>(device, config, consumer),
        cpal::SampleFormat::U16 => build_stream_typed::<u16>(device, config, consumer),
        cpal::SampleFormat::I32 => build_stream_typed::<i32>(device, config, consumer),
        cpal::SampleFormat::F64 => build_stream_typed::<f64>(device, config, consumer),
        fmt => Err(Error::Audio(format!(
            "unsupported device sample format: {fmt:?}"
        ))),
    }
}

fn build_stream_typed<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: rtrb::Consumer<f32>,
) -> Result<cpal::Stream, Error>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    *sample = T::from_sample(consumer.pop().unwrap_or(0.0));
                }
            },
            |err| log::error!("audio stream error: {err}"),
            None,
        )
        .map_err(|e| Error::Audio(e.to_string()))
}

// -- Tauri state & commands --

pub type EngineState<'a> = tauri::State<'a, Arc<RwLock<Engine>>>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaybackPayload {
    progress_frames: u64,
    total_frames: u64,
    is_finished: bool,
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
