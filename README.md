# rust-jpegli

Rust workspace with a safe encode-first wrapper over the official Google JPEGli.

- Upstream: official `google/jpegli`
- Pinned upstream commit: `c4e35be4af1c8c359e2c7063b5a2a767330007ce`
- Default build mode: vendored, reproducible, no network access from `build.rs`
- License stack: permissive only; no AGPL dependencies

## Workspace

- `crates/jpegli-sys`: build integration, FFI shim, pregenerated shim bindings
- `crates/jpegli`: safe Rust API for in-memory JPEG encoding
- `crates/jpegli-sys/vendor/jpegli`: vendored upstream source tree pinned to a specific commit

## Build Modes

### `vendored` (default)

Builds JPEGli from
[crates/jpegli-sys/vendor/jpegli](/home/andrey/projects/rust-jpegli/crates/jpegli-sys/vendor/jpegli)
with CMake.

The build disables upstream tools, docs, benchmarks and tests, and targets `jpegli-static`.

### `system`

Optional mode for linking against an already built system JPEGli that exposes
`jpegli_*` symbols.

Supported overrides:

- `JPEGLI_SYS_USE_SYSTEM=1`
- `JPEGLI_SYS_ROOT=/path/to/jpegli/build-or-install-root`
- `JPEGLI_SYS_STATIC=1`
- `JPEGLI_SYS_CMAKE_TOOLCHAIN_FILE=/path/to/toolchain.cmake`
- `JPEGLI_SYS_PKG_CONFIG=/path/to/pkg-config`

Example:

```bash
JPEGLI_SYS_USE_SYSTEM=1 \
JPEGLI_SYS_ROOT=/opt/jpegli \
cargo build -p jpegli --no-default-features --features system,static
```

## Safe API

`jpegli` currently exposes encode-only v0.1 functionality:

- `Rgb8`, `Rgba8`, `Gray8`
- memory-to-memory encoding into `Vec<u8>`
- `quality` or `distance`
- progressive on/off
- `Auto` / `444` / `422` / `420` subsampling
- optional ICC profile pass-through
- optional optimize-coding flag
- optional baseline-compatibility flag
- stride-aware borrowed input views

`distance` is the preferred modern quality knob. `psnr` is intentionally not
exposed in the high-level API in v0.1.

For `Rgba8`, alpha is dropped because baseline JPEG has no alpha channel.

## Example

```rust
use jpegli::{ChromaSubsampling, Encoder, EncoderConfig, ImageView, PixelFormat};

let encoder = Encoder::new(EncoderConfig {
    distance: Some(1.0),
    progressive: false,
    subsampling: ChromaSubsampling::Cs444,
    ..EncoderConfig::default()
})?;

let rgb = vec![
    255, 0, 0, 0, 255, 0,
    0, 0, 255, 255, 255, 0,
];

let jpeg = encoder.encode(&ImageView {
    width: 2,
    height: 2,
    format: PixelFormat::Rgb8,
    stride: 2 * 3,
    data: &rgb,
})?;
```

A runnable example lives at
[crates/jpegli/examples/memory_encode.rs](/home/andrey/projects/rust-jpegli/crates/jpegli/examples/memory_encode.rs).

## Limitations

Out of scope in v0.1:

- decode API
- streaming writer API
- coefficient/raw MCU APIs
- `psnr` in the safe API
- wasm/mobile support

## Development

Typical local checks:

```bash
cargo build
cargo test
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
```

## License

This project's wrapper code is licensed under either:

- MIT license
- Apache License, Version 2.0

at your option.

Vendored upstream Google JPEGli code remains under BSD-3-Clause.
The vendored upstream tree also preserves third-party components and their
upstream license files as shipped by upstream.
See [THIRD_PARTY_NOTICES.md](/home/andrey/projects/rust-jpegli/THIRD_PARTY_NOTICES.md)
and the vendored upstream license files for details.
