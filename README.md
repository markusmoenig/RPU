# RPU

The Game Language. Build. Run. Everywhere.

RPU is a language and runtime for interactive 2D games and apps, with declarative scenes, lightweight scripting, hot reload, and a CLI-based workflow. Build for every platform through the power of Rust.

## Build a Scene

RPU combines declarative scenes with lightweight scripting. You describe what is on screen, attach behavior where it belongs, and run it through the CLI.

Start with a single sprite:

```rpu
scene Main {
    sprite Hero {
        pos = (48, 56)
        texture = "hero.png"
        color = #f4f8ff
    }
}
```

This does three things:

1. Creates a scene named `Main`.
2. Places a sprite called `Hero`.
3. Draws `hero.png` with a starting position and tint.

## Add Behavior

Then attach a script directly to the same node. The scene still owns structure. The script only owns behavior.

```rpu
scene Main {
    sprite Hero {
        pos = (48, 56)
        texture = "hero.png"

        on update(dt) {
            if input_left() {
                self.x = self.x - 120.0 * dt
            }
        }
    }
}
```

That is the core RPU model: scene files describe what exists, scripts describe what changes, and the CLI builds and runs the project.

Docs: https://rpu-lang.org
