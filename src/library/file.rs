use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct File {
    path: PathBuf,
}

impl File {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
