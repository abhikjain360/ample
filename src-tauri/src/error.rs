#[derive(Debug, serde::Serialize)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}

macro_rules! define_error {
    ($($variant:ident => $ty:ty),* $(,)?) => {
        #[derive(Debug, serde::Serialize)]
        #[serde(rename_all = "snake_case")]
        pub enum ErrorKind {
            $($variant),*
        }


        $(
            impl From<$ty> for Error {
                fn from(err: $ty) -> Self {
                    Self {
                        kind: ErrorKind::$variant,
                        message: err.to_string(),
                    }
                }
            }
        )*
    };
}

define_error!(
    IO => std::io::Error,
    Settings => crate::settings::SettingsInitError,
    Audio => crate::miniaudio::Error,
);
