#[derive(Debug)]
pub(crate) struct LoadingLibrary {
    pub(crate) builder: crate::DirTreeBuilder,
    pub(crate) init: super::Init,
}
