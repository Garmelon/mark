use image::RgbaImage;
use palette::{Hsl, IntoColor, Lab, LinSrgb, Oklab, Srgb};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BwMethod {
    SrgbAverage,
    LinSrgbAverage,
    Hsl,
    Cielab,
    Oklab,
}

impl BwMethod {
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

pub fn bw(image: &mut RgbaImage, method: BwMethod) {
    for pixel in image.pixels_mut() {
        let [r, g, b, _] = pixel.0;
        let srgb = Srgb::new(r, g, b).into_format::<f32>();
        let srgb = method.to_bw(srgb).into_format::<u8>();
        pixel.0[0] = srgb.red;
        pixel.0[1] = srgb.green;
        pixel.0[2] = srgb.blue;
    }
}
