use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct LoadingLibrary {
    pub(crate) settings: crate::Settings,
    #[expect(dead_code)]
    pub(crate) path: PathBuf,
}
