use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{num::NonZeroUsize, sync::Arc};

use crate::Error;
pub(crate) use status::StreamStatus;

mod status;

pub(crate) struct Stream {
    status: Arc<std::sync::Mutex<StreamStatus>>,
    pub(crate) tx: gil::spsc::Sender<f32>,
}

const AUDIO_CHANNEL_CAPACITY: NonZeroUsize = NonZeroUsize::new(65_536).unwrap();

impl Stream {
    pub(crate) fn new() -> Result<Self, Error> {
        let (tx, mut rx) = gil::spsc::channel(AUDIO_CHANNEL_CAPACITY);

        let host = cpal::default_host();
        let Some(device) = host.default_output_device() else {
            return Err(Error::DefaultAudioDevice);
        };

        let config = device
            .supported_output_configs()?
            .next()
            .ok_or(Error::SupportedAudioConfigs)?
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
                *status = StreamStatus::StreamError(error);
            },
            None,
        )?;
        stream.pause().map_err(|_| Error::UnsupportedDevice)?;

        let mut lock = status.lock().unwrap();
        *lock = StreamStatus::Active(stream);
        drop(lock);

        Ok(Self { status, tx })
    }

    #[expect(dead_code)]
    fn pause(&self) -> Result<(), Error> {
        let mut lock = self.status.lock().unwrap();
        if let StreamStatus::Active(stream) = &mut *lock {
            return stream.pause().map_err(|_| Error::UnsupportedDevice);
        }
        Ok(())
    }

    #[expect(dead_code)]
    fn play(&self) -> Result<(), Error> {
        let mut lock = self.status.lock().unwrap();
        if let StreamStatus::Active(stream) = &mut *lock {
            return stream.play().map_err(|_| Error::UnsupportedDevice);
        }
        Ok(())
    }
}
