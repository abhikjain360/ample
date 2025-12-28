#[derive(Debug, thiserror::Error)]
enum Error {
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
