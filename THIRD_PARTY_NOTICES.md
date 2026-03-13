# Third-Party Notices

The `jpegli-sys` crate publishes a vendored subset of the upstream Google JPEGli
source tree together with upstream-preserved third-party dependencies required
for local builds.

## Google JPEGli

- Component name: Google JPEGli
- Origin: official upstream Google JPEGli repository
- License: BSD-3-Clause
- Notes: primary vendored source used by the `jpegli-sys` crate for local builds
- Preserved upstream files:
  - [crates/jpegli-sys/vendor/jpegli/LICENSE](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/LICENSE)
  - [crates/jpegli-sys/vendor/jpegli/AUTHORS](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/AUTHORS)
  - [crates/jpegli-sys/vendor/jpegli/PATENTS](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/PATENTS)

## Highway

- Component name: Highway
- Origin: vendored by upstream Google JPEGli
- License: upstream-preserved Apache-2.0 and BSD-3-Clause notice files
- Notes: SIMD support code included in the vendored upstream tree
- Preserved upstream files:
  - [crates/jpegli-sys/vendor/jpegli/third_party/highway/LICENSE](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/third_party/highway/LICENSE)
  - [crates/jpegli-sys/vendor/jpegli/third_party/highway/LICENSE-BSD3](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/third_party/highway/LICENSE-BSD3)

## skcms

- Component name: skcms
- Origin: vendored by upstream Google JPEGli
- License: BSD-3-Clause
- Notes: color management code included in the vendored upstream tree
- Preserved upstream file:
  - [crates/jpegli-sys/vendor/jpegli/third_party/skcms/LICENSE](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/third_party/skcms/LICENSE)

## libjpeg-turbo files

- Component name: libjpeg-turbo files vendored by upstream Google JPEGli
- Origin: vendored by upstream Google JPEGli
- License: upstream-preserved IJG, BSD-style, and zlib terms
- Notes: headers and related files used by the vendored JPEGli build
- Preserved upstream files:
  - [crates/jpegli-sys/vendor/jpegli/third_party/libjpeg-turbo/LICENSE.md](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/third_party/libjpeg-turbo/LICENSE.md)
  - [crates/jpegli-sys/vendor/jpegli/third_party/libjpeg-turbo/README.ijg](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli/third_party/libjpeg-turbo/README.ijg)
