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

**[How to run](../../how_to_run.md)**
