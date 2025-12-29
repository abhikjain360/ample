use iced::{
    Element,
    Length::Fill,
    widget::{self, container, text},
};

use crate::{Message, state};

pub(crate) fn view(_opened: &state::status::Init) -> Element<'_, Message> {
    container(text("loading library"))
        .center_x(Fill)
        .center_y(Fill)
        .into()
}
