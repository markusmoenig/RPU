# Hello With Module

Build the C module, copy its portable artifact into the parent cartridge, then run the parent:

```bash
rpu build examples/hello_module
cp examples/hello_module/build/main.wasm examples/hello_with_module/modules/hello_module.wasm
rpu run examples/hello_with_module
```

The module initializes before the parent cartridge's `rpu_main` entry point.
