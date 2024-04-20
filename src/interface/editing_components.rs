use iced::Element;
use iced::widget::{Button, checkbox, Column, Text};

use crate::fairplay::Message;
use crate::interface::components::{named_slider, ranged_named_slider};
use crate::models::modifier::{BoxBlurOptions, GrayscaleOptions, LightnessCorrectionOptions, MedianBlurOptions, Modifier, NegativeOptions, SobelOptions, ThresholdingOptions, UnsharpMaskingOptions};

pub fn modifier_options<'a>(modifier: &'a Modifier) -> Element<'a, Message> {
    let opts = match modifier {
        Modifier::Negative(opts) => { negative_modopts(opts) }
        Modifier::Thresholding(opts) => { thresholding_modopts(opts) }
        Modifier::Grayscale(opts) => { grayscale_modopts(opts) }
        Modifier::LightnessCorrection(opts) => { lightness_correction_modopts(opts) }
        Modifier::BoxBlur(opts) => { box_blur_modopts(opts) }
        Modifier::MedianBlur(opts) => { median_blur_modopts(opts) }
        Modifier::Sobel(opts) => { sobel_modopts(opts) }
        Modifier::Sharpening => { return Column::new().into() }
        Modifier::UnsharpMasking(opts) => { unsharp_masking_modopts(opts) }
    };

    let apply = Button::new("Apply")
        .on_press(Message::ModifierOptionsApplied);

    Column::new()
        .push(opts)
        .push(apply)
        .spacing(10)
        .padding(10)
        .into()
}

fn negative_modopts<'a>(opts: &NegativeOptions) -> Element<'a, Message> {
    checkbox("Grayscale", opts.grayscale).on_toggle(|v| Message::ModifierOptionsChanged(Modifier::Negative(NegativeOptions { grayscale: v }))).into()
}

fn thresholding_modopts<'a>(opts: &'a ThresholdingOptions) -> Element<'a, Message> {
    Column::new()
        .push(checkbox("Grayscale", opts.grayscale).on_toggle(|v| Message::ModifierOptionsChanged(Modifier::Thresholding(ThresholdingOptions { grayscale: v, threshold: opts.threshold }))))
        .push(named_slider("Threshold", opts.threshold, |x| Message::ModifierOptionsChanged(Modifier::Thresholding(ThresholdingOptions { grayscale: opts.grayscale, threshold: x }))))
        .into()
}

fn grayscale_modopts<'a>(opts: &'a GrayscaleOptions) -> Element<'a, Message> {
    Column::new()
        .push(Text::new("Color weights:"))
        .push(named_slider("Red", opts.red_weight, |x| Message::ModifierOptionsChanged(Modifier::Grayscale(GrayscaleOptions { red_weight: x, ..opts.clone() }))))
        .push(named_slider("Green", opts.green_weight, |x| Message::ModifierOptionsChanged(Modifier::Grayscale(GrayscaleOptions { green_weight: x, ..opts.clone() }))))
        .push(named_slider("Blue", opts.blue_weight, |x| Message::ModifierOptionsChanged(Modifier::Grayscale(GrayscaleOptions { blue_weight: x, ..opts.clone() }))))
        .into()
}

fn lightness_correction_modopts<'a>(opts: &LightnessCorrectionOptions) -> Element<'a, Message> {
    named_slider("Exponent", opts.exponent, |x| Message::ModifierOptionsChanged(Modifier::LightnessCorrection(LightnessCorrectionOptions { exponent: x }))).into()
}

fn box_blur_modopts<'a>(opts: &BoxBlurOptions) -> Element<'a, Message> {
    ranged_named_slider("Box size", 3..=25, 2, opts.size, |x| Message::ModifierOptionsChanged(Modifier::BoxBlur(BoxBlurOptions { size: x })))
}

fn median_blur_modopts<'a>(opts: &MedianBlurOptions) -> Element<'a, Message> {
    ranged_named_slider("Box size", 3..=25, 2, opts.size, |x| Message::ModifierOptionsChanged(Modifier::MedianBlur(MedianBlurOptions { size: x })))
}

fn sobel_modopts<'a>(opts: &'a SobelOptions) -> Element<'a, Message> {
    Column::new()
        .push(checkbox("Horizontal", opts.horizontal).on_toggle(|v| Message::ModifierOptionsChanged(Modifier::Sobel(SobelOptions { horizontal: v, ..opts.clone() }))))
        .push(checkbox("Vertical", opts.vertical).on_toggle(|v| Message::ModifierOptionsChanged(Modifier::Sobel(SobelOptions { vertical: v, ..opts.clone() }))))
        .into()
}

fn unsharp_masking_modopts<'a>(opts: &UnsharpMaskingOptions) -> Element<'a, Message> {
    ranged_named_slider("Box size", 3..=25, 2, opts.blur_size, |x| Message::ModifierOptionsChanged(Modifier::UnsharpMasking(UnsharpMaskingOptions { blur_size: x })))
}