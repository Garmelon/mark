//! Various dithering algorithms and supporting types.
//!
//! The cumbersome types in this module are there for performance. For example,
//! the program should not switch on the configured difference whenever it
//! compares two colors. Instead, a version of each algorithm should be compiled
//! for each color space and difference combination.

use std::marker::PhantomData;

use image::RgbaImage;
use palette::{
    color_difference::{Ciede2000, EuclideanDistance, HyAb},
    Clamp, IntoColor, Lab, Srgb,
};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::util;

//////////////////////
// Color difference //
//////////////////////

pub trait Difference<C> {
    fn diff(a: C, b: C) -> f32;
}

pub struct DiffClamp<D> {
    _phantom: PhantomData<D>,
}

impl<C: Clamp, D: Difference<C>> Difference<C> for DiffClamp<D> {
    fn diff(a: C, b: C) -> f32 {
        D::diff(a.clamp(), b.clamp())
    }
}

pub struct DiffEuclid;

impl<C: EuclideanDistance<Scalar = f32>> Difference<C> for DiffEuclid {
    fn diff(a: C, b: C) -> f32 {
        a.distance(b)
    }
}

pub struct DiffHyAb;

impl<C: IntoColor<Lab>> Difference<C> for DiffHyAb {
    fn diff(a: C, b: C) -> f32 {
        let a: Lab = a.into_color();
        let b: Lab = b.into_color();
        a.hybrid_distance(b)
    }
}

pub struct DiffCiede2000;

impl<C: IntoColor<Lab>> Difference<C> for DiffCiede2000 {
    fn diff(a: C, b: C) -> f32 {
        let a: Lab = a.into_color();
        let b: Lab = b.into_color();
        a.difference(b)
    }
}

pub struct DiffManhattan;

impl<C: AsRef<[f32; 3]>> Difference<C> for DiffManhattan {
    fn diff(a: C, b: C) -> f32 {
        let [a1, a2, a3] = a.as_ref();
        let [b1, b2, b3] = b.as_ref();
        (a1 - b1).abs() + (a2 - b2).abs() + (a3 - b3).abs()
    }
}

pub struct DiffManhattanSquare;

impl<C: AsRef<[f32; 3]>> Difference<C> for DiffManhattanSquare {
    fn diff(a: C, b: C) -> f32 {
        let [a1, a2, a3] = a.as_ref();
        let [b1, b2, b3] = b.as_ref();
        (a1 - b1).powi(2) + (a2 - b2).powi(2) + (a3 - b3).powi(3)
    }
}

/////////////
// Palette //
/////////////

pub struct Palette<C> {
    colors: Vec<C>,
}

impl<C> Palette<C> {
    pub fn new(colors: Vec<C>) -> Self {
        Self { colors }
    }

    fn nearest<D>(&self, to: C) -> C
    where
        C: Copy,
        D: Difference<C>,
    {
        self.colors
            .iter()
            .copied()
            .map(|c| (c, D::diff(c, to)))
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .expect("palette was empty")
            .0
    }
}

////////////////
// Algorithms //
////////////////

pub trait Algorithm<C, D> {
    fn run(image: RgbaImage, palette: Palette<C>) -> RgbaImage;
}

pub struct AlgoThreshold;

impl<C, D> Algorithm<C, D> for AlgoThreshold
where
    Srgb: IntoColor<C>,
    C: Copy,
    C: IntoColor<Srgb> ,
    D: Difference<C>,
{
    fn run(mut image: RgbaImage, palette: Palette<C>) -> RgbaImage {
        for pixel in image.pixels_mut() {
            let color: C = util::pixel_to_color(*pixel);
            let color = palette.nearest::<D>(color);
            util::update_pixel_with_color(pixel, color);
        }
        image
    }
}

// TODO Fix probability calculation
//
// Choose probability for each color such that the expected value of a pixel is
// its actual color (or as close as possible).
//
// We want to represent a pixel as a linear combination of palette colors with
// factors in the range [0, 1] that sum up to 1. Then we can use those factors
// as probabilities. This may not work for every palette.
//
// As a secondary optimization target, we might want to miminize the amount of
// nonzero factors, if possible.
pub struct AlgoRandom;

impl<C, D> Algorithm<C, D> for AlgoRandom
where
    Srgb: IntoColor<C>,
    C: AsMut<[f32; 3]>,
    C: Copy,
    C: IntoColor<Srgb>,
    D: Difference<C>,
{
    fn run(mut image: RgbaImage, palette: Palette<C>) -> RgbaImage {
        let mut rng = SmallRng::seed_from_u64(0);
        let range_radius = 1.0;

        for pixel in image.pixels_mut() {
            let mut color: C = util::pixel_to_color(*pixel);
            color.as_mut()[0] += rng.gen_range(-range_radius..=range_radius);
            color.as_mut()[1] += rng.gen_range(-range_radius..=range_radius);
            color.as_mut()[2] += rng.gen_range(-range_radius..=range_radius);
            let color = palette.nearest::<D>(color);
            util::update_pixel_with_color(pixel, color);
        }
        image
    }
}
