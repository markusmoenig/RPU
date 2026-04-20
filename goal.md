# RPU — A Scene Runtime for Fast Visual Apps

**RPU** is a CLI-first runtime for building interactive 2D applications using a combination of **declarative scenes (DSL)** and **lightweight scripting with bytecode execution**.

It is designed for one goal:

> Go from idea → running visual application as fast as possible.

---

## Core Idea

RPU is not just a rendering library.

It is a **runtime for describing and executing scenes**.

Instead of writing low-level rendering code, you:

- describe **what exists** (DSL)  
- define **how it behaves** (scripts)  
- let the runtime handle execution and rendering  

---

## The Three Layers

### 1. Scenes (DSL)

Scenes define structure.

They describe:
- objects  
- layout  
- resources  
- relationships  

They are **declarative and mostly static**.

Example:

    scene Main {
        sprite Player {
            texture = "player.png"
            pos = (0, 0)
            script = "player.rpu"
        }
    }

---

### 2. Scripts (Bytecode-Executed)

Scripts define behavior.

They describe:
- movement  
- interaction  
- logic  
- state changes  

Scripts are:
- event-driven  
- dynamically executed  
- **compiled to bytecode and executed by the runtime**  

Example:

    on update(dt) {
        if key_down("Left")  { self.pos.x -= 100 * dt }
        if key_down("Right") { self.pos.x += 100 * dt }
    }

---

### 3. Runtime (SceneVM)

The runtime:
- executes scenes  
- runs scripts  
- manages state  
- renders output (via GPU)  

Users do not interact with it directly.

---

## Embedded Scripts in DSL

For small behaviors, scripts can be embedded directly inside scene definitions.

Example:

    sprite Torch {
        texture = "torch.png"
        pos = (40, 20)

        on update(dt) {
            self.rotation = sin(time()) * 0.05
        }
    }

Embedded scripts are useful for:
- quick prototyping  
- localized behavior  
- reducing file fragmentation  

Larger logic should still be placed in separate script files.

---

## Mental Model

Think of RPU like this:

- **Scenes** = the world  
- **Scripts** = what happens in the world  
- **Runtime** = how the world runs  

Or:

    DSL → defines  
    Script → mutates  
    Runtime → executes  

---

## CLI-First Workflow

RPU is driven by a single command-line tool.

    rpu new my_app
    cd my_app
    rpu run

From there:

- edit scene → instant update  
- edit script → instant behavior change  
- no manual build step required for iteration  

---

## Project Structure

RPU projects are simple and explicit.

    my_app/
      rpu.toml
      scenes/
      scripts/
      assets/

- **scenes/** → structure  
- **scripts/** → behavior  
- **assets/** → data  

---

## Design Principles

### Fast Feedback

Changes should be visible immediately.

Hot reload is a core feature.

---

### Data-Driven

Scenes are data, not code.

This makes them:
- easy to edit  
- easy to generate  
- easy to visualize  

---

### Separation of Concerns

Structure and behavior are separate.

- DSL defines the world  
- scripts control the world  

---

### Minimal Surface

RPU avoids large APIs.

Instead, it provides:
- a small DSL  
- a small scripting model  
- a focused runtime  

---

### Optional Rust Integration

For advanced use cases, RPU can be extended with Rust.

Rust is optional, not required.

---

## What RPU Is (and Isn’t)

### RPU is:

- a scene runtime  
- a CLI-driven toolchain  
- a fast iteration environment  
- a data-first system  

### RPU is not:

- a traditional game engine  
- a low-level rendering API  
- an ECS framework  
- a heavy editor-first system  

---

## Why It Exists

Most tools fall into two extremes:

- low-level APIs (too much boilerplate)  
- full engines (too heavy, too complex)  

RPU aims for the middle:

> Simple enough to start instantly  
> Powerful enough to build real applications  

---

## Roadmap Direction

- 2D first (sprites, camera, input)  
- JIT scripting system  
- hot reload and debugging tools  
- packaging for desktop and mobile  
- optional 3D extension  

---

## One-Line Summary

> RPU is a runtime where you describe a world, define its behavior, and see it run instantly.
