pub(crate) mod snackbar;
pub(crate) mod status;

use std::path::PathBuf;

use crate::{Config, Error, Settings};
pub(crate) use snackbar::Snackbar;
pub(crate) use status::Status;

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) snackbar: Option<Snackbar>,
    pub(crate) status: Status,
    pub(crate) config: Config,
}

impl State {
    pub(crate) fn new(settings_path: Option<PathBuf>, config_path: Option<PathBuf>) -> Self {
        let config = Config::load(config_path)
            .inspect_err(|e| tracing::error!("error while loading config: {e}"))
            .unwrap_or_default();

        let mut settings = match Settings::load_or_create(settings_path) {
            Ok(settings) => settings,
            Err(error) => {
                return Self {
                    snackbar: None,
                    status: Status::UnrecoverableError(Error::SettingsInitError(error)),
                    config,
                };
            }
        };

        loop {
            let Some(library) = settings.libraries.front() else {
                return Self {
                    snackbar: None,
                    status: Status::Welcome(settings),
                    config,
                };
            };

            if !library.is_dir() {
                settings.libraries.pop_front();
                continue;
            }

            let path = library.clone();
            return Self {
                snackbar: None,
                status: Status::ShouldLoadLibrary(status::ShouldLoadLibrary {
                    path: path.clone(),
                    settings,
                }),
                config,
            };
        }
    }
}
