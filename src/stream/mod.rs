use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{num::NonZeroUsize, sync::Arc};

pub(crate) use status::StreamStatus;

mod status;

const AUDIO_CHANNEL_CAPACITY: NonZeroUsize = NonZeroUsize::new(65_536).unwrap();

pub(crate) struct Stream {
    status: Arc<std::sync::Mutex<StreamStatus>>,
    #[expect(dead_code)]
    pub(crate) tx: gil::spsc::Sender<f32>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum StreamInitError {
    #[error("no default audio device found")]
    DefaultAudioDevice,
    #[error("unable to fetch supported configs")]
    SupportedAudioConfigsFetch(#[from] cpal::SupportedStreamConfigsError),
    #[error("unable to build stream to the default audio device")]
    BuildStream(#[from] cpal::BuildStreamError),
    #[error("default audio device supports no configs")]
    SupportedAudioConfigs,
    #[error("unsupported device")]
    UnsupportedDevice,
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum StreamError {
    #[error("stream error: {0}")]
    StreamError(#[from] cpal::StreamError),
    #[error("pause error: {0}")]
    PauseError(#[from] cpal::PauseStreamError),
    #[error("play error: {0}")]
    PlayError(#[from] cpal::PlayStreamError),
}

impl Stream {
    pub(crate) fn new() -> Result<Self, StreamInitError> {
        let (tx, mut rx) = gil::spsc::channel(AUDIO_CHANNEL_CAPACITY);

        let host = cpal::default_host();
        let Some(device) = host.default_output_device() else {
            return Err(StreamInitError::DefaultAudioDevice);
        };

        let config = device
            .supported_output_configs()?
            .next()
            .ok_or(StreamInitError::SupportedAudioConfigs)?
            .with_max_sample_rate()
            .config();

        let status = Arc::new(std::sync::Mutex::new(StreamStatus::Uninitialized));
        let status_clone = status.clone();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                let recved = rx.read_buffer();
                let min_len = recved.len().min(data.len());

                data[..min_len].copy_from_slice(&recved[..min_len]);
                unsafe { rx.advance(min_len) };
            },
            move |error| {
                let Ok(mut status) = status_clone.lock() else {
                    return;
                };
                *status = StreamStatus::StreamError(StreamError::StreamError(error));
            },
            None,
        )?;
        stream
            .pause()
            .map_err(|_| StreamInitError::UnsupportedDevice)?;

        let mut lock = status.lock().unwrap();
        *lock = StreamStatus::Active(stream);
        drop(lock);

        Ok(Self { status, tx })
    }

    #[expect(dead_code)]
    fn pause(&self) -> Result<(), StreamError> {
        let mut lock = self.status.lock().unwrap();
        if let StreamStatus::Active(stream) = &mut *lock {
            stream.pause()?;
        }
        Ok(())
    }

    #[expect(dead_code)]
    fn play(&self) -> Result<(), StreamError> {
        let mut lock = self.status.lock().unwrap();
        if let StreamStatus::Active(stream) = &mut *lock {
            stream.play()?;
        }
        Ok(())
    }
}
