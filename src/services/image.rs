use std::sync::Arc;

use image::{ImageBuffer, Pixel, Rgba, RgbaImage};

pub async fn grayscale(image: Arc<RgbaImage>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);
        let r = if p.channels()[0] as u16 * 3 > 255 { 255 } else { p.channels()[0] * 3 };
        let g = p.channels()[1];
        let b = p.channels()[2];
        let a = p.channels()[3];
        Rgba([r, g, b, a])
    })
}