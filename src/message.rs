use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    #[expect(dead_code)]
    Error(crate::Error),
    Tick,
    CloseSnackbar,
    ShowLibraryAdder,
    SelectedLibrary(Option<PathBuf>),
    LibraryLoaded(Result<Arc<crate::Library>, crate::IoError>),
}
