use std::sync::Arc;

use image::{ImageBuffer, Pixel, Rgba, RgbaImage};

use crate::models::modifier::Modifier;

pub async fn apply(image: Arc<RgbaImage>, modifiers: Vec<Modifier>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = RgbaImage::from_raw(image.width(), image.height(), image.to_vec()).unwrap();
    for modifier in modifiers {
        img = match modifier {
            Modifier::Grayscale => {
                grayscale(img).await
            }
        }
    }

    img
}

async fn grayscale(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);
        let r = if p.channels()[0] as u16 * 3 > 255 { 255 } else { p.channels()[0] * 3 };
        let g = p.channels()[1];
        let b = p.channels()[2];
        let a = p.channels()[3];
        Rgba([r, g, b, a])
    })
}