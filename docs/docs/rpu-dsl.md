---
id: rpu-dsl
title: RPU DSL
sidebar_position: 6
---

# RPU DSL

The RPU DSL is the friendly native frontend for the cartridge runtime.

It currently provides:

- declarative scenes for app cartridges
- bytecode scripts for behavior
- inline handlers inside visual nodes
- headless `on run()` scripts for CLI cartridges

The DSL is not the whole platform. It is one way to produce cartridge resources.

Today the DSL compiles to RPU bytecode. Long term its CPU code should also target WASM so it uses the same cartridge ABI as C, Rust, Zig, Odin, Denrim graphs, and other frontends. Its restricted GPU kernel subset should target WGSL and use the same portable `graphics` service.

As a later target, declarative RPU UI and layout may also compile to semantic HTML and CSS for websites and browser applications. HTML would be a renderer output rather than part of the portable runtime API.

Other frontends can eventually target the same runtime:

- Tiny C
- Rust
- Zig
- Odin
- Denrim graphs
- future Denrim DSLs

## App Example

```rpu
scene Main {
    sprite Hero {
        texture = "hero.png"

        on update(dt) {
            if input_left() {
                self.x = self.x - 120.0 * dt
            }
        }
    }
}
```

## CLI Example

```rpu
on run() {
    print("Hello from CLI")
}
```

## GPU Direction

RPU will eventually use one language surface for ordinary CPU code and explicitly marked GPU kernels. CPU code creates resources and dispatches work; typed `@compute`, vertex, and fragment functions compile to WGSL.

This is a planned language capability, not current DSL syntax. See [GPU Architecture](./gpu).

## References

Use these pages for the current DSL details:

- [Scenes](./scenes)
- [Scripts](./scripts)
