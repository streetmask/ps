use std::{cell::RefCell, rc::Rc};

use iced::{
    button::State as ButtonState,
    canvas::{self, Canvas as IcedCanvas, Cursor, Frame, Geometry, Path, Stroke},
    canvas::{
        event::{self, Event},
        path::Builder,
    },
    mouse, Alignment, Button, Column, Command, Element, Length, Point, Rectangle, Text,
};

use crate::app::{message::CanvasMessage, Flags};
use crate::app::{utils::get_size, UserSettings};

use super::Component;

#[derive(Default)]
pub struct Canvas {
    state: State,
    curves: Vec<Curve>,
    button_state: ButtonState,
}

impl Component for Canvas {
    type Message = CanvasMessage;

    fn new(_flags: &mut Flags) -> (Self, Command<Self::Message>) {
        (Canvas::default(), Command::none())
    }

    fn update(
        &mut self,
        message: CanvasMessage,
        _settings: Rc<RefCell<UserSettings>>,
    ) -> Command<CanvasMessage> {
        match message {
            CanvasMessage::AddCurve(curve) => {
                self.curves.push(curve);
                self.state.request_redraw();
            }
            CanvasMessage::Clear => {
                self.state = State::default();
                self.curves.clear();
            }
        }
        Command::none()
    }

    fn view(&mut self, _settings: Rc<RefCell<UserSettings>>) -> Element<CanvasMessage> {
        Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(self.state.view(&self.curves).map(CanvasMessage::AddCurve))
            .push(
                Button::new(&mut self.button_state, Text::new("Clear"))
                    .padding(8)
                    .on_press(CanvasMessage::Clear),
            )
            .into()
    }
}

#[derive(Default)]
pub struct State {
    curve: Curve,
    cache: canvas::Cache,
}

impl State {
    pub fn update(&mut self, mouse_event: mouse::Event, new: Point) -> Option<Curve> {
        match mouse_event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                let labor: usize = self.curve.labor().into();
                if self.curve.points.len() + 1 < labor {
                    self.curve.points.push(new);
                } else if self.curve.points.len() + 1 == labor {
                    let mut curve = Curve::new(self.curve.kind);
                    curve.points.append(&mut self.curve.points);
                    curve.points.push(new);
                    return Some(curve);
                }
                None
            }
            _ => None,
        }
    }

    pub fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Geometry {
        let mut frame = Frame::new(bounds.size());

        if let Some(cursor_position) = cursor.position_in(&bounds) {
            let path = Path::new(|p| match self.curve.kind {
                CurveKind::Rectangle => {
                    if self.curve.points.len() == 1 {
                        let top_left = self.curve.points[0];
                        let right_bottom = cursor_position;
                        p.rectangle(top_left, get_size(top_left, right_bottom));
                    }
                }
            });
            frame.stroke(&path, Stroke::default().with_width(2.0))
        }

        frame.into_geometry()
    }

    pub fn view<'a>(&'a mut self, curves: &'a [Curve]) -> Element<'a, Curve> {
        IcedCanvas::new(Curves {
            state: self,
            curves,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}

struct Curves<'a> {
    state: &'a mut State,
    curves: &'a [Curve],
}

impl<'a> canvas::Program<Curve> for Curves<'a> {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Curve>) {
        let cursor_position = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => (
                event::Status::Captured,
                self.state.update(mouse_event, cursor_position),
            ),
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let content = self.state.cache.draw(bounds.size(), |frame: &mut Frame| {
            Curve::draw_all(self.curves, frame);

            frame.stroke(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                Stroke::default(),
            );
        });

        let pending_curve = self.state.draw(bounds, cursor);

        vec![content, pending_curve]
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CurveKind {
    #[default]
    Rectangle,
}

#[derive(Default, Debug, Clone)]
pub struct Curve {
    points: Vec<Point>,
    kind: CurveKind,
}

impl Curve {
    pub fn new(kind: CurveKind) -> Self {
        Curve {
            points: vec![],
            kind,
        }
    }

    pub fn labor(&self) -> u16 {
        match self.kind {
            CurveKind::Rectangle => 2,
        }
    }

    #[inline(always)]
    pub fn draw(curve: &Curve, builder: &mut Builder) {
        assert!(curve.points.len() == curve.labor().into());
        match curve.kind {
            CurveKind::Rectangle => {
                if let [top_left, right_bottom] = curve.points[..] {
                    builder.rectangle(top_left, get_size(top_left, right_bottom));
                }
            }
        }
    }

    fn draw_all(curves: &[Curve], frame: &mut Frame) {
        let curves = Path::new(|p| {
            for curve in curves {
                Curve::draw(curve, p);
            }
        });

        frame.stroke(&curves, Stroke::default().with_width(2.0));
    }
}