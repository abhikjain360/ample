pub(crate) enum StreamStatus {
    Uninitialized,
    Active(cpal::Stream),
    #[expect(dead_code)]
    StreamError(super::StreamError),
}
