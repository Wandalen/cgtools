# Feature: 3D Line Rendering

`line_tools` renders 3D polylines as camera-facing quads (`d3::Line`), with optional per-vertex colors, screen- or world-space width, alpha-to-coverage or alpha-tested anti-aliasing, and dashing.

### Scope

- **Purpose**: Give 3D applications perspective-correct, anti-aliased polyline rendering (trails, debug lines, effects) without hand-writing WebGL draw calls.
- **Responsibility**: Cross-reference the source, shaders, and known pitfalls that make up 3D line rendering.
- **In Scope**: Width modes, per-vertex colors, dashing, anti-aliasing, and the WebGL mesh/uniform contract.
- **Out of Scope**: 2D line rendering (caps/joins) — see [feature/001_2d_line_rendering.md](001_2d_line_rendering.md).

### Design

A `d3::Line` renders each segment as a camera-facing quad built from the four-piece rectangle geometry in `src/helpers.rs`. Unlike `d2::Line`, 3D lines have no join or cap geometry.

**Width modes**: `world_units_use( bool )` toggles between constant screen-space pixel width (default) and world-space width that shrinks with distance from the camera.

**Vertex colors**: `vertex_color_use( true )` enables a per-point color, interpolated along the line.

**Anti-aliasing**: `alpha_to_coverage_use( true )` switches the fragment shader from alpha-testing (hard `discard` past the edge) to MSAA alpha-to-coverage (smooth coverage falloff via `smoothstep`) — see `src/d3/shaders/main.frag`.

**Dashing** (requires the `distance` feature): `dash_use( true )` plus `dash_pattern_set( DashPattern::V1..V4 )` and `dash_offset_set`. `DashPattern` variants carry an increasing number of segment lengths (`V1( f32 )` … `V4( [ f32; 4 ] )`). Dashing depends on the cumulative per-point arc length (`total_distance_get`, `distances_get`) tracked as points are added.

**Performance**: point/color `add` is O(1) amortized, `remove` is O(n), and mesh (re)creation is O(n) in point count.

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/003_close_points_segment_overlap.md](../pitfall/003_close_points_segment_overlap.md) | Points closer together than the line width cause heavy segment overlap |
| [pitfall/004_zero_length_segment_break.md](../pitfall/004_zero_length_segment_break.md) | Coincident neighbouring points break the line; `d3::Line`'s point-adding methods have no epsilon guard equivalent to `d2::Line`'s |

### Sources

| File | Relationship |
|------|--------------|
| `src/d3.rs` | `d3` layer declaration (`mod_interface!`) |
| `src/d3/line.rs` | `d3::Line` — point/color storage, mesh, draw |
| `src/helpers.rs` | Shared geometry helpers (four-piece rectangle) |
| `src/d3/shaders/` | Vertex/fragment shaders (perspective width, dashing, anti-aliasing) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/webgl/dash.rs` | Dash pattern, offset, toggle |
| `tests/webgl/distance.rs` | Cumulative distance tracking |
| `tests/webgl/points.rs` | Point add/remove operations |
