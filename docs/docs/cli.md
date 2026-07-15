---
id: cli
title: CLI
sidebar_position: 3
---

# CLI

The top-level crate is `rpu`.

Current commands:

```bash
rpu new my_app
rpu run path/to/cartridge
rpu run path/to/cli-cartridge -- arg1 arg2
rpu build path/to/cartridge
rpu build-web path/to/cartridge
rpu serve-web path/to/cartridge --port 8123
rpu export-xcode path/to/cartridge --output /tmp/apple-export
```

## `rpu new`

Creates a new cartridge source directory with:

- `rpu.toml`
- `scenes/main.rpu`
- `scripts/main.rpu`
- `assets/`

## `rpu run`

Runs a cartridge.

Current behavior:

- loads the cartridge source directory
- compiles scenes and scripts
- for `kind = "app"`, opens a window, renders the scene, and hot reloads source changes
- for `kind = "cli"`, runs headless `on run()` script handlers
- for C/WASM source cartridges, compiles `src/*.c`, packages a `.cart` directory, and runs it with Wasmer
- for built `.cart` directories, runs the packaged entry directly without a language toolchain

CLI cartridge arguments are passed after `--` and are available through `arg_count()` and `arg(index)`.

## `rpu build-web`

Builds a browser export for an app cartridge.

Preflight behavior:

- checks that `cargo` and `rustup` are available
- automatically installs the missing Rust target with:
  - `rustup target add wasm32-unknown-unknown`
- requires `wasm-bindgen-cli`
  - if missing, install it with:
    - `cargo install wasm-bindgen-cli --version 0.2.126 --locked`
  - use `--force` to replace an older version

Current output goes to:

```text
build/web/
```

This currently emits:

- `index.html`
- wasm-bindgen JS glue
- `.wasm`
- copied/bundled cartridge scenes, scripts, and assets through the generated launcher
- a generated hidden launcher crate under `build/web/.app`

The generated web build is self-contained and suitable for local preview or embedding into a website.

## `rpu serve-web`

Builds the web export and serves it locally.

Example:

```bash
rpu serve-web examples/warped_space_shooter --port 8123
```

This is useful for checking wasm/browser behavior without wiring your own local server.

The local server:

- serves the generated `build/web/` output
- uses the authored project resolution and responsive browser fitting
- is the easiest way to validate web input, rendering, and audio behavior during development

## `rpu build`

For RPU bytecode cartridges, current build output is a placeholder cartridge summary written to:

```text
build/BUILD.txt
```

It currently reports:

- scene count
- script count
- draw counts
- handler/op counts
- diagnostics

For C/WASM cartridges, `rpu build` invokes a WebAssembly-capable Clang and LLD and writes:

```text
build/<name>.cart/
  manifest.toml
  main.wasm
```

The generated `.cart` directory is the portable runtime artifact. `build/main.wasm` may exist as an intermediate compiler output and is not the cartridge contract.

See [SDK Installation](./sdks) for toolchain setup and [C SDK](./c-sdk) for source layout.

## `rpu export-xcode`

Exports a native Apple project that uses the generated Xcode host plus the Rust renderer through FFI.

Current output goes to:

```text
build/apple/
```

Current preflight behavior:

- requires macOS
- requires `xcodebuild`
- requires `cargo`

Example:

```bash
rpu export-xcode examples/warped_space_shooter
```

If you run it on a non-macOS machine, it fails early with a clear message instead of trying to build the bridge anyway.

The generated export currently includes:

- `App/`
- `RustBridge/`
- `Project/`
- `RPUAppleApp.xcodeproj/`
- `RPUAppleTVApp.xcodeproj/`
- `tvOS-Info.plist`

The generated macOS host is a native AppKit app. The generated tvOS host is a native UIKit app using the `UIScene` lifecycle. Both create a native `CAMetalLayer`, while Rust renders into that surface through FFI using the same renderer as the normal desktop runtime.

Current metadata and sizing behavior:

- uses `[meta].display_name` for the app display name
- uses `[meta].bundle_id` for the bundle identifier
- uses `[meta].development_team` for generated Xcode signing settings when present
- uses `[window].width`, `[window].height`, and `[window].default_scale` for the startup content size

For tvOS builds, Xcode runs `RustBridge/build-rust.sh` during the app build. If a required Rust target is missing, install it with the exact command printed by the build log. Common targets are:

```bash
rustup target add aarch64-apple-tvos
rustup target add aarch64-apple-tvos-sim
```

tvOS remote and controller input is normalized to the same RPU keys used elsewhere. Directional input maps to movement keys, while action input maps to `Space`. Audio uses the Apple host bridge, so sound effects and background music work in the generated tvOS app as well as on desktop.
