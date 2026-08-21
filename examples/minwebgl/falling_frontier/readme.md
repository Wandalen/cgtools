# Falling Frontier

**Keywords:** Tactical Grid, Object Picking, Transform Gizmo, WebGL2

A tactical space-scene demo: a fleet of selectable ships and a station orbit an asteroid field,
tracked by a shader-driven tactical grid whose selection-driven view-zone ribbon wraps around
blocking asteroids as a faceted boundary polyline. Selecting a unit shows its info in a HUD
card and a movable/rotatable transform gizmo (translate XZ / rotate Y); fleets follow
Catmull-Rom patrol paths, with each ship's route drawable as a trajectory ribbon. Object
picking uses an off-screen ID buffer (`gpu_picking`) rather than CPU-side raycasting. A dev
tuning panel exposes every tactical-grid shader parameter live; the nebula backdrop is baked
once into a cube map at startup and sampled every frame rather than re-evaluated per pixel.

Controls:
- Left-click drag - orbit camera
- Right-click drag - pan camera
- Scroll wheel - zoom camera
- Click a ship / asteroid / the station - select it (a drag past ~6px counts as a camera drag, not a click)
- G - switch the gizmo to translate mode (requires a selection)
- R - switch the gizmo to rotate mode (requires a selection)
- Escape - deselect
- HUD toggle buttons (Tactical Grid / Vector Trajectories / Sensor Ranges & Rings / CRT Scanlines / Animate Ships Motion) - toggle each overlay independently
- HUD Pause / Play / Fast buttons - control simulation speed
- HUD "Reset Camera View" button - restores the initial camera framing
- Dev tuning panel - exposes every tactical-grid shader parameter live

**[How to run](../../how_to_run.md)**
