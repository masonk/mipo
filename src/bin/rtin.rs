use anyhow::Result;
use clap::Parser;
use env_logger;
use log::warn;
use mipo::rtin::preprocess_heightmap_from_img_path;
use std::path::PathBuf;
// Preprocess an image for rtin meshing.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    img_path: PathBuf,
}

pub fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    if cfg!(feature = "serde") {
        let _ = preprocess_heightmap_from_img_path(args.img_path)?;
    } else {
        warn!("The 'serde' feature is not enabled, but it must be enabled to write .rtin files.")
    }

    Ok(())
}
