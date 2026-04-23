---
id: project-structure
title: Project Structure
sidebar_position: 4
---

# Project Structure

RPU projects use a simple layout:

```text
my_app/
  rpu.toml
  scenes/
  scripts/
  assets/
```

## `rpu.toml`

The manifest is lowercase:

```toml
[project]
name = "my_app"
version = "0.1.0"

[window]
width = 272
height = 160
default_scale = 4.0
resize = "letterbox"

[meta]
bundle_id = "org.rpu.my_app"
display_name = "My App"
```

`window.width` / `window.height` define the authored base resolution.

`window.default_scale` controls the default startup window size relative to that base resolution.

The important part is that this base should match the gameplay canvas you are authoring against, not a larger promo or menu image. If your in-game background is `272x160`, use that as the base resolution and let `default_scale` make the startup window larger.

That means:

- scenes and scripts are authored against the base size
- `screen_width()` / `screen_height()` return that base size
- the initial native window opens at `width * default_scale` by `height * default_scale`
- resizing maps the base canvas into the real window using the configured resize mode

Current resize modes:

- `letterbox`
- `stretch`

`letterbox` preserves aspect ratio and adds bars when the real window does not match the project aspect ratio.

`stretch` fills the window exactly, even if that distorts the image.

For pixel-art or fixed-layout games, `letterbox` is usually the right default.

## `meta`

`meta` holds project metadata that can be reused by platform exporters.

Current fields:

- `bundle_id`
- `display_name`

Example:

```toml
[meta]
bundle_id = "org.rpu.warped_space_shooter"
display_name = "Warped Space Shooter"
```

Right now these are used by the Apple/Xcode export, but the section is intentionally platform-neutral so the same metadata can be reused by future exporters.

## `scenes/`

Contains `.rpu` scene files.

Scenes can hold structure only, or structure plus embedded script functions and handlers inside `rect` and `sprite` nodes.

## `scripts/`

Contains optional external `.rpu` script files used by scene nodes.

This directory is still useful for:

- reusable scripts shared across multiple nodes
- keeping larger behaviors out of scene files
- mixing shared file-based code with node-local inline handlers

## `assets/`

Contains textures and other project assets.

Sprite textures are currently resolved relative to `assets/`.
