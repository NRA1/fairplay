use std::sync::Arc;

use image::{ImageBuffer, Pixel, Rgba, RgbaImage};

use crate::models::modifier::{BoxBlurOptions, GrayscaleOptions, LightnessCorrectionOptions, MedianBlurOptions, Modifier, NegativeOptions, SobelOptions, ThresholdingOptions, UnsharpMaskingOptions};
use crate::services::functions::{median, pitagora};

pub async fn apply(image: Arc<RgbaImage>, modifiers: Vec<Modifier>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = RgbaImage::from_raw(image.width(), image.height(), image.to_vec()).unwrap();
    for modifier in modifiers {
        img = match modifier {
            Modifier::Negative(opts) => { negative(opts, img).await }
            Modifier::Thresholding(opts) => { thresholding(opts, img).await }
            Modifier::Grayscale(opts) => { grayscale(opts, img).await }
            Modifier::LightnessCorrection(opts) => { lightness_correction(opts, img).await }
            Modifier::BoxBlur(opts) => { box_blur(opts, &img).await }
            Modifier::MedianBlur(opts) => { median_blur(opts, img).await }
            Modifier::Sobel(opts) => { sobel(opts, img).await }
            Modifier::Sharpening => { sharpening(img).await }
            Modifier::UnsharpMasking(opts) => { unsharp_masking(opts, img).await }
        }
    }

    img
}

async fn negative(opts: NegativeOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = image;
    if opts.grayscale {
        img = grayscale(GrayscaleOptions::default(), img).await;
    }

    RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        let r = u8::MAX - p.channels()[0];
        let g = u8::MAX - p.channels()[1];
        let b = u8::MAX - p.channels()[2];
        let a = p.channels()[3];
        Rgba([r, g, b, a])
    })
}

async fn thresholding(opts: ThresholdingOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = image;
    if opts.grayscale {
        img = grayscale(GrayscaleOptions::default(), img).await;
    }

    RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        let r = if opts.threshold > p.channels()[0] { u8::MIN } else { u8::MAX };
        let g = if opts.threshold > p.channels()[1] { u8::MIN } else { u8::MAX };
        let b = if opts.threshold > p.channels()[2] { u8::MIN } else { u8::MAX };
        let a = p.channels()[3];
        Rgba([r, g, b, a])
    })
}

async fn grayscale(opts: GrayscaleOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let sum = opts.red_weight as u16 + opts.green_weight as u16 + opts.blue_weight as u16;
    let multiplier = (u8::MAX as f32 / sum as f32) / u8::MAX as f32;

    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);
        let v = opts.red_weight as f32 * multiplier * p.channels()[0] as f32 +
            opts.green_weight as f32 * multiplier * p.channels()[1] as f32 +
            opts.blue_weight as f32 * multiplier * p.channels()[2] as f32;
        let v = v.round() as u8;
        let a = p.channels()[3];
        Rgba([v, v, v, a])
    })
}

async fn lightness_correction(opts: LightnessCorrectionOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let exp = opts.exponent as f32 / ((u8::MAX as f32) / 2f32);

    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);

        let rv = (p.channels()[0] as f32).powf(exp);
        let gv = (p.channels()[1] as f32).powf(exp);
        let bv = (p.channels()[2] as f32).powf(exp);

        let r = if rv > u8::MAX as f32 { u8::MAX } else { rv.round() as u8 };
        let g = if gv > u8::MAX as f32 { u8::MAX } else { gv.round() as u8 };
        let b = if bv > u8::MAX as f32 { u8::MAX } else { bv.round() as u8 };
        let a = p.channels()[3];
        Rgba([r, g, b, a])
    })
}
async fn box_blur(opts: BoxBlurOptions, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let min = -((opts.size / 2) as i64);
    let max = (opts.size / 2) as i64;

    let width = image.width() as i64;
    let height = image.height() as i64;

    RgbaImage::from_fn(width as u32, height as u32, |x, y| {
        let x = x as i64;
        let y = y as i64;

        let mut count = 0u32;
        let mut rsum = 0u32;
        let mut gsum = 0u32;
        let mut bsum = 0u32;
        let mut asum = 0u32;
        for ix in min..=max {
            for iy in min..=max {
                if x + ix < 0 || y + iy < 0 || x + ix > width - 1 || y + iy > height - 1 {
                    continue
                }

                count += 1;
                let p = image.get_pixel((x + ix) as u32, (y + iy) as u32);
                rsum += p.channels()[0] as u32;
                gsum += p.channels()[1] as u32;
                bsum += p.channels()[2] as u32;
                asum += p.channels()[3] as u32;
            }
        }

        let r = (rsum / count) as u8;
        let g = (gsum / count) as u8;
        let b = (bsum / count) as u8;
        let a = (asum / count) as u8;
        Rgba([r, g, b, a])
    })
}

async fn median_blur(opts: MedianBlurOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {

    let min = -((opts.size / 2) as i64);
    let max = (opts.size / 2) as i64;

    let width = image.width() as i64;
    let height = image.height() as i64;

    RgbaImage::from_fn(width as u32, height as u32, |x, y| {
        let x = x as i64;
        let y = y as i64;

        let mut rvals = Vec::with_capacity((opts.size as usize).pow(2));
        let mut gvals = Vec::with_capacity((opts.size as usize).pow(2));
        let mut bvals = Vec::with_capacity((opts.size as usize).pow(2));
        let mut a = 0;

        for ix in min..=max {
            for iy in min..=max {
                if x + ix < 0 || y + iy < 0 || x + ix > width - 1 || y + iy > height - 1 {
                    continue
                }

                let p = image.get_pixel((x + ix) as u32, (y + iy) as u32);
                rvals.push(p.channels()[0]);
                gvals.push(p.channels()[1]);
                bvals.push(p.channels()[2]);

                if ix == 0 && iy == 0 {
                    a = p.channels()[3];
                }
            }
        }

        let r = median(&mut rvals);
        let g = median(&mut gvals);
        let b = median(&mut bvals);
        Rgba([r, g, b, a])
    })
}

async fn sobel(opts: SobelOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let filter_horizontal: [[i8; 3]; 3] = [
        [1i8, 0, -1],
        [2, 0, -2],
        [1, 0, -1]
    ];
    let filter_vertical: [[i8; 3]; 3] = [
        [1i8, 2, 1],
        [0, 0, 0],
        [-1, -2, -1]
    ];

    let horizontal_opt = if opts.horizontal {
        Some(apply_filter(&filter_horizontal, &image).await)
    } else { None };

    let vertical_opt = if opts.vertical {
        Some(apply_filter(&filter_vertical, &image).await)
    } else { None };

    if horizontal_opt.is_some() && vertical_opt.is_some() {
        let horizontal = horizontal_opt.unwrap();
        let vertical = vertical_opt.unwrap();

        RgbaImage::from_fn(image.width(), image.height(), |x, y| {
            let ph = horizontal.get_pixel(x, y);
            let pv = vertical.get_pixel(x, y);

            let r = pitagora(ph.channels()[0], pv.channels()[0]);
            let g = pitagora(ph.channels()[1], pv.channels()[1]);
            let b = pitagora(ph.channels()[2], pv.channels()[2]);

            let a = ph.channels()[3] as u8;

            let r = if r >= 0 { r } else { 0 };
            let g = if g >= 0 { g } else { 0 };
            let b = if b >= 0 { b } else { 0 };

            let r = if r > u8::MAX as i16 { u8::MAX } else { r as u8 };
            let g = if g > u8::MAX as i16 { u8::MAX } else { g as u8 };
            let b = if b > u8::MAX as i16 { u8::MAX } else { b as u8 };

            Rgba([r, g, b, a])
        })
    } else {
        if let Some(horizontal) = horizontal_opt {
            RgbaImage::from_fn(horizontal.width(), horizontal.height(), |x, y| {
                let p = horizontal.get_pixel(x, y);
                let r = p.channels()[0];
                let g = p.channels()[1];
                let b = p.channels()[2];
                let a = p.channels()[3] as u8;

                let r = if r >= 0 { r } else { 0 };
                let g = if g >= 0 { g } else { 0 };
                let b = if b >= 0 { b } else { 0 };

                let r = if r > u8::MAX as i16 { u8::MAX } else { r as u8 };
                let g = if g > u8::MAX as i16 { u8::MAX } else { g as u8 };
                let b = if b > u8::MAX as i16 { u8::MAX } else { b as u8 };

                Rgba([r, g, b, a])
            })
        } else if let Some(vertical) = vertical_opt {
            RgbaImage::from_fn(vertical.width(), vertical.height(), |x, y| {
                let p = vertical.get_pixel(x, y);
                let r = p.channels()[0];
                let g = p.channels()[1];
                let b = p.channels()[2];
                let a = p.channels()[3] as u8;

                let r = if r >= 0 { r } else { 0 };
                let g = if g >= 0 { g } else { 0 };
                let b = if b >= 0 { b } else { 0 };

                let r = if r > u8::MAX as i16 { u8::MAX } else { r as u8 };
                let g = if g > u8::MAX as i16 { u8::MAX } else { g as u8 };
                let b = if b > u8::MAX as i16 { u8::MAX } else { b as u8 };

                Rgba([r, g, b, a])
            })
        } else {
            image
        }
    }
}

async fn sharpening(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let filter: [[i8; 3]; 3] = [
        [1, 1, 1],
        [1, -8, 1],
        [1, 1, 1]
    ];

    let laplace = apply_filter(&filter, &image).await;
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let l = laplace.get_pixel(x, y);
        let p = image.get_pixel(x, y);

        let r = p.channels()[0] as i16 - l.channels()[0];
        let g = p.channels()[1] as i16 - l.channels()[1];
        let b = p.channels()[2] as i16 - l.channels()[2];

        let r = if r >= 0 { r } else { 0 };
        let g = if g >= 0 { g } else { 0 };
        let b = if b >= 0 { b } else { 0 };

        let r = if r > u8::MAX as i16 { u8::MAX } else { r as u8 };
        let g = if g > u8::MAX as i16 { u8::MAX } else { g as u8 };
        let b = if b > u8::MAX as i16 { u8::MAX } else { b as u8 };

        let a = p.channels()[3];

        Rgba([r, g, b, a])
    })
}

async fn unsharp_masking(opts: UnsharpMaskingOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let blur = box_blur(BoxBlurOptions { size: opts.blur_size}, &image).await;
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let b = blur.get_pixel(x, y);
        let p = image.get_pixel(x, y);

        let r = (p.channels()[0] as i16) * 2 - b.channels()[0] as i16;
        let g = (p.channels()[1] as i16) * 2 - b.channels()[1] as i16;
        let b = (p.channels()[2] as i16) * 2 - b.channels()[2] as i16;

        let r = if r >= 0 { r } else { 0 };
        let g = if g >= 0 { g } else { 0 };
        let b = if b >= 0 { b } else { 0 };

        let r = if r > u8::MAX as i16 { u8::MAX } else { r as u8 };
        let g = if g > u8::MAX as i16 { u8::MAX } else { g as u8 };
        let b = if b > u8::MAX as i16 { u8::MAX } else { b as u8 };

        let a = p.channels()[3];

        Rgba([r, g, b, a])
    })
}

async fn apply_filter(filter: &[[i8; 3]; 3], image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<i16>, Vec<i16>> {
    let width = image.width() as i64;
    let height = image.height() as i64;

    ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        let x = x as i64;
        let y = y as i64;

        let mut rsum = 0i32;
        let mut gsum = 0i32;
        let mut bsum = 0i32;
        let mut a = 0;
        for ix in (-1)..=1 {
            for iy in (-1)..=1 {
                let mut sx = ix;
                let mut sy = iy;
                if x + ix < 0 ||  x + ix > width - 1 {
                    sx = 0;
                }
                if y + iy < 0 || y + iy > height - 1 {
                    sy = 0;
                }

                let multiplier = filter[(1 + iy) as usize][(1 + ix) as usize] as i32;

                let p = image.get_pixel((x + sx) as u32, (y + sy) as u32);
                rsum += (p.channels()[0] as i32) * multiplier;
                gsum += (p.channels()[1] as i32) * multiplier;
                bsum += (p.channels()[2] as i32) * multiplier;

                if ix == 0 && iy == 0 {
                    a = p.channels()[3] as i16;
                }
            }
        }

        let r = rsum as i16;
        let g = gsum as i16;
        let b = bsum as i16;
        Rgba::<i16>([r, g, b, a])
    })
}