use std::path::PathBuf;

#[derive(Debug)]
pub struct ShouldLoadLibrary {
    pub(crate) settings: crate::Settings,
    pub(crate) path: PathBuf,
}
