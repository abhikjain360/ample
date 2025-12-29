use std::path::PathBuf;

use crate::Message;
pub(crate) use dirtree::{DirTree, DirTreeBuilder};
pub(crate) use file::File;

mod dirtree;
mod file;

pub(crate) struct Library {
    dir: DirTree,
}

impl Library {
    pub(crate) fn walker(path: PathBuf) -> (DirTreeBuilder, impl Future<Output = ()>) {
        DirTreeBuilder::new(path)
    }
}
