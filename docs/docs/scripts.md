---
id: scripts
title: Scripts
sidebar_position: 6
---

# Scripts

Scripts are compiled to bytecode and executed by the runtime.

They are currently event-driven. They can live in external files under `scripts/` or be embedded directly inside `rect` and `sprite` scene nodes.

External example:

```rpu
on ready() {
    log("boot")
}
```

Inline scene example:

```rpu
rect Hero {
    on update(dt) {
        self.x = self.x + 10.0 * dt
    }
}
```

## Event handlers

Current handlers are defined as:

```rpu
on ready() {
    log("boot")
}

on update(dt) {
    self.x = self.x + 10.0 * dt
}
```

## Current language features

Supported today:

- assignments
- numeric expressions
- property reads and writes
- local bindings with `let`
- persistent per-entity script state with `state`
- entity-local event dispatch with `emit(...)`
- `if` / `else`
- boolean conditions with `&&`, `||`, `!`
- top-level reusable functions
- runtime query calls like `input_left()`, `input_action()`, and `key("Space")`
- action builtins like `play_sound(...)`, `play_music(...)`, and `stop_music()`

## Properties

Current readable/writable properties:

- `self.x`
- `self.y`
- `self.width`
- `self.height`
- `self.pos`
- `self.size`
- `self.rotation`
- `self.color`
- `self.texture` for sprites
- `self.animation` for named sprite animations
- `self.flip_x` for sprites
- `self.flip_y` for sprites
- `self.vx` for platformer sprites
- `self.vy` for platformer sprites
- `self.move_x` for platformer sprites
- `self.jump` for platformer sprites
- `self.grounded` for platformer sprites
- `self.text` for text nodes
- `self.some_state`
- `Name.x`
- `Name.y`
- `Name.width`
- `Name.height`
- `Name.pos`
- `Name.size`
- `Name.rotation`
- `Name.color`
- `Name.texture` for sprites
- `Name.animation` for named sprite animations
- `Name.flip_x`
- `Name.flip_y`
- `Name.vx`
- `Name.vy`
- `Name.move_x`
- `Name.jump`
- `Name.grounded`
- `Name.text` for text nodes
- `Name.some_state`

Example:

```rpu
Accent.color = #7ce0ff
self.pos = Mascot.pos
self.width = 96.0
self.rotation = self.rotation + 1.6 * dt
```

`flip_x`, `flip_y`, `jump`, and `grounded` are represented as scalar `0` or `1` values in scripts.

## Script Events

Scripts can dispatch entity-local events:

```rpu
emit("motion", "run")
```

The same entity can receive them through a generic event handler:

```rpu
on event(event, value) {
    if event == "motion" {
        if value == "run" {
            self.animation = "run"
        } else if value == "idle" {
            self.animation = "idle"
        }
    }
}
```

String equality and inequality are supported in conditions, so `event == "motion"` and `value != "idle"` are valid.

## Platformer Input

For sprites using `physics = platformer`, scripts should express input intent and let the runtime handle acceleration, friction, gravity, jumping, and collision:

```rpu
on update(dt) {
    if input_left() {
        self.move_x = -1
        self.flip_x = 1
    } else if input_right() {
        self.move_x = 1
        self.flip_x = 0
    } else {
        self.move_x = 0
    }

    if input_action() && self.grounded {
        self.jump = 1
    }
}
```

The runtime updates `self.vx`, `self.vy`, and `self.grounded`.

## Locals

Handler-local values can be introduced with `let`:

```rpu
let next_x = Mascot.x - 12.0 * dt
Mascot.x = next_x
```

Locals are shared with nested `if` blocks and with called functions in the same handler execution.

## Persistent state

Scripts can declare persistent state that survives across frames on the bound runtime entity:

```rpu
state score = 0
state lives = 3
state invulnerable_until = 0
```

State values can be read as bare variables inside the same script:

```rpu
score = score + 10
```

They can also be accessed through entity properties:

```rpu
self.score = self.score + 10
HudState.lives = HudState.lives - 1
```

Use `let` for temporary per-handler values and `state` for data that must persist between updates.

## Conditions

Current condition features:

- `<`
- `<=`
- `>`
- `>=`
- `==`
- `!=`
- `&&`
- `||`
- `!`
- grouping with parentheses

Example:

```rpu
if next_x < 120.0 || (Accent.x < 260.0 && !(self.y < 200.0)) {
    Mascot.x = 520.0
} else {
    Accent.color = #7ce0ff
}
```

Bare query calls are also valid conditions. They are treated as truthy when non-zero:

```rpu
if input_left() {
    self.x = self.x - 120.0 * dt
}
```

## Functions

Current functions are top-level, can take parameters, and can return a value:

```rpu
fn accent_color(limit) {
    if limit < 120.0 {
        return #ff8899
    } else {
        return #7ce0ff
    }
}
```

Called as a statement:

```rpu
call sync_accent(next_x)
```

Audio actions use the same `call ...` statement form:

```rpu
call play_sound("shot.wav")
call play_music("music-game.ogg")
call stop_music()
```

`play_sound(...)` is intended for short one-shot effects.

`play_music(...)` starts looping background music and replaces any currently playing music.

`stop_music()` stops the current music track.

These actions work on both native and wasm/web. On the web, browsers may delay music start until the first user interaction because of autoplay rules.

Called as an expression:

```rpu
Accent.color = accent_color(next_x)
```

## Runtime queries

Current built-in runtime queries:

- `input_left()`
- `input_right()`
- `input_up()`
- `input_down()`
- `input_action()`
- `key("Space")`
- `exists("Name")`
- `first_overlap("group")`
- `high_score_name(index)`
- `high_score_value(index)`
- `lerp(a, b, t)`
- `pulse(period)`
- `smoothstep(edge0, edge1, x)`
- `alpha(color, alpha)`
- `time()`
- `difficulty()`
- `every(seconds)`
- `every(min_seconds, max_seconds)`
- `rand(min, max)`
- `screen_width()`
- `screen_height()`

Example:

```rpu
if input_action() {
    self.color = #ffbf47
}

if key("Space") {
    self.color = #ff8899
}

self.x = clamp(self.x, 80.0, screen_width() - 160.0)

if every(1.2, 2.0) {
    spawn("EnemyTemplate", screen_width() + 80.0, rand(48.0, screen_height() - 96.0))
}
```

`input_action()` is the generic shoot/confirm/action abstraction. On desktop it currently maps to `Space`, `Enter`, `Z`, and `X`.

`exists("Name")` returns whether a live runtime instance with that name currently exists. This is useful for spawn gating:

```rpu
if !exists("PlanetTop") && every(10.0, 14.0) {
    spawn("PlanetTopTemplate", "PlanetTop", screen_width() + 96.0, -92.0)
}
```

`first_overlap("group")` returns the name of the first overlapping live entity in that group, or an empty string if there is no hit. This is useful for simple projectile collisions:

```rpu
let hit = first_overlap("hostile")
if exists(hit) {
    destroy(hit)
    destroy(self)
}
```

`high_score_name(index)` and `high_score_value(index)` read from the runtime high-score table using 1-based indices.

`lerp(a, b, t)` linearly interpolates between two scalar values.

`pulse(period)` returns a repeating `0..1` pulse over the given period in seconds.

`smoothstep(edge0, edge1, x)` returns a smoothed `0..1` interpolation factor, useful for eased motion and fades.

`alpha(color, alpha)` returns the given color with a replaced alpha channel.

`difficulty()` is currently a simple time-based level that increases as the session runs.

`every(seconds)` is a per-script-line timer query that returns true when the interval elapses.

`every(min_seconds, max_seconds)` schedules the next trigger with a randomized interval in that range.

`rand(min, max)` returns a random scalar inside the given range.

## Runtime instancing

Current runtime instance statements:

- `spawn("TemplateName", "InstanceName", x, y)`
- `spawn("TemplateName", x, y)`
- `destroy("InstanceName")`
- `destroy(name_expr)`
- `destroy(self)`

Example:

```rpu
on ready() {
    spawn("EnemyTemplate", self.x + self.width, rand(self.y, self.y + self.height))
}

on update(dt) {
    ScoutOne.x = ScoutOne.x - 180.0 * dt
}
```

## High scores

Scripts can submit scores to the runtime high-score table:

```rpu
call submit_score("UNKNOWN", HudState.score)
```

This updates the internal table used by `highscore` scene nodes and the `high_score_name(...)` / `high_score_value(...)` queries.

Current limitations:

- no closures
- no column-precise diagnostics; warnings/errors currently report file and line

## Compatibility helpers

The runtime still supports older helper-style script ops such as:

- `move_by(...)`
- `move_by_dt(...)`
- `set_pos(...)`
- `set_color(...)`
- `copy_pos(...)`
- `clamp_x(...)`
- `clamp_y(...)`

The direction now is to prefer assignments and expressions instead of adding more one-off built-ins.
