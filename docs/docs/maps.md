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

    legend {
        x = marker
        # = #c58c35
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
- `legend`
- `ascii`
- color-mapped cells rendered as rects
- marker cells used for sprite placement
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

## Current limits

The map system does not yet provide:

- tile rules
- procedural terrain generation
- collision generation
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
