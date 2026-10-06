# Falling Frontier

**Keywords:** Tactical Grid, Object Picking, Transform Gizmo, WebGL2

A tactical space-scene demo: a fleet of selectable ships and a station orbit an asteroid field,
tracked by a shader-driven tactical grid whose selection-driven view-zone ribbon wraps around
blocking asteroids as a faceted boundary polyline. Selecting a unit shows its info in a HUD
card and a movable/rotatable transform gizmo (translate XZ / rotate Y); fleets follow
Catmull-Rom patrol paths. Object picking uses an off-screen ID buffer (`gpu_picking`) rather
than CPU-side raycasting. A dev tuning panel exposes every tactical-grid shader parameter live;
the nebula backdrop is baked once into a cube map at startup and sampled every frame rather
than re-evaluated per pixel.

Controls:
- Left-click drag - orbit camera
- Right-click drag - pan camera
- Scroll wheel - zoom camera
- Click a ship / asteroid / the station - select it (a drag past ~6px counts as a camera drag, not a click)
- G - switch the gizmo to translate mode (requires a selection)
- R - switch the gizmo to rotate mode (requires a selection)
- Escape - deselect; during a gizmo drag, cancel the drag instead
- HUD Pause / Play / Fast buttons - control simulation speed
- HUD "Reset Camera" button - restores the initial camera framing
- Render Layers panel (bottom left) - click a row to switch it. Scene layers (grid, background,
  starfield, asteroids, ships, station): right click a row to show only that layer, Shift + right
  click to hide it and show the other scene layers (mouse only). Overlays and lighting (view-zone
  ribbon, selection gizmo, lighting, shadows, CRT scanlines) are left alone by those gestures; the
  ribbon row is greyed out while the grid is off, and shadows while lighting is off
- Dev tuning panel (bottom right) - exposes every tactical-grid shader parameter live

**[How to run](../../how_to_run.md)**
