use image::Rgba;
use palette::{IntoColor, Srgb};

pub fn pixel_to_srgb(pixel: Rgba<u8>) -> Srgb {
    let [r, g, b, _] = pixel.0;
    Srgb::new(r, g, b).into_format::<f32>()
}

pub fn update_pixel_with_srgb(pixel: &mut Rgba<u8>, srgb: Srgb) {
    let srgb = srgb.into_format::<u8>();
    pixel.0[0] = srgb.red;
    pixel.0[1] = srgb.green;
    pixel.0[2] = srgb.blue;
}

pub fn pixel_to_color<C>(pixel: Rgba<u8>) -> C
where
    Srgb: IntoColor<C>,
{
    pixel_to_srgb(pixel).into_color()
}

pub fn update_pixel_with_color<C>(pixel: &mut Rgba<u8>, color: C)
where
    C: IntoColor<Srgb>,
{
    update_pixel_with_srgb(pixel, color.into_color())
}
