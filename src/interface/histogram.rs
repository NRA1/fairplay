use iced::{Color, Point, Rectangle, Renderer, Size, Theme};
use iced::mouse::Cursor;
use iced::widget::{canvas, Canvas};
use iced::widget::canvas::{Cache, Geometry};

pub fn histogram<Message>(data: Vec<u32>, color: Color) -> Canvas<HistogramProgram, Message> {
    canvas(HistogramProgram::new(data, color))
}

#[derive(Default, Clone, Debug)]
pub struct Histogram {
    pub(crate) lightness: Vec<u32>,
    pub(crate) red: Vec<u32>,
    pub(crate) green: Vec<u32>,
    pub(crate) blue: Vec<u32>
}

pub struct HistogramProgram {
    data: Vec<u32>,
    cache: Cache,
    color: Color
}

impl HistogramProgram {
    pub fn new(data: Vec<u32>, color: Color) -> HistogramProgram {
        HistogramProgram {
            data,
            cache: Default::default(),
            color
        }
    }
}

impl<Message> canvas::Program<Message> for HistogramProgram {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let histogram = self.cache.draw(renderer, bounds.size(), |frame| {
            let width = bounds.width / self.data.len() as f32;
            let max_val = self.data.iter().max();
            let multiplier = if let Some(max_val) = max_val { bounds.height as f64 / *max_val as f64 } else { return; };

            for (idx, val) in self.data.iter().enumerate() {
                let height = ((*val as f64) * multiplier) as f32;
                let x = width * idx as f32;
                let y = bounds.height - height;
                frame.fill_rectangle(Point::new(x, y), Size { width, height }, self.color)
            }
        });
        vec![histogram]
    }
}