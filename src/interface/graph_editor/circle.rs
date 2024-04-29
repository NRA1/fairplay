use iced::{Color, mouse, Rectangle, Renderer, Theme};
use iced::event::Status;
use iced::mouse::{Button, Cursor, Interaction};
use iced::widget::{canvas, Canvas};
use iced::widget::canvas::Event;
use iced_graphics::geometry::Path;


pub struct Circle<Message> {
    handler: Box<dyn Fn(Event) -> (Status, Option<Message>)>,
    cache: canvas::Cache,
}

impl<Message> Circle<Message> {
    pub fn new(handler: Box<dyn Fn(Event) -> (Status, Option<Message>)>) -> Circle<Message> {
        Circle {
            handler,
            cache: Default::default(),
        }
    }
}

impl<Message> canvas::Program<Message> for Circle<Message> {
    type State = ();

    fn update(&self, _state: &mut Self::State, event: Event, _bounds: Rectangle, _cursor: Cursor) -> (Status, Option<Message>) {
        match event {
            Event::Mouse(mouse) => {
                self.handler(mouse)
            },
            _ => { (Status::Ignored, None) }
        }
    }

    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<<iced::Renderer as iced_graphics::geometry::Renderer>::Geometry> {
        let circle = self.cache.draw(renderer, bounds.size(), |frame| {
            let circle = Path::circle(frame.center(), frame.width().min(frame.height()) / 2.0);

            frame.fill(&circle, Color::from_rgb(0.5, 0.5, 0.5));
        });
        vec![circle]
    }

    fn mouse_interaction(&self, _state: &Self::State, _bounds: Rectangle, _cursor: Cursor) -> Interaction {
        Interaction::Crosshair
    }
}