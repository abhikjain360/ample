use std::sync::Arc;

use iced::{Element, Task, Theme, widget};

pub(crate) use crate::{
    config::Config,
    error::{Error, IoError},
    library::*,
    message::Message,
    miniaudio::Engine,
    settings::Settings,
    state::{Snackbar, State},
};

mod cli;
mod config;
mod error;
mod font;
mod library;
mod log;
mod message;
mod miniaudio;
mod settings;
mod state;
mod ui;

fn update(state: &mut State, message: Message) -> Task<Message> {
    use state::Status::*;

    match (&mut state.status, message) {
        (_, Message::Error(error)) => state.status = UnrecoverableError(error),

        (UnrecoverableError(e), _) => {
            tracing::error!("unrecoverable error: {}", e);
        }

        (Welcome(_), Message::ShowLibraryAdder) => {
            tracing::info!("showing library selector");
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

        (Welcome(_), Message::LibraryLoaded(Result::Err(err))) => {
            tracing::error!("unable to load library: {err}");
            state.snackbar = Some(Snackbar::error(widget::text(format!(
                "unable to load library: {err}"
            ))));
            return Task::none();
        }

        (_, message) => {
            return update_with_ownership(state, message);
        }
    }

    Task::none()
}

fn update_with_ownership(state: &mut State, message: Message) -> Task<Message> {
    use state::{Status::*, status};

    let status = std::mem::replace(&mut state.status, Transistioning);
    match (status, message) {
        (Welcome(settings), Message::SelectedLibrary(path_opt)) => {
            tracing::info!("selected library: {:?}", path_opt);

            let Some(path) = path_opt else {
                state.snackbar = Some(Snackbar::error(widget::text("no library selected")));
                state.status = Welcome(settings);
                return Task::none();
            };

            if !path.is_dir() {
                tracing::error!("invalid library path: {}", path.display());
                state.snackbar = Some(Snackbar::error(widget::text(format!(
                    "invalid library path: {}",
                    path.display()
                ))));
                state.status = Welcome(settings);
                return Task::none();
            }

            state.status = LoadingLibrary(status::LoadingLibrary {
                settings,
                path: path.clone(),
            });

            tracing::info!("loading library: {}", path.display());
            return Task::perform(Library::walker(path), Message::LibraryLoaded);
        }

        (ShouldLoadLibrary(status::ShouldLoadLibrary { settings, path }), Message::Tick) => {
            state.status = LoadingLibrary(status::LoadingLibrary {
                settings,
                path: path.clone(),
            });

            tracing::info!("loading library: {}", path.display());
            return Task::perform(Library::walker(path), Message::LibraryLoaded);
        }

        (
            LoadingLibrary(status::LoadingLibrary { settings, .. }),
            Message::LibraryLoaded(Ok(library)),
        ) => {
            let library = Arc::try_unwrap(library).unwrap();
            state.status = Home(status::Home { settings, library });
        }

        (status, Message::Tick) => state.status = status,

        (status, message) => {
            tracing::error!(
                "invalid message/status combination:\n\tstatus={status:#?}\n\tmessage={message:#?}"
            );
            unreachable!(
                "invalid message/state combination:\n\tstatus={status:#?}\n\tmessage={message:#?}"
            )
        }
    }

    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    use state::Status::*;

    let elem = match &state.status {
        UnrecoverableError(error) => widget::text(format!("error: {error}")).into(),
        Home(home) => ui::home::view(home),
        Welcome(opened) => ui::welcome::view(opened),
        LoadingLibrary(_) | Transistioning | ShouldLoadLibrary(_) => ui::spinner::view(),
    };

    if let Some(snackbar) = &state.snackbar {
        return snackbar.overlay(elem);
    }

    elem
}

fn subscription(state: &State) -> iced::Subscription<Message> {
    use state::Status::*;
    match state.status {
        LoadingLibrary(_) | Transistioning | ShouldLoadLibrary(_) => {
            let millis = 1000 / state.config.refresh_rate;
            iced::time::every(std::time::Duration::from_millis(millis)).map(|_| Message::Tick)
        }
        _ => iced::Subscription::none(),
    }
}

fn main() -> iced::Result {
    let cli::Opts {
        log_file,
        settings_path,
        config_path,
    } = argh::from_env();

    log::init(log_file);

    iced::application(
        move || State::new(settings_path.clone(), config_path.clone()),
        update,
        view,
    )
    .subscription(subscription)
    .theme(Theme::CatppuccinMocha)
    .font(font::FIRA_BOLD_BYTES)
    .font(font::FIRA_REGULAR_BYTES)
    .default_font(font::FIRA_REGULAR)
    .run()
}
