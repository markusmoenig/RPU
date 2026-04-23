---
id: getting-started
title: Getting Started
sidebar_position: 1
---

# Getting Started

## Install Rust

RPU is built with Rust. Install the current stable toolchain first:

- [Install Rust with rustup](https://www.rust-lang.org/tools/install)

## Create Or Run A Project

Current starting points:

```bash
rpu new my_game
rpu run path/to/project
```

For the maintained example:

```bash
rpu run examples/warped_space_shooter
```

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
cargo install wasm-bindgen-cli
```

## Export For Xcode

On macOS, RPU can generate an Xcode project:

```bash
rpu export-xcode examples/warped_space_shooter
```

This requires:

- macOS
- Xcode / `xcodebuild`
- Rust / Cargo

If your project defines:

```toml
[meta]
bundle_id = "org.rpu.my_game"
display_name = "My Game"
```

the Xcode export will use that metadata for the generated app.

## Next

More setup steps will go here.
