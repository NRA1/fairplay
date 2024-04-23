use iced::{Element, Length, Vector};
use iced::widget::{container, column, button, text};
use crate::fairplay::Message;
use crate::interface::{graph, graph_editor};
use crate::models::node;


pub fn graph<'a>(nodes: &Vec<node::Node>, scaling: f32, translation: Vector) -> Element<'a, Message> {
    let node_content = |kind: node::Kind| -> Element<'a, Message> {
        match kind {
            node::Kind::A => text("Node A").into(),
            node::Kind::B => column![text("Node B"), text("Some description...")]
                .spacing(5)
                .into(),
            node::Kind::C => column![
                    text("Node C"),
                ]
                .spacing(5)
                .into(),
            node::Kind::D => column![
                    text("Node D"),
                ]
                .spacing(5)
                .into(),
        }
    };

    let nodes = nodes
        .iter()
        .map(|node| graph_editor::Node::new(node_content(node.kind), node.offset, node.edges.clone()))
        .collect();

    container(
        graph_editor::Editor::new(nodes, Message::Graph)
            .scaling(scaling)
            .translation(translation)
    )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
