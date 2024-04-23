use iced::advanced::widget::{tree, Tree};
use iced::{event, advanced::layout, mouse, advanced::renderer, Element, advanced::Layout, advanced::Shell, Point, Vector, Rectangle, Padding, Border, Color, Background, Size};
use iced::advanced::Renderer as _;
use iced::border::Radius;
use iced::mouse::Cursor;
use iced::Renderer;
use iced::widget::Text;
use iced::widget::text::layout;
use crate::fairplay;

use super::editor::Event;

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
        Theme: StyleSheet,
{
    name: Element<'a, fairplay::Message, Theme, Renderer>,
    content: Element<'a, fairplay::Message, Theme, Renderer>,
    offset: Vector,
    pub edges: Vec<usize>,
    style: <Theme as StyleSheet>::Style,
}

impl<'a, Theme> Node<'a, Theme>
    where
        Theme: StyleSheet + iced::widget::text::StyleSheet + iced::widget::text::StyleSheet + 'a,
{
    pub fn new(
        name: String,
        content: impl Into<Element<'a, fairplay::Message, Theme, Renderer>>,
        offset: Vector,
        edges: Vec<usize>,
    ) -> Self {
        Self {
            name: Text::new(name).into(),
            content: content.into(),
            offset,
            edges,
            style: Default::default(),
        }
    }

    pub fn style(mut self, style: impl Into<<Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Theme> Node<'a, Theme>
    where
        Theme: StyleSheet,
{
    pub fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    pub fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    pub fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    pub fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    pub fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let padding: Padding = [20, 5, 5, 5].into();

        // let name = self.name.as_widget().layout(&mut tree.children[0], renderer, &limits);

        let content = self
            .content
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits.clone().shrink(padding));

        let node = content.size().expand(padding);

        let offset = Vector::new(padding.left, padding.top);

        layout::Node::with_children(node, vec![content.translate(offset)]).translate(self.offset)
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
        let in_bounds = cursor.is_over(bounds);

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

        let appearance = theme.appearance(self.style);

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
