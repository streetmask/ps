use std::{cell::RefCell, rc::Rc};

use iced::{
    button::State as ButtonState,
    canvas::event::{self, Event},
    canvas::{self, Canvas as IcedCanvas, Cursor, Frame, Geometry, Path, Stroke},
    mouse, Alignment, Button, Color, Column, Command, Element, Length, Point, Rectangle, Row, Text,
};

use svg::node::element::path::Data;
use svg::node::element::Path as SvgPath;
use svg::Document;

use super::Component;
use crate::app::{file_dialog::save as save_file, message::CurvesMessage};
use crate::app::{message::CanvasMessage, Flags};
use crate::app::{utils::get_size, UserSettings};

pub struct Canvas {
    pub pending: Pending,
    curves: Vec<Curve>,
    clear_button: ButtonState,
    save_button: ButtonState,
    back_button: ButtonState,
    selected_curve: Option<usize>,
}

impl Component for Canvas {
    type Message = CanvasMessage;

    fn new(_flags: &mut Flags) -> (Self, Command<Self::Message>) {
        (
            Canvas {
                pending: Pending::new(),
                curves: vec![],
                clear_button: ButtonState::new(),
                save_button: ButtonState::new(),
                back_button: ButtonState::new(),
                selected_curve: None,
            },
            Command::none(),
        )
    }

    fn update(
        &mut self,
        message: CanvasMessage,
        _settings: Rc<RefCell<UserSettings>>,
    ) -> Command<CanvasMessage> {
        match message {
            CanvasMessage::CurvesMessage(cm) => match cm {
                CurvesMessage::AddCurve(curve) => {
                    self.curves.push(curve);
                }
                CurvesMessage::SelectCurve(i) => {
                    if self.selected_curve == i {
                        self.selected_curve = None;
                    } else {
                        self.selected_curve = i
                    }
                }
            },
            CanvasMessage::Clear => {
                self.pending.curve.points.clear();
                self.curves.clear();
            }
            CanvasMessage::Save => Curves::save(&self.curves),
            _ => {}
        }
        self.pending.request_redraw();
        Command::none()
    }

    fn view(&mut self, _settings: Rc<RefCell<UserSettings>>) -> Element<CanvasMessage> {
        Column::new()
            .width(Length::FillPortion(5))
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(
                self.pending
                    .view(&self.curves, &self.selected_curve)
                    .map(CanvasMessage::CurvesMessage),
            )
            .push(
                Row::new()
                    .push(
                        Button::new(&mut self.back_button, Text::new("Back"))
                            .padding(8)
                            .on_press(CanvasMessage::Back),
                    )
                    .push(
                        Button::new(&mut self.clear_button, Text::new("Clear"))
                            .padding(8)
                            .on_press(CanvasMessage::Clear),
                    )
                    .push(
                        Button::new(&mut self.save_button, Text::new("Save"))
                            .padding(8)
                            .on_press(CanvasMessage::Save),
                    ),
            )
            .into()
    }
}

pub struct Pending {
    curve: Curve,
    cache: canvas::Cache,
}

impl Pending {
    pub fn new() -> Self {
        Pending {
            curve: Curve::new(ShapeKind::Rectangle, Color::BLACK, 2.0),
            cache: canvas::Cache::new(),
        }
    }

    pub fn update(&mut self, new: Point) -> Option<Curve> {
        let labor: usize = self.curve.labor().into();
        self.curve.points.push(new);
        if self.curve.points.len() == labor {
            Some(Curve {
                points: self.curve.points.drain(..).collect(),
                ..self.curve
            })
        } else {
            None
        }
    }

    pub fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Geometry {
        let mut frame = Frame::new(bounds.size());

        if let Some(cursor_position) = cursor.position_in(&bounds) {
            let path = Path::new(|p| match self.curve.kind {
                ShapeKind::Rectangle => {
                    if self.curve.points.len() == 1 {
                        let top_left = self.curve.points[0];
                        let right_bottom = cursor_position;
                        p.rectangle(top_left, get_size(top_left, right_bottom));
                    }
                }
                ShapeKind::Triangle => {
                    let len = self.curve.points.len();
                    if len == 1 {
                        p.move_to(self.curve.points[0]);
                        p.line_to(cursor_position);
                    } else if len == 2 {
                        p.move_to(self.curve.points[0]);
                        p.line_to(self.curve.points[1]);
                        p.line_to(cursor_position);
                        p.line_to(self.curve.points[0]);
                    }
                }
            });
            frame.stroke(
                &path,
                Stroke {
                    width: self.curve.width,
                    color: self.curve.color,
                    ..Stroke::default()
                },
            )
        }

        frame.into_geometry()
    }

    pub fn view<'a>(
        &'a mut self,
        curves: &'a [Curve],
        selected: &'a Option<usize>,
    ) -> Element<'a, CurvesMessage> {
        IcedCanvas::new(Curves {
            pending: self,
            curves,
            selected,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }

    pub fn change_shape(&mut self, s: ShapeKind) {
        self.curve.kind = s;
    }
}

struct Curves<'a> {
    pending: &'a mut Pending,
    curves: &'a [Curve],
    selected: &'a Option<usize>,
}

impl<'a> canvas::Program<CurvesMessage> for Curves<'a> {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<CurvesMessage>) {
        let cursor_position = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        //如果pending是空的，则判定是否落在已有curve的points上
                        if self.pending.curve.points.is_empty() {
                            for (index, curve) in self.curves.iter().enumerate() {
                                for point in curve.points.iter() {
                                    if cursor_position.distance(*point) < 5.0 {
                                        return (
                                            event::Status::Captured,
                                            Some(CurvesMessage::SelectCurve(Some(index))),
                                        );
                                    }
                                }
                            }
                        }
                        if let Some(curve) = self.pending.update(cursor_position) {
                            return (
                                event::Status::Captured,
                                Some(CurvesMessage::AddCurve(curve)),
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        (event::Status::Ignored, None)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        dbg!(self.selected);
        let content = self.pending.cache.draw(bounds.size(), |frame: &mut Frame| {
            if let Some(selected) = self.selected {
                self.curves.iter().enumerate().for_each(|(index, curve)| {
                    if index == *selected {
                        Curve::draw(curve, frame, true);
                    } else {
                        Curve::draw(curve, frame, false);
                    }
                });
            } else {
                self.curves
                    .iter()
                    .for_each(|curve| Curve::draw(curve, frame, false))
            }

            frame.stroke(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                Stroke::default(),
            );
        });

        let pending_curve = self.pending.draw(bounds, cursor);

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

impl<'a> Curves<'a> {
    fn save(curves: &'a [Curve]) {
        if let Some(pathbuf) = save_file() {
            let data = curves.iter().fold(Data::new(), |acc, x| x.save(acc));

            let path = SvgPath::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 3)
                .set("d", data.close());

            let document = Document::new().add(path);

            svg::save(pathbuf, &document).unwrap();
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeKind {
    #[default]
    Rectangle,
    Triangle,
}

#[derive(Debug, Clone)]
pub struct Curve {
    points: Vec<Point>,
    kind: ShapeKind,
    color: Color,
    width: f32,
}

impl Curve {
    pub fn new(kind: ShapeKind, color: Color, width: f32) -> Self {
        Curve {
            points: vec![],
            kind,
            color,
            width,
        }
    }

    pub fn labor(&self) -> u16 {
        match self.kind {
            ShapeKind::Rectangle => 2,
            ShapeKind::Triangle => 3,
        }
    }

    #[inline(always)]
    pub fn draw(curve: &Curve, frame: &mut Frame, selected: bool) {
        assert!(curve.points.len() == curve.labor().into());
        let path = Path::new(|builder| match curve.kind {
            ShapeKind::Rectangle => {
                if let [top_left, right_bottom] = curve.points[..] {
                    builder.rectangle(top_left, get_size(top_left, right_bottom));
                }
            }
            ShapeKind::Triangle => {
                if let [a, b, c] = curve.points[..] {
                    builder.move_to(a);
                    builder.line_to(b);
                    builder.line_to(c);
                    builder.line_to(a);
                }
            }
        });
        frame.stroke(
            &path,
            Stroke {
                width: curve.width,
                color: curve.color,
                ..Stroke::default()
            },
        );

        if selected {
            let selection_highlight = Path::new(|b| {
                for point in curve.points.iter() {
                    b.circle(*point, 8.0);
                }
            });

            frame.stroke(
                &selection_highlight,
                Stroke {
                    width: 1.0,
                    color: Color::from_rgb(255.0, 255.0, 0.0),
                    ..Stroke::default()
                },
            );
        }
    }

    #[inline(always)]
    pub fn save(&self, data: Data) -> Data {
        match self.kind {
            ShapeKind::Rectangle => {
                assert!(self.points.len() == 2);
                let Point { x: x1, y: y1 } = self.points[0];
                let Point { x: x2, y: y2 } = self.points[1];
                data.move_to((x1, y1))
                    .line_to((x2, y1))
                    .line_to((x2, y2))
                    .line_to((x1, y2))
                    .line_to((x1, y1))
            }
            ShapeKind::Triangle => {
                assert!(self.points.len() == 3);
                let Point { x: ax, y: ay } = self.points[0];
                let Point { x: bx, y: by } = self.points[1];
                let Point { x: cx, y: cy } = self.points[2];
                data.move_to((ax, ay))
                    .line_to((bx, by))
                    .line_to((cx, cy))
                    .line_to((ax, ay))
            }
        }
    }
}
