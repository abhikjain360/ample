use iced::{
    Element, Event, Length, Radians, Renderer, Theme, mouse,
    widget::{
        Canvas,
        canvas::{self, Action, Cache, Geometry, Path, Stroke},
        center,
    },
};
use std::f32::consts::PI;

use crate::Message;

const SPINNER_SIZE: f32 = 48.0;
const STROKE_WIDTH: f32 = 4.0;
const ARC_LENGTH: f32 = PI * 1.5;

pub fn view() -> Element<'static, Message> {
    center(
        Canvas::new(SpinnerCanvas::default())
            .width(Length::Fixed(SPINNER_SIZE))
            .height(Length::Fixed(SPINNER_SIZE)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[derive(Debug, Default)]
struct SpinnerCanvas {
    cache: Cache,
}

#[derive(Debug, Default)]
pub struct SpinnerState {
    rotation: f32,
}

impl canvas::Program<Message> for SpinnerCanvas {
    type State = SpinnerState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Event,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        state.rotation += 0.06;
        if state.rotation > 2.0 * PI {
            state.rotation -= 2.0 * PI;
        }
        Some(Action::request_redraw())
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let spinner = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let radius = (SPINNER_SIZE / 2.0) - (STROKE_WIDTH / 2.0);

            let palette = theme.palette();

            let arc = Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle: Radians(state.rotation),
                    end_angle: Radians(state.rotation + ARC_LENGTH),
                });
            });

            frame.stroke(
                &arc,
                Stroke::default()
                    .with_width(STROKE_WIDTH)
                    .with_color(palette.primary)
                    .with_line_cap(canvas::LineCap::Round),
            );
        });

        vec![spinner]
    }
}
