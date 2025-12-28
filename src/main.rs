use iced::{Element, widget};

pub(crate) use crate::{error::Error, state::State, status::Status, stream::Stream};

mod error;
mod message;
mod state;
mod status;
mod stream;

#[derive(Debug)]
enum Message {
    Error(Error),
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Error(error) => state.status = Status::UnrecoveableError(error),
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match &state.status {
        Status::Starting => widget::text("starting").into(),
        Status::UnrecoveableError(error) => widget::text(format!("error: {error}")).into(),
        Status::Idle => widget::text("idle").into(),
    }
}

fn main() -> iced::Result {
    iced::application(State::new, update, view).run()
}
