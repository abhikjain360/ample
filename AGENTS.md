a music player. uses tauri.
backend: symphonia (decode) + cpal (output) + rubato (sinc resample) + rtrb (lock-free ring buffer)
frontend: react + typescript

## architecture

three layers, each on its own thread:

1. **cpal callback** (audio thread) — pops f32 from rtrb ring buffer, fills silence when empty. zero logic here, no locks/atomics.
2. **decode thread** (per song) — symphonia decode → channel convert → rubato resample (if rates differ) → push to ring buffer. backpressure via 1ms sleep when buffer full, checks stop signal each iteration.
3. **Engine** (main thread) — owns cpal device + optional PlaybackSession. session = stream + decode thread + ring buffer + shared state. seeking/song change drops old session, creates new one.

playback position tracked via wall-clock time (Instant), adjusted for pauses. progress sent to frontend every 100ms via tauri Channel as (progress_frames, total_frames, is_finished). "frames" are actually milliseconds — frontend just does the ratio.

resampler: 256-point sinc, cubic interpolation, BlackmanHarris2 window, 256x oversampling. created per-song only when source rate != device rate.

end-of-song detection: decode thread finishes → polls producer.slots() until ring buffer drains → sets playback_finished atomic → frontend triggers next song.
