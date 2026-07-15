# Examples

The projects in this directory are maintained alongside the current `rpu-core`,
`rpu-runtime`, and `rpu-scenevm` compiler/runtime path.

Use them as smoke-test projects while evolving the DSL and renderer.

`hello_shapes`
- Declaration-focused example for `rect`, `sprite`, camera, colors, layering, and simple entity-bound scripting.

`hello_cli`
- Headless cartridge example for the first CLI execution path.

`hello_c`
- Freestanding C cartridge compiled with Clang to WASM and executed through the RPU ABI.

`hello_module` and `hello_with_module`
- A freestanding C WASM module and a CLI cartridge that loads it before starting its main entry point.

`warped_space_shooter`
- Small game-focused example for keyboard movement, runtime query calls, textured sprites, and a concrete “progress target” built from the bundled shooter art reference.
