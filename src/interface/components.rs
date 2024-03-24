use iced::{Color, Element};
use iced::widget::{button};
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