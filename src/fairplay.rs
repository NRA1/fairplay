use iced::{Application, Command, Element, executor, font, Sandbox, Theme};
use image::{ImageBuffer, Rgba, RgbaImage};

use crate::{update, view};
use crate::interface::editing::EditingView;
use crate::interface::home::HomeView;

pub enum Fairplay {
    Home(HomeView),
    Editing(EditingView)
}

#[derive(Debug, Clone)]
pub enum Message {
    Started,
    OpenPicker,
    Open(RgbaImage),
    Grayscale,
    Loaded,
    ImageModified(ImageBuffer<Rgba<u8>, Vec<u8>>)
}

impl Application for Fairplay {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::Home(HomeView::default()), font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(|x| Message::Started))
    }

    fn title(&self) -> String {
        String::from("Fairplay")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        update::update(self, message)
    }

    fn view(&self) -> Element<Self::Message> {
        view::view(self)
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
