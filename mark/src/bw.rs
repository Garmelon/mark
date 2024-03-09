use image::RgbaImage;
use palette::{Hsl, Hsv, IntoColor, Lab, LinSrgb, Oklab, Srgb};

use crate::util;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Method {
    SrgbAverage,
    LinSrgbAverage,
    Hsl,
    Hsv,
    Cielab,
    Oklab,
}

impl Method {
    fn to_bw(self, pixel: Srgb) -> Srgb {
        match self {
            Self::SrgbAverage => {
                let value = (pixel.red + pixel.green + pixel.blue) / 3.0;
                Srgb::new(value, value, value)
            }
            Self::LinSrgbAverage => {
                let pixel: LinSrgb = pixel.into_color();
                let value = (pixel.red + pixel.green + pixel.blue) / 3.0;
                LinSrgb::new(value, value, value).into_color()
            }
            Self::Hsl => {
                let mut pixel: Hsl = pixel.into_color();
                pixel.saturation = 0.0;
                pixel.into_color()
            }
            Self::Hsv => {
                let mut pixel: Hsv = pixel.into_color();
                pixel.saturation = 0.0;
                pixel.into_color()
            }
            Self::Cielab => {
                let mut pixel: Lab = pixel.into_color();
                pixel.a = 0.5;
                pixel.b = 0.5;
                pixel.into_color()
            }
            Self::Oklab => {
                let mut pixel: Oklab = pixel.into_color();
                pixel.a = 0.0;
                pixel.b = 0.0;
                pixel.into_color()
            }
        }
    }
}

pub fn bw(image: &mut RgbaImage, method: Method) {
    for pixel in image.pixels_mut() {
        let srgb = util::pixel_to_srgb(*pixel);
        let srgb = method.to_bw(srgb);
        util::update_pixel_with_srgb(pixel, srgb);
    }
}
