use iced::{Element, Task, Theme, widget};

pub(crate) use crate::{
    error::{Error, IoError},
    library::*,
    message::Message,
    settings::Settings,
    state::{Snackbar, State},
    stream::Stream,
};

mod cli;
mod config;
mod error;
mod font;
mod library;
mod message;
mod settings;
mod state;
mod stream;
mod ui;

fn update(state: &mut State, message: Message) -> Task<Message> {
    match (&mut state.status, message) {
        (_, Message::Error(error)) => state.status = state::Status::UnrecoverableError(error),

        (state::Status::UnrecoverableError(_), _) => {}

        (state::Status::Welcome(_), Message::ShowLibraryAdder) => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(Into::into)
                },
                Message::SelectedLibrary,
            );
        }

        (state::Status::Welcome(_), Message::SelectedLibrary(path_opt)) => {
            let Some(path) = path_opt else {
                return Task::none();
            };

            if !path.is_dir() {
                state.snackbar = Some(Snackbar::error(widget::text(format!(
                    "Invalid library path: {}",
                    path.display()
                ))));
            }

            let (builder, walker) = Library::walker(path);
            state.status = state::Status::;
        }

        (_, message) => {
            unreachable!(
                "invalid message/state combination:\n\tstate={state:#?}\n\tmessage={message:#?}"
            )
        }
    }

    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    let elem = match &state.status {
        state::Status::UnrecoverableError(error) => widget::text(format!("error: {error}")).into(),
        state::Status::Idle { .. } => widget::text("idle").into(),
        state::Status::Welcome(_opened) => ui::welcome::view(_opened),
        state::Status::LoadingLibrary(_opened) => ui::loading::view(_opened),
    };

    if let Some(snackbar) = &state.snackbar {
        return snackbar.overlay(elem);
    }

    elem
}

fn main() -> iced::Result {
    iced::application(State::new, update, view)
        .theme(Theme::CatppuccinMocha)
        .font(font::FIRA_BOLD_BYTES)
        .font(font::FIRA_REGULAR_BYTES)
        .default_font(font::FIRA_REGULAR)
        .run()
}
