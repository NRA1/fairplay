use std::io::Cursor;

use iced::{Alignment, alignment, Command, Element, Length};
use iced::widget::{Button, Column, Container, Row, Text};
use iced_aw::{BOOTSTRAP_FONT, BootstrapIcon};
use iced_aw::graphics::icons::icon_to_char;
use image::io::Reader as ImageReader;
use rfd::AsyncFileDialog;

use crate::fairplay::{Fairplay, Message};
use crate::interface::components::with_spinner;
use crate::interface::editing::EditingView;
use crate::interface::View;

#[derive(Default)]
pub struct HomeView {
    loading: bool
}

impl View for HomeView {
    fn update(app: &mut Fairplay, message: Message) -> Command<Message> {
        let Fairplay::Home(state) = app else { panic!("Invalid call") };
        match message {
            Message::OpenPicker => {
                state.loading = true;

                return Command::perform(async {
                        let file = AsyncFileDialog::new()
                            .add_filter("image", &["png", "jpg"])
                            .pick_file()
                            .await;
                        file.unwrap().read().await
                    }, |data| {
                        let img = ImageReader::new(Cursor::new(data))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap()
                            .into_rgba8();

                        Message::Open(img)
                    }
                )
            }
            Message::Open(data) => {
                *app = Fairplay::Editing(EditingView::new(data));
            }
            Message::Started => {

            }
            _ => {
                panic!("Invalid message");
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let open_btn = Button::new(
            Row::new()
                .push(Text::new(String::from(icon_to_char(BootstrapIcon::FoldertwoOpen))).font(BOOTSTRAP_FONT))
                .push(Text::new("Open"))
                .spacing(10)
        )
            .on_press(Message::OpenPicker)
            .width(Length::Fill);

        let column = Container::new(
            Column::new()
                .push(open_btn)
                .width(Length::Fixed(300_f32))
        )
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        let row = Row::new()
            .push(column)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center);

        if self.loading {
            with_spinner(row)
        } else { row.into() }
    }
}