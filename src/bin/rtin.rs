use anyhow::Result;
use ciborium::into_writer;
use clap::Parser;
use mipo::rtin::preprocess_heightmap_from_img_path;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

// Preprocess an image for rtin meshing.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    img_path: PathBuf,
}

pub fn main() -> Result<()> {
    let args = Args::parse();
    let mut out = args.img_path.clone();

    let rtin_data = preprocess_heightmap_from_img_path(args.img_path)?;

    out.set_extension("rtin");
    let of = BufWriter::new(File::create(out)?);
    into_writer(&rtin_data, of)?;

    Ok(())
}
