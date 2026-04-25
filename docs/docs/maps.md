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

## Direct Tile Textures

For tile-based games that do not want terrain synthesis, legend entries can map symbols directly to texture filenames:

```rpu
map Terrain {
    origin = (0, 84)
    cell = (24, 24)

    legend {
        # = "tile-grass-top.png"
        d = "tile-dirt.png"
        < = "tile-slope-left.png"
        > = "tile-slope-right.png"
    }

    ascii {
         ####
    ############
    dddddddddddd
    }
}
```

Quoted texture entries are drawn one sprite per map cell using the map `cell` size. This path is deterministic and does not use terrain classification, material stacks, WFC, or synthesized caps.

Direct texture cells currently also generate solid platformer collision. Spawn and marker cells do not collide.

## Current limits

The map system does not yet provide:

- tile rules
- procedural terrain generation
- per-tile collision policies such as `one_way` or `none`
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
