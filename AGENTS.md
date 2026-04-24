a music player. uses tauri.
backend: libmpv2 (mpv) for decode + output
frontend: react + typescript

## architecture

- **Engine** (main thread) owns a single `libmpv2::Mpv` instance, initialized with `vo=null`, `video=no`, `keep-open=no`, `idle=yes`.
- `song_start` loads a file via mpv's `loadfile` command and spawns a tokio task that polls mpv properties every 100ms.
- Polling reads `time-pos`, `duration`, `eof-reached`, and `idle-active`. Progress is sent to the frontend via a tauri `Channel<PlaybackPayload>` as `(progress_ms, total_ms, is_finished)`.
- `is_finished` is `true` when `eof-reached == "yes"` or `idle-active == "yes"`. Properties are read as `String` (`"yes"`/`"no"`) to avoid a buffer-overflow bug in `libmpv2`'s `bool` deserialization.
- `keep-open=no` ensures mpv unloads the file at EOF so `idle-active` reliably becomes true, which is used to trigger the next song on the frontend.
- seeking / song change issues a new `loadfile` command and bumps a generation counter; the old poll task breaks when it sees the generation mismatch.
