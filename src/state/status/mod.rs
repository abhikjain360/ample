use crate::Settings;
pub(crate) use home::Home;
pub(crate) use loading_library::LoadingLibrary;
pub(crate) use should_load_library::ShouldLoadLibrary;

mod home;
mod loading_library;
mod should_load_library;

#[derive(Debug)]
pub(crate) enum Status {
    Transistioning,
    UnrecoverableError(crate::Error),
    ShouldLoadLibrary(ShouldLoadLibrary),
    Welcome(Settings),
    LoadingLibrary(LoadingLibrary),
    Home(Home),
}
