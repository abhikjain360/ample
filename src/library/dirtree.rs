use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct DirTree {
    pub(crate) path: PathBuf,
    pub(crate) children: Vec<PathBuf>,
}

impl DirTree {
    pub(crate) fn new(path: PathBuf) -> Self {
        DirTree {
            path,
            children: vec![],
        }
    }
}
