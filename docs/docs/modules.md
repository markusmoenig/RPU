---
id: modules
title: Modules
sidebar_position: 5
---

# Modules

Modules are portable code units loaded by a cartridge.

These cartridge modules are distinct from trusted [runtime service crates](./runtime-services). A cartridge module consumes granted services; it does not implement privileged APIs such as `rpu_system` or `rpu_graphics`.

The first implemented path loads WASM modules in manifest order before a CLI cartridge starts. Each module is an independent Wasmer instance and receives the parent cartridge's declared capability set.

The cartridge build target and module backends should share one runtime ABI. A cartridge written in the RPU DSL and compiled to WASM should not receive a different API from a cartridge or module produced by C, Rust, Zig, Odin, or Denrim tooling.

```toml
[[modules]]
name = "mesh_simplifier"
backend = "wasm"
path = "modules/mesh_simplifier.wasm"
```

Current backend names:

- `bytecode`
- `wasm`
- `native`

Only `wasm` modules are executable today. Declaring another backend produces an explicit runtime error.

## Loading

A WASM module must implement the common ABI exports plus:

```text
rpu_module_init() -> i32
```

The host checks the module's imports, required exports, and ABI version before calling its initializer. `0` means success; any other result stops cartridge startup with an error. Duplicate module names, missing files, unsupported backends, and invalid module paths are rejected.

Modules remain alive for the duration of the parent cartridge run. Version one deliberately has no dynamic linking, shared linear memory, or direct calls between WASM guests. Future module APIs will use typed, host-mediated handles and calls.

## C Example

The source module implements `rpu_module_main`:

```c
#include <rpu.h>

int32_t rpu_module_main(void) {
    rpu_print("Hello from the loaded module");
    return 0;
}
```

Build it and place the resulting WASM file at the path declared by the parent:

```bash
rpu build examples/hello_module
cp examples/hello_module/build/main.wasm examples/hello_with_module/modules/hello_module.wasm
rpu run examples/hello_with_module
```

## Intended Roles

`bytecode` is for RPU DSL scripts compiled into the native bytecode VM.

`wasm` is for portable modules produced by languages such as C, Rust, Zig, Odin, and future DSLs. It is the first executable module backend.

`native` is for trusted host extensions that need platform-specific access or maximum performance.

## Direction

The runtime API is the stable contract. Capabilities belong to the cartridge manifest, not to the source language.

WASM is one backend that can call that API. It should not become the whole platform, and RPU bytecode should not become a privileged separate platform.

```text
RPU bytecode VM
WASM engine
trusted native modules
        |
        v
system / graphics / audio / network
```

This lets RPU keep fast hot-reloadable scripting while still supporting cross-language modules.

Long term, this should also be valid for the RPU DSL itself:

```toml
[build]
language = "rpu"
backend = "wasm"
```

See [WASM ABI](./wasm-abi) for the first shared cartridge/module contract.
