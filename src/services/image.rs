use std::sync::Arc;

use image::{ImageBuffer, Pixel, Rgba, RgbaImage};
use crate::interface::histogram::Histogram;

use crate::models::modifier::{BoxBlurOptions, ChannelOptions, GaussianBlurOptions, GrayscaleOptions, LightnessCorrectionOptions, MedianBlurOptions, Modifier, NegativeOptions, SobelOptions, ThresholdingOptions, UnsharpMaskingOptions};
use crate::services::functions::{median, pitagora};

pub async fn apply(image: Arc<RgbaImage>, modifiers: Vec<Modifier>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = RgbaImage::from_raw(image.width(), image.height(), image.to_vec()).unwrap();
    for modifier in modifiers {
        img = match modifier {
            Modifier::Negative(opts) => { negative(opts, img).await }
            Modifier::Thresholding(opts) => { thresholding(opts, img).await }
            Modifier::Grayscale(opts) => { grayscale(opts, img).await }
            Modifier::Channels(opts) => { channels(opts, img).await }
            Modifier::LightnessCorrection(opts) => { lightness_correction(opts, img).await }
            Modifier::BoxBlur(opts) => { box_blur(opts, &img).await }
            Modifier::GaussianBlur(opts) => { gaussian_blur(opts, img).await },
            Modifier::MedianBlur(opts) => { median_blur(opts, img).await }
            Modifier::Sobel(opts) => { sobel(opts, img).await }
            Modifier::Laplace => { laplace(img).await }
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

async fn channels(opts: ChannelOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);
        let r = if opts.red_enabled {
            let v = ((p.channels()[0] as f32) * (opts.red_weight as f32 / 100.0)).round() as u16;
            if v > u8::MAX as u16 { u8::MAX } else { v as u8 }
        } else { 0 };
        let g = if opts.green_enabled {
            let v = ((p.channels()[1] as f32) * (opts.green_weight as f32 / 100.0)).round() as u16;
            if v > u8::MAX as u16 { u8::MAX } else { v as u8 }
        } else { 0 };
        let b = if opts.blue_enabled {
            let v = ((p.channels()[2] as f32) * (opts.blue_weight as f32 / 100.0)).round() as u16;
            if v > u8::MAX as u16 { u8::MAX } else { v as u8 }
        } else { 0 };
        let a = p.channels()[3];
        Rgba([r, g, b, a])
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

async fn gaussian_blur(opts: GaussianBlurOptions, image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let o = (opts.size as f32) / 6f32;

    let min = -((opts.size / 2) as i64);
    let max = (opts.size / 2) as i64;

    let mut filter = Vec::with_capacity(opts.size as usize);

    for x in min..=max {
        let mut vec = Vec::with_capacity(opts.size as usize);

        for y in min..=max {
            let topow2 = 2f32 * o.powi(2);
            let exponent = -(((x.pow(2) + y.pow(2)) as f32) / topow2);
            let multiplier = 1f32 / (topow2 * std::f32::consts::PI);
            let val = multiplier * std::f32::consts::E.powf(exponent);
            vec.push(val)
        }

        filter.push(vec);
    }

    let gaussian = apply_filter(&filter, &image).await;

    RgbaImage::from_fn(gaussian.width(), gaussian.height(), |x, y| {
        let p = gaussian.get_pixel(x, y);
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
    let filter_horizontal = [
        [1.0, 0.0, -1.0].to_vec(),
        [2.0, 0.0, -2.0].to_vec(),
        [1.0, 0.0, -1.0].to_vec()
    ].to_vec();
    let filter_vertical = [
        [1.0, 2.0, 1.0].to_vec(),
        [0.0, 0.0, 0.0].to_vec(),
        [-1.0, -2.0, -1.0].to_vec()
    ].to_vec();

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

async fn laplace(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let filter = [
        [1.0, 1.0, 1.0].to_vec(),
        [1.0, -8.0, 1.0].to_vec(),
        [1.0, 1.0, 1.0].to_vec()
    ].to_vec();

    let laplace = apply_filter(&filter, &image).await;

    RgbaImage::from_fn(laplace.width(), laplace.height(), |x, y| {
        let p = laplace.get_pixel(x, y);
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
}

async fn sharpening(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let filter = [
        [1.0, 1.0, 1.0].to_vec(),
        [1.0, -8.0, 1.0].to_vec(),
        [1.0, 1.0, 1.0].to_vec()
    ].to_vec();

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

async fn apply_filter(filter: &Vec<Vec<f32>>, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<i16>, Vec<i16>> {
    let width = image.width() as i64;
    let height = image.height() as i64;

    ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        let x = x as i64;
        let y = y as i64;

        let min = -(filter.len() as i64 / 2);
        let max = filter.len() as i64 / 2;

        let mut rsum = 0f32;
        let mut gsum = 0f32;
        let mut bsum = 0f32;
        let mut a = 0;
        for ix in min..=max {
            for iy in min..=max {
                let mut sx = ix;
                let mut sy = iy;
                if x + ix < 0 ||  x + ix > width - 1 {
                    sx = 0;
                }
                if y + iy < 0 || y + iy > height - 1 {
                    sy = 0;
                }

                let multiplier = filter[(max + iy) as usize][(max + ix) as usize];

                let p = image.get_pixel((x + sx) as u32, (y + sy) as u32);
                rsum += (p.channels()[0] as f32) * multiplier;
                gsum += (p.channels()[1] as f32) * multiplier;
                bsum += (p.channels()[2] as f32) * multiplier;

                if ix == 0 && iy == 0 {
                    a = p.channels()[3] as i16;
                }
            }
        }

        let r = rsum.round() as i16;
        let g = gsum.round() as i16;
        let b = bsum.round() as i16;
        Rgba::<i16>([r, g, b, a])
    })
}

pub async fn histogram(image: RgbaImage) -> Histogram {
    let mut lightness = vec![0; 32];
    let mut red = vec![0; 32];
    let mut green = vec![0; 32];
    let mut blue = vec![0; 32];
    let step = (u8::MAX as u16 + 1) as f32 / 32.0;

    image.pixels().for_each(|p| {
        let r = p.channels()[0];
        red[(r as f32 / step) as usize] += 1;
        let g = p.channels()[1];
        green[(g as f32 / step) as usize] += 1;
        let b = p.channels()[2];
        blue[(b as f32 / step) as usize] += 1;
        let l = r as f32 + g as f32 + b as f32;
        lightness[(l / (step * 3.0)) as usize] += 1;
    });

    Histogram {
        lightness,
        red,
        green,
        blue
    }
}