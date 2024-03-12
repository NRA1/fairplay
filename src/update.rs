
use iced::{Application, Command};

use crate::interface::View;
use crate::fairplay::{Fairplay, Message};
use crate::interface::editing::EditingView;
use crate::interface::home::HomeView;

pub fn update(app: &mut Fairplay, message: Message) -> Command<Message> {
    match app {
        Fairplay::Home(_) => { HomeView::update(app, message) }
        Fairplay::Editing(_) => { EditingView::update(app, message) }
    }
}
