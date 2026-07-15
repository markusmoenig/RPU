# RPU

RPU is a tiny creative computer for portable cartridges.

RPU is evolving into a cross-platform cartridge runtime for tools, apps, games, and creative modules. The RPU DSL remains the friendly built-in authoring language, but the runtime is the platform: cartridges should eventually be buildable from RPU, C, Rust, Zig, Denrim graphs, or other frontends.

Trusted runtime service crates provide capabilities such as system and graphics. `rpu-system` provides the baseline WASM API, while `rpu-graphics` owns the portable frame model, the first capability-gated `rpu_graphics` frame commands, and the cross-platform GPU renderer. Sandboxed cartridges and modules consume only the services granted by their manifests.

## Run a Cartridge

RPU cartridges declare their execution kind and required runtime service families.

Example CLI cartridge manifest:

```toml
[project]
name = "hello_cli"
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

Then run it:

```bash
rpu run examples/hello_cli -- RPU
```

Freestanding C cartridges compile to WASM through Clang/LLD and run against the same RPU system API:

```bash
rpu run examples/hello_c -- RPU
```

Build a portable cartridge directory, then run it without the C source or compiler:

```bash
rpu build examples/hello_c
rpu run examples/hello_c/build/hello_c.cart -- RPU
```

WASM modules can be declared inside a cartridge and are initialized before its main entry point. See [the module example](examples/hello_with_module/README.md).

## Use the RPU DSL

The RPU DSL is the friendly built-in frontend. It can describe app scenes or headless CLI tools.

```rpu
on run() {
    print("Hello from an RPU CLI cartridge")

    if arg_count() > 0 {
        print("first arg: " + arg(0))
    }
}
```

App cartridges can also use declarative scenes and inline scripts. C CLI cartridges already compile to WASM; future RPU/Rust/Zig WASM paths will use the same runtime service model.

Docs: https://rpu-lang.org

Concept: [concept.md](concept.md)

CLI cartridge example:

```bash
rpu run examples/hello_cli
rpu run examples/hello_cli -- RPU
```
