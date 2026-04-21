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
