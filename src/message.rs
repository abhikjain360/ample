use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    #[expect(dead_code)]
    Error(crate::Error),
    ShowLibraryAdder,
    SelectedLibrary(Option<PathBuf>),
    CloseSnackbar,
}
