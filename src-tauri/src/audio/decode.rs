use std::{
    fs::File,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use audiopus::coder::GenericCtl;
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{DecoderOptions, CODEC_TYPE_NULL, CODEC_TYPE_OPUS},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, Packet, SeekMode, SeekTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};

// Opus always decodes to 48 kHz PCM regardless of the encoder's original rate.
const OPUS_DECODE_RATE: u32 = 48_000;
// Max opus frame duration is 120 ms → 5760 samples/channel at 48 kHz.
const OPUS_MAX_FRAME_SAMPLES: usize = 5760;

use super::SharedState;

// ── Adapter implementations for rubato ──────────────────────────────────────

/// Read-only interleaved f32 buffer for rubato input.
struct InterleavedInput<'a> {
    data: &'a [f32],
    channels: usize,
    frames: usize,
}

impl<'a> rubato::audioadapter::Adapter<'a, f32> for InterleavedInput<'a> {
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> f32 {
        debug_assert!(frame < self.frames && channel < self.channels);
        *self.data.get_unchecked(frame * self.channels + channel)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.frames
    }
}

/// Mutable interleaved f32 buffer for rubato output.
struct InterleavedOutput<'a> {
    data: &'a mut [f32],
    channels: usize,
    frames: usize,
}

impl<'a> rubato::audioadapter::Adapter<'a, f32> for InterleavedOutput<'a> {
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> f32 {
        debug_assert!(frame < self.frames && channel < self.channels);
        *self.data.get_unchecked(frame * self.channels + channel)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.frames
    }
}

impl<'a> rubato::audioadapter::AdapterMut<'a, f32> for InterleavedOutput<'a> {
    unsafe fn write_sample_unchecked(
        &mut self,
        channel: usize,
        frame: usize,
        value: &f32,
    ) -> bool {
        debug_assert!(frame < self.frames && channel < self.channels);
        *self.data.get_unchecked_mut(frame * self.channels + channel) = *value;
        true
    }
}

// ── File probing ────────────────────────────────────────────────────────────

/// Probe an audio file to get its total duration and source sample rate.
pub fn probe_file(path: &PathBuf) -> Result<(f64, u32), String> {
    let file = File::open(path).map_err(|e| format!("failed to open file: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("unsupported format: {e}"))?;

    let format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "no supported audio track found".to_string())?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let duration_secs = match (track.codec_params.n_frames, track.codec_params.time_base) {
        (Some(n_frames), Some(tb)) => {
            let time = tb.calc_time(track.codec_params.start_ts + n_frames);
            time.seconds as f64 + time.frac
        }
        (Some(n_frames), None) => n_frames as f64 / sample_rate as f64,
        _ => 0.0,
    };

    Ok((duration_secs, sample_rate))
}

// ── Decode thread ───────────────────────────────────────────────────────────

/// Main decode thread entry point. Opens the file, decodes, optionally
/// resamples, and pushes interleaved f32 samples to the ring buffer.
pub fn run(
    path: PathBuf,
    start_position_secs: f64,
    mut producer: rtrb::Producer<f32>,
    ring_capacity: usize,
    stop_signal: Arc<AtomicBool>,
    shared: Arc<SharedState>,
    device_sample_rate: u32,
    device_channels: u16,
) -> Result<(), String> {
    // Open and probe the file
    let file = File::open(&path).map_err(|e| format!("failed to open file: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("unsupported format: {e}"))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "no supported audio track found".to_string())?;

    let track_id = track.id;
    let is_opus = track.codec_params.codec == CODEC_TYPE_OPUS;
    let source_channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2);

    // Opus streams always decode to 48 kHz regardless of the codec_params rate
    // (which reflects the encoder's original input rate, not decoder output).
    let source_sample_rate = if is_opus {
        OPUS_DECODE_RATE
    } else {
        track.codec_params.sample_rate.unwrap_or(44100)
    };

    let mut decoder = if is_opus {
        AudioDecoder::new_opus(source_channels)?
    } else {
        let sym = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("unsupported codec: {e}"))?;
        AudioDecoder::Symphonia {
            decoder: sym,
            sample_buf: None,
        }
    };

    // Seek if starting from a non-zero position
    if start_position_secs > 0.0 {
        if let Err(e) = format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: Time::from(start_position_secs),
                track_id: Some(track_id),
            },
        ) {
            log::warn!("seek failed: {e}, starting from beginning");
        }
        decoder.reset();
    }

    // Create resampler if source and device sample rates differ
    let needs_resample = source_sample_rate != device_sample_rate;
    let chunk_size: usize = 1024;

    let mut resampler: Option<Box<dyn Resampler<f32>>> = if needs_resample {
        log::info!(
            "resampling: {} Hz -> {} Hz (sinc interpolation)",
            source_sample_rate,
            device_sample_rate
        );
        Some(create_resampler(
            source_sample_rate as usize,
            device_sample_rate as usize,
            device_channels as usize,
            chunk_size,
        )?)
    } else {
        None
    };

    // Pre-allocate resampler output buffer
    let resampler_output_max = resampler
        .as_ref()
        .map(|r| r.output_frames_max() * device_channels as usize)
        .unwrap_or(0);
    let mut resample_output_buf = vec![0.0f32; resampler_output_max];

    // Intermediate buffer to accumulate decoded frames for fixed-size resampler chunks
    let mut intermediate_buf: Vec<f32> = Vec::with_capacity(chunk_size * device_channels as usize);

    // ── Decode loop ──
    loop {
        if stop_signal.load(Ordering::Relaxed) {
            return Ok(());
        }

        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(_) => break, // EOF or unrecoverable error
        };

        if packet.track_id() != track_id {
            continue;
        }

        let samples = match decoder.decode(&packet) {
            Ok(s) => s,
            Err(DecodeErr::Skip(msg)) => {
                log::warn!("decode error (skipping packet): {msg}");
                continue;
            }
            Err(DecodeErr::Fatal(msg)) => {
                log::warn!("fatal decode error: {msg}");
                break;
            }
        };

        if samples.is_empty() {
            continue;
        }

        // Channel conversion (source → device channel count)
        let converted = convert_channels(&samples, source_channels, device_channels as usize);

        if let Some(ref mut resampler) = resampler {
            // Accumulate decoded frames, process in fixed-size chunks
            intermediate_buf.extend_from_slice(&converted);

            let samples_per_chunk = chunk_size * device_channels as usize;

            while intermediate_buf.len() >= samples_per_chunk {
                let chunk: Vec<f32> = intermediate_buf.drain(..samples_per_chunk).collect();
                let produced = resample_chunk(
                    resampler.as_mut(),
                    &chunk,
                    device_channels as usize,
                    chunk_size,
                    &mut resample_output_buf,
                    None,
                )?;

                let out = &resample_output_buf[..produced * device_channels as usize];
                if !push_samples(&mut producer, out, &stop_signal) {
                    return Ok(());
                }
            }
        } else {
            // No resampling: push decoded samples directly
            if !push_samples(&mut producer, &converted, &stop_signal) {
                return Ok(());
            }
        }
    }

    // Flush remaining samples in the intermediate buffer (partial chunk)
    if let Some(ref mut resampler) = resampler {
        if !intermediate_buf.is_empty() {
            let remaining_frames = intermediate_buf.len() / device_channels as usize;

            // Zero-pad to full chunk size so the adapter has enough data
            intermediate_buf.resize(chunk_size * device_channels as usize, 0.0);

            let produced = resample_chunk(
                resampler.as_mut(),
                &intermediate_buf,
                device_channels as usize,
                chunk_size,
                &mut resample_output_buf,
                Some(remaining_frames),
            )?;

            let out = &resample_output_buf[..produced * device_channels as usize];
            if !push_samples(&mut producer, out, &stop_signal) {
                return Ok(());
            }
        }
    }

    // Decoding is complete
    shared.decode_finished.store(true, Ordering::Release);

    // Wait for the ring buffer to fully drain (all audio played)
    loop {
        if stop_signal.load(Ordering::Relaxed) || producer.is_abandoned() {
            return Ok(());
        }

        // All write slots available = buffer is empty
        if producer.slots() >= ring_capacity {
            break;
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    shared.playback_finished.store(true, Ordering::Release);

    Ok(())
}

// ── Unified decoder (symphonia codecs + audiopus for Opus) ──────────────────

enum AudioDecoder {
    Symphonia {
        decoder: Box<dyn symphonia::core::codecs::Decoder>,
        sample_buf: Option<SampleBuffer<f32>>,
    },
    Opus {
        decoder: audiopus::coder::Decoder,
        channels: usize,
        scratch: Vec<f32>,
    },
}

enum DecodeErr {
    Skip(String),
    Fatal(String),
}

impl AudioDecoder {
    fn new_opus(channels: usize) -> Result<Self, String> {
        let ch = match channels {
            1 => audiopus::Channels::Mono,
            2 => audiopus::Channels::Stereo,
            n => return Err(format!("unsupported opus channel count: {n}")),
        };
        let decoder = audiopus::coder::Decoder::new(audiopus::SampleRate::Hz48000, ch)
            .map_err(|e| format!("failed to init opus decoder: {e}"))?;
        Ok(Self::Opus {
            decoder,
            channels,
            scratch: vec![0.0; OPUS_MAX_FRAME_SAMPLES * channels],
        })
    }

    fn reset(&mut self) {
        match self {
            Self::Symphonia { decoder, .. } => decoder.reset(),
            Self::Opus { decoder, .. } => {
                let _ = decoder.reset_state();
            }
        }
    }

    fn decode(&mut self, packet: &Packet) -> Result<Vec<f32>, DecodeErr> {
        match self {
            Self::Symphonia { decoder, sample_buf } => {
                let decoded = match decoder.decode(packet) {
                    Ok(d) => d,
                    Err(SymphoniaError::DecodeError(e)) => {
                        return Err(DecodeErr::Skip(e.to_string()))
                    }
                    Err(SymphoniaError::IoError(e)) => {
                        return Err(DecodeErr::Skip(e.to_string()))
                    }
                    Err(e) => return Err(DecodeErr::Fatal(e.to_string())),
                };
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    *sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }
                let buf = sample_buf.as_mut().unwrap();
                buf.copy_interleaved_ref(decoded);
                Ok(buf.samples().to_vec())
            }
            Self::Opus { decoder, channels, scratch } => {
                let input = packet.buf();
                let samples_per_ch = decoder
                    .decode_float(Some(input), &mut scratch[..], false)
                    .map_err(|e| DecodeErr::Skip(format!("opus decode: {e}")))?;
                Ok(scratch[..samples_per_ch * *channels].to_vec())
            }
        }
    }
}

// ── Resampler creation ──────────────────────────────────────────────────────

fn create_resampler(
    source_rate: usize,
    device_rate: usize,
    channels: usize,
    chunk_size: usize,
) -> Result<Box<dyn Resampler<f32>>, String> {
    // ratio = output_rate / input_rate
    let ratio = device_rate as f64 / source_rate as f64;

    let sinc_len = 256;
    let window = WindowFunction::BlackmanHarris2;
    let f_cutoff: f32 = rubato::calculate_cutoff(sinc_len, window);

    let params = SincInterpolationParameters {
        sinc_len,
        f_cutoff,
        oversampling_factor: 256,
        interpolation: SincInterpolationType::Cubic,
        window,
    };

    let resampler = Async::<f32>::new_sinc(
        ratio,
        1.1, // allow small ratio adjustment margin
        &params,
        chunk_size,
        channels,
        FixedAsync::Input,
    )
    .map_err(|e| format!("failed to create resampler: {e}"))?;

    Ok(Box::new(resampler))
}

// ── Resampling a single chunk ───────────────────────────────────────────────

fn resample_chunk(
    resampler: &mut dyn Resampler<f32>,
    interleaved_input: &[f32],
    channels: usize,
    chunk_frames: usize,
    output_buf: &mut [f32],
    partial_len: Option<usize>,
) -> Result<usize, String> {
    let input = InterleavedInput {
        data: interleaved_input,
        channels,
        frames: chunk_frames,
    };

    let output_frames_max = output_buf.len() / channels;
    let mut output = InterleavedOutput {
        data: output_buf,
        channels,
        frames: output_frames_max,
    };

    let indexing = partial_len.map(|len| rubato::Indexing {
        input_offset: 0,
        output_offset: 0,
        partial_len: Some(len),
        active_channels_mask: None,
    });

    let (_, produced) = resampler
        .process_into_buffer(&input, &mut output, indexing.as_ref())
        .map_err(|e| format!("resample error: {e}"))?;

    Ok(produced)
}

// ── Channel conversion ──────────────────────────────────────────────────────

/// Convert interleaved audio from `from_ch` channels to `to_ch` channels.
///
/// - Mono → Stereo: duplicate the mono channel.
/// - Stereo → Mono: average left and right.
/// - N → M where N > M: take the first M channels.
/// - N → M where N < M: copy existing channels, fill extra with channel 0.
fn convert_channels(samples: &[f32], from_ch: usize, to_ch: usize) -> Vec<f32> {
    if from_ch == to_ch {
        return samples.to_vec();
    }

    let frames = samples.len() / from_ch;
    let mut out = Vec::with_capacity(frames * to_ch);

    // Special case: stereo → mono (average L+R for quality)
    if from_ch == 2 && to_ch == 1 {
        for frame in 0..frames {
            let l = samples[frame * 2];
            let r = samples[frame * 2 + 1];
            out.push((l + r) * 0.5);
        }
        return out;
    }

    for frame in 0..frames {
        let src = frame * from_ch;
        for ch in 0..to_ch {
            if ch < from_ch {
                out.push(samples[src + ch]);
            } else {
                // Fill extra output channels with channel 0 (mono expansion)
                out.push(samples[src]);
            }
        }
    }

    out
}

// ── Ring buffer push with backpressure ───────────────────────────────────────

/// Push interleaved samples to the ring buffer, sleeping when full.
/// Returns `false` if the stop signal was received (caller should exit).
fn push_samples(
    producer: &mut rtrb::Producer<f32>,
    samples: &[f32],
    stop_signal: &AtomicBool,
) -> bool {
    let mut offset = 0;
    while offset < samples.len() {
        if stop_signal.load(Ordering::Relaxed) {
            return false;
        }

        match producer.push(samples[offset]) {
            Ok(()) => offset += 1,
            Err(_) => {
                // Buffer full — sleep briefly and retry
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
    true
}
