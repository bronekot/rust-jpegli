// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef JPEGLI_RS_SHIM_H_
#define JPEGLI_RS_SHIM_H_

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum jpegli_rs_status {
  JPEGLI_RS_STATUS_OK = 0,
  JPEGLI_RS_STATUS_INVALID_ARGUMENT = 1,
  JPEGLI_RS_STATUS_ENCODE_ERROR = 2,
  JPEGLI_RS_STATUS_INTERNAL_ERROR = 3,
} jpegli_rs_status;

typedef enum jpegli_rs_pixel_format {
  JPEGLI_RS_PIXEL_FORMAT_GRAY8 = 1,
  JPEGLI_RS_PIXEL_FORMAT_RGB8 = 3,
  JPEGLI_RS_PIXEL_FORMAT_RGBA8 = 4,
} jpegli_rs_pixel_format;

typedef enum jpegli_rs_subsampling {
  JPEGLI_RS_SUBSAMPLING_AUTO = 0,
  JPEGLI_RS_SUBSAMPLING_444 = 1,
  JPEGLI_RS_SUBSAMPLING_422 = 2,
  JPEGLI_RS_SUBSAMPLING_420 = 3,
} jpegli_rs_subsampling;

typedef struct jpegli_rs_encoder_config {
  uint8_t has_quality;
  uint8_t quality;
  uint8_t has_distance;
  uint8_t progressive;
  uint8_t optimize_coding;
  uint8_t baseline_compatible;
  uint8_t _reserved0;
  float distance;
  uint32_t subsampling;
  const uint8_t* icc_profile;
  size_t icc_profile_len;
} jpegli_rs_encoder_config;

typedef struct jpegli_rs_image_view {
  uint32_t width;
  uint32_t height;
  size_t stride;
  uint32_t pixel_format;
  const uint8_t* data;
  size_t data_len;
} jpegli_rs_image_view;

typedef struct jpegli_rs_output {
  uint8_t* data;
  size_t len;
  char* error_message;
} jpegli_rs_output;

jpegli_rs_status jpegli_rs_encode(const jpegli_rs_encoder_config* config,
                                  const jpegli_rs_image_view* image,
                                  jpegli_rs_output* output);

void jpegli_rs_free_output(jpegli_rs_output* output);

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // JPEGLI_RS_SHIM_H_
