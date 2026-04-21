# RPU

RPU is a language and runtime for interactive 2D games and apps.

It combines:

- declarative scene files
- lightweight scripting compiled to bytecode
- a CLI-first workflow for creating, running, and building projects

Current repo layout:

- `src/`: top-level `rpu` CLI crate
- `crates/`: core runtime, scene/runtime backend, and build support
- `examples/`: maintained example projects
- `docs/`: the `rpu-lang.org` documentation site

Useful commands:

```bash
cargo run -- new my_game
cargo run -- run examples/warped_space_shooter
cargo run -- build examples/warped_space_shooter
```

Docs: `https://rpu-lang.org`
