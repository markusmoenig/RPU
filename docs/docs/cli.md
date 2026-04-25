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
rpu run path/to/project
rpu build path/to/project
rpu build-web path/to/project
rpu serve-web path/to/project --port 8123
rpu export-xcode path/to/project --output /tmp/apple-export
```

## `rpu new`

Creates a new project with:

- `rpu.toml`
- `scenes/main.rpu`
- `scripts/main.rpu`
- `assets/`

## `rpu run`

Runs the project in a native window using `rpu-runtime` and `rpu-scenevm`.

Current behavior:

- loads the project
- compiles scenes and scripts
- opens a window
- renders the scene
- polls for source changes and hot reloads

## `rpu build-web`

Builds a browser export for a project.

Preflight behavior:

- checks that `cargo` and `rustup` are available
- automatically installs the missing Rust target with:
  - `rustup target add wasm32-unknown-unknown`
- requires `wasm-bindgen-cli`
  - if missing, install it with:
    - `cargo install wasm-bindgen-cli`

Current output goes to:

```text
build/web/
```

This currently emits:

- `index.html`
- wasm-bindgen JS glue
- `.wasm`
- copied/bundled project scenes, scripts, and assets through the generated launcher
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

Current build output is a placeholder build summary written to:

```text
build/BUILD.txt
```

It currently reports:

- scene count
- script count
- draw counts
- handler/op counts
- diagnostics

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
