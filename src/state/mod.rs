pub(crate) mod snackbar;
pub(crate) mod status;

use crate::{Error, Settings, Stream, cli};
pub(crate) use snackbar::Snackbar;
use status::Idle;
pub(crate) use status::{Init, Status};

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) snackbar: Option<Snackbar>,
    pub(crate) status: Status,
}

impl State {
    pub(crate) fn new() -> Self {
        let stream = match Stream::new() {
            Ok(stream) => stream,
            Err(error) => {
                return Self {
                    snackbar: None,
                    status: Status::UnrecoverableError(Error::StreamInitError(error)),
                };
            }
        };

        let opts: cli::Opts = argh::from_env();

        let settings = match Settings::load_or_create(opts.settings_path) {
            Ok(settings) => settings,
            Err(error) => {
                return Self {
                    snackbar: None,
                    status: Status::UnrecoverableError(Error::SettingsInitError(error)),
                };
            }
        };

        let mut init = Init { stream, settings };

        loop {
            let Some(library) = init.settings.libraries.front() else {
                return Self {
                    snackbar: None,
                    status: Status::Welcome(init),
                };
            };

            if !library.is_dir() {
                init.settings.libraries.pop_front();
                continue;
            }

            return Self {
                snackbar: None,
                status: Status::Idle(Idle {
                    path: library.clone(),
                    init,
                }),
            };
        }
    }
}
