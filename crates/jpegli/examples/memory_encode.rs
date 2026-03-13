// SPDX-License-Identifier: MIT OR Apache-2.0

use std::env;
use std::fs;
use std::path::PathBuf;

use jpegli::{ChromaSubsampling, Encoder, EncoderConfig, ImageView, PixelFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("example.jpg"));

    let width = 32u32;
    let height = 32u32;
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            pixels.push((x * 8) as u8);
            pixels.push((y * 8) as u8);
            pixels.push(((x + y) * 4) as u8);
        }
    }

    let encoder = Encoder::new(EncoderConfig {
        distance: Some(1.0),
        progressive: false,
        subsampling: ChromaSubsampling::Cs444,
        ..EncoderConfig::default()
    })?;

    let jpeg = encoder.encode(&ImageView {
        width,
        height,
        format: PixelFormat::Rgb8,
        stride: width as usize * 3,
        data: &pixels,
    })?;

    fs::write(&output, jpeg)?;
    eprintln!("wrote {}", output.display());
    Ok(())
}
