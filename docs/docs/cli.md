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
