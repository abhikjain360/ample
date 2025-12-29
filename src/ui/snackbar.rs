use iced::{
    Alignment::{Center, Start},
    Element,
    Length::Fill,
    widget::{button, container, stack, text},
};

impl crate::state::Snackbar {
    pub(crate) fn overlay<'a>(
        &'a self,
        content: Element<'a, crate::Message>,
    ) -> Element<'a, crate::Message> {
        let overlay = container(
            button(text("Operation Successful! (Click to dismiss)"))
                .on_press_with(|| crate::Message::CloseSnackbar),
        )
        .padding(10)
        .width(Fill)
        .height(Fill)
        .align_x(Center)
        .align_y(Start);

        stack![content, overlay].into()
    }
}
