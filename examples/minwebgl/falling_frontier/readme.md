# Falling Frontier

**Keywords:** Tactical Grid, Object Picking, Transform Gizmo, WebGL2

A tactical space-scene demo: a fleet of selectable ships and a station orbit an asteroid field,
tracked by a shader-driven tactical grid whose selection-driven view-zone ribbon wraps around
blocking asteroids as a faceted boundary polyline. Selecting a unit shows its info in a HUD
card and a movable/rotatable transform gizmo (translate XZ / rotate Y); fleets move along
Catmull-Rom trajectories with live path and sensor-ring overlays. Object picking uses an
off-screen ID buffer (`gpu_picking`) rather than CPU-side raycasting. A dev tuning panel
exposes every tactical-grid shader parameter live.

**[How to run](../../how_to_run.md)**
