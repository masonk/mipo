use anyhow::Result;
use clap::Parser;
use mipo::rtin::{from_img_path, Options};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    img_path: PathBuf,
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    from_img_path(args.img_path, Options::default())?;
    Ok(())
}
