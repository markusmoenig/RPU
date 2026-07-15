---
id: system-api
title: System API
sidebar_position: 4
---

# System API

`requires.system = true` enables the baseline system service family.

Today this is most visible in CLI cartridges.

The native WASM implementation lives in the `rpu-system` service crate. `rpu-wasm` registers its namespace only when the capability is enabled. See [Runtime Services](./runtime-services) for the host architecture.

## CLI Entry Point

CLI cartridges use `on run()`:

```rpu
on run() {
    print("Hello from CLI")

    if arg_count() > 0 {
        print("first arg: " + arg(0))
    }
}
```

Run it with arguments after `--`:

```bash
rpu run examples/hello_cli -- RPU
```

## Builtins

Current CLI system builtins:

- `arg_count()`
- `arg(index)`
- `print(value)`
- `eprint(value)`
- `exit(code)`

`print(...)` writes to stdout with a newline.

`eprint(...)` writes to stderr with a newline.

`arg_count()` returns the number of arguments passed after `--`.

`arg(index)` returns the argument at the zero-based index, or an empty string when the index is out of range.

`exit(code)` stops the CLI cartridge and returns the process exit code.

## Current Limits

The CLI runner is intentionally small.

It currently supports:

- strings
- numbers
- string concatenation with `+`
- numeric binary expressions
- comparisons
- `if` / `else`
- the CLI builtins above

It does not yet expose filesystem, environment variables, random numbers, or dynamic module discovery to RPU bytecode. Declared WASM modules are loaded by the cartridge host before startup.

## WASM Mapping

The first WASM ABI maps these system builtins to host imports in the `rpu_system` namespace.

See [WASM ABI](./wasm-abi) for the exact import/export names and string passing rules.
