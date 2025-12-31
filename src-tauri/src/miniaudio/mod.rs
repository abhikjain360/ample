use std::{
    ffi::c_char,
    path::PathBuf,
    ptr::{self, NonNull},
    sync::{Arc, RwLock},
    time::Duration,
};

mod error;
pub(crate) mod sys;

pub use error::Error;
use sys::*;

use crate::library::LibraryState;

#[derive(Debug)]
pub struct Engine {
    inner: NonNull<ma_engine>,
    is_sound_init: bool,
    sound: NonNull<ma_sound>,
    sound_str: Box<[c_char]>,
    generation: u64,
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl Engine {
    pub fn init() -> Result<Self, Error> {
        // TODO: we can probably make a single allocation and allocate these side-by-side
        let engine = Box::<ma_engine>::new_uninit();
        let sound = Box::<ma_sound>::new_uninit();
        unsafe {
            let inner = NonNull::new_unchecked(Box::into_raw(engine)).cast();
            let sound = NonNull::new_unchecked(Box::into_raw(sound)).cast();

            let result = ma_engine_init(ptr::null(), inner.as_ptr());
            Error::from_i32(result)?;

            Ok(Self {
                inner,
                sound,
                is_sound_init: false,
                sound_str: Box::new([]),
                generation: 0,
            })
        }
    }

    pub fn start(&mut self, path: PathBuf) -> Result<(), Error> {
        unsafe {
            if self.is_sound_init {
                ma_sound_uninit(self.sound.as_ptr());
            }
            // drops the box, deallocs the str AFTER the sound referncing it has been uninit too
            self.sound_str = path_to_c_chars(path);

            Error::from_i32(ma_sound_init_from_file(
                self.inner.as_ptr(),
                self.sound_str.as_mut_ptr(),
                ma_sound_flags_MA_SOUND_FLAG_STREAM,
                ptr::null_mut(),
                ptr::null_mut(),
                self.sound.as_ptr(),
            ))?;

            self.is_sound_init = true;
            self.generation = self.generation.wrapping_add(1);

            Error::from_i32(ma_sound_start(self.sound.as_ptr()))
        }
    }

    pub fn get_status(&self) -> (PlaybackPayload, u64) {
        let mut cursor: u64 = 0;
        let mut length: u64 = 0;
        let mut is_end: u32 = 0;

        if self.is_sound_init {
            unsafe {
                ma_sound_get_cursor_in_pcm_frames(self.sound.as_ptr(), &mut cursor);
                ma_sound_get_length_in_pcm_frames(self.sound.as_ptr(), &mut length);
                is_end = ma_sound_at_end(self.sound.as_ptr());
            }
        }
        let payload = PlaybackPayload {
            progress_frames: cursor,
            total_frames: length,
            is_finished: is_end == MA_TRUE,
        };
        (payload, self.generation)
    }

    pub fn play(&mut self) -> Result<(), Error> {
        if self.is_sound_init {
            Error::from_i32(unsafe { ma_sound_start(self.sound.as_ptr()) })?;
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        if self.is_sound_init {
            Error::from_i32(unsafe { ma_sound_stop(self.sound.as_ptr()) })?;
        }
        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            let engine = self.inner.as_ptr();
            let sound = self.sound.as_ptr();

            ma_engine_uninit(engine);
            if self.is_sound_init {
                ma_sound_uninit(sound);
            }

            _ = Box::from_raw(engine);
            _ = Box::from_raw(sound);
        }
    }
}

fn path_to_c_chars(path: PathBuf) -> Box<[c_char]> {
    let mut bytes = path.into_os_string().into_encoded_bytes();
    bytes.push(0);
    unsafe { std::mem::transmute::<Box<[u8]>, Box<[i8]>>(bytes.into_boxed_slice()) }
}

pub type EngineState<'a> = tauri::State<'a, Arc<RwLock<Engine>>>;

#[derive(Debug, serde::Serialize)]
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
        return Err(Error::DoesNotExist)?;
    };
    let Some(file) = library.files.get(id).map(|file| file.path.clone()) else {
        log::error!("invalid id to play");
        return Err(Error::InvalidArgs)?;
    };
    drop(lock);

    let mut engine_gaurd = engine.write().unwrap();
    engine_gaurd.start(file)?;
    let generation = engine_gaurd.generation;
    drop(engine_gaurd);

    let engine = Arc::clone(&engine);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let guard = engine.read().unwrap();
            let (payload, new_generation) = guard.get_status();
            drop(guard);
            if generation == new_generation {
                on_event.send(payload).unwrap();
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
    engine.pause()?;

    Ok(())
}

#[tauri::command]
pub fn song_play(engine: EngineState<'_>) -> crate::Result<()> {
    let mut engine = engine.write().unwrap();
    engine.play()?;

    Ok(())
}
