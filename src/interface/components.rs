use std::ops::RangeInclusive;
use iced::{Color, Element};
use iced::widget::{button, Row, slider, Text};
use iced::widget::button::Appearance;
#[cfg(not(target_arch = "wasm32"))]
use iced_aw::{floating_element, Spinner};
#[cfg(not(target_arch = "wasm32"))]
use iced::widget::{Container, container};
#[cfg(not(target_arch = "wasm32"))]
use iced::{alignment, Background, Length};
use crate::fairplay::Message;

#[cfg(not(target_arch = "wasm32"))]
pub fn with_spinner<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    floating_element(
        content,
        Container::new(
            Spinner::new()
                .width(Length::Fixed(100_f32))
                .height(Length::Fixed(100_f32))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(container::Appearance::default()
                .with_background(Background::from(Color::new(0.0, 0.0, 0.0, 0.3)))
            )
    ).into()
}

#[cfg(target_arch = "wasm32")]
pub fn with_spinner<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    content.into()
}

pub fn named_slider<'a>(name: &'a str, val: u8, on_change: impl Fn(u8) -> Message + 'a) -> Element<'a, Message> {
    ranged_named_slider(name, u8::MIN..=u8::MAX, 1, val, on_change)
}

pub fn ranged_named_slider<'a>(name: &'a str, range: RangeInclusive<u8>, step: u8, val: u8, on_change: impl Fn(u8) -> Message + 'a) -> Element<'a, Message> {
    Row::new()
        .push(Text::new(name))
        .push(slider(range, val, on_change).step(step))
        .push(Text::new(val.to_string()))
        .spacing(10)
        .into()
}
#[derive(Default)]
pub struct TransparentButtonStyle;

impl button::StyleSheet for TransparentButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: None,
            text_color: Color::WHITE,
            ..Appearance::default()
        }
    }
}

#[derive(Default)]
pub struct SelectedButtonStyle;

impl button::StyleSheet for SelectedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(Color::new(1f32, 1f32, 1f32, 0.3))),
            text_color: Color::WHITE,
            ..Appearance::default()
        }
    }
}