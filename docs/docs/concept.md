---
id: concept
title: Concept
sidebar_position: 2
---

# Fantasy Computer

RPU is evolving into a tiny creative computer for portable cartridges.

The runtime is the machine. A cartridge is the software unit. The RPU DSL is one friendly frontend for that machine, not the whole platform.

```text
RPU DSL       C / Rust / Zig       Denrim graphs
   \              |                    /
    \             |                   /
          cartridge resources
                  |
                  v
        RPU cartridge runtime
                  |
                  v
  system / graphics / audio / network
```

The GPU is a core programmable part of this machine, not only the implementation of drawing. CPU code and GPU kernels share cartridge resources and the portable `graphics` service.

```text
CPU code ----------------> WASM ------------> RPU runtime API
GPU kernels -------------> WGSL ------------> graphics service
                                                 |
                                                 v
                                     Metal / Vulkan / D3D / WebGPU
```

## Current Reality

Today RPU supports:

- app cartridges that open a window and run scene/script code
- CLI cartridges that run headless `on run()` script handlers
- a compact cartridge manifest in `rpu.toml`
- build declarations through `[build]`
- capability declarations through `[requires]`
- a capability-driven host service registry with `rpu-system` providing the first guest ABI
- an extracted `rpu-graphics` host service owning frame commands and the cross-platform GPU renderer
- capability-gated `rpu_graphics` WASM frame commands for cross-language drawing
- isolated WASM modules initialized in manifest order before CLI cartridge startup
- freestanding C CLI cartridges compiled to WASM and executed through the RPU ABI
- inspectable `.cart` directory artifacts that run independently of source projects and language toolchains
- web and Xcode export for app cartridges

## Direction

The long-term direction is:

- cartridges instead of native app projects
- `system`, `graphics`, `audio`, and `network` as runtime service families
- separate trusted runtime service crates and sandboxed cartridge modules
- integrated CPU/GPU programming: CPU code targets WASM; GPU kernels target WGSL
- explicit GPU resource creation, dispatch, synchronization, and readback through `graphics`
- RPU bytecode for fast native DSL scripting while bootstrapping
- WASM as the shared long-term execution ABI for RPU, C, Rust, Zig, Odin, Denrim, and other frontends
- a small ABI surface starting with lifecycle exports, memory/string passing, CLI args, stdout/stderr, exit codes, and time
- optional native modules for trusted host extensions
- Denrim operators that can become portable modules
- HTML and CSS as a future output backend for declarative RPU UI and layout

The goal is not a full game engine or a desktop app framework.

The goal is a small, understandable, hackable creative computer.

## GPU Rule

The RPU DSL should eventually provide a typed kernel subset for compute, vertex, and fragment functions. Kernels are packaged as WGSL resources and dispatched by ordinary cartridge code.

This should be seamless in language design, but explicit in execution: the compiler must not silently move arbitrary CPU code onto the GPU. Dispatch, resource access, and readback remain visible so cartridges stay portable and performance remains understandable.

GPU compute is part of `requires.graphics`, not a separate capability. A cartridge never talks directly to Metal, Vulkan, Direct3D, or browser WebGPU APIs.

See [GPU Architecture](./gpu) for the intended contract.

## Future HTML Target

Once the declarative UI and layout model is stable, RPU may render it through multiple backends:

```text
RPU UI / layout
        |
        +-- native immediate UI
        +-- HTML + CSS
        +-- canvas / WebGPU
```

This would allow RPU to produce document-style websites, static sites, and browser applications with optional WASM behavior. HTML is an output format, not the machine API; raw DOM access would be an explicit web-only escape hatch rather than part of the portable cartridge contract.

This is a future target, not a current implementation commitment.

## Design Rule

Every new feature should answer two questions:

- Does this improve the tiny computer?
- Or is this only convenience for the RPU DSL frontend?

Both can matter, but they belong in different layers.
