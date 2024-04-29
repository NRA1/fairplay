use iced::{Element, Length, Vector};
use iced::widget::{container, column, button, text};
use crate::fairplay::Message;
use crate::interface::{graph, graph_editor};
use crate::interface::editing_components::modifier_options;
use crate::models::modifier::Modifier;
use crate::models::node;


pub fn graph<'a>(nodes: &'a Vec<node::Node>, scaling: f32, translation: Vector) -> Element<'a, Message> {
    let nodes = nodes
        .iter()
        .map(|node| graph_editor::Node::new(format!("{}", node.modifier), modifier_options(&node.modifier), node.offset, vec!["Test1".to_string()], vec!["Test2".to_string()]))
        .collect();

    container(
        graph_editor::Editor::new(nodes, Message::Graph)
            .scaling(scaling)
            .translation(translation)
    )
        .width(Length::FillPortion(4))
        .height(Length::Fill)
        .into()
}
