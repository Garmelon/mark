#![forbid(unsafe_code)]
// Rustc lint groups
#![warn(future_incompatible)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
// Rustc lints
#![warn(noop_method_call)]
#![warn(single_use_lifetimes)]
// Clippy lints
#![warn(clippy::use_self)]

use std::{
    error::Error,
    fmt,
    io::{Cursor, Read, Write},
    num::ParseIntError,
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;
use image::{ImageFormat, ImageReader, RgbaImage};
use mark::{
    bw,
    dither::{
        AlgoFloydSteinberg, AlgoRandom, AlgoStucki, AlgoThreshold, Algorithm, DiffCiede2000,
        DiffClamp, DiffEuclid, DiffHyAb, DiffManhattan, Difference, Palette,
    },
};
use palette::{Clamp, IntoColor, Lab, Lch, LinSrgb, Luv, Okhsl, Okhsv, Oklab, Srgb};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum BwMethod {
    SrgbAverage,
    LinSrgbAverage,
    Hsl,
    Hsv,
    Cielab,
    Oklab,
}

impl From<BwMethod> for bw::Method {
    fn from(value: BwMethod) -> Self {
        match value {
            BwMethod::SrgbAverage => Self::SrgbAverage,
            BwMethod::LinSrgbAverage => Self::LinSrgbAverage,
            BwMethod::Hsl => Self::Hsl,
            BwMethod::Hsv => Self::Hsv,
            BwMethod::Cielab => Self::Cielab,
            BwMethod::Oklab => Self::Oklab,
        }
    }
}

#[derive(Debug, clap::Parser)]
/// Convert images into black and white.
struct BwCmd {
    #[arg(long, short)]
    method: BwMethod,
}

impl BwCmd {
    fn run(self, mut image: RgbaImage) -> RgbaImage {
        bw::bw(&mut image, self.method.into());
        image
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum DitherAlgorithm {
    Threshold,
    Random,
    FloydSteinberg,
    Stucki,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum DitherColorSpace {
    Srgb,
    LinSrgb,
    Cielab,
    Cieluv,
    Cielch,
    Oklab,
    Okhsl,
    Okhsv,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum DitherDifference {
    Euclid,
    EuclidClamp,
    HyAb,
    HyAbClamp,
    Ciede2000,
    Ciede2000Clamp,
    Manhattan,
    ManhattanClamp,
}

#[derive(Debug, Clone, Copy)]
struct SrgbColor(Srgb<u8>);

#[derive(Debug)]
enum ParseSrgbColorError {
    MustBeSixHexDigits,
    ParseIntError(ParseIntError),
}

impl fmt::Display for ParseSrgbColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MustBeSixHexDigits => write!(f, "a color must consist of six hexadecimal digits"),
            Self::ParseIntError(e) => e.fmt(f),
        }
    }
}

impl Error for ParseSrgbColorError {}

impl From<ParseIntError> for ParseSrgbColorError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl FromStr for SrgbColor {
    type Err = ParseSrgbColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 6 {
            return Err(ParseSrgbColorError::MustBeSixHexDigits);
        }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseSrgbColorError::MustBeSixHexDigits);
        }
        let r = u8::from_str_radix(&s[0..2], 16)?;
        let g = u8::from_str_radix(&s[2..4], 16)?;
        let b = u8::from_str_radix(&s[4..6], 16)?;
        Ok(Self(Srgb::new(r, g, b)))
    }
}

#[derive(Debug, clap::Parser)]
/// Dither images.
struct DitherCmd {
    #[arg(long, short)]
    algorithm: DitherAlgorithm,
    #[arg(long, short)]
    color_space: DitherColorSpace,
    #[arg(long, short)]
    difference: DitherDifference,
    /// Add a hex color to the palette used for dithering.
    #[arg(long, short)]
    palette: Vec<SrgbColor>,
}

impl DitherCmd {
    fn run(self, image: RgbaImage) -> RgbaImage {
        match self.color_space {
            DitherColorSpace::Srgb => self.run_c::<Srgb>(image),
            DitherColorSpace::LinSrgb => self.run_c::<LinSrgb>(image),
            DitherColorSpace::Cielab => self.run_c::<Lab>(image),
            DitherColorSpace::Cieluv => self.run_c::<Lch>(image),
            DitherColorSpace::Cielch => self.run_c::<Luv>(image),
            DitherColorSpace::Oklab => self.run_c::<Oklab>(image),
            DitherColorSpace::Okhsl => self.run_c::<Okhsl>(image),
            DitherColorSpace::Okhsv => self.run_c::<Okhsv>(image),
        }
    }

    fn run_c<C>(self, image: RgbaImage) -> RgbaImage
    where
        C: AsMut<[f32; 3]>,
        C: AsRef<[f32; 3]>,
        C: Clamp,
        C: Copy,
        C: IntoColor<Lab>,
        C: IntoColor<Srgb>,
        Srgb: IntoColor<C>,
    {
        use DitherDifference::*;
        match self.difference {
            Euclid => self.run_cd::<C, DiffEuclid>(image),
            EuclidClamp => self.run_cd::<C, DiffClamp<DiffEuclid>>(image),
            HyAb => self.run_cd::<C, DiffHyAb>(image),
            HyAbClamp => self.run_cd::<C, DiffClamp<DiffHyAb>>(image),
            Ciede2000 => self.run_cd::<C, DiffCiede2000>(image),
            Ciede2000Clamp => self.run_cd::<C, DiffClamp<DiffCiede2000>>(image),
            Manhattan => self.run_cd::<C, DiffManhattan>(image),
            ManhattanClamp => self.run_cd::<C, DiffClamp<DiffManhattan>>(image),
        }
    }

    fn run_cd<C, D>(self, image: RgbaImage) -> RgbaImage
    where
        C: AsMut<[f32; 3]>,
        C: Clamp,
        C: Copy,
        C: IntoColor<Srgb>,
        D: Difference<C>,
        Srgb: IntoColor<C>,
    {
        use DitherAlgorithm::*;
        match self.algorithm {
            Threshold => self.run_acd::<AlgoThreshold, C, D>(image),
            Random => self.run_acd::<AlgoRandom, C, D>(image),
            FloydSteinberg => self.run_acd::<AlgoFloydSteinberg, C, D>(image),
            Stucki => self.run_acd::<AlgoStucki, C, D>(image),
        }
    }

    fn run_acd<A, C, D>(self, image: RgbaImage) -> RgbaImage
    where
        A: Algorithm<C, D>,
        Srgb: IntoColor<C>,
    {
        let colors = self
            .palette
            .into_iter()
            .map(|c| c.0.into_format().into_color())
            .collect::<Vec<C>>();
        let palette = Palette::<C>::new(colors);
        A::run(image, &palette)
    }
}

#[derive(Debug, clap::Parser)]
enum Cmd {
    Bw(BwCmd),
    Dither(DitherCmd),
}

impl Cmd {
    fn run(self, image: RgbaImage) -> RgbaImage {
        match self {
            Self::Bw(cmd) => cmd.run(image),
            Self::Dither(cmd) => cmd.run(image),
        }
    }
}

#[derive(Debug, clap::Parser)]
struct Args {
    /// Load image from file instead of stdin.
    #[arg(long, short)]
    r#in: Option<PathBuf>,

    /// Output image to file instead of stdout.
    #[arg(long, short)]
    out: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

fn load_image(r#in: &Option<PathBuf>) -> RgbaImage {
    if let Some(path) = r#in {
        eprintln!("Loading image from {}", path.display());
        ImageReader::open(path)
            .expect("failed to load image from file")
            .decode()
            .expect("failed to decode image data")
    } else {
        eprintln!("Loading image from stdin");
        let mut buf = vec![];
        std::io::stdin()
            .read_to_end(&mut buf)
            .expect("failed to read stdin");
        ImageReader::new(Cursor::new(buf))
            .with_guessed_format()
            .expect("failed to guess image format")
            .decode()
            .expect("failed to decode image data")
    }
    .into_rgba8()
}

fn save_image(out: &Option<PathBuf>, image: RgbaImage) {
    if let Some(path) = out {
        eprintln!("Writing image to {}", path.display());
        image.save(path).expect("failed to save image to file");
    } else {
        eprintln!("Writing image to stdout");
        let mut buf = vec![];
        image
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .expect("failed to export image to bytes");
        std::io::stdout()
            .write_all(&buf)
            .expect("failed to write image to stdout");
    }
}

fn main() {
    let args = Args::parse();
    let image = load_image(&args.r#in);
    let image = args.cmd.run(image);
    save_image(&args.out, image);
}
