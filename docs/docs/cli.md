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

Current Apple export output is also a placeholder.

It prepares an output directory and writes a summary README for the future Apple flow.
