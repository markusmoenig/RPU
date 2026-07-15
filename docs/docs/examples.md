---
id: examples
title: Examples
sidebar_position: 8
---

# Examples

## Hello CLI

Source:

- [examples/hello_cli](https://github.com/markusmoenig/RPU/tree/main/examples/hello_cli)

Run it locally:

```bash
rpu run examples/hello_cli
rpu run examples/hello_cli -- RPU
```

## Hello C

Source:

- [examples/hello_c](https://github.com/markusmoenig/RPU/tree/main/examples/hello_c)

Build or run the freestanding C-to-WASM cartridge:

```bash
rpu build examples/hello_c
rpu run examples/hello_c -- RPU
rpu run examples/hello_c/build/hello_c.cart -- RPU
```

## Hello Module

Sources:

- [examples/hello_module](https://github.com/markusmoenig/RPU/tree/main/examples/hello_module)
- [examples/hello_with_module](https://github.com/markusmoenig/RPU/tree/main/examples/hello_with_module)

Run the parent cartridge with its bundled WASM module:

```bash
rpu run examples/hello_with_module
```

Rebuild the module from C and replace the bundled artifact:

```bash
rpu build examples/hello_module
cp examples/hello_module/build/main.wasm examples/hello_with_module/modules/hello_module.wasm
```

The module's initializer runs before the parent cartridge's main entry point.

## Warped Space Shooter

Source:

- [examples/warped_space_shooter](https://github.com/markusmoenig/RPU/tree/main/examples/warped_space_shooter)

Play it here:

<iframe
  src="/games/warped-space-shooter/index.html"
  title="Warped Space Shooter"
  style={{
    width: '100%',
    maxWidth: '960px',
    aspectRatio: '16 / 10',
    border: '1px solid var(--ifm-color-emphasis-300)',
    borderRadius: '12px',
    background: '#000',
    display: 'block',
    marginTop: '1rem',
  }}
/>
