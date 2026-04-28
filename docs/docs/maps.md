---
id: maps
title: ASCII Maps
sidebar_position: 7
---

# ASCII Maps

RPU currently supports embedded ASCII maps directly inside scene files.

This is intentionally basic and is meant as a foundation for later terrain experiments.

## Syntax

```rpu
map Terrain {
    origin = (80, 176)
    cell = (48, 48)
    render = basic

    legend {
        x = marker
        # = #c58c35
        g = "tile-grass-top.png"
        - = #4dc7ff
        / = #7b8cff
        \ = #5970d8
    }

    ascii {
        x
        ###/
          /----
             \##
    }
}
```

## Current behavior

Current map support includes:

- `origin`
- `cell`
- `render`
- `legend`
- `ascii`
- color-mapped cells rendered as rects
- texture-mapped cells rendered as sprites
- marker cells used for sprite placement
- typed spawn markers used for entity placement
- solid collision rect generation for platformer physics
- terrain legend entries with topology + material parsing
- derived terrain shape classification for debug rendering

## Shape Maps

`shape_map` is the first vector-map path for games such as pinball. It still uses an ASCII grid, but the cells define named control points instead of tile collision.

```rpu
shape_map Table {
    origin = (40, 40)
    cell = (8, 8)
    debug_labels = true

    legend {
        a = point("left_top")
        b = point("left_mid")
        c = point("left_bottom")
        l = point("lane_top", 0, -8)
        k = point("lane_bottom")
        r = point("lane_exit_edge")
        q = point("lane_exit_guide")
        x = point("bumper_left")
    }

    ascii {
        a......q.
        ........lr
        ....x...
        b.......
        c........k
    }

    wall LeftRail {
        points = [left_top, left_mid, left_bottom]
        corner = round
        radius = 12
        segments = 8
        thickness = 5
        color = #9ad8ff
        bounce = 0.9
    }

    polyline PlayfieldBoundary {
        points = [left_bottom, left_mid, left_top, lane_exit_guide, lane_top]
        closed = false
        radius = 3
        smooth = 8
        corner = round
        corner_radius = 16
        segments = 10
        color = #9ad8ff
        bounce = 0.9
    }

    pipe ShooterLane {
        points = [lane_top, lane_bottom]
        width = 20
        thickness = 4
        color = #ffe08a
        bounce = 0.6
    }

    sdf_wall ShooterExitGuide {
        points = [lane_exit_edge, lane_exit_guide, left_top]
        radius = 4
        smooth = 8
        corner = round
        corner_radius = 12
        segments = 8
        color = #ffe08a
        bounce = 0.9
    }

    bumper LeftBumper {
        point = bumper_left
        radius = 13
        color = #ff4f9a
        bounce = 1.7
    }

    flipper LeftFlipper {
        pivot = left_bottom
        length = 34
        thickness = 5
        rest_angle = 0.28
        active_angle = -0.55
        up_speed = 24
        down_speed = 12
        impulse = 1.25
        input = left
        color = #fff39a
        bounce = 1.45
    }
}
```

Current shape-map behavior:

- `origin` and `cell` define the control-point grid.
- `cell` can be non-square, for example `(8, 36)`, to stretch a compact ASCII sketch over a taller playfield.
- `debug_labels = true` draws each authored point symbol at its resolved world position.
- `legend` maps ASCII symbols to named points with `point("name")`.
- `point("name", dx, dy)` applies an optional world-space offset to a control point, useful when the ASCII grid is too coarse for final pinball tuning.
- `wall` connects named points into prototype polyline geometry.
- `corner = round` rounds interior wall corners; omit it or use `corner = sharp` for raw line segments.
- `radius` controls how far the rounded corner cuts back along each adjacent segment.
- `segments` controls how many short debug/render segments approximate each rounded corner.
- `pipe` creates two parallel wall rails from a named centerline. It is useful for launcher lanes, tubes, and other constrained ball paths.
- `pipe.width` is the rail-to-rail center distance; `pipe.thickness`, `color`, and `bounce` use the same meaning as walls.
- `sdf_wall` creates a smooth signed-distance wall from named points. Collision samples the SDF and uses the field gradient as the response normal.
- `sdf_wall.radius` is the wall radius, and `smooth` blends neighboring segments so bends are less brittle than raw polylines.
- `sdf_wall` also supports `corner = round`, `corner_radius`, and `segments` to round the authored polyline before SDF collision sampling.
- `polyline` is the same rounded SDF collision/render path as `sdf_wall`, but intended for larger authored boundaries. Use `closed = true` to connect the last point back to the first.
- `bumper` places prototype circular bumper geometry at a named point.
- `flipper` creates an input-driven pinball segment around a named `pivot`.
- `rest_angle` and `active_angle` are radians in world coordinates, where `0` points right and positive angles rotate downward.
- `up_speed` and `down_speed` control flipper angular motion in radians per second.
- `impulse` scales the extra velocity transferred from moving flippers into the ball.
- `input` can be `left`, `right`, `up`, `down`, `action`, or a concrete key name.
- `bounce` controls the restitution used by `physics = pinball` balls.

Pinball balls are regular sprites:

```rpu
sprite Ball {
    pos = (154, 72)
    size = (12, 12)
    collider_size = (12, 12)
    texture = "generated://circle/ball"
    physics = pinball
    gravity = 180
    max_speed = 420
}
```

`physics = pinball` treats the sprite collider as a circle and collides it against all shape-map walls and bumpers in the scene. `generated://circle/...` creates an internal circular alpha texture, so prototypes do not need a separate ball image asset.

## Markers

Sprites can resolve their position from a symbol:

```rpu
sprite Player {
    symbol = x
    size = (144, 144)
    color = #ffd447
    script = "main.rpu"
}
```

Typed spawn markers bind a map cell to an entity name:

```rpu
map Terrain {
    legend {
        p = spawn(Player)
    }

    ascii {
        p
        ####
    }
}

sprite Player {
    physics = platformer
    texture = "foxy_idle1.png"
}
```

`spawn(Player)` places the scene sprite named `Player` at that map cell. This is often more readable than assigning a separate `pos` to the sprite.

Repeated spawn cells can instantiate hidden sprite definitions:

```rpu
legend {
    c = spawn(Coin)
    o = spawn(Opossum)
}

ascii {
    c   o   c
}

sprite Coin {
    visible = false
    group = "pickup"
    texture = ["gem-1.png", "gem-2.png"]
}
```

The runtime creates visible instances named from the template, such as `Coin_1`, `Coin_2`, and `Opossum_1`. This keeps collectible, enemy, and prop placement in the map instead of scattering positions through sprite definitions.

## Direct Tiles

For tile-based games that do not want terrain synthesis, legend entries can map symbols directly to texture filenames and collision policies:

```rpu
map Terrain {
    origin = (0, 84)
    cell = (24, 24)

    legend {
        # = tile("tile-grass-top.png", solid)
        d = tile("tile-dirt.png", solid)
        - = tile("platform.png", one_way)
        b = tile("bush.png", none)
    }

    ascii {
         ----
    ############
    dddddddddddd
    b          b
    }
}
```

`tile("name.png", policy)` draws one sprite per map cell using the map `cell` size. This path is deterministic and does not use terrain classification, material stacks, WFC, or synthesized caps.

Collision policies:

- `solid` generates full AABB collision.
- `one_way` only collides when a platformer body lands from above.
- `none` draws the tile without collision.

Quoted texture entries such as `# = "tile-grass-top.png"` still work as a compatibility shorthand for a solid tile. Spawn and marker cells do not collide.

## Current limits

The map system does not yet provide:

- tile rules
- procedural terrain generation
- slope semantics beyond authored symbols
- texture synthesis

That work is expected to grow later on top of the current embedded map representation.

## Terrain Legend Entries

Map legend symbols can also represent terrain cells instead of only flat colors.

Examples:

```rpu
legend {
    # = rock
    # = grass>dirt>rock
    / = slope_up:grass
    \ = slope_down:grass
}
```

Current rules:

- bare material like `rock` means:
  - `solid` topology
  - material `rock`
- explicit forms are:
  - `solid:rock`
  - `slope_up:grass`
  - `slope_down:grass`
- stacked materials are also allowed:
  - `grass>dirt>rock`
  - `solid:grass>dirt>rock`
  - `slope_up:grass>dirt>rock`

Current stack behavior:

- top-facing outer surfaces use the first material
- side and underside outer surfaces fall back to the next material
- deeper cells step further into the stack by boundary distance

From those authored cells, RPU derives a neighborhood-based terrain shape such as:

- `Top`
- `Left`
- `TopLeftOuter`
- `Interior`

Right now terrain entries render as generated debug colors so these derived shape classes are visible while the system is still being built out.

## Terrain Render Modes

Terrain maps can choose how the main terrain render/preview path behaves:

```rpu
map Terrain {
    origin = (80, 176)
    cell = (48, 48)
    render = basic
}
```

Supported values:

- `debug`
  - uses the structural debug-colored terrain view
- `basic`
  - uses the explicit cap/body terrain renderer
  - no WFC or solved surface strip is used
- `synth`
  - uses the current solved surface-cap path on top of the same body fill

This lets users keep the same topology/material authoring and choose later whether they want:

- pure debugging
- a simple non-synth terrain render
- or the experimental synthesized surface pass

## Terrain Style Controls

Terrain maps can also tune the generated cap and shoulder shape:

```rpu
map Terrain {
    render = synth
    cap_depth = 0.68
    ramp_cap_depth = 0.62
    join_cap_depth = 0.72
    shoulder_width = 0.82
    surface_roughness = 0.045
    shoulder_shape = bend
}
```

Supported style properties:

- `cap_depth`
  - grass/cap thickness on flat top surfaces, as a fraction of tile height
- `ramp_cap_depth`
  - cap thickness on explicit `/` and `\` ramp tiles
- `join_cap_depth`
  - cap thickness on plateau cells that connect to a ramp
- `shoulder_width`
  - how much of a plateau join cell is shaped by the ramp shoulder
- `surface_roughness`
  - small world-coordinate surface waviness applied to caps
  - `0.0` keeps authored terrain perfectly geometric
- `shoulder_shape`
  - `linear` for a straight shoulder
  - `bend` for a smoother eased shoulder

These controls affect `basic` and `synth` terrain rendering. They do not change the ASCII topology; they only change how the generated terrain surface is shaped and sampled.
