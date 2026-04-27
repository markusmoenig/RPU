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
        follow = Player
        follow_offset = (0, -18)
        background = (0.07, 0.09, 0.14, 1.0)
    }
}
```

## Supported nodes

Current scene nodes:

- `camera`
- `rect`
- `sprite`
- `text`
- `stack`
- `highscore`
- `map`

## Cameras

Camera properties:

- `pos`
- `zoom`
- `background`
- `follow`
- `follow_offset`
- `follow_smoothing`
- `dead_zone`
- `bounds_min`
- `bounds_max`

`pos` is the authored world-space camera center. If `follow` is set, the runtime resolves the named entity each frame and centers the camera on that entity instead.

```rpu
camera MainCamera {
    pos = (160, 90)
    follow = Player
    follow_offset = (0, -18)
    zoom = 1.0
}
```

`follow_offset` shifts the followed camera center in world units. For platformers this is useful to show more space ahead or above the player while still keeping the camera tied to movement.

`follow_smoothing` eases camera movement toward the target. `0` disables smoothing. Larger values catch up faster.

`dead_zone = (width, height)` keeps the camera still while the followed target remains inside that centered world-space box.

`bounds_min` and `bounds_max` clamp the camera center after follow is applied:

```rpu
camera MainCamera {
    follow = Player
    follow_smoothing = 7.0
    dead_zone = (48, 28)
    bounds_min = (160, 90)
    bounds_max = (880, 90)
}
```

## Visual node properties

`rect` and `sprite` currently share this core property set:

- `visible`
- `template`
- `group`
- `parent`
- `order`
- `anchor`
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

`parent = "StackName"` attaches the node to a `stack` layout container.

`order = 10` defines the node order inside that stack.

`anchor` controls viewport-relative placement for UI-style nodes. Current values are:

- `world`
- `top_left`
- `top`
- `top_right`
- `left`
- `center`
- `right`
- `bottom_left`
- `bottom`
- `bottom_right`

`anchor = "world"` is the default and keeps the current gameplay/world-space behavior.

Non-`world` anchors place the node relative to the authored virtual window size, with `pos` acting as an offset from that anchor.

## Stack Layout

`stack` is a lightweight layout container for menus and HUDs.

Current stack properties:

- `anchor`
- `pos`
- `size`
- `direction`
- `gap`
- `align`

Example:

```rpu
stack MenuContent {
    anchor = top
    pos = (-80, 18)
    size = (160, 128)
    direction = vertical
    gap = 6.0
    align = center
}

text TitleLabel {
    parent = "MenuContent"
    order = 0
    value = "WARPED SPACE SHOOTER"
    font = "BetterPixels.ttf"
    font_size = 14.0
    color = #f4f8ff
}
```

Current stack directions:

- `vertical`
- `horizontal`

Current stack alignment values:

- `start`
- `center`
- `end`

Stacks are intentionally small. They help with menu and HUD composition, but they are not a full retained UI system.

For `sprite` nodes, `size` is optional. If a sprite has a `texture` and no explicit `size`, RPU uses the texture's natural pixel dimensions. If you do declare `size`, that overrides the texture size.

Sprites also support:

- `texture`
- `animation <name> { ... }`
- `animation_<name>`
- `animation_<name>_fps`
- `animation_<name>_mode`
- `animation_fps`
- `animation_mode`
- `destroy_on_animation_end`
- `symbol`
- `physics`
- `acceleration`
- `friction`
- `max_speed`
- `gravity`
- `jump_speed`
- `max_fall_speed`
- `rotation`
- `scroll`
- `repeat_x`
- `repeat_y`
- `flip_x`
- `flip_y`
- `collider_offset`
- `collider_size`

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

Named animations keep reusable frame lists in scene data instead of script logic:

```rpu
sprite Player {
    texture = "foxy_idle1.png"

    animation idle {
        frames = ["foxy_idle1.png", "foxy_idle2.png"]
        fps = 1.25
    }

    animation run {
        frames = ["foxy_run1.png", "foxy_run2.png", "foxy_run3.png"]
        fps = 9.0
    }

    animation hurt {
        frames = "foxy_hurt.png"
        mode = once
    }
}
```

Animation blocks support `frames`, `fps`, `mode = loop|once`, and `loop = true|false`. The older `animation_<name>` property form remains supported for compact declarations.

Scripts switch named animations through `self.animation = "idle"` or `self.animation = "run"`.

## Platformer Physics

Sprites can opt into the built-in kinematic platformer physics layer:

```rpu
sprite Player {
    physics = platformer
    acceleration = 520.0
    friction = 840.0
    max_speed = 96.0
    gravity = 560.0
    jump_speed = 255.0
    max_fall_speed = 280.0
    coyote_time = 0.10
    jump_buffer = 0.12
    collider_offset = (4, 2)
    collider_size = (16, 22)
}
```

The runtime applies acceleration, friction, gravity, jump impulse, and axis-separated AABB collision against map cells with collision.

Direct `tile(...)` map cells control collision explicitly with `solid`, `one_way`, or `none`. Legacy color, quoted texture, and terrain map cells are treated as solid. `marker` and `spawn(...)` cells are ignored.

`coyote_time` keeps jump eligibility alive briefly after leaving a platform. `jump_buffer` remembers a jump press briefly before landing. Both values are in seconds and make platforming less brittle without script-side timing code.

`collider_offset` and `collider_size` define the runtime collision rectangle relative to the sprite's visual top-left. If omitted, the sprite's full visual `size` is used. Rendering still uses the visual size, so sprites with transparent padding can use tighter collision bounds.

Scripts drive platformer physics through intent properties:

```rpu
self.move_x = -1
self.move_x = 1
self.move_x = 0

if input_action() && self.grounded {
    self.jump = 1
}
```

Runtime physics exposes:

- `self.vx`
- `self.vy`
- `self.move_x`
- `self.jump`
- `self.grounded`

`rotation = 1.57` rotates the sprite in radians around its center. Rotation is also script-visible through `self.rotation` and `Name.rotation`.

## Text

`text` nodes render strings using a font file from `assets/`.

Current text properties:

- `value`
- `font`
- `font_size`
- `visible`
- `template`
- `group`
- `layer`
- `z`
- `pos`
- `color`
- `script`
- `anchor`
- `align`

Example:

```rpu
text Score {
    anchor = top_right
    align = right
    pos = (-12, 8)
    value = "SCORE 000000"
    font = "BetterPixels.ttf"
    font_size = 16.0
    color = #f4f8ff
}
```

The `font` property should point to a `.ttf` file in `assets/`.

`align` currently supports:

- `left`
- `center`
- `right`

This is especially useful with anchored HUD/menu text.

## High Score

`highscore` is a built-in UI node backed by the runtime high-score table. It renders a compact name/score list without requiring one text node per row.

Current highscore properties:

- `font`
- `font_size`
- `items`
- `gap`
- `score_digits`
- `visible`
- `layer`
- `z`
- `pos`
- `size`
- `color`
- `anchor`

Example:

```rpu
highscore HighScoreTable {
    anchor = top
    pos = (0, 64)
    size = (96, 72)
    color = #f4f8ff
    font = "BetterPixels.ttf"
    font_size = 8.0
    items = 8
    gap = 8.0
    score_digits = 4
}
```

The runtime table currently starts with placeholder `UNKNOWN / 0000` rows until scores are submitted.

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

## High Scores

`highscore` is a built-in scene object backed by the runtime high-score table.

Current highscore properties:

- `font`
- `font_size`
- `items`
- `gap`
- `score_digits`
- normal visual properties such as `anchor`, `pos`, `size`, `layer`, `z`, and `color`

Example:

```rpu
highscore HighScoreTable {
    anchor = top
    pos = (-40, 64)
    size = (80, 72)
    color = #f4f8ff
    font = "BetterPixels.ttf"
    font_size = 8.0
    items = 8
    gap = 8.0
    score_digits = 4
}
```

This is driven by the internal runtime high-score struct. You do not need to create one text node per row.
