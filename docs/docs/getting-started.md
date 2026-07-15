---
id: getting-started
title: Getting Started
sidebar_position: 1
---

# Getting Started

RPU requires Rust 1.93 or newer. This is also the minimum supported by the embedded Wasmer 7 runtime.

RPU is a tiny creative computer for portable cartridges.

## Install Rust

RPU is built with Rust. Install the current stable toolchain first:

- [Install Rust with rustup](https://www.rust-lang.org/tools/install)

## Create Or Run A Cartridge

Current starting points:

```bash
rpu new my_cart
rpu run path/to/cartridge
```

For a windowed app cartridge:

```bash
rpu run examples/warped_space_shooter
```

For a headless CLI cartridge:

```bash
rpu run examples/hello_cli
rpu run examples/hello_cli -- RPU
```

For a freestanding C cartridge compiled to WASM:

```bash
brew install llvm lld # macOS toolchain setup
rpu run examples/hello_c -- RPU
rpu build examples/hello_c
rpu run examples/hello_c/build/hello_c.cart -- RPU
```

See [SDK Installation](./sdks) for other platforms, verification, and toolchain overrides. The [C SDK](./c-sdk) page documents source layout and its API.

## Build For The Web

RPU can currently export a wasm/web build:

```bash
rpu build-web examples/warped_space_shooter
```

To preview it locally:

```bash
rpu serve-web examples/warped_space_shooter --port 8123
```

If the wasm target is missing, RPU will try to install it automatically with `rustup`.

If `wasm-bindgen-cli` is missing, install it with:

```bash
cargo install wasm-bindgen-cli --version 0.2.126 --locked
```

The CLI version must match RPU's `wasm-bindgen` crate version. Use `--force` to replace an older installation.

## Export For Xcode

On macOS, RPU can generate an Xcode project:

```bash
rpu export-xcode examples/warped_space_shooter
```

This requires:

- macOS
- Xcode / `xcodebuild`
- Rust / Cargo

If your cartridge defines:

```toml
[meta]
bundle_id = "org.rpu.my_game"
display_name = "My Game"
```

the Xcode export will use that metadata for the generated app.

## Next

Read the [concept](./concept), then the [cartridge manifest](./cartridges) and [system API](./system-api) pages.
