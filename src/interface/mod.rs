use iced::{Command, Element};
use crate::fairplay::{Fairplay, Message};

pub mod home;
pub mod editing;
mod components;
mod editing_components;
pub mod graph_editor;
mod graph;

pub trait View {
    fn update(app: &mut Fairplay, message: Message) -> Command<Message>;
    fn view(&self) -> Element<Message>;
}
