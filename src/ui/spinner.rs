use std::{
    f32::consts::PI,
    time::{SystemTime, UNIX_EPOCH},
};

use iced::{
    Element, Event, Length, Radians, Renderer, Theme, mouse,
    widget::{
        Canvas,
        canvas::{self, Action, Frame, Geometry, Path, Stroke},
        center,
    },
};

use crate::Message;

const SPINNER_SIZE: f32 = 48.0;
const STROKE_WIDTH: f32 = 4.0;
const ARC_LENGTH: f32 = PI * 1.5;

pub fn view() -> Element<'static, Message> {
    center(
        Canvas::new(SpinnerCanvas)
            .width(Length::Fixed(SPINNER_SIZE))
            .height(Length::Fixed(SPINNER_SIZE)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[derive(Debug)]
struct SpinnerCanvas;

impl canvas::Program<Message> for SpinnerCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());

        let center = frame.center();
        let radius = (SPINNER_SIZE / 2.0) - (STROKE_WIDTH / 2.0);

        let palette = theme.palette();

        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 100_000;
        let rotation = elapsed as f32 * 2.0 * PI / 3_000.0;

        let arc = Path::new(|builder| {
            builder.arc(canvas::path::Arc {
                center,
                radius,
                start_angle: Radians(rotation),
                end_angle: Radians(rotation + ARC_LENGTH),
            });
        });

        frame.stroke(
            &arc,
            Stroke::default()
                .with_width(STROKE_WIDTH)
                .with_color(palette.primary)
                .with_line_cap(canvas::LineCap::Round),
        );

        vec![frame.into_geometry()]
    }
}
