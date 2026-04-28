# Retro Pinball

Make a retro-style, large pinball game with a dense feature loop and visual effects all over the table. The table should feel like an arcade cabinet: bright inserts, flashing bumpers, animated targets, score callouts, and strong audio feedback.

The playfield should be wider than the current prototype, but still fully visible horizontally. The height should be roughly three visible screens tall. The camera follows the ball vertically with smooth scrolling, while keeping the launch lane, flippers, and active ball readable.

## Core Direction

- Use the ASCII point map for layout authoring, so table geometry remains readable in text.
- Use rounded SDF/polyline geometry for smooth rails, lanes, ramps, pockets, and guides.
- Use proper pinball physics for the ball, flippers, bumpers, drain, plunger, rollover triggers, and future targets.
- Keep generated debug outputs for every map so placement issues can be checked without guessing from screenshots.
- Build the visual layer from generated shapes first, then later replace or augment them with shaders and sprite art.

## Current Prototype

- One ball with pinball physics.
- One spring/plunger launcher.
- Two rounded flippers.
- Three circular bumpers with flash sprites.
- Basic SDF/polyline playfield boundaries.
- Rollover and drain triggers.
- Score text and a simple script-driven score counter.
- Build-time shape debug output:
  - `build/debug/maps/*__shape_layout.png`
  - `build/debug/maps/*__shape_runtime.txt`

## Feature Backlog

### Table Layout

- Wider playfield with stronger left/right lane separation.
- Three-screen vertical table with smooth camera follow.
- Launcher lane connected cleanly into the upper playfield.
- Lower flipper area with inlanes, outlanes, slingshots, and center drain.
- Mid-table bumper nest with lanes around it.
- Upper orbit and side loops.
- At least one ramp or elevated lane once layered geometry exists.
- Bonus saucers or capture pockets that hold and release the ball.
- Extra flipper at the upper left, aimed at a target/light bank on the upper right.
- Extra flipper at the lower right, aimed across the table at a lower/mid target bank.
- Enemy mouth / exit lane that can eat the ball if the player fails to control danger modes.

### Gameplay Loop

- Score for bumpers, rollovers, targets, ramps, lanes, and combos.
- Light inserts that turn on/off as objectives are completed.
- Lane completion bonus.
- Multiplier progression.
- Skill shot from the plunger lane.
- Ball save after launch and short post-drain grace period.
- Extra ball / special target later.
- Clear mode goals, for example:
  - Complete top lanes to enable multiplier.
  - Hit all drop targets to open a lock.
  - Lock balls to start multiball.
  - Complete orbit loops for jackpot.
- Light-bank objective: use the extra upper-left and lower-right flippers to hit all lit targets in a bank.
- Enemy hazard mode: an enemy object shoots missiles at the ball, trying to redirect it toward an eating mouth / exit.
- Counterplay loop: hit the enemy for score and disable its missile attacks for a short cooldown window.
- Risk/reward: while the enemy is active, targets score more or advance a jackpot faster, but the mouth hazard becomes dangerous.

### Pinball Objects

- Circular bumpers with impulse and flash effects.
- Slingshots near the flippers.
- Rollovers / lanes.
- Drop targets.
- Stand-up targets.
- Spinners.
- Kickers / saucers.
- Gates.
- Ball locks.
- Ramps and habitrails.
- Optional magnets for special modes.
- Additional upper-left and lower-right flippers.
- Light target banks / insert rows.
- Enemy turret / creature object that can fire ball-deflecting missiles.
- Eating mouth / hazard exit with warning lights and temporary lockout states.
- Missile projectiles that interact with the ball by applying impulse, not by damaging it directly.

### Special Modes

- Light Hunt: clear all lit targets in a bank before the timer expires.
- Creature Attack: enemy fires missiles toward the ball, trying to push it into the mouth lane.
- Creature Disabled: hitting the enemy turns off missiles temporarily and opens a scoring window.
- Mouth Panic: mouth lane opens, warning lights flash, and the player must escape via a side orbit or hit the enemy.
- Revenge Jackpot: after disabling the enemy, selected targets become high-value jackpot shots.

### Physics

- Stable ball integration with substeps.
- Reliable spring/plunger contact that carries the ball during pull and launches on release.
- Tunable bounce, friction, and restitution per object.
- Flippers as tapered capsules with angular velocity impulse.
- One-way gates and ramp transitions later.
- Better drain/outlane trigger definitions to avoid accidental resets inside launcher lanes.
- Projectile impulses for missile hits, with tunable strength and direction.
- Hazard mouth trigger that can drain/eat the ball only while active.
- Temporary disable state for enemy physics/attack behavior after a direct hit.

### Visuals

- Retro neon palette: cyan rails, magenta/yellow bumpers, dark blue table, bright insert lights.
- Generated circular and capsule primitives for early prototyping.
- Shader support later for:
  - Glow around inserts and bumpers.
  - Animated SDF rails.
  - Procedural table gradients.
  - Additive flashes and score popups.
  - Scanline / CRT-style optional overlay.
- Animated bumper pulses only on hit, not on constant contact.
- Score popups near hits.
- Plunger spring should visibly compress downward and release upward.
- Light banks should visibly toggle per target, with completion sweeps across the row.
- Enemy object should have clear states: idle, charging, firing, stunned, recovering.
- Missile shots should have obvious trails and impact flashes so the player understands why the ball changed direction.
- Mouth hazard should telegraph activation with lights, glow, or animated teeth.

### Audio

- Plunger pull/release sounds.
- Ball rolling ambience later.
- Bumper hit sounds.
- Flipper sounds.
- Rollover ticks.
- Drain sound.
- Mode start / jackpot / bonus fanfare.
- Missile launch and missile impact sounds.
- Enemy hit / stunned sounds.
- Mouth open / mouth eat sounds.

### UI / HUD

- Score at top or fixed overlay.
- Ball count.
- Multiplier.
- Current mode/objective text.
- Temporary callouts for jackpots, combos, and skill shots.
- Enemy state indicator, for example `CREATURE ACTIVE`, `STUNNED`, or `MOUTH OPEN`.
- Light-bank completion indicator.
- Debug overlay toggle for physics contacts and shape-map labels.

## Shape Map Design

The point map should remain the main authoring tool for pinball tables. Points are defined in ASCII, then connected by named objects:

- `polyline` for rails and continuous boundaries.
- `sdf_wall` for simple smooth walls.
- `spring` for the plunger.
- `flipper` for pivot-based tapered capsule flippers.
- `bumper` for circular bumper geometry.
- Future objects should follow the same pattern: define points in ASCII, then create semantic objects from those point names.

The in-game debug labels are controlled by:

```rpu
debug_labels = true
```

This should only be used while editing. Build-time debug PNG/text output should always remain available, independent of the in-game labels.

## Generated Debug Directory

The generated debug directory should make visual problems easy to inspect:

- `__shape_layout.png` should show the current camera-space layout with visible generated geometry, invisible triggers, sprite centers, and labels if enabled.
- `__shape_runtime.txt` should list resolved point coordinates, bumper centers, spring endpoints, trigger centers, sprite centers, and useful suggested positions.
- The debug output should be regenerated by `cargo run --release -- build examples/pinball`.
- Generated debug files belong under `build/debug/maps/` and should not be hand-edited.

## Near-Term Implementation Steps

1. Enlarge the table width and height while preserving a full-width camera view.
2. Add smooth camera follow with vertical bounds.
3. Extend the ASCII point map for a taller table.
4. Add slingshots and proper inlane/outlane geometry.
5. Add rollover lanes and objective lights.
6. Add drop targets and a basic mode progression.
7. Add upper-left and lower-right auxiliary flippers.
8. Add light banks that are intentionally aimed by those auxiliary flippers.
9. Add the enemy hazard loop: turret/creature, missile impulses, mouth exit, temporary disable scoring.
10. Add score popups and hit-specific visual effects.
11. Add audio hooks for the main pinball interactions.
12. Keep improving debug PNG/text output whenever placement becomes ambiguous.
