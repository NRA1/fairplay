use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Negative(NegativeOptions),
    Thresholding(ThresholdingOptions),
    Grayscale(GrayscaleOptions),
    LightnessCorrection(LightnessCorrectionOptions),
    BoxBlur(BoxBlurOptions),
    MedianBlur(MedianBlurOptions),
    Sobel(SobelOptions),
    Sharpening,
    UnsharpMasking(UnsharpMaskingOptions)
}

impl Display for Modifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                Modifier::Grayscale(_) => { "Grayscale" }
                Modifier::Negative(_) => { "Negative" }
                Modifier::Thresholding(_) => { "Thresholding" }
                Modifier::LightnessCorrection(_) => { "Lightness correction" }
                Modifier::BoxBlur(_) => { "Box blur" }
                Modifier::MedianBlur(_) => { "Median blur" }
                Modifier::Sobel(_) => { "Sobel" }
                Modifier::Sharpening => { "Sharpening" }
                Modifier::UnsharpMasking(_) => { "Unsharp masking" }
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NegativeOptions {
    pub grayscale: bool
}

impl Default for NegativeOptions {
    fn default() -> Self {
        NegativeOptions {
            grayscale: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThresholdingOptions {
    pub grayscale: bool,
    pub threshold: u8
}

impl Default for ThresholdingOptions {
    fn default() -> Self {
        ThresholdingOptions {
            grayscale: false,
            threshold: u8::MAX / 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrayscaleOptions {
    pub red_weight: u8,
    pub blue_weight: u8,
    pub green_weight: u8
}

impl Default for GrayscaleOptions {
    fn default() -> Self {
        GrayscaleOptions {
            red_weight: 72,
            green_weight: 149,
            blue_weight: 34,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LightnessCorrectionOptions {
    pub exponent: u8
}

impl Default for LightnessCorrectionOptions {
    fn default() -> Self {
        LightnessCorrectionOptions {
            exponent: u8::MAX / 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoxBlurOptions {
    pub size: u8
}

impl Default for BoxBlurOptions {
    fn default() -> Self {
        BoxBlurOptions {
            size: 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MedianBlurOptions {
    pub size: u8
}

impl Default for MedianBlurOptions {
    fn default() -> Self {
        MedianBlurOptions {
            size: 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SobelOptions {
    pub horizontal: bool,
    pub vertical: bool
}

impl Default for SobelOptions {
    fn default() -> Self {
        SobelOptions {
            horizontal: true,
            vertical: true
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnsharpMaskingOptions {
    pub blur_size: u8
}

impl Default for UnsharpMaskingOptions {
    fn default() -> Self {
        UnsharpMaskingOptions {
            blur_size: 3,
        }
    }
}