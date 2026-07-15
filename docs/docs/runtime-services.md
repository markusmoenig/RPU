---
id: runtime-services
title: Runtime Services
sidebar_position: 4
---

# Runtime Services

Runtime services are the privileged capabilities of the RPU computer.

They are different from cartridge modules:

| Runtime services | Cartridge modules |
| --- | --- |
| Implement machine capabilities | Implement application or tool functionality |
| Trusted host code | Sandboxed cartridge code |
| Expose ABI namespaces such as `rpu_system` | Export cartridge module interfaces |
| Enabled through `[requires]` | Declared through `[[modules]]` |
| May have different host implementations | Remain portable WASM or bytecode artifacts |

## Service Registry

The host creates a service registry for each WASM instance. It registers only the service families enabled by the cartridge manifest.

```toml
[requires]
system = true
graphics = false
audio = false
network = false
```

With this declaration, the host registers `rpu_system.*`. It does not register graphics, audio, or network imports. A guest that imports a disabled or unknown service is rejected before its lifecycle starts.

```text
WASM cartridge or module
          |
          v
   service registry
          |
          +-- rpu_system
          +-- rpu_graphics
          +-- rpu_audio     (future)
          +-- rpu_network   (future)
```

## Current Implementation

The first service crate is `rpu-system`. It owns the `rpu_system` namespace implementation:

- command-line arguments
- stdout and stderr
- process-style exit codes
- monotonic time
- safe access to guest memory for strings and argument buffers

`rpu-wasm` owns WASM compilation, lifecycle validation, instance creation, and the capability-driven registry. System behavior no longer lives inside the WASM executor.

The second service crate is `rpu-graphics`. It currently owns:

- the portable frame and 2D draw-command model
- GPU surfaces, devices, queues, textures, and pipelines
- image loading and text rasterization
- native, Apple layer, and browser canvas rendering through `wgpu`

`rpu-scenevm` consumes this crate while retaining application lifecycle, input, audio, and window event handling. The same crate now implements the capability-gated `rpu_graphics` WASM namespace.

Graphics ABI v1 begins with a small immediate frame surface:

- `begin_frame(width, height)`
- `clear(red, green, blue, alpha)`
- `draw_rect(x, y, width, height, red, green, blue, alpha)`
- `end_frame()`

These commands produce the same portable frame type consumed by the existing renderer. WASM app lifecycle execution is still pending, so the service can record guest frames but the host does not yet open a window for a WASM app cartridge.

These names describe separate layers:

| Layer | Name |
| --- | --- |
| RPU DSL | `graphics.*` |
| Cross-language WASM imports | `rpu_graphics` |
| Trusted Rust service crate | `rpu-graphics` |

RPU authors should see the short `graphics` API. The prefixed name exists only to keep the shared WASM ABI explicit and collision-resistant.

Runtime service crates are statically linked into the native host today. The registry still installs them per cartridge, so a disabled service has no guest-visible namespace. Dynamic host-service loading can be considered later without changing the cartridge ABI.

## Tiny Kernel

A small execution kernel remains built into the host. It validates the cartridge format and ABI, creates guest instances, attaches memory, and invokes lifecycle exports.

Everything that represents a useful computer capability should live behind a service family. Graphics, including GPU rendering and compute, should follow the same registration model as system.
