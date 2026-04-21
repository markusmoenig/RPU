---
id: install
title: Install
sidebar_position: 2
---

# Install

RPU is currently developed as a Rust workspace.

## Prerequisites

- Rust toolchain
- Cargo

For the docs site itself:

- Node.js
- npm

## Workspace

From the repo root:

```bash
cargo check
```

That validates the current Rust workspace.

## Docs site

From the repo root:

```bash
cd docs
npm install
npm run start
```

For a production docs build:

```bash
cd docs
npm run build
```
