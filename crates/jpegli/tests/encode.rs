// SPDX-License-Identifier: MIT OR Apache-2.0

use image::{load_from_memory_with_format, DynamicImage, GenericImageView, ImageFormat};
use jpegli::{ChromaSubsampling, Encoder, EncoderConfig, ImageView, PixelFormat};

fn rgb_fixture() -> Vec<u8> {
    load_from_memory_with_format(
        include_bytes!("fixtures/rgb_gradient.ppm"),
        ImageFormat::Pnm,
    )
    .unwrap()
    .into_rgb8()
    .into_raw()
}

fn gray_fixture() -> Vec<u8> {
    load_from_memory_with_format(
        include_bytes!("fixtures/gray_gradient.pgm"),
        ImageFormat::Pnm,
    )
    .unwrap()
    .into_luma8()
    .into_raw()
}

fn generated_rgb(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.push((x * 255 / (width - 1)) as u8);
            pixels.push((y * 255 / (height - 1)) as u8);
            pixels.push(((x + y) * 255 / (width + height - 2)) as u8);
        }
    }
    pixels
}

fn decode_jpeg(bytes: &[u8]) -> DynamicImage {
    load_from_memory_with_format(bytes, ImageFormat::Jpeg).unwrap()
}

fn encoder_with_quality(quality: u8) -> Encoder {
    Encoder::new(EncoderConfig {
        quality: Some(quality),
        progressive: false,
        subsampling: ChromaSubsampling::Cs444,
        ..EncoderConfig::default()
    })
    .unwrap()
}

fn encoder_with_distance(distance: f32) -> Encoder {
    Encoder::new(EncoderConfig {
        distance: Some(distance),
        progressive: false,
        subsampling: ChromaSubsampling::Cs444,
        ..EncoderConfig::default()
    })
    .unwrap()
}

#[test]
fn rgb_encode_success() {
    let pixels = rgb_fixture();
    let encoder = encoder_with_quality(90);
    let jpeg = encoder
        .encode(&ImageView {
            width: 8,
            height: 8,
            format: PixelFormat::Rgb8,
            stride: 8 * 3,
            data: &pixels,
        })
        .unwrap();

    assert!(jpeg.starts_with(&[0xFF, 0xD8]));
    assert!(jpeg.ends_with(&[0xFF, 0xD9]));
    assert!(jpeg.len() > 100);

    let decoded = decode_jpeg(&jpeg);
    assert_eq!(decoded.dimensions(), (8, 8));
}

#[test]
fn rgba_encode_drops_alpha() {
    let rgb = rgb_fixture();
    let mut rgba = Vec::with_capacity(8 * 8 * 4);
    for (idx, pixel) in rgb.chunks_exact(3).enumerate() {
        rgba.extend_from_slice(pixel);
        rgba.push((idx as u8).wrapping_mul(13));
    }

    let encoder = encoder_with_quality(90);
    let rgb_jpeg = encoder
        .encode(&ImageView {
            width: 8,
            height: 8,
            format: PixelFormat::Rgb8,
            stride: 8 * 3,
            data: &rgb,
        })
        .unwrap();
    let rgba_jpeg = encoder
        .encode(&ImageView {
            width: 8,
            height: 8,
            format: PixelFormat::Rgba8,
            stride: 8 * 4,
            data: &rgba,
        })
        .unwrap();

    assert_eq!(rgb_jpeg, rgba_jpeg);
}

#[test]
fn gray_encode_success() {
    let pixels = gray_fixture();
    let encoder = encoder_with_quality(90);
    let jpeg = encoder
        .encode(&ImageView {
            width: 8,
            height: 8,
            format: PixelFormat::Gray8,
            stride: 8,
            data: &pixels,
        })
        .unwrap();

    let decoded = decode_jpeg(&jpeg);
    assert_eq!(decoded.dimensions(), (8, 8));
}

#[test]
fn smoke_fixture_decodes_and_has_expected_properties() {
    let pixels = rgb_fixture();
    let encoder = encoder_with_distance(1.0);
    let jpeg = encoder
        .encode(&ImageView {
            width: 8,
            height: 8,
            format: PixelFormat::Rgb8,
            stride: 8 * 3,
            data: &pixels,
        })
        .unwrap();

    let decoded = decode_jpeg(&jpeg).into_rgb8();
    assert_eq!(decoded.dimensions(), (8, 8));
    assert!(jpeg.len() > 100);
    assert!(decoded.pixels().any(|pixel| pixel.0 != [0, 0, 0]));
}

#[test]
fn quality_affects_output_size() {
    let pixels = generated_rgb(64, 64);
    let low = encoder_with_quality(40)
        .encode(&ImageView {
            width: 64,
            height: 64,
            format: PixelFormat::Rgb8,
            stride: 64 * 3,
            data: &pixels,
        })
        .unwrap();
    let high = encoder_with_quality(95)
        .encode(&ImageView {
            width: 64,
            height: 64,
            format: PixelFormat::Rgb8,
            stride: 64 * 3,
            data: &pixels,
        })
        .unwrap();

    assert!(high.len() > low.len());
}

#[test]
fn distance_affects_output_size() {
    let pixels = generated_rgb(64, 64);
    let detailed = encoder_with_distance(0.8)
        .encode(&ImageView {
            width: 64,
            height: 64,
            format: PixelFormat::Rgb8,
            stride: 64 * 3,
            data: &pixels,
        })
        .unwrap();
    let smaller = encoder_with_distance(4.0)
        .encode(&ImageView {
            width: 64,
            height: 64,
            format: PixelFormat::Rgb8,
            stride: 64 * 3,
            data: &pixels,
        })
        .unwrap();

    assert!(detailed.len() > smaller.len());
}
