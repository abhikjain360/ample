use std::{fmt, path::PathBuf};

use lofty::{file::TaggedFile, probe::Probe};

pub(crate) struct File {
    path: PathBuf,
    metadata: TaggedFile,
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("path", &self.path).finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileInitError {
    #[error("failed to read metadata: {0}")]
    Lofty(#[from] lofty::error::LoftyError),
}

impl File {
    pub fn new(path: PathBuf) -> Result<Self, FileInitError> {
        let metadata = Probe::open(&path)?.read()?;
        Ok(Self { path, metadata })
    }
}
