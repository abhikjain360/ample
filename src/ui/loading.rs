use iced::{
    Element,
    Length::Fill,
    widget::{container, text},
};

use crate::{Message, state};

pub(crate) fn view(_loading_library: &state::status::LoadingLibrary) -> Element<'_, Message> {
    container(text("loading library"))
        .center_x(Fill)
        .center_y(Fill)
        .into()
}
