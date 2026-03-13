// SPDX-License-Identifier: MIT OR Apache-2.0

//! Safe Rust API for encoding JPEG images with Google JPEGli.

use std::error::Error as StdError;
use std::fmt;

/// High-level encoder with validated configuration.
#[derive(Clone, Debug)]
pub struct Encoder {
    cfg: EncoderConfig,
}

/// Configuration for high-level JPEG encoding.
#[derive(Clone, Debug, PartialEq)]
pub struct EncoderConfig {
    pub quality: Option<u8>,
    pub distance: Option<f32>,
    pub progressive: bool,
    pub subsampling: ChromaSubsampling,
    pub optimize_coding: bool,
    pub baseline_compatible: bool,
    pub icc_profile: Option<Vec<u8>>,
}

/// Output chroma subsampling mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChromaSubsampling {
    Auto,
    Cs444,
    Cs422,
    Cs420,
}

/// Borrowed source image passed into the encoder.
#[derive(Clone, Copy, Debug)]
pub struct ImageView<'a> {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub stride: usize,
    pub data: &'a [u8],
}

/// Supported input pixel formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    Rgb8,
    Rgba8,
    Gray8,
}

/// High-level encoder errors.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidConfig(&'static str),
    InvalidImage(&'static str),
    EncodeFailed(String),
    NullPointer,
    Internal(&'static str),
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            quality: None,
            distance: None,
            progressive: false,
            subsampling: ChromaSubsampling::Auto,
            optimize_coding: true,
            baseline_compatible: false,
            icc_profile: None,
        }
    }
}

impl PixelFormat {
    fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba8 => 4,
            PixelFormat::Gray8 => 1,
        }
    }

    fn as_sys(self) -> jpegli_sys::raw::jpegli_rs_pixel_format {
        match self {
            PixelFormat::Rgb8 => {
                jpegli_sys::raw::jpegli_rs_pixel_format::JPEGLI_RS_PIXEL_FORMAT_RGB8
            }
            PixelFormat::Rgba8 => {
                jpegli_sys::raw::jpegli_rs_pixel_format::JPEGLI_RS_PIXEL_FORMAT_RGBA8
            }
            PixelFormat::Gray8 => {
                jpegli_sys::raw::jpegli_rs_pixel_format::JPEGLI_RS_PIXEL_FORMAT_GRAY8
            }
        }
    }
}

impl EncoderConfig {
    fn validate(&self) -> Result<(), Error> {
        if self.quality.is_some() && self.distance.is_some() {
            return Err(Error::InvalidConfig(
                "quality and distance are mutually exclusive",
            ));
        }

        if self.progressive && !self.optimize_coding {
            return Err(Error::InvalidConfig(
                "progressive encoding requires optimize_coding=true",
            ));
        }

        if let Some(quality) = self.quality {
            if !(1..=100).contains(&quality) {
                return Err(Error::InvalidConfig("quality must be in the range 1..=100"));
            }
        }

        if let Some(distance) = self.distance {
            if !distance.is_finite() || !(0.0..=25.0).contains(&distance) {
                return Err(Error::InvalidConfig(
                    "distance must be finite and in the range 0.0..=25.0",
                ));
            }
        }

        if matches!(self.icc_profile.as_ref(), Some(profile) if profile.is_empty()) {
            return Err(Error::InvalidConfig("icc_profile must not be empty"));
        }

        Ok(())
    }

    fn as_sys(&self) -> jpegli_sys::EncodeConfig<'_> {
        jpegli_sys::EncodeConfig {
            quality: self.quality,
            distance: self.distance,
            progressive: self.progressive,
            optimize_coding: self.optimize_coding,
            baseline_compatible: self.baseline_compatible,
            subsampling: self.subsampling.as_sys(),
            icc_profile: self.icc_profile.as_deref(),
        }
    }
}

impl ChromaSubsampling {
    fn as_sys(self) -> jpegli_sys::raw::jpegli_rs_subsampling {
        match self {
            ChromaSubsampling::Auto => {
                jpegli_sys::raw::jpegli_rs_subsampling::JPEGLI_RS_SUBSAMPLING_AUTO
            }
            ChromaSubsampling::Cs444 => {
                jpegli_sys::raw::jpegli_rs_subsampling::JPEGLI_RS_SUBSAMPLING_444
            }
            ChromaSubsampling::Cs422 => {
                jpegli_sys::raw::jpegli_rs_subsampling::JPEGLI_RS_SUBSAMPLING_422
            }
            ChromaSubsampling::Cs420 => {
                jpegli_sys::raw::jpegli_rs_subsampling::JPEGLI_RS_SUBSAMPLING_420
            }
        }
    }
}

impl Encoder {
    pub fn new(cfg: EncoderConfig) -> Result<Self, Error> {
        cfg.validate()?;
        Ok(Self { cfg })
    }

    /// Encodes the image into an in-memory JPEG byte buffer.
    ///
    /// For `PixelFormat::Rgba8`, the alpha channel is ignored because baseline
    /// JPEG has no alpha support.
    pub fn encode(&self, image: &ImageView<'_>) -> Result<Vec<u8>, Error> {
        image.validate()?;

        jpegli_sys::encode(&self.cfg.as_sys(), &image.as_sys()).map_err(map_sys_error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidConfig(message) => write!(f, "{message}"),
            Error::InvalidImage(message) => write!(f, "{message}"),
            Error::EncodeFailed(message) => write!(f, "{message}"),
            Error::NullPointer => write!(f, "unexpected null pointer from FFI"),
            Error::Internal(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {}

impl<'a> ImageView<'a> {
    fn validate(&self) -> Result<(), Error> {
        if self.width == 0 || self.height == 0 {
            return Err(Error::InvalidImage("width and height must be non-zero"));
        }

        let row_bytes = usize::try_from(self.width)
            .ok()
            .and_then(|width| width.checked_mul(self.format.bytes_per_pixel()))
            .ok_or(Error::InvalidImage("image row size overflow"))?;

        if self.stride < row_bytes {
            return Err(Error::InvalidImage(
                "stride must be at least width * bytes_per_pixel",
            ));
        }

        let total_bytes = self
            .stride
            .checked_mul((self.height - 1) as usize)
            .and_then(|prefix| prefix.checked_add(row_bytes))
            .ok_or(Error::InvalidImage("image buffer size overflow"))?;

        if self.data.len() < total_bytes {
            return Err(Error::InvalidImage(
                "input buffer is too small for the image view",
            ));
        }

        Ok(())
    }

    fn as_sys(&self) -> jpegli_sys::ImageView<'_> {
        jpegli_sys::ImageView {
            width: self.width,
            height: self.height,
            stride: self.stride,
            pixel_format: self.format.as_sys(),
            data: self.data,
        }
    }
}

fn map_sys_error(error: jpegli_sys::EncodeError) -> Error {
    match error.kind {
        jpegli_sys::EncodeErrorKind::InvalidConfig => {
            Error::InvalidConfig("ffi rejected the provided encoder config")
        }
        jpegli_sys::EncodeErrorKind::InvalidArgument => {
            Error::InvalidImage("ffi rejected the provided image")
        }
        jpegli_sys::EncodeErrorKind::EncodeFailed => Error::EncodeFailed(
            error
                .message
                .unwrap_or_else(|| "jpegli encode failed".to_owned()),
        ),
        jpegli_sys::EncodeErrorKind::Internal => {
            if let Some(message) = error.message {
                Error::EncodeFailed(message)
            } else {
                Error::Internal("jpegli internal error")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb_image<'a>(data: &'a [u8]) -> ImageView<'a> {
        ImageView {
            width: 2,
            height: 2,
            format: PixelFormat::Rgb8,
            stride: 6,
            data,
        }
    }

    #[test]
    fn rejects_quality_and_distance_together() {
        let cfg = EncoderConfig {
            quality: Some(90),
            distance: Some(1.2),
            ..EncoderConfig::default()
        };

        let err = Encoder::new(cfg).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidConfig("quality and distance are mutually exclusive")
        );
    }

    #[test]
    fn rejects_invalid_dimensions() {
        let encoder = Encoder::new(EncoderConfig {
            quality: Some(90),
            ..EncoderConfig::default()
        })
        .unwrap();
        let image = ImageView {
            width: 0,
            height: 2,
            format: PixelFormat::Rgb8,
            stride: 6,
            data: &[0; 12],
        };

        let err = encoder.encode(&image).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidImage("width and height must be non-zero")
        );
    }

    #[test]
    fn rejects_empty_buffer() {
        let encoder = Encoder::new(EncoderConfig {
            quality: Some(90),
            ..EncoderConfig::default()
        })
        .unwrap();

        let err = encoder.encode(&rgb_image(&[])).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidImage("input buffer is too small for the image view")
        );
    }

    #[test]
    fn rejects_wrong_stride() {
        let encoder = Encoder::new(EncoderConfig {
            quality: Some(90),
            ..EncoderConfig::default()
        })
        .unwrap();
        let image = ImageView {
            width: 2,
            height: 2,
            format: PixelFormat::Rgb8,
            stride: 5,
            data: &[0; 12],
        };

        let err = encoder.encode(&image).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidImage("stride must be at least width * bytes_per_pixel")
        );
    }
}
