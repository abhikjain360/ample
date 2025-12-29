use std::fmt;

use crate::{Settings, Stream};
pub(crate) use idle::Idle;
pub(crate) use loading_library::LoadingLibrary;

mod idle;
mod loading_library;

#[derive(Debug)]
pub(crate) enum Status {
    UnrecoverableError(crate::Error),
    Welcome(Init),
    LoadingLibrary(LoadingLibrary),
    Idle(Idle),
}

pub(crate) struct Init {
    pub(super) stream: Stream,
    pub(crate) settings: Settings,
}

impl fmt::Debug for Init {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Opened")
            .field("settings", &self.settings)
            .finish()
    }
}
