use iced::Element;
use crate::fairplay::{Fairplay, Message};
use crate::interface::View;

pub fn view(app: &Fairplay) -> Element<Message> {
    match app {
        Fairplay::Home(view) => { view.view() }
        Fairplay::Editing(view) => { view.view() }
    }
}
