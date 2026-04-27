# Sunnyland

Prototype for side-view platform movement, jump physics, pickups, and simple hazards using the Sunnyland asset set.

The scene uses:

- Sunnyland Foxy/Opossum/items/props/background assets
- ASCII terrain with direct Sunnyland tile texture mapping
- map-authored `spawn(Player)`, `spawn(Coin)`, and `spawn(Opossum)` cells
- built-in platformer physics with map collision
- explicit tile collision policies via `tile("asset.png", solid|one_way|none)`
- camera follow via `follow = Player`
- runtime physics overlays via `[debug].physics`
- runtime physics overlay toggle via `F3`
- pickup and hazard interactions via `on collision_enter(other, group)`
- stomp-vs-hurt enemy behavior via `is_stomping(other)`

The scene intentionally avoids terrain synthesis. Map keys directly select Sunnyland tiles and their collision behavior, which keeps the prototype readable while the platformer physics work is expanded.
