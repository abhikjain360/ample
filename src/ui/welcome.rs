use iced::{
    Alignment::Center,
    Border, Element,
    Length::Fill,
    widget::{self, button, column, container, text},
};

use crate::{Message, font, state};

pub(crate) fn view(state: &state::Init) -> Element<'_, Message> {
    let libraries = &state.settings.libraries;

    let add_new_row = container(
        widget::row![
            text("+").size(24).font(font::FIRA_BOLD),
            widget::space().width(16),
            text("Add New Library").size(16),
        ]
        .align_y(Center),
    )
    .padding([16, 24])
    .style(|_| container::Style {
        border: Border {
            color: iced::color!(0x505050),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .width(Fill);
    let add_new_button = button(add_new_row).on_press(Message::ShowLibraryAdder);

    let library_list: Element<'_, Message> = if libraries.is_empty() {
        container(
            widget::column![
                text("No libraries yet")
                    .size(14)
                    .color(iced::color!(0x999999)),
                widget::space().height(8),
                text("Add a library to get started")
                    .size(12)
                    .color(iced::color!(0x666666)),
            ]
            .align_x(Center),
        )
        .padding([40, 20])
        .center_x(Fill)
        .into()
    } else {
        column(libraries.iter().map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());

            let path_display = path.display().to_string();

            container(widget::column![
                text(name).size(16).font(font::FIRA_BOLD),
                widget::space().height(4),
                text(path_display).size(12).color(iced::color!(0x808080)),
            ])
            .padding([14, 20])
            .width(Fill)
            .style(|_| container::Style {
                border: Border {
                    color: iced::color!(0x404040),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
        }))
        .spacing(10)
        .into()
    };

    let content = widget::column![
        text("ample").size(52).font(font::FIRA_BOLD),
        widget::space().height(8),
        text("simplified vim-like music player")
            .size(14)
            .color(iced::color!(0x808080)),
        widget::space().height(48),
        add_new_button,
        widget::space().height(32),
        text("Recent Libraries")
            .size(12)
            .color(iced::color!(0x666666)),
        widget::space().height(16),
        library_list,
    ]
    .align_x(Center)
    .width(420);

    container(content).center_x(Fill).center_y(Fill).into()
}
