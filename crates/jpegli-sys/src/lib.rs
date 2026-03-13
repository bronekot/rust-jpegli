// SPDX-License-Identifier: MIT OR Apache-2.0

//! Low-level Rust bindings and build integration for vendored Google JPEGli.

use std::error::Error as StdError;
use std::ffi::CStr;
use std::fmt;
use std::ptr;
use std::slice;

/// Raw FFI declarations for the narrow encoder shim ABI.
pub mod raw {
    #![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

    #[cfg(feature = "generate-bindings")]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[cfg(not(feature = "generate-bindings"))]
    include!("bindings.rs");
}

/// Low-level encoder configuration passed to the C shim.
#[derive(Clone, Copy, Debug)]
pub struct EncodeConfig<'a> {
    pub quality: Option<u8>,
    pub distance: Option<f32>,
    pub progressive: bool,
    pub optimize_coding: bool,
    pub baseline_compatible: bool,
    pub subsampling: raw::jpegli_rs_subsampling,
    pub icc_profile: Option<&'a [u8]>,
}

/// Low-level borrowed image view passed to the C shim.
#[derive(Clone, Copy, Debug)]
pub struct ImageView<'a> {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixel_format: raw::jpegli_rs_pixel_format,
    pub data: &'a [u8],
}

/// Broad category of a low-level encode error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeErrorKind {
    InvalidConfig,
    InvalidArgument,
    EncodeFailed,
    Internal,
}

/// Error returned by the low-level encoder shim.
#[derive(Debug)]
pub struct EncodeError {
    pub kind: EncodeErrorKind,
    pub message: Option<String>,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{message}"),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

impl StdError for EncodeError {}

/// Encodes the provided image into a JPEG byte buffer via the C shim.
pub fn encode(config: &EncodeConfig<'_>, image: &ImageView<'_>) -> Result<Vec<u8>, EncodeError> {
    config.validate()?;
    image.validate()?;

    let mut output = raw::jpegli_rs_output {
        data: ptr::null_mut(),
        len: 0,
        error_message: ptr::null_mut(),
    };

    let raw_config = config.as_raw();
    let raw_image = image.as_raw();

    let status = unsafe { raw::jpegli_rs_encode(&raw_config, &raw_image, &mut output) };

    match status {
        raw::jpegli_rs_status::JPEGLI_RS_STATUS_OK => {
            if output.data.is_null() {
                unsafe { raw::jpegli_rs_free_output(&mut output) };
                return Err(EncodeError {
                    kind: EncodeErrorKind::InvalidArgument,
                    message: Some("encoder returned success without an output buffer".to_owned()),
                });
            }

            let bytes = unsafe { slice::from_raw_parts(output.data, output.len) }.to_vec();
            unsafe { raw::jpegli_rs_free_output(&mut output) };
            Ok(bytes)
        }
        raw::jpegli_rs_status::JPEGLI_RS_STATUS_INVALID_ARGUMENT => {
            let err = take_error(EncodeErrorKind::InvalidArgument, &mut output);
            Err(err)
        }
        raw::jpegli_rs_status::JPEGLI_RS_STATUS_ENCODE_ERROR => {
            let err = take_error(EncodeErrorKind::EncodeFailed, &mut output);
            Err(err)
        }
        raw::jpegli_rs_status::JPEGLI_RS_STATUS_INTERNAL_ERROR => {
            let err = take_error(EncodeErrorKind::Internal, &mut output);
            Err(err)
        }
    }
}

impl<'a> EncodeConfig<'a> {
    fn validate(&self) -> Result<(), EncodeError> {
        if self.quality.is_some() && self.distance.is_some() {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidConfig,
                message: Some("quality and distance are mutually exclusive".to_owned()),
            });
        }

        if self.progressive && !self.optimize_coding {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidConfig,
                message: Some("progressive encoding requires optimize_coding=true".to_owned()),
            });
        }

        if let Some(quality) = self.quality
            && !(1..=100).contains(&quality)
        {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidConfig,
                message: Some("quality must be in the range 1..=100".to_owned()),
            });
        }

        if let Some(distance) = self.distance
            && (!distance.is_finite() || !(0.0..=25.0).contains(&distance))
        {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidConfig,
                message: Some("distance must be finite and in the range 0.0..=25.0".to_owned()),
            });
        }

        if matches!(self.icc_profile, Some(profile) if profile.is_empty()) {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidConfig,
                message: Some("icc_profile must not be empty".to_owned()),
            });
        }

        Ok(())
    }

    fn as_raw(&self) -> raw::jpegli_rs_encoder_config {
        raw::jpegli_rs_encoder_config {
            has_quality: u8::from(self.quality.is_some()),
            quality: self.quality.unwrap_or(0),
            has_distance: u8::from(self.distance.is_some()),
            progressive: u8::from(self.progressive),
            optimize_coding: u8::from(self.optimize_coding),
            baseline_compatible: u8::from(self.baseline_compatible),
            _reserved0: 0,
            distance: self.distance.unwrap_or(0.0),
            subsampling: self.subsampling as u32,
            icc_profile: self
                .icc_profile
                .map(|icc| icc.as_ptr())
                .unwrap_or(ptr::null()),
            icc_profile_len: self.icc_profile.map(|icc| icc.len()).unwrap_or(0),
        }
    }
}

impl<'a> ImageView<'a> {
    fn validate(&self) -> Result<(), EncodeError> {
        if self.width == 0 || self.height == 0 {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidArgument,
                message: Some("width and height must be non-zero".to_owned()),
            });
        }

        let bytes_per_pixel = self.pixel_format as usize;
        let row_bytes = usize::try_from(self.width)
            .ok()
            .and_then(|width| width.checked_mul(bytes_per_pixel))
            .ok_or_else(|| EncodeError {
                kind: EncodeErrorKind::InvalidArgument,
                message: Some("image row size overflow".to_owned()),
            })?;

        if self.stride < row_bytes {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidArgument,
                message: Some("stride must be at least width * bytes_per_pixel".to_owned()),
            });
        }

        let total_bytes = self
            .stride
            .checked_mul((self.height - 1) as usize)
            .and_then(|prefix| prefix.checked_add(row_bytes))
            .ok_or_else(|| EncodeError {
                kind: EncodeErrorKind::InvalidArgument,
                message: Some("image buffer size overflow".to_owned()),
            })?;

        if self.data.len() < total_bytes {
            return Err(EncodeError {
                kind: EncodeErrorKind::InvalidArgument,
                message: Some("input buffer is too small for the image view".to_owned()),
            });
        }

        Ok(())
    }

    fn as_raw(&self) -> raw::jpegli_rs_image_view {
        raw::jpegli_rs_image_view {
            width: self.width,
            height: self.height,
            stride: self.stride,
            pixel_format: self.pixel_format as u32,
            data: self.data.as_ptr(),
            data_len: self.data.len(),
        }
    }
}

fn take_error(kind: EncodeErrorKind, output: &mut raw::jpegli_rs_output) -> EncodeError {
    let message = if output.error_message.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(output.error_message) }
                .to_string_lossy()
                .into_owned(),
        )
    };
    unsafe { raw::jpegli_rs_free_output(output) };
    EncodeError { kind, message }
}
