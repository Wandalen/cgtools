# Feature: 2D Line Rendering

`line_tools` renders thick, anti-aliased 2D polylines in WebGL 2.0 (`d2::Line`) as instanced geometry, with independently configurable caps and joins and incremental point editing.

### Scope

- **Purpose**: Give 2D applications configurable, anti-aliased polyline rendering without hand-writing WebGL draw calls.
- **Responsibility**: Cross-reference the source, shaders, and known pitfalls that make up 2D line rendering.
- **In Scope**: Cap/join configuration, incremental point editing, serialization of styling types, and the WebGL mesh/uniform contract.
- **Out of Scope**: 3D line rendering (see [feature/002_3d_line_rendering.md](002_3d_line_rendering.md)); shared mesh/program plumbing (see `src/mesh.rs`, `src/program.rs`).

### Design

A `d2::Line` is a list of 2D points rendered as instanced quads: one instanced draw per interior segment (body), one per join, and one per line terminal (cap). Joins and caps are separate geometry from the segment body, composited in the same draw pass — this is the source of [pitfall/001](../pitfall/001_overlapping_geometry_at_joins_and_caps.md) (visible overlap under blending).

**Caps** (`Cap` enum): `Butt` (default, no extra geometry), `Round( segments )`, `Square` — cover the two line terminals.

**Joins** (`Join` enum): `Miter( h, v )`, `Round( h, v )`, `Bevel( h, v )` — connect consecutive segments; `h`/`v` control triangulation precision. The miter join computes a bend direction and offset corner per join in `join_miter.vert`; see [pitfall/002](../pitfall/002_small_angle_segment_overlap.md) and [pitfall/006](../pitfall/006_parallel_segment_division_by_zero.md) for its known edge cases.

**Point editing**: `point_add` deduplicates a new point against the last-added point within `1e-8` (`EPSILON`) on both axes, silently dropping the duplicate before it reaches the mesh — the concrete mitigation for [pitfall/004](../pitfall/004_zero_length_segment_break.md). Editing is incremental: `mesh_update` re-uploads only the buffers that changed since the previous call.

**Serialization**: `Cap` and `Join` implement `serde::{ Serialize, Deserialize }` under the `serialization` feature.

**Overlap and anti-aliasing**: because joins, caps, and the segment body are drawn as separate, independently-triangulated geometry, blending — or a sufficiently small join angle, or points closer together than the line width — can expose the seams between them; see [pitfall/001](../pitfall/001_overlapping_geometry_at_joins_and_caps.md), [pitfall/002](../pitfall/002_small_angle_segment_overlap.md), and [pitfall/003](../pitfall/003_close_points_segment_overlap.md).

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_overlapping_geometry_at_joins_and_caps.md](../pitfall/001_overlapping_geometry_at_joins_and_caps.md) | Visible overlap under blending — inherent to separately-drawn join/cap/body geometry |
| [pitfall/002_small_angle_segment_overlap.md](../pitfall/002_small_angle_segment_overlap.md) | Small join angle + large width causes neighbouring segments to overlap |
| [pitfall/003_close_points_segment_overlap.md](../pitfall/003_close_points_segment_overlap.md) | Points closer together than the line width causes heavy segment overlap |
| [pitfall/004_zero_length_segment_break.md](../pitfall/004_zero_length_segment_break.md) | Coincident neighbouring points break the line via a zero-length direction vector |
| [pitfall/005_non_neighbor_segment_overlap.md](../pitfall/005_non_neighbor_segment_overlap.md) | Side effect of the corner-case-2 clamp — unusual overlap between non-neighbouring segments |
| [pitfall/006_parallel_segment_division_by_zero.md](../pitfall/006_parallel_segment_division_by_zero.md) | Parallel neighbouring segments cause a division by zero in the join geometry |
| [pitfall/007_uv_coordinate_flip_at_width.md](../pitfall/007_uv_coordinate_flip_at_width.md) | UV coordinates shrink and flip sign as line width increases |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2.rs` | `d2` layer declaration (`mod_interface!`) |
| `src/d2/line.rs` | `d2::Line` — point storage, mesh (re)generation, draw |
| `src/caps.rs` | `Cap` enum and cap geometry generation |
| `src/joins.rs` | `Join` enum and join geometry generation |
| `src/d2/shaders/` | Body, terminal, join (miter/round/bevel), and cap (round/square) vertex shaders |

### Tests

| File | Relationship |
|------|--------------|
| `tests/webgl/points.rs` | Point add/remove operations |
| `tests/webgl/distance.rs` | Cumulative distance tracking shared with `d3::Line` |
