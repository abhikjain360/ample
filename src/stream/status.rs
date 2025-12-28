pub(crate) enum StreamStatus {
    Uninitialized,
    Active(cpal::Stream),
    StreamError(cpal::StreamError),
}
