// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi::c_char;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum jpegli_rs_status {
    JPEGLI_RS_STATUS_OK = 0,
    JPEGLI_RS_STATUS_INVALID_ARGUMENT = 1,
    JPEGLI_RS_STATUS_ENCODE_ERROR = 2,
    JPEGLI_RS_STATUS_INTERNAL_ERROR = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum jpegli_rs_pixel_format {
    JPEGLI_RS_PIXEL_FORMAT_GRAY8 = 1,
    JPEGLI_RS_PIXEL_FORMAT_RGB8 = 3,
    JPEGLI_RS_PIXEL_FORMAT_RGBA8 = 4,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum jpegli_rs_subsampling {
    JPEGLI_RS_SUBSAMPLING_AUTO = 0,
    JPEGLI_RS_SUBSAMPLING_444 = 1,
    JPEGLI_RS_SUBSAMPLING_422 = 2,
    JPEGLI_RS_SUBSAMPLING_420 = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct jpegli_rs_encoder_config {
    pub has_quality: u8,
    pub quality: u8,
    pub has_distance: u8,
    pub progressive: u8,
    pub optimize_coding: u8,
    pub baseline_compatible: u8,
    pub _reserved0: u8,
    pub distance: f32,
    pub subsampling: u32,
    pub icc_profile: *const u8,
    pub icc_profile_len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct jpegli_rs_image_view {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixel_format: u32,
    pub data: *const u8,
    pub data_len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct jpegli_rs_output {
    pub data: *mut u8,
    pub len: usize,
    pub error_message: *mut c_char,
}

unsafe extern "C" {
    pub fn jpegli_rs_encode(
        config: *const jpegli_rs_encoder_config,
        image: *const jpegli_rs_image_view,
        output: *mut jpegli_rs_output,
    ) -> jpegli_rs_status;

    pub fn jpegli_rs_free_output(output: *mut jpegli_rs_output);
}
