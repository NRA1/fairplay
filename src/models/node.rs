use iced::Vector;
use crate::models::modifier::Modifier;

#[derive(Debug, Clone)]
pub struct Node {
    pub modifier: Modifier,
    pub offset: Vector,
}
