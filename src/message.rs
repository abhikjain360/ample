use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    #[expect(dead_code)]
    Error(crate::Error),
    CloseSnackbar,
    ShowLibraryAdder,
    SelectedLibrary(Option<PathBuf>),
    LibraryLoaded(Result<Arc<RwLock<crate::Library>>, crate::IoError>),
}
