use std::sync::{Arc, Mutex};
use iced::{Alignment, Background, Color, Command, Element, Length};
use iced::widget::{button, Button, Column, combo_box, Container, container, pick_list, Row, Text};
use iced::widget::image::Handle as ImageHandle;
use iced_aw::{BOOTSTRAP_FONT, BootstrapIcon};
use iced_aw::graphics::icons::icon_to_char;
use image::{RgbaImage};

use crate::fairplay::{Fairplay, Message};
use crate::interface::components::{TransparentButtonStyle, with_spinner};
use crate::interface::View;
use crate::models::modifier::Modifier;
use crate::services;

pub struct EditingView {
    pub(crate) image: Arc<RgbaImage>,
    pub(crate) handle: ImageHandle,

    pub(crate) loading: bool,
    pub(crate) modifiers: Vec<Modifier>,
}

impl View for EditingView {
    fn update(app: &mut Fairplay, message: Message) -> Command<Message> {
        let Fairplay::Editing(state) = app else { panic!("Invalid call!") };
        match message {
            Message::ModifierAdded(modifier) => {
                state.loading = true;
                state.modifiers.push(modifier);
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ModifierRemoved(idx) => {
                state.loading = true;
                state.modifiers.remove(idx);
                return Command::perform(services::image::apply(state.image.clone(), state.modifiers.clone()), |r| Message::ImageModified(r));
            }
            Message::ImageModified(image) => {
                state.handle = ImageHandle::from_pixels(image.width(), image.height(), image.into_vec());
                state.loading = false;
            }
            _ => { panic!("Invalid message") }
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let dropdown = pick_list(
            vec![
                Modifier::Grayscale
            ],
            None::<Modifier>,
            |modifier: Modifier| {
                Message::ModifierAdded(modifier)
            }
        )
            .placeholder("Add a modifier")
            .width(Length::Fill);

        let mut modifiers = Column::new();
        for (i, modifier) in self.modifiers.iter().enumerate() {
            modifiers = modifiers.push(
                Row::new()
                    .push(
                        Text::new(format!("{}", modifier))
                            .width(Length::Fill)
                    ).push(
                        Button::new(Text::new(String::from(icon_to_char(BootstrapIcon::X))).font(BOOTSTRAP_FONT))
                            .on_press(Message::ModifierRemoved(i))
                            .style(iced::theme::Button::Custom(Box::new(TransparentButtonStyle)))
                    )
                    .align_items(Alignment::Center)
            );
        }

        let image = iced::widget::image::viewer(self.handle.clone())
            .max_scale(0.5)
            .width(Length::FillPortion(4))
            .height(Length::Fill);



        let panel = Container::new(
            Column::new()
                .push(dropdown)
                .push(modifiers)
                .spacing(10)
                .width(Length::FillPortion(1))
                .padding(10)
                .align_items(Alignment::Start)
                .height(Length::Fill)
        ).style(container::Appearance::default()
            .with_background(Background::from(Color::new(0.0, 0.0, 0.0, 0.3)))
        );

        let row = Row::new()
            .push(image)
            .push(panel)
            .into();

        if self.loading {
            with_spinner(row)
        } else {
            row
        }
    }
}