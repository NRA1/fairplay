use std::sync::Arc;
use iced::{Alignment, Command, Element, Length};
use iced::widget::{button, Column, Row};
use iced::widget::image::Handle as ImageHandle;
use image::{RgbaImage};

use crate::fairplay::{Fairplay, Message};
use crate::interface::components::with_spinner;
use crate::interface::View;
use crate::services;

pub struct EditingView {
    pub(crate) image: Arc<RgbaImage>,
    pub(crate) handle: ImageHandle,

    pub(crate) loading: bool
}

impl View for EditingView {
    fn update(app: &mut Fairplay, message: Message) -> Command<Message> {
        let Fairplay::Editing(state) = app else { panic!("Invalid call!") };
        match message {
            Message::Grayscale => {
                state.loading = true;
                return Command::perform(services::image::grayscale(state.image.clone()), |r| Message::ImageModified(r));
            }
            Message::ImageModified(image) => {
                state.handle = ImageHandle::from_pixels(image.width(), image.height(), image.to_vec());
                state.image = Arc::new(image);
                state.loading = false;
            }
            _ => { panic!("Invalid message") }
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let grayscale = button("Grayscale")
            .on_press(Message::Grayscale)
            .width(Length::Fill);
        let image = iced::widget::image(self.handle.clone());


        let img_container = Row::new()
            .push(image)
            .width(Length::FillPortion(4))
            .align_items(Alignment::Center)
            .height(Length::Fill);

        let panel = Column::new()
            .push(grayscale)
            .spacing(10)
            .width(Length::FillPortion(1))
            .align_items(Alignment::Start)
            .height(Length::Fill);

        let row = Row::new()
            .push(img_container)
            .push(panel)
            .padding(20)
            .spacing(20)
            .into();

        if self.loading {
            with_spinner(row)
        } else {
            row
        }
    }
}