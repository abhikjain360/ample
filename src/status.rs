#[derive(Debug, Default)]
pub(crate) enum Status {
    #[default]
    Starting,
    UnrecoveableError(crate::Error),
    Idle,
}
