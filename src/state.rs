use crate::{Status, Stream};

#[derive(Default)]
pub(crate) struct State {
    stream: Option<Stream>,
    pub(crate) status: Status,
}

impl State {
    pub(crate) fn new() -> Self {
        let mut me = Self::default();

        match Stream::new() {
            Ok(stream) => {
                me.stream = Some(stream);
                me.status = Status::Idle;
            }
            Err(error) => me.status = Status::UnrecoveableError(error),
        }

        me
    }
}
