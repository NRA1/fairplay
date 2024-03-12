use iced::{alignment, Background, Color, Element, Length};
use iced::widget::{Container, container};
use iced_aw::{floating_element, Spinner};
use crate::fairplay::Message;

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