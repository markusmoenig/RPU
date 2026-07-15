---
id: gpu
title: GPU Architecture
sidebar_position: 5
---

# GPU Architecture

The GPU is a core programmable service of the RPU fantasy computer.

It is not a platform-specific escape hatch and not only the renderer's internal implementation. Cartridges can create GPU resources, run portable kernels, and use their results in tools, apps, games, and Denrim operators.

## Naming

RPU programs use the compact `graphics.*` API. They do not spell the platform name again inside every call.

The lower-level WASM import namespace is `rpu_graphics`, allowing C, Rust, Zig, and other frontends to target the same ABI without ambiguous imports. The trusted host implementation lives in the Rust crate `rpu-graphics`. These are implementation-layer names, not extra user-facing APIs.

Today `rpu-graphics` owns the frame model and renderer used by DSL scenes. Graphics ABI v1 also gives WASM guests capability-gated `begin_frame`, `clear`, `draw_rect`, and `end_frame` imports. Windowed WASM app lifecycle execution is planned next.

## One Language, Two Execution Domains

RPU should provide a coherent language surface for CPU code and GPU code, with an explicit boundary between them.

```text
RPU cartridge source
  |
  +-- ordinary code ----------> WASM ----------> system / graphics API
  |
  +-- @compute / shader code -> WGSL ----------> graphics pipeline
                                                     |
                                                     v
                                      Metal / Vulkan / D3D / WebGPU
```

Ordinary code controls the cartridge: it loads resources, creates textures and buffers, supplies parameters, dispatches work, and decides when to read results. GPU kernels execute in parallel over explicit resources.

The source language should make this pleasant, but it must not pretend that CPU and GPU execution are identical. Dispatch, resource access, synchronization, and readback remain visible.

## Portable Kernel Format

RPU kernels compile to WGSL. The runtime then maps them through its WebGPU-style graphics implementation to the host backend.

This means cartridges do not contain Metal source and do not call Metal, Vulkan, Direct3D, or browser WebGPU APIs directly. WGSL is the portable cartridge artifact; the graphics service is the stable runtime contract.

## Planned Kernel Subset

The RPU DSL should eventually support a restricted, statically typed kernel subset for:

- `@compute` functions
- vertex functions
- fragment functions
- scalar and vector math
- texture and buffer access
- uniforms and push-style parameters

It should reject features that cannot be represented safely and portably in WGSL, rather than allowing a kernel to depend on one native GPU backend.

Illustrative future syntax:

```rpu
@compute(workgroup_size = (8, 8))
fn tint(target: storage_texture2d, color: vec4) {
    let pixel = target.read(gid.xy)
    target.write(gid.xy, pixel * color)
}

on run() {
    let texture = graphics.texture(viewport_size())
    graphics.dispatch(tint, texture, color)
}
```

This is an architectural example, not implemented DSL syntax.

## Resource Model

The host runtime owns GPU devices, queues, pipelines, textures, buffers, and synchronization. A cartridge receives portable handles and uses commands exposed by `graphics`.

The resource model should cover:

- textures, samplers, and render targets
- storage, vertex, index, and uniform buffers
- shader and compute pipelines
- command encoding and dispatch
- render passes
- explicit synchronization and readback

CPU and GPU code should refer to the same logical cartridge resources. Moving data between their execution domains must be deliberate and observable.

## Capabilities And Packaging

GPU work is enabled by:

```toml
[requires]
graphics = true
```

Compute is part of the graphics service, rather than a separate capability. A cartridge can package WGSL artifacts under `shaders/` alongside its WASM code, scenes, assets, and modules.

## Why It Matters

This makes RPU useful for more than a visual scene runtime:

- image processing and texture tools
- materials and rendering
- simulations
- mesh, voxel, and terrain processing
- procedural generation
- GPU-accelerated Denrim operators

It is a defining RPU idea: a small portable computer where creative code can naturally span CPU and GPU without being locked to a single graphics API.
