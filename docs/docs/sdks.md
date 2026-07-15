---
id: sdks
title: SDKs
sidebar_position: 7
---

# SDKs

Language SDKs connect cartridge source code to the stable RPU ABI.

An SDK provides language-friendly functions, lifecycle glue, memory conventions, and build integration. It does not replace the runtime and does not grant extra capabilities: every language uses the service families declared in `rpu.toml`.

```text
source language -> language SDK -> WASM ABI -> RPU host services
```

## Current SDKs

| SDK | Status | Output | Additional toolchain |
| --- | --- | --- | --- |
| RPU DSL | Built in | RPU bytecode | None |
| C | Available for CLI cartridges | WASM | Clang/LLVM and LLD |
| Rust, Zig, Odin | Planned | WASM | Not integrated yet |

The C SDK is the first cross-language implementation. It targets `wasm32-unknown-unknown` and imports the RPU API directly, without WASI.

## Install RPU From Source

Install the current Rust toolchain, then build and install the CLI from the repository checkout:

```bash
git clone https://github.com/markusmoenig/RPU.git
cd RPU
cargo install --path .
```

Cargo places `rpu` in its binary directory, normally `~/.cargo/bin`. Make sure that directory is on `PATH`.

Verify the CLI:

```bash
rpu --help
```

The language SDK files currently ship inside the repository. There is no separate SDK installer yet.

## Install The C Toolchain

The C SDK requires an upstream Clang with the WebAssembly target and the LLD WebAssembly linker.

### macOS

Apple's bundled Clang does not provide the WebAssembly target. Install LLVM and LLD with Homebrew:

```bash
brew install llvm lld
```

RPU checks the Homebrew LLVM location automatically, so changing the shell's default compiler is not required.

### Linux

Install Clang and LLD with the system package manager. On Debian and Ubuntu:

```bash
sudo apt install clang lld
```

Both `clang` and `wasm-ld` must be available on `PATH`.

### Windows

Install an upstream LLVM distribution that includes Clang and LLD, then add its `bin` directory to `PATH`. Confirm that both `clang.exe` and `wasm-ld.exe` are available in a new terminal.

## Verify The Toolchain

Check that Clang lists the WebAssembly target:

```bash
clang --print-targets
```

The output must include `wasm32`. Also check the linker:

```bash
wasm-ld --version
```

Finally, build and run the example cartridge:

```bash
rpu build examples/hello_c
rpu run examples/hello_c -- RPU
rpu run examples/hello_c/build/hello_c.cart -- RPU
```

Expected output:

```text
Hello from a C cartridge
First argument:
RPU
```

The final command executes the packaged cartridge and does not require Clang or the C SDK. This separates language-specific build infrastructure from the portable runtime artifact.

## Toolchain Overrides

RPU supports these environment variables:

| Variable | Purpose |
| --- | --- |
| `RPU_CLANG` | Path to a WebAssembly-capable Clang executable |
| `RPU_C_SDK` | Path to an alternate C SDK directory containing `include/` and `src/` |

Example:

```bash
RPU_CLANG=/custom/llvm/bin/clang \
RPU_C_SDK=/custom/rpu-sdk/c \
rpu build my_cartridge
```

## Versioning Direction

SDK versions should follow the RPU WASM ABI they target. As distribution matures, each supported language should receive a versioned SDK package and reproducible toolchain setup. Until then, SDK files from the same RPU checkout should be used with its CLI and runtime.

Continue with the [C SDK](./c-sdk) for source layout, entry points, and the current API.
