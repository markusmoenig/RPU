# RPU Fantasy Computer Concept

It is evolving from a 2D game language into a small creative computer.

The important shift is this:

> RPU is not the whole platform. RPU is one language that targets the platform.

The platform is the runtime. Applications are cartridges. Languages, tools, and modules are frontends that produce cartridge content for the same small machine.

---

## Core Idea

RPU should become a cross-platform, cross-language cartridge runtime for tools, apps, games, and creative modules.

Think:

- Commodore 64
- Amiga
- Pico-8

but modern:

- WebGPU-style graphics
- integrated CPU and GPU programming
- WASM modules
- hot reload
- portable cartridges
- explicit runtime capabilities
- small immediate UI
- creative tooling
- AI-assisted development

The goal is not another game engine, Qt, Electron, or native app framework.

The goal is a tiny, opinionated creative computer.

---

## The Machine

The runtime is the machine.

It exposes a small set of stable services:

- graphics
- audio
- input
- storage
- resources
- timing
- window
- UI
- networking, optional

Applications do not talk directly to Metal, Vulkan, Direct3D, or browser APIs.

Instead:

```text
Application / Module
        |
        v
Fantasy Computer API
        |
        v
Host Runtime
        |
        v
Metal / Vulkan / D3D / WebGPU / Browser
```

This keeps cartridges portable. The cartridge asks the machine for services. The host decides how those services are implemented on each platform.

---

## Integrated GPU Programming

The GPU is not only a rendering backend. It is a core programmable part of the fantasy computer.

RPU should let a cartridge use CPU code and GPU kernels within one coherent language and resource model. This should feel integrated in source code while keeping the execution boundary explicit: CPU code creates resources, supplies parameters, dispatches kernels, and decides when results are read; kernels run in parallel over textures and buffers.

```text
RPU source
  |\
  | \-- CPU code ------------> WASM ------------> machine API
  |
  \---- @compute kernels ----> WGSL ------------> graphics service
                                                       |
                                                       v
                                           Metal / Vulkan / D3D / WebGPU
```

The RPU DSL should eventually have a restricted, statically typed kernel subset for `@compute`, vertex, and fragment functions. A kernel is compiled to WGSL and packaged with the cartridge; ordinary RPU code is compiled to the same WASM ABI used by C, Rust, Zig, Odin, Denrim, and other frontends.

The runtime owns GPU devices, queues, textures, buffers, pipelines, and synchronization. Cartridges use portable handles and commands through the `graphics` service. They never target Metal directly.

This is deliberately not automatic offloading. Parallelism, dispatch, resource access, and readback must remain visible in code so that performance and data flow stay understandable.

The model should support both creative graphics and computation:

- rendering and materials
- image processing and texture tools
- mesh, voxel, and terrain operators
- simulations
- procedural generation
- GPU-accelerated Denrim operators

GPU compute belongs to `requires.graphics`; it is part of one portable GPU service, not a separate platform capability.

---

## Palette-Based Pixel Graphics

RPU should also treat indexed-color pixel graphics as a first-class part of `graphics`. This is not a separate retro capability and not merely a visual filter. A cartridge should be able to create a genuinely palette-indexed surface where each pixel stores a color index rather than a full RGBA value.

The portable model should eventually include:

- 4-bit and 8-bit indexed surfaces
- cartridge-defined and runtime-editable palettes
- a transparent palette index
- pixel, line, rectangle, sprite, and blit operations
- palette swapping without rewriting pixel data
- fixed logical resolutions and integer scaling
- nearest-neighbor presentation without filtering
- scanline or region palette changes for raster-style effects
- explicit conversion between indexed surfaces and regular GPU textures

Illustrative future RPU syntax:

```rpu
let screen = graphics.indexed_surface(320, 180, palette)
graphics.pixel(screen, x, y, color_index)
graphics.blit(screen, sprite, x, y)
graphics.present(screen)
```

The host may accelerate these operations with the modern GPU, but indexed pixels, palette changes, and integer presentation remain observable parts of the portable graphics contract. The same resources must be available through the `rpu_graphics` ABI so C, Rust, Zig, and other cartridge languages can build authentic palette-based applications and games too.

---

## Cartridges

Applications are cartridges, not native executables.

A cartridge is a packaged unit of code, data, assets, and declared capabilities.

Example:

```text
my_app.cart/
  manifest.toml
  main.wasm
  assets/
  shaders/
  modules/
  scenes/
```

The runtime loads the cartridge and provides the machine API.

The first implemented `.cart` format is an inspectable directory bundle with a normalized `manifest.toml`. Source projects use `rpu.toml`; built cartridges contain only runtime metadata and artifacts. Archive packaging may be added later without changing the logical cartridge layout.

Long term, the same cartridge should run on:

- macOS
- iPadOS
- Windows
- Linux
- browser

without modification.

---

## Cartridge Kinds

Cartridges should not all have to open a window.

The runtime should eventually support different cartridge kinds:

- `app` for windowed interactive applications
- `cli` for headless command-line tools
- `module` for code loaded by another cartridge or host

Example windowed cartridge:

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

Example CLI cartridge:

```toml
[project]
name = "mesh_optimizer"
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

A CLI cartridge still runs inside the fantasy computer. It just uses a headless host instead of a windowed host.

It should receive:

- command-line arguments
- stdout and stderr
- logging
- resources
- storage access, if declared
- time
- optional network
- optional module loading

This makes the cartridge model useful for:

- mesh conversion
- voxel processing
- terrain baking
- texture packing
- asset validation
- procedural generation
- batch export
- build steps
- Denrim operators

Example:

```bash
rpu run mesh_optimizer.cart -- input.obj output.glb
```

or eventually:

```bash
rpu exec mesh_optimizer.cart input.obj output.glb
```

CLI cartridges are important because they make portable creative tools possible without requiring a GUI. They are also one of the cleanest paths toward Denrim operators becoming reusable modules.

---

## Capabilities And Modules

The system should be modular from the beginning.

Cartridges should declare what they need:

```toml
[requires]
system = true
graphics = true
audio = true
network = false
```

This has several benefits:

- small hosts can load only the APIs they support
- security and sandboxing become easier
- cartridges are easier to inspect
- modules can be shared between tools
- Denrim operators can become portable runtime modules

The first module path loads declared WASM files in manifest order before the parent cartridge starts. Each module is an isolated instance, validated against the same ABI and limited to the parent's capability set. Direct guest-to-guest linking and shared memory are intentionally deferred; typed calls should cross host-managed interfaces instead.

Runtime services and cartridge modules are deliberately different layers. Trusted host service crates implement machine capabilities such as `system`, `graphics`, `audio`, and `network`. Sandboxed cartridge modules consume only the services granted to their parent cartridge. The host builds a service registry for each guest and installs only the ABI namespaces enabled by `[requires]`.

The first extracted service crates are `rpu-system` and `rpu-graphics`. `rpu-system` owns the `rpu_system` Wasmer imports. `rpu-graphics` owns the portable frame model and the host GPU renderer, including surfaces, textures, text rasterization, pipelines, and the WebGPU-style mapping through `wgpu`. `rpu-scenevm` now handles application lifecycle, input, audio, and window events without owning GPU implementation details.

The names belong to different layers. RPU source should expose the concise `graphics.*` API because the program is already running inside RPU. The lower-level cross-language WASM namespace is `rpu_graphics`, and the trusted Rust implementation crate is `rpu-graphics`. Those internal names must not make ordinary RPU code verbose.

Graphics ABI v1 now registers `rpu_graphics` only for cartridges with `graphics = true`. Its first immediate frame commands are `begin_frame`, `clear`, `draw_rect`, and `end_frame`; they produce the same portable frame representation used by existing DSL scenes. This deliberately proves capability gating and cross-language command submission before introducing persistent GPU resource handles.

The next graphics milestone is WASM app lifecycle execution: call `rpu_start`, drive `rpu_update`, consume each completed frame through `rpu-graphics`, and use a small C graphics cartridge as the first cross-language visual acceptance test. Textures, buffers, WGSL pipelines, compute dispatch, and readback should then grow as versioned additions to the same service.

The runtime API should feel like a tiny OS, but it should not grow into a general-purpose operating system.

Every new service should answer:

> Is this a platform capability that many languages and cartridges can use?

If yes, it belongs near the runtime API.

If no, it probably belongs in a frontend, tool, module, or library.

---

## Language Independence

The runtime must not depend on the RPU DSL.

Multiple languages should be able to target the same machine:

- RPU
- Tiny C
- Rust
- Zig
- Odin
- Denrim graphs
- future Denrim DSL

These frontends can produce:

- WASM modules
- WGSL GPU kernels
- runtime bytecode
- serialized scene data
- assets and resources

The runtime does not care which frontend produced the cartridge.

```text
RPU DSL --------+
Tiny C --------+
Rust ----------+
Zig -----------+
Denrim Graph --+
                \
                 v
              Cartridge
                 |
                 v
        Fantasy Computer Runtime
```

---

## RPU's Role

RPU remains important, but its role changes.

Old model:

```text
RPU
 |
 v
Runtime
```

New model:

```text
RPU DSL ----+
Tiny C -----+
Rust -------+
Zig --------+
Denrim -----+
             |
             v
      Cartridge Runtime
```

RPU becomes the friendly built-in language of the fantasy computer.

It can keep its convenient hybrid style:

```rpu
scene Main {
    sprite Player {
        texture = "player.png"

        on update(dt) {
            self.x = self.x + 20.0 * dt
        }
    }
}
```

That is fine because it is an authoring format, not the universal runtime model.

Internally, RPU should compile into the same runtime concepts other languages use:

- scene data
- bytecode while bootstrapping
- WASM as the long-term shared ABI
- WGSL kernels for portable GPU work
- resource references
- calls into the machine API

A C cartridge may never use scene syntax at all:

```c
#include <rpuos.h>

void app_init(void) {
    rpuos_scene_create("Main");
    rpuos_sprite_create("Player");
}

void app_update(float dt) {
    rpuos_entity_move("Player", 20.0f * dt, 0.0f);
}
```

Both approaches should run on the same machine.

---

## API Naming

The runtime API should not be named `rpu.*` if it is meant to be language-independent.

`rpu.*` makes the RPU language feel like it owns the platform.

Better options:

- `rpuos.*`
- `tinyos.*`
- `cart.*`
- another dedicated machine name

The exact name can change, but the architectural rule should remain:

> The platform API is not the RPU language API.

RPU the DSL can have its own syntax and helpers. The cartridge runtime should have a neutral machine API.

---

## Denrim

Denrim can become both a host and an authoring environment for this machine.

Examples:

Denrim Forge:

- mesh operators
- procedural generators
- material tools
- timeline operators

Denrim Voxel:

- voxel generators
- filters
- importers

Denrim Terrain:

- terrain generators
- noise operators
- erosion modules

The key idea:

> Every useful operator should eventually be able to become a portable module.

That means Denrim work does not stay trapped inside one editor. It can become part of the cartridge ecosystem.

---

## WASM

WASM is the natural portable execution format.

Advantages:

- sandboxed
- cross-platform
- browser-capable
- native-capable
- fast startup
- good fit for plugin loading
- works across many source languages

WASM should be treated as the shared cartridge execution layer, not as a replacement for the whole runtime.

The runtime still provides the machine services. WASM modules call into those services through a stable ABI. The RPU DSL should eventually be able to compile to that same ABI, so RPU code and C/Rust/Zig/Odin/Denrim code do not receive different capability models. GPU kernels are a companion portable artifact: they compile to WGSL and are dispatched through the `graphics` service, rather than attempting to execute GPU code inside WASM itself.

The first implemented cross-language path is a freestanding C CLI cartridge compiled by Clang/LLD to `wasm32-unknown-unknown` and executed by the embedded Wasmer 7 runtime. It imports only RPU services and does not require WASI.

The first ABI surface should stay deliberately small:

- `memory`
- `rpu_abi_version()`
- `rpu_alloc(...)` / `rpu_dealloc(...)`
- `rpu_run()` for CLI cartridges
- `rpu_start()` / `rpu_update(...)` / `rpu_stop()` for app cartridges
- `rpu_module_init()` for module cartridges
- `rpu_system.print(...)` and `rpu_system.eprint(...)`
- `rpu_system.arg_count(...)`, `arg_len(...)`, and `arg_read(...)`
- `rpu_system.exit(...)`
- `rpu_system.now_ms()`

Strings are UTF-8 ranges in guest memory, passed as `(ptr, len)`.

---

## UI

The runtime should provide a tiny immediate-mode UI.

Useful primitives:

- label
- button
- slider
- checkbox
- text input
- panel
- menu

This is not meant to imitate native widgets.

Creative applications can build their own look on top. The built-in UI exists so cartridges can expose simple controls, inspectors, debug panels, and tools without pulling in a large framework.

### Future HTML Renderer

In the future, the declarative RPU UI and layout model may also target HTML and CSS. HTML should be an output backend, not the core machine API.

```text
RPU UI / layout
        |
        +-- native immediate UI
        +-- HTML + CSS
        +-- canvas / WebGPU
```

This could support document-style websites, static-site export, and browser applications with optional WASM behavior. Browser hosts should use real semantic HTML elements where possible so generated sites retain accessibility, text selection, forms, navigation, and normal browser layout.

Raw DOM access should remain an explicit web-only escape hatch rather than part of the portable cartridge contract. The portable RPU model describes UI, content, layout, and interaction; each renderer decides how those concepts are represented by its host.

This is a long-term target. It should be explored only after the declarative UI and layout model has become stable enough to map cleanly to both native and web renderers.

---

## What This Is Not

RPU should not try to become:

- a full native widget toolkit
- a full desktop app framework
- a full game engine
- a general-purpose operating system
- a giant editor-first ecosystem

The strength is smallness.

The whole system should stay small enough that one person can understand it.

---

## Why This Is Worth Doing

There are many adjacent projects:

- fantasy consoles
- game engines
- creative coding runtimes
- app runtimes
- WASM plugin systems
- node-based creative tools

RPU should not compete with them on size.

The interesting identity is the intersection:

- tiny fantasy-computer runtime
- cartridge app format
- cross-language modules
- tools, apps, and games
- Denrim as host and module authoring system
- RPU DSL as friendly native language
- explicit runtime capabilities
- small, hackable, portable core

That is different from just being a 2D engine.

---

## Design Rules

1. The runtime is the platform.
2. Applications are cartridges.
3. RPU is one frontend, not the platform itself.
4. Runtime APIs should be language-neutral.
5. Capabilities belong to cartridges, not source languages.
6. Modules should be loadable and portable.
7. WASM should be the main portable code format for RPU and non-RPU frontends.
8. Scene data should remain serializable and inspectable.
9. Denrim operators should be able to become modules.
10. CLI cartridges should be first-class, not an afterthought.
11. CPU and GPU programming should share one resource model while keeping dispatch and synchronization explicit.
12. HTML and CSS may be future renderer outputs, but the DOM should not become the portable runtime API.
13. Keep the system small enough to understand.

---

## Near-Term Direction

The next version of RPU should move toward this structure gradually.

Practical steps:

1. Keep the current scene and scripting path working.
2. Remove old experimental game-specific systems from the core.
3. Introduce cartridge terminology and manifest structure.
4. Separate frontend concepts from runtime API concepts.
5. Define a first minimal runtime capability set.
6. Add a `kind` field for windowed, CLI, and module cartridges.
7. Add a `build` field for source language and backend.
8. Design a stable WASM ABI for cartridges and modules.
9. Expand declarative module loading from initialization into typed host-mediated calls.
10. Treat RPU scenes as one kind of cartridge resource.
11. Define the portable graphics API for GPU resources, kernels, and dispatch.
12. Add indexed surfaces, palettes, pixel operations, and integer-scaled presentation to `graphics`.
13. Add an RPU kernel subset that compiles to WGSL alongside CPU WASM.
14. Keep examples focused on portable apps, CLI tools, and modules, not engine features.

The guiding question for every new feature:

> Does this improve the tiny creative computer, or only the RPU language?

Both can matter, but they belong in different layers.

---

## One-Line Summary

RPU is becoming a tiny creative computer where portable cartridges use one coherent CPU and GPU model through a small stable runtime API.
