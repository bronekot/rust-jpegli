// SPDX-License-Identifier: MIT OR Apache-2.0

#include "jpegli_rs_shim.h"

#include <setjmp.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include <string>

#include "lib/jpegli/common.h"
#include "lib/jpegli/encode.h"

namespace {

struct RustErrorManager {
  jpeg_error_mgr pub;
  jmp_buf env;
  std::string message;
};

struct EncoderContext {
  jpeg_compress_struct cinfo = {};
  RustErrorManager err = {};
  unsigned char* encoded = nullptr;
  unsigned long encoded_len = 0;  // NOLINT(runtime/int)
};

char* DuplicateString(const char* message) {
  if (message == nullptr) {
    return nullptr;
  }
  const size_t len = strlen(message);
  char* copy = static_cast<char*>(malloc(len + 1));
  if (copy == nullptr) {
    return nullptr;
  }
  memcpy(copy, message, len + 1);
  return copy;
}

void ErrorExit(j_common_ptr cinfo) {
  auto* err = reinterpret_cast<RustErrorManager*>(cinfo->err);
  char buffer[JMSG_LENGTH_MAX];
  buffer[0] = '\0';
  (*cinfo->err->format_message)(cinfo, buffer);
  err->message.assign(buffer);
  longjmp(err->env, 1);
}

void OutputMessage(j_common_ptr /*cinfo*/) {}

int ComponentsForFormat(uint32_t pixel_format) {
  switch (pixel_format) {
    case JPEGLI_RS_PIXEL_FORMAT_GRAY8:
      return 1;
    case JPEGLI_RS_PIXEL_FORMAT_RGB8:
      return 3;
    case JPEGLI_RS_PIXEL_FORMAT_RGBA8:
      return 4;
    default:
      return 0;
  }
}

J_COLOR_SPACE ColorSpaceForFormat(uint32_t pixel_format) {
  switch (pixel_format) {
    case JPEGLI_RS_PIXEL_FORMAT_GRAY8:
      return JCS_GRAYSCALE;
    case JPEGLI_RS_PIXEL_FORMAT_RGB8:
      return JCS_EXT_RGB;
    case JPEGLI_RS_PIXEL_FORMAT_RGBA8:
      return JCS_EXT_RGBA;
    default:
      return JCS_UNKNOWN;
  }
}

bool ValidateConfig(const jpegli_rs_encoder_config* config, jpegli_rs_output* output) {
  if (config->has_quality != 0 && config->has_distance != 0) {
    output->error_message = DuplicateString("quality and distance are mutually exclusive");
    return false;
  }

  if (config->has_quality != 0 &&
      (config->quality < 1 || config->quality > 100)) {
    output->error_message = DuplicateString("quality must be in the range 1..=100");
    return false;
  }

  if (config->has_distance != 0 &&
      (!(config->distance >= 0.0f && config->distance <= 25.0f))) {
    output->error_message =
        DuplicateString("distance must be finite and in the range 0.0..=25.0");
    return false;
  }

  if (config->progressive != 0 && config->optimize_coding == 0) {
    output->error_message =
        DuplicateString("progressive encoding requires optimize_coding=true");
    return false;
  }

  if (config->icc_profile == nullptr && config->icc_profile_len != 0) {
    output->error_message = DuplicateString("icc_profile pointer is null");
    return false;
  }

  return true;
}

bool ValidateImage(const jpegli_rs_image_view* image, jpegli_rs_output* output,
                   int* components, J_COLOR_SPACE* color_space) {
  if (image->width == 0 || image->height == 0 || image->data == nullptr) {
    output->error_message = DuplicateString("image is empty");
    return false;
  }

  *components = ComponentsForFormat(image->pixel_format);
  *color_space = ColorSpaceForFormat(image->pixel_format);
  if (*components == 0 || *color_space == JCS_UNKNOWN) {
    output->error_message = DuplicateString("unsupported pixel format");
    return false;
  }

  const size_t row_bytes = static_cast<size_t>(image->width) * (*components);
  if (image->stride < row_bytes) {
    output->error_message =
        DuplicateString("stride must be at least width * bytes_per_pixel");
    return false;
  }

  const size_t total_bytes =
      (static_cast<size_t>(image->height) - 1) * image->stride + row_bytes;
  if (image->data_len < total_bytes) {
    output->error_message =
        DuplicateString("input buffer is too small for the image view");
    return false;
  }

  return true;
}

void ApplySubsampling(j_compress_ptr cinfo, uint32_t subsampling) {
  if (cinfo->num_components < 3 || subsampling == JPEGLI_RS_SUBSAMPLING_AUTO) {
    return;
  }

  switch (subsampling) {
    case JPEGLI_RS_SUBSAMPLING_444:
      cinfo->comp_info[0].h_samp_factor = 1;
      cinfo->comp_info[0].v_samp_factor = 1;
      break;
    case JPEGLI_RS_SUBSAMPLING_422:
      cinfo->comp_info[0].h_samp_factor = 2;
      cinfo->comp_info[0].v_samp_factor = 1;
      break;
    case JPEGLI_RS_SUBSAMPLING_420:
      cinfo->comp_info[0].h_samp_factor = 2;
      cinfo->comp_info[0].v_samp_factor = 2;
      break;
    default:
      return;
  }

  for (int i = 1; i < cinfo->num_components; ++i) {
    cinfo->comp_info[i].h_samp_factor = 1;
    cinfo->comp_info[i].v_samp_factor = 1;
  }
}

void ResetOutput(jpegli_rs_output* output) {
  output->data = nullptr;
  output->len = 0;
  output->error_message = nullptr;
}

void CleanupContext(EncoderContext* ctx) {
  jpegli_destroy(reinterpret_cast<j_common_ptr>(&ctx->cinfo));
  free(ctx->encoded);
  ctx->encoded = nullptr;
}

void InitializeErrorHandling(EncoderContext* ctx) {
  ctx->cinfo.err = jpegli_std_error(&ctx->err.pub);
  ctx->cinfo.err->error_exit = ErrorExit;
  ctx->cinfo.err->output_message = OutputMessage;
}

void ConfigureCompressor(EncoderContext* ctx,
                         const jpegli_rs_encoder_config* config,
                         const jpegli_rs_image_view* image, int components,
                         J_COLOR_SPACE color_space) {
  jpegli_create_compress(&ctx->cinfo);
  jpegli_mem_dest(&ctx->cinfo, &ctx->encoded, &ctx->encoded_len);

  ctx->cinfo.image_width = image->width;
  ctx->cinfo.image_height = image->height;
  ctx->cinfo.input_components = components;
  ctx->cinfo.in_color_space = color_space;
  jpegli_set_defaults(&ctx->cinfo);
  ApplySubsampling(&ctx->cinfo, config->subsampling);

  if (config->has_quality) {
    jpegli_set_quality(&ctx->cinfo, config->quality,
                       config->baseline_compatible ? TRUE : FALSE);
  } else if (config->has_distance) {
    jpegli_set_distance(&ctx->cinfo, config->distance,
                        config->baseline_compatible ? TRUE : FALSE);
  }

  jpegli_set_progressive_level(&ctx->cinfo, config->progressive ? 2 : 0);
  ctx->cinfo.optimize_coding = config->optimize_coding ? TRUE : FALSE;
}

bool EncodeRows(EncoderContext* ctx, const jpegli_rs_image_view* image,
                jpegli_rs_output* output) {
  const uint8_t* row_base = image->data;
  while (ctx->cinfo.next_scanline < ctx->cinfo.image_height) {
    JSAMPROW row = const_cast<JSAMPROW>(
        row_base + static_cast<size_t>(ctx->cinfo.next_scanline) * image->stride);
    const JDIMENSION written = jpegli_write_scanlines(&ctx->cinfo, &row, 1);
    if (written != 1) {
      output->error_message = DuplicateString("jpegli wrote zero scanlines");
      return false;
    }
  }
  return true;
}

}  // namespace

extern "C" jpegli_rs_status jpegli_rs_encode(
    const jpegli_rs_encoder_config* config, const jpegli_rs_image_view* image,
    jpegli_rs_output* output) {
  if (config == nullptr || image == nullptr || output == nullptr) {
    return JPEGLI_RS_STATUS_INVALID_ARGUMENT;
  }

  ResetOutput(output);

  if (!ValidateConfig(config, output)) {
    return JPEGLI_RS_STATUS_INVALID_ARGUMENT;
  }

  int components = 0;
  J_COLOR_SPACE color_space = JCS_UNKNOWN;
  if (!ValidateImage(image, output, &components, &color_space)) {
    return JPEGLI_RS_STATUS_INVALID_ARGUMENT;
  }

  EncoderContext ctx;
  InitializeErrorHandling(&ctx);

  if (setjmp(ctx.err.env) != 0) {
    CleanupContext(&ctx);
    output->error_message =
        DuplicateString(ctx.err.message.empty() ? "jpegli encode failed"
                                                : ctx.err.message.c_str());
    return JPEGLI_RS_STATUS_ENCODE_ERROR;
  }

  ConfigureCompressor(&ctx, config, image, components, color_space);
  jpegli_start_compress(&ctx.cinfo, TRUE);

  if (config->icc_profile != nullptr && config->icc_profile_len > 0) {
    jpegli_write_icc_profile(&ctx.cinfo, config->icc_profile,
                             static_cast<unsigned int>(config->icc_profile_len));
  }

  if (!EncodeRows(&ctx, image, output)) {
    CleanupContext(&ctx);
    return JPEGLI_RS_STATUS_INTERNAL_ERROR;
  }

  jpegli_finish_compress(&ctx.cinfo);
  jpegli_destroy_compress(&ctx.cinfo);

  output->data = ctx.encoded;
  output->len = static_cast<size_t>(ctx.encoded_len);
  return JPEGLI_RS_STATUS_OK;
}

extern "C" void jpegli_rs_free_output(jpegli_rs_output* output) {
  if (output == nullptr) {
    return;
  }
  free(output->data);
  free(output->error_message);
  ResetOutput(output);
}
