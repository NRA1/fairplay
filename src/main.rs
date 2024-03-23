use iced::{Application, Settings};
use crate::fairplay::Fairplay;

mod interface;
mod fairplay;
mod update;
mod view;
mod services;
mod models;


pub fn main() -> iced::Result {
    Fairplay::run(Settings::default())
}
