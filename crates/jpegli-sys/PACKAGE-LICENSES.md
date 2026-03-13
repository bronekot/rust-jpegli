# Package Licensing

This package contains source files under multiple licenses.

Wrapper, build, bindings, and shim code written in this repository are dual-licensed under either:

- MIT
- Apache License, Version 2.0

The full texts are provided in:

- `LICENSE-MIT`
- `LICENSE-APACHE`

Vendored upstream and upstream-preserved third-party components included in this package:

- Google JPEGli
  - Origin: official upstream Google JPEGli repository
  - License: BSD-3-Clause
  - License files:
    - `vendor/jpegli/LICENSE`
    - `vendor/jpegli/AUTHORS`
    - `vendor/jpegli/PATENTS`

- Highway
  - Origin: vendored by upstream Google JPEGli
  - Upstream-preserved license files:
    - `vendor/jpegli/third_party/highway/LICENSE`
    - `vendor/jpegli/third_party/highway/LICENSE-BSD3`

- skcms
  - Origin: vendored by upstream Google JPEGli
  - License: BSD-3-Clause
  - License file:
    - `vendor/jpegli/third_party/skcms/LICENSE`

- libjpeg-turbo files vendored by upstream Google JPEGli
  - Origin: vendored by upstream Google JPEGli
  - Upstream-preserved license files:
    - `vendor/jpegli/third_party/libjpeg-turbo/LICENSE.md`
    - `vendor/jpegli/third_party/libjpeg-turbo/README.ijg`

All upstream copyright and license notices shipped in the vendored tree are preserved in place.
