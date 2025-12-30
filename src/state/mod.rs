pub(crate) mod snackbar;
pub(crate) mod status;

use std::path::PathBuf;

use crate::{Error, Settings};
pub(crate) use snackbar::Snackbar;
pub(crate) use status::Status;

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) snackbar: Option<Snackbar>,
    pub(crate) status: Status,
}

impl State {
    pub(crate) fn new(settings_path: Option<PathBuf>) -> Self {
        let mut settings = match Settings::load_or_create(settings_path) {
            Ok(settings) => settings,
            Err(error) => {
                return Self {
                    snackbar: None,
                    status: Status::UnrecoverableError(Error::SettingsInitError(error)),
                };
            }
        };

        loop {
            let Some(library) = settings.libraries.front() else {
                return Self {
                    snackbar: None,
                    status: Status::Welcome(settings),
                };
            };

            if !library.is_dir() {
                settings.libraries.pop_front();
                continue;
            }

            return Self {
                snackbar: None,
                status: Status::ShouldLoadLibrary(status::ShouldLoadLibrary {
                    path: library.clone(),
                    settings,
                }),
            };
        }
    }
}
