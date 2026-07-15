---
id: wasm-abi
title: WASM ABI
sidebar_position: 6
---

# WASM ABI

The WASM ABI is the shared execution contract for cartridges and modules.

RPU bytecode remains useful for fast local iteration, but WASM is the long-term portable path for RPU, C, Rust, Zig, Odin, Denrim, and future frontends.

The native host currently executes CLI WASM cartridges and modules with the embedded Wasmer 7 runtime. These cartridges target the RPU ABI directly and do not require WASI.

## Version

The first ABI version is:

```text
1
```

Every WASM cartridge must export:

```text
rpu_abi_version() -> i32
```

The host expects this to return `1`.

## Memory And Strings

WASM cartridges export linear memory as:

```text
memory
```

Strings are passed as UTF-8 byte ranges inside guest memory:

```text
ptr: i32
len: i32
```

Strings are not null-terminated.

The host never owns guest memory directly. The guest must export:

```text
rpu_alloc(len: i32, align: i32) -> i32
rpu_dealloc(ptr: i32, len: i32, align: i32)
```

## CLI Lifecycle

CLI cartridges export:

```text
rpu_run() -> i32
```

The return value is the cartridge exit code.

Command-line arguments are provided through system imports rather than through `rpu_run` parameters.

## App Lifecycle

App cartridges export:

```text
rpu_start() -> i32
rpu_update(dt: f32) -> i32
rpu_stop()
```

`dt` is elapsed seconds since the previous update.

The integer return values are status codes. `0` means continue/success.

## Module Lifecycle

Module cartridges export:

```text
rpu_module_init() -> i32
```

The host calls module initializers in manifest order before the main cartridge lifecycle starts. `0` means success. A nonzero status aborts startup.

Each module currently runs as an isolated WASM instance. It inherits the parent manifest's capabilities but does not share memory or dynamically link against the parent or other modules. Future module-specific calls will cross a typed host-mediated boundary.

## System Imports

When `[requires].system = true`, the host provides imports from:

```text
rpu_system
```

Current imports:

```text
arg_count() -> i32
arg_len(index: i32) -> i32
arg_read(index: i32, ptr: i32, len: i32) -> i32
print(ptr: i32, len: i32)
eprint(ptr: i32, len: i32)
exit(code: i32)
now_ms() -> i32
```

`arg_len` returns the UTF-8 byte length of an argument, or `0` for an out-of-range index.

`arg_read` writes up to `len` bytes into guest memory and returns the number of bytes written.

`print` and `eprint` write UTF-8 text to stdout and stderr. They do not require null terminators.

`exit` stops the cartridge with the given process-style exit code.

`now_ms` returns host monotonic time in milliseconds, truncated to `i32` for ABI version 1.

## Graphics Imports

When `[requires].graphics = true`, the host provides imports from:

```text
rpu_graphics
```

Graphics ABI v1 starts with an immediate frame command surface:

```text
begin_frame(width: i32, height: i32)
clear(red: f32, green: f32, blue: f32, alpha: f32)
draw_rect(x: f32, y: f32, width: f32, height: f32,
          red: f32, green: f32, blue: f32, alpha: f32)
end_frame()
```

Commands must occur between `begin_frame` and `end_frame`. Dimensions must be positive, rectangle sizes cannot be negative, and all floating-point arguments must be finite. Colors are linear RGBA values clamped to `0...1` by the host.

The completed command stream becomes the same portable frame representation used by RPU DSL scenes. Textures, buffers, pipelines, kernels, compute dispatch, and readback are not part of this first surface.

## Capability Rule

The source language does not decide which APIs are available.

The manifest does:

```toml
[requires]
system = true
graphics = false
audio = false
network = false
```

That same capability set applies to RPU-generated WASM and WASM produced by C, Rust, Zig, Odin, Denrim, or another frontend.

See [SDKs](./sdks) for installation and [C SDK](./c-sdk) for the first implemented cross-language API.
