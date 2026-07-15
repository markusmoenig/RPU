---
id: cartridges
title: Cartridges
sidebar_position: 3
---

# Cartridges

RPU cartridges currently use a simple source layout:

```text
my_app/
  rpu.toml
  scenes/
  scripts/
  assets/
```

`rpu.toml` is still the source manifest name for compatibility. Conceptually, it is the cartridge manifest.

## Built Cartridge

`rpu build` turns a supported source project into a language-neutral runtime bundle:

```text
hello_c.cart/
  manifest.toml
  main.wasm
  assets/
  shaders/
  modules/
```

The first `.cart` format is an inspectable directory. A compressed single-file transport may be added later while preserving this logical layout.

Source projects use `rpu.toml`, which describes the source language and build backend. Built cartridges use `manifest.toml`, which contains only runtime information:

```toml
[cartridge]
format_version = 1
abi_version = 1

[project]
name = "hello_c"
version = "0.1.0"
kind = "cli"

[entry]
backend = "wasm"
path = "main.wasm"

[requires]
system = true
graphics = false
audio = false
network = false
```

Build and execute it with:

```bash
rpu build examples/hello_c
rpu run examples/hello_c/build/hello_c.cart -- RPU
```

Running the built cartridge does not invoke Clang or read the original C source. The bundle can be copied elsewhere and executed by a compatible RPU host.

The loader rejects:

- unsupported cartridge format or ABI versions
- absolute entry or module paths
- paths containing `.` or `..`
- missing entry or module files
- symbolic links anywhere inside the bundle
- imports outside declared capability families

## Manifest

```toml
[project]
name = "my_app"
version = "0.1.0"
kind = "app"

[build]
language = "rpu"
backend = "bytecode"

[requires]
system = true
graphics = true
audio = true
network = false

[window]
width = 272
height = 160
default_scale = 4.0
resize = "letterbox"
```

## Project

`project` identifies the cartridge.

Current fields:

- `name`
- `version`
- `start_scene`
- `kind`

`kind` selects the execution mode:

- `app` opens a window and runs the scene loop
- `cli` runs headless through `on run()` script handlers
- `module` is built for loading by another cartridge or host and cannot be run directly

Older manifests without `kind` default to `app`.

WASM module cartridges can be declared through `[[modules]]`. The host validates and initializes them in declaration order before a CLI cartridge starts. See [Modules](./modules).

## Build

`build` describes the frontend and execution format for the cartridge's own code.

Current fields:

- `language`
- `backend`

Current language names:

- `rpu`
- `c`
- `rust`
- `zig`
- `odin`
- `denrim`

Current backend names:

- `bytecode`
- `wasm`
- `native`

The default is:

```toml
[build]
language = "rpu"
backend = "bytecode"
```

The intended long-term shared path is:

```toml
[build]
language = "rpu"
backend = "wasm"
```

That should use the same cartridge ABI as WASM produced by C, Rust, Zig, Odin, or Denrim tooling.

## Requires

`requires` declares which runtime service families the cartridge expects.

Current fields:

- `system`
- `graphics`
- `audio`
- `network`

`system` covers baseline computer services such as script execution, command-line arguments, stdout/stderr, time, cartridge resources, and basic storage.

`graphics` covers the windowed visual surface, rendering, GPU resources and compute dispatch, input, and tiny UI. GPU kernels are portable WGSL cartridge resources; applications do not access Metal, Vulkan, Direct3D, or browser WebGPU directly.

`audio` and `network` stay separate because they are optional host services with clear platform implications.

Older manifests without `requires` still load with the legacy default capability set.

## App Cartridge

```toml
[project]
name = "paint_lab"
kind = "app"

[build]
language = "rpu"
backend = "bytecode"

[requires]
system = true
graphics = true
audio = true
network = false
```

## CLI Cartridge

```toml
[project]
name = "hello_cli"
kind = "cli"

[build]
language = "rpu"
backend = "bytecode"

[requires]
system = true
graphics = false
audio = false
network = false
```

CLI cartridges do not need `scenes/`.

## Window

`window.width` and `window.height` define the authored base resolution for app cartridges.

`window.default_scale` controls the default startup window size relative to that base resolution.

Current resize modes:

- `letterbox`
- `stretch`

## Meta

`meta` holds package metadata reused by platform exporters.

Current fields:

- `bundle_id`
- `display_name`
- `development_team`

Right now these are used by the Apple/Xcode export. `development_team` is Apple-specific in practice, but it lives in `meta` because signing and package metadata are exporter concerns.
