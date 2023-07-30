use std::{
    io::{Cursor, Read, Write},
    path::PathBuf,
};

use clap::Parser;
use image::{ImageFormat, RgbaImage};
use mark::bw;

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
            BwMethod::SrgbAverage => bw::Method::SrgbAverage,
            BwMethod::LinSrgbAverage => bw::Method::LinSrgbAverage,
            BwMethod::Hsl => bw::Method::Hsl,
            BwMethod::Hsv => bw::Method::Hsv,
            BwMethod::Cielab => bw::Method::Cielab,
            BwMethod::Oklab => bw::Method::Oklab,
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

#[derive(Debug, clap::Parser)]
enum Cmd {
    Bw(BwCmd),
}

impl Cmd {
    fn run(self, image: RgbaImage) -> RgbaImage {
        match self {
            Cmd::Bw(cmd) => cmd.run(image),
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
        image::io::Reader::open(path)
            .expect("failed to load image from file")
            .decode()
            .expect("failed to decode image data")
    } else {
        eprintln!("Loading image from stdin");
        let mut buf = vec![];
        std::io::stdin()
            .read_to_end(&mut buf)
            .expect("failed to read stdin");
        image::io::Reader::new(Cursor::new(buf))
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
