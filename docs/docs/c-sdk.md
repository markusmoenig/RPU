---
id: c-sdk
title: C SDK
sidebar_position: 7
---

# C SDK

RPU can compile a freestanding C CLI cartridge to WebAssembly and execute it through the RPU ABI.

```text
C source -> Clang / LLVM -> wasm-ld -> main.wasm -> Wasmer -> RPU services
```

This path targets `wasm32-unknown-unknown`. It does not use WASI, Emscripten, a browser JavaScript runtime, or a native operating-system API.

## Installation

The C SDK currently ships in the RPU source tree and does not require a separate SDK installer:

```text
sdk/c/
  include/rpu.h
  src/rpu.c
```

It requires Clang/LLVM and LLD. Follow [SDK Installation](./sdks) for macOS, Linux, and Windows setup, verification commands, and toolchain overrides.

RPU automatically checks common LLVM locations and `clang` on `PATH`. Set `RPU_CLANG` to select another compiler executable or `RPU_C_SDK` when using an SDK directory outside the source tree.

## Cartridge Layout

```text
hello_c/
  rpu.toml
  src/
    main.c
```

```toml
[project]
name = "hello_c"
kind = "cli"

[build]
language = "c"
backend = "wasm"

[requires]
system = true
graphics = false
audio = false
network = false
```

## Entry Point

A C CLI cartridge implements `rpu_main`:

```c
#include <rpu.h>

int32_t rpu_main(void) {
    rpu_print("Hello from C");
    return 0;
}
```

The SDK supplies the exported ABI lifecycle, memory allocation, and system import declarations.

A C module cartridge uses `kind = "module"` and implements `rpu_module_main` instead:

```c
#include <rpu.h>

int32_t rpu_module_main(void) {
    rpu_print("Module initialized");
    return 0;
}
```

The build selects the module SDK lifecycle automatically and exports `rpu_module_init()`.

Current helpers:

- `rpu_arg_count()`
- `rpu_arg_len(index)`
- `rpu_arg_read(index, buffer, capacity)`
- `rpu_print(text)`
- `rpu_eprint(text)`
- `rpu_exit(code)`
- `rpu_now_ms()`

`rpu_arg_read` writes a null-terminated string when the buffer has capacity. Strings passed across the underlying ABI remain UTF-8 `(ptr, len)` ranges.

## Build And Run

```bash
rpu build examples/hello_c
rpu run examples/hello_c -- RPU
```

The build artifact is:

```text
examples/hello_c/build/hello_c.cart/
  manifest.toml
  main.wasm
```

Running the source directory rebuilds C before execution. Running `hello_c.cart` directly does not require Clang, the C SDK, or source files.

```bash
rpu run examples/hello_c/build/hello_c.cart -- RPU
```

The WASM host validates required exports, rejects undeclared or unsupported imports, supplies only declared capability families, and checks the ABI version before calling `rpu_run` or `rpu_module_init`.

## Graphics Frame API

The C SDK exposes the first Graphics ABI v1 commands:

```c
rpu_graphics_begin_frame(320, 180);
rpu_graphics_clear(0.05f, 0.06f, 0.09f, 1.0f);
rpu_graphics_draw_rect(24.0f, 24.0f, 96.0f, 48.0f,
                       0.2f, 0.8f, 0.5f, 1.0f);
rpu_graphics_end_frame();
```

The cartridge must declare `graphics = true`. These calls already target the real `rpu_graphics` host service, but windowed WASM app execution is not implemented yet. The first visual C cartridge will arrive with that lifecycle runner.

## Current Scope

The SDK is deliberately freestanding. It does not yet provide a full C standard library, filesystem access, formatted printing, persistent GPU resources, or typed module calls. Those should grow as small RPU services and libraries rather than introducing an implicit WASI dependency.
