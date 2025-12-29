use std::{fmt, num::NonZeroUsize, path::PathBuf};

use super::File;

#[derive(Debug)]
pub(crate) struct DirTree {
    path: PathBuf,
}

#[derive(Debug)]
pub(crate) enum Node {
    Directory(DirTree),
    File(File),
}

impl DirTree {}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DirTreeBuilderError {
    #[error("failed to read the directory")]
    IoError(#[from] crate::IoError),
}

pub(crate) struct DirTreeBuilder {
    partial: DirTree,
    rx: gil::spsc::Receiver<PathBuf>,
}

impl fmt::Debug for DirTreeBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DirTreeBuilder")
            .field("partial", &self.partial)
            .finish()
    }
}

impl DirTreeBuilder {
    pub(crate) fn new(path: PathBuf) -> (Self, impl Future<Output = ()>) {
        const CHANNEL_SIZE: NonZeroUsize = NonZeroUsize::new(1024).unwrap();
        let (mut tx, rx) = gil::spsc::channel(CHANNEL_SIZE);
        (
            Self {
                partial: DirTree { path: path.clone() },
                rx,
            },
            async move {},
        )
    }
}
