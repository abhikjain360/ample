use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Idle {
    pub(crate) init: super::Init,
    pub(crate) path: PathBuf,
}
