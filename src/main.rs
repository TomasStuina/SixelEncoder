use std::{env, ffi::OsString, io};
use encoders::sixel::{ColorMode, SixelEncoder};

pub mod encoders;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<OsString> = env::args_os().collect();

    if args.len() != 2 {
        return Err("Missing argument: image path.".into());
    }
    let stdout = io::stdout();
    let mut buf_writer = io::BufWriter::new(stdout);
    let image_path = &args[1];
    let image = bmp::open(image_path)?;
    let mut encoder = SixelEncoder::new(&image, ColorMode::RGB);

    encoder.encode(&mut buf_writer)
}