#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum Error {
    #[error("unable to load settings: {0}")]
    SettingsInitError(#[from] crate::settings::SettingsInitError),
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("I/O error ({kind}): {message}")]
pub(crate) struct IoError {
    kind: std::io::ErrorKind,
    message: String,
}

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: err.kind(),
            message: err.to_string(),
        }
    }
}
