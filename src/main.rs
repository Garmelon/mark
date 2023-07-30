use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, clap::Parser)]
struct BwCmd {}

#[derive(Debug, clap::Parser)]
enum Cmd {
    Bw(BwCmd),
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long, short)]
    r#in: Option<PathBuf>,
    #[arg(long, short)]
    out: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

fn main() {
    let args = Args::parse();
    println!("{args:#?}");
}
