---
id: getting-started
title: Getting Started
sidebar_position: 1
---

# Getting Started

## Install Rust

RPU is built with Rust. Install the current stable toolchain first:

- [Install Rust with rustup](https://www.rust-lang.org/tools/install)

## Create Or Run A Project

Current starting points:

```bash
rpu new my_game
rpu run path/to/project
```

For the maintained example:

```bash
rpu run examples/warped_space_shooter
```

## Build For The Web

RPU can currently export a wasm/web build:

```bash
rpu build-web examples/warped_space_shooter
```

To preview it locally:

```bash
rpu serve-web examples/warped_space_shooter --port 8123
```

## Next

More setup steps will go here.
