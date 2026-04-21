---
id: scenes
title: Scenes
sidebar_position: 5
---

# Scenes

Scenes describe the structure of the world.

Current top-level scene syntax:

```rpu
scene Main {
    meta {
        title = "Hello Shapes"
    }

    camera MainCamera {
        pos = (360, 240)
        zoom = 1.0
        background = (0.07, 0.09, 0.14, 1.0)
    }
}
```

## Supported nodes

Current scene nodes:

- `camera`
- `rect`
- `sprite`
- `map`

## Visual node properties

`rect` and `sprite` currently share this core property set:

- `visible`
- `template`
- `group`
- `layer`
- `z`
- `pos`
- `size`
- `color`
- `script`

Example:

```rpu
rect Hero {
    visible = true
    layer = 0
    z = 20
    pos = (80, 40)
    size = (280, 160)
    color = #ff4455
    script = "main.rpu"
}
```

`visible = false` keeps the node in the scene but skips rendering it.

`template = true` marks a visual node as a runtime spawn template. Template nodes are not part of the initial rendered world; scripts can instantiate them later with `spawn(...)`.

`group = "name"` assigns the node to a logical runtime group. This is useful for gameplay queries such as overlap checks against all `hostile` entities.

For `sprite` nodes, `size` is optional. If a sprite has a `texture` and no explicit `size`, RPU uses the texture's natural pixel dimensions. If you do declare `size`, that overrides the texture size.

Sprites also support:

- `texture`
- `animation_fps`
- `animation_mode`
- `destroy_on_animation_end`
- `symbol`
- `scroll`
- `repeat_x`
- `repeat_y`
- `flip_x`
- `flip_y`

`scroll = (x, y)` applies a continuous authored-space offset over time.

`repeat_x = true` repeats the sprite horizontally to fill the viewport.

`repeat_y = true` repeats the sprite vertically to fill the viewport.

`flip_x = true` horizontally mirrors the sprite texture.

`flip_y = true` vertically mirrors the sprite texture.

`texture` can be either a single image:

```rpu
texture = "player1.png"
```

or an animated frame list:

```rpu
texture = ["shoot1.png", "shoot2.png"]
```

`animation_fps = 18.0` advances animated sprite frames at that rate.

`animation_mode = "loop"` repeats the frame list.

`animation_mode = "once"` plays the frame list once and then holds on the last frame.

`destroy_on_animation_end = true` removes a runtime sprite instance automatically when a `once` animation finishes. This is useful for short-lived effects such as explosions.

Visual nodes can also embed script functions and handlers directly:

```rpu
rect Hero {
    pos = (80, 40)
    size = (280, 160)
    color = #ff4455

    fn wrap_x(next_x) {
        if next_x < 120.0 {
            return 520.0
        }
        return next_x
    }

    on update(dt) {
        let next_x = self.x - 12.0 * dt
        self.x = wrap_x(next_x)
    }
}
```

Inline scripts compile through the same bytecode path as external `scripts/*.rpu` files.

If a node defines both `script = "shared.rpu"` and inline handlers/functions, RPU currently composes them into one effective script for that node: external script contents first, then inline scene-local code.

## Colors

Current color formats:

- tuple RGBA: `(0.10, 0.72, 0.88, 1.0)`
- hex RGB: `#ff4455`
- hex RGBA: `#ff4455cc`

## Camera

Current camera properties:

- `pos`
- `zoom`
- `background`

The runtime currently uses the first available camera as the active camera.
