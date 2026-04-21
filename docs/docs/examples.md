---
id: examples
title: Examples
sidebar_position: 8
---

# Examples

The repo currently maintains three example projects.

## `hello_shapes`

Declaration-focused example.

It demonstrates:

- `rect`
- `sprite`
- camera setup
- layering
- textures
- inline scene-local scripts
- script assignments
- locals
- boolean conditions
- reusable functions
- return values

Repo path:

```text
examples/hello_shapes
```

## `terrain_playground`

Map-focused example.

It demonstrates:

- embedded `map`
- `legend`
- `ascii`
- marker placement
- `symbol = x` sprite spawning
- keyboard input queries
- screen-size queries

Repo path:

```text
examples/terrain_playground
```

## `warped_space_shooter`

Small game-focused example.

It demonstrates:

- a concrete playable goal
- project-defined gameplay base resolution
- keyboard movement with `input_left()` / `input_right()` / `input_up()` / `input_down()`
- `key("Space")`
- screen-size query calls
- difficulty-driven spawning via templates
- script-driven enemy and asteroid motion
- layered gameplay backgrounds (`bg-back` + `bg-stars`)
- textured sprites using the bundled shooter art reference

Repo path:

```text
examples/warped_space_shooter
```
