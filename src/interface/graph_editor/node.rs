use std::ops::Index;
use iced::advanced::widget::{tree, Tree};
use iced::{event, advanced::layout, mouse, advanced::renderer, Element, advanced::Layout, advanced::Shell, Point, Vector, Rectangle, Padding, Border, Color, Background, Size, Length, Alignment};
use iced::advanced::Renderer as _;
use iced::border::Radius;
use iced::mouse::Cursor;
use iced::Renderer;
use iced::widget::{canvas, Column, Row, Space, Text, text};
use crate::fairplay;
use crate::interface::graph_editor;
use crate::interface::graph_editor::circle::{Circle};

use super::editor::Event;

pub enum ConnectorDirection {
    In,
    Out
}

pub struct Connector {
    pub name: String,
    pub direction: ConnectorDirection,
}

#[derive(Debug)]
pub enum State {
    Idle,
    Hovered,
    Translating { started_at: Point, offset: Vector },
}

impl State {
    pub fn adjusted_bounds(&self, bounds: Rectangle) -> Rectangle {
        match self {
            State::Idle | State::Hovered => bounds,
            State::Translating { offset, .. } => bounds + *offset,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

pub struct Node<'a, Theme>
    where
        Theme: StyleSheet + iced::widget::text::StyleSheet + iced::widget::text::StyleSheet + 'a,
{
    name: Element<'a, fairplay::Message, Theme, Renderer>,
    content: Element<'a, fairplay::Message, Theme, Renderer>,
    offset: Vector,
    connectors_in: Vec<Connector>,
    connectors_out: Vec<Connector>,
    connectors_widget: Element<'a, fairplay::Message, Theme, Renderer>,
    pub edges: Vec<usize>,
    style: <Theme as StyleSheet>::Style,
}

impl<'a, Theme> Node<'a, Theme>
    where
        Theme: StyleSheet + iced::widget::text::StyleSheet + iced::widget::text::StyleSheet + 'a, iced::Element<'a, fairplay::Message, Theme, iced::Renderer>: From<iced::widget::Column<'a, fairplay::Message>>
{
    pub fn new(
        name: String,
        content: impl Into<Element<'a, fairplay::Message, Theme, Renderer>>,
        offset: Vector,
        connectors_in: Vec<String>,
        connectors_out: Vec<String>,
    ) -> Self {
        let connectors_in = connectors_in.iter().map(move |x| Connector { name: x.clone(), direction: ConnectorDirection::In }).collect();
        let connectors_out = connectors_out.iter().map(move |x| Connector { name: x.clone(), direction: ConnectorDirection::Out }).collect();

        Self {
            name: Text::new(name).size(20).into(),
            content: content.into(),
            offset,
            connectors_widget: Self::build_connectors_widget(&connectors_in, &connectors_out),
            connectors_in,
            connectors_out,
            edges: vec![],
            style: Default::default(),
        }
    }

    pub fn style(mut self, style: impl Into<<Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }

    fn build_connectors_widget(inc: &Vec<Connector>, outc: &Vec<Connector>) -> Element<'a, fairplay::Message, Theme, Renderer> {
        let mut inc_it = inc.iter();
        let mut outc_it = outc.iter();

        let in_conn = |conn: &Connector| -> Element<fairplay::Message> {
            Row::new()
                .push(canvas(Circle::new()).width(Length::Fixed(10.0)).height(Length::Fixed(10.0)))
                .push(text(conn.name.clone()))
                .spacing(10)
                .align_items(Alignment::Center)
                .into()
        };
        let out_conn = |conn: &Connector| -> Element<fairplay::Message> {
            Row::new()
                .push(text(conn.name.clone()))
                .push(canvas(Circle::new()).width(Length::Fixed(10.0)).height(Length::Fixed(10.0)))
                .spacing(10)
                .align_items(Alignment::Center)
                .into()
        };

        let mut col = Column::new();

        loop {
            let fst = inc_it.next();
            let snd = outc_it.next();

            if fst.is_none() && snd.is_none() {
                break;
            }

            col = col.push(
                Row::new()
                    .push_maybe(fst.map(|x| in_conn(x)))
                    .push(Space::new(Length::Fill, Length::Shrink))
                    .push_maybe(snd.map(|x| out_conn(x)))
            );
        }

        col.into()
    }
}

impl<'a, Theme> Node<'a, Theme>
    where
        Theme: StyleSheet + iced::widget::text::StyleSheet,
{
    pub fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    pub fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.name), Tree::new(&self.connectors_widget)]
    }

    pub fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content, &self.name, &self.connectors_widget]);
    }

    pub fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    pub fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let padding: Padding = [5, 5, 5, 5].into();

        let name = self.name.as_widget().layout(&mut tree.children[1], renderer, &limits.clone().shrink(padding));

        let content_padding: Padding = [5.0, 5.0, 5.0, 5.0].into();

        let connectors = self.connectors_widget.as_widget()
            .layout(&mut tree.children[2], renderer, &limits.clone().shrink(content_padding));

        let content = self
            .content
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits.clone().shrink(content_padding));


        let node = Size {
            width: content.size().width.max(name.size().width).max(connectors.size().width),
            height: name.size().height + content.size().height + connectors.size().height,
        } .expand(padding);

        let conn_offset = Vector::new(padding.left, name.size().height + content_padding.top);
        let cont_offset = Vector::new(padding.left, name.size().height + content_padding.top + connectors.size().height);

        layout::Node::with_children(node, vec![content.translate(cont_offset), name.translate(Vector::new(15.0, 5.0)), connectors.translate(conn_offset)]).translate(self.offset)
    }

    pub fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut Shell<'_, fairplay::Message>,
        viewport: &Rectangle,
        index: usize,
        on_event: &dyn Fn(Event) -> fairplay::Message,
    ) -> event::Status {
        let bounds = layout.bounds();
        let connectors = layout.children().nth(2).expect("Index out of range").bounds();
        let content = layout.children().nth(0).expect("Index out of range").bounds();
        let in_bounds = cursor.is_over(bounds) && !cursor.is_over(connectors) && !cursor.is_over(content);

        let state = tree.state.downcast_mut::<State>();

        if let State::Translating { started_at, offset } = state {
            if let iced::Event::Mouse(event) = event {
                match event {
                    mouse::Event::CursorMoved { .. } => {
                        *offset = cursor.position().expect("Cursor not set") - *started_at;
                        return event::Status::Captured;
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        shell.publish((on_event)(Event::NodeMoved {
                            index,
                            offset: self.offset + *offset,
                        }));
                        *state = in_bounds.then_some(State::Hovered).unwrap_or(State::Idle);
                    }
                    mouse::Event::CursorLeft => {
                        shell.publish((on_event)(Event::NodeMoved {
                            index,
                            offset: self.offset + *offset,
                        }));
                        *state = State::Idle;
                    }
                    _ => {}
                }
            }

            event::Status::Ignored
        } else {
            let status = self.content.as_widget_mut().on_event(
                tree.children.first_mut().unwrap(),
                event.clone(),
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport
            );

            if matches!(status, event::Status::Ignored) {
                if let iced::Event::Mouse(event) = event {
                    match event {
                        mouse::Event::CursorMoved { .. }
                        if in_bounds && matches!(*state, State::Idle) =>
                            {
                                *state = State::Hovered;
                                return event::Status::Captured;
                            }
                        mouse::Event::CursorMoved { .. }
                        if !in_bounds && matches!(*state, State::Hovered) =>
                            {
                                *state = State::Idle;
                                return event::Status::Captured;
                            }
                        mouse::Event::ButtonPressed(mouse::Button::Left)
                        if matches!(*state, State::Hovered) =>
                            {
                                *state = State::Translating {
                                    started_at: cursor.position().expect("Cursor not set"),
                                    offset: Vector::default(),
                                };
                                return event::Status::Captured;
                            }
                        _ => {}
                    }
                }

                event::Status::Ignored
            } else {
                status
            }
        }
    }

    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let appearance = StyleSheet::appearance(theme, self.style);

        let draw = |renderer: &mut Renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        color: appearance.border_color,
                        width: appearance.border_width,
                        radius: Radius::from(appearance.border_radius),
                    },
                    shadow: Default::default(),
                },
                appearance
                    .background
                    .unwrap_or_else(|| Color::TRANSPARENT.into()),
            );

            self.name.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                &renderer::Style {
                    text_color: appearance.text_color.unwrap_or(style.text_color),
                },
                layout.children().nth(1).unwrap(),
                cursor,
                viewport
            );

            self.connectors_widget.as_widget().draw(
                &tree.children[2],
                renderer,
                theme,
                &renderer::Style {
                    text_color: appearance.text_color.unwrap_or(style.text_color),
                },
                layout.children().nth(2).unwrap(),
                cursor,
                viewport
            );

            self.content.as_widget().draw(
                tree.children.first().unwrap(),
                // tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: appearance.text_color.unwrap_or(style.text_color),
                },
                layout.children().next().unwrap(),
                cursor,
                viewport
            )
        };

        if let State::Translating { offset, .. } = state {
            renderer.with_translation(*offset, |renderer| {
                draw(renderer);
            });
        } else {
            draw(renderer);
        }
    }

    pub fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        match state {
            State::Idle => mouse::Interaction::default(),
            State::Hovered => mouse::Interaction::Grab,
            State::Translating { .. } => mouse::Interaction::Grabbing,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}
