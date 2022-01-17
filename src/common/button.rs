use crate::app::message::MessageType;
use iced::{button, Button, Length, Text};

pub fn toolbar<'a, T>(state: &'a mut button::State, text: &str) -> Button<'a, T>
where
    T: MessageType + Clone,
{
    Button::new(state, Text::new(text))
        .padding(10)
        .style(super::style::Button::Toolbar)
}

//TODO: 想办法让按钮变成一个竖长条，目前想到的是用很多个按钮，但是明显不合适。。
pub fn navigator<'a, T>(state: &'a mut button::State, text: &str) -> Button<'a, T>
where
    T: MessageType + Clone,
{
    Button::new(state, Text::new(text))
        .height(Length::Fill)
        .padding(10)
        .style(super::style::Button::Navigator)
}

pub fn entry<'a, T>(state: &'a mut button::State, text: &str) -> Button<'a, T>
where
    T: MessageType + Clone,
{
    Button::new(state, Text::new(text))
        .padding(10)
        .style(super::style::Button::PickImage)
}
