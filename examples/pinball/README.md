# Pinball Prototype

Prototype for the vector/shape-map authoring path.

The table uses `shape_map` with a small ASCII grid to place named control points. Wall, pipe, bumper, and flipper declarations then connect those points into prototype geometry without requiring long coordinate lists.

The ball is a normal sprite using `physics = pinball`. Runtime pinball physics resolves the ball as a circle against the same wall and bumper geometry used by the shape-map debug PNG.

Left/right input drives the prototype flippers.

Hold and release action to launch the ball from the pipe-based shooter lane. Drained balls reset to the launcher.

Press F3 while running to toggle the physics overlay.
