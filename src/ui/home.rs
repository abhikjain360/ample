use std::borrow::Cow;

use iced::{
    Element, Length,
    widget::{column, container, row, scrollable, text},
};
use lofty::{file::TaggedFileExt, prelude::*};

use crate::{Message, font, state::status::Home};

pub(crate) fn view(home: &Home) -> Element<'_, Message> {
    let list = column(home.library.files.iter().map(|file| {
        let tag = file.metadata.primary_tag();

        let title = tag
            .and_then(|t| t.title())
            .unwrap_or_else(|| file.path.file_stem().unwrap_or_default().to_string_lossy());

        let artist = tag
            .and_then(|t| t.artist())
            .unwrap_or_else(|| Cow::Borrowed("Unknown Artist"));

        let duration = file.metadata.properties().duration();
        let duration_secs = duration.as_secs();
        let duration_str = format!("{}:{:02}", duration_secs / 60, duration_secs % 60);

        container(
            row![
                text(title)
                    .font(font::FIRA_REGULAR)
                    .size(14)
                    .width(Length::FillPortion(4)),
                text(artist)
                    .font(font::FIRA_REGULAR)
                    .size(14)
                    .color(iced::color!(0x808080))
                    .width(Length::FillPortion(3)),
                text(duration_str)
                    .font(font::FIRA_REGULAR)
                    .size(14)
                    .color(iced::color!(0x808080))
                    .width(Length::Shrink),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .padding(5)
        .into()
    }))
    .spacing(2);

    container(scrollable(list))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}
