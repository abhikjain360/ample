use crate::Settings;
pub(crate) use idle::Idle;
pub(crate) use loading_library::LoadingLibrary;
pub(crate) use should_load_library::ShouldLoadLibrary;

mod idle;
mod loading_library;
mod should_load_library;

#[derive(Debug)]
pub(crate) enum Status {
    Transistioning,
    UnrecoverableError(crate::Error),
    ShouldLoadLibrary(ShouldLoadLibrary),
    Welcome(Settings),
    #[expect(dead_code)]
    LoadingLibrary(LoadingLibrary),
    #[expect(dead_code)]
    Idle(Idle),
}
