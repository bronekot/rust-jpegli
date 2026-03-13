# Third-Party Notices

This package publishes a vendored subset of the upstream Google JPEGli source
tree together with upstream-preserved third-party dependencies required for
local builds.

## Google JPEGli

- Component name: Google JPEGli
- Origin: official upstream Google JPEGli repository
- License: BSD-3-Clause
- Notes: primary vendored source used by the `jpegli-sys` crate for local builds
- Preserved upstream files:
  - `vendor/jpegli/LICENSE`
  - `vendor/jpegli/AUTHORS`
  - `vendor/jpegli/PATENTS`

## Highway

- Component name: Highway
- Origin: vendored by upstream Google JPEGli
- License: upstream-preserved Apache-2.0 and BSD-3-Clause notice files
- Notes: SIMD support code included in the vendored upstream tree
- Preserved upstream files:
  - `vendor/jpegli/third_party/highway/LICENSE`
  - `vendor/jpegli/third_party/highway/LICENSE-BSD3`

## skcms

- Component name: skcms
- Origin: vendored by upstream Google JPEGli
- License: BSD-3-Clause
- Notes: color management code included in the vendored upstream tree
- Preserved upstream file:
  - `vendor/jpegli/third_party/skcms/LICENSE`

## libjpeg-turbo files

- Component name: libjpeg-turbo files vendored by upstream Google JPEGli
- Origin: vendored by upstream Google JPEGli
- License: upstream-preserved IJG, BSD-style, and zlib terms
- Notes: headers and related files used by the vendored JPEGli build
- Preserved upstream files:
  - `vendor/jpegli/third_party/libjpeg-turbo/LICENSE.md`
  - `vendor/jpegli/third_party/libjpeg-turbo/README.ijg`
