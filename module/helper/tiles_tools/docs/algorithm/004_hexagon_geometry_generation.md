# Algorithm: Hexagon Geometry Generation

### Scope

- **Purpose**: Document `hexagon_triangles`/`hexagon_triangles_with_transform`/`hexagon_lines`/`hexagon_vertices`, the flat 2D vertex-buffer generators for rendering a single hexagon.
- **Responsibility**: Document the independent-primitive constraint the generators satisfy (`TRIANGLES`/`LINES` draw modes only — never `TRIANGLE_FAN`/`LINE_LOOP`) and the fan-patterned topology of the shared triangulation helper.
- **In Scope**: `hexagon_vertices`, `hexagon_triangles`, `hexagon_triangles_with_transform`, `hexagon_lines`, `triangles_from_vertices` (private helper).
- **Out of Scope**: The coordinate systems these buffers are positioned by (see `type/001`); any renderer that consumes the returned `Vec<f32>` (`tiles_tools` has none of its own).

### Abstract

`src/geometry.rs` generates flat `Vec<f32>` position buffers for a single unit-radius hexagon centered at the origin: 6 vertices (`hexagon_vertices`), a triangulated fill (`hexagon_triangles`/`hexagon_triangles_with_transform`), and an outline (`hexagon_lines`). These are pure geometry generators with no dependency on any grid coordinate type — a caller positions the returned buffer in world space by translating/scaling it per-cell using whichever coordinate system's `to_pixel` conversion it's using (see `algorithm/005`).

**Independent-primitive constraint** (stated in the module doc): every generator returns a primitive soup drawable with plain `TRIANGLES` or `LINES` mode; `TRIANGLE_FAN`/`LINE_LOOP` modes are never required. The constraint exists because `from_iter` concatenates many cells into one buffer drawn in a single call, and mode-level fans/loops cannot express disjoint shapes within one draw call. An earlier revision of this doc described the fill's fan-patterned *topology* as contradicting the then-bare in-source `no fans or loops` comment; resolving that marker (task 063) distinguished the two readings — the constraint governs required *draw modes*, which the shipped independent-triangle encoding satisfies, while fan topology (a shared anchor vertex) is the standard triangulation of a convex polygon and is not what the constraint forbids.

### Algorithm

`hexagon_vertices()` returns the 6 unit-hexagon corner points, counterclockwise from `(1, 0)` (vertex `i` at angle `60° × i`).

`triangles_from_vertices(&[F32x2; 6])` (private helper shared by both fill generators) anchors every triangle at `points[0]` and walks the remaining 5 points pairwise via `.windows(2)`, emitting one standalone triangle per window — `(first, window[0], window[1])` for each of the 4 windows. Fan-patterned topology, independent-triangle encoding: 4 triangles, 24 floats, `TRIANGLES`-ready.

`hexagon_triangles()` delegates directly: `triangles_from_vertices(&hexagon_vertices())`.

`hexagon_triangles_with_transform(transform: F32x3x3)` transforms each corner point by the supplied 3×3 matrix, then runs the same shared helper. Renamed by task 063 from the previously-shipped misspelling `hexagon_triangles_with_tranform`, which had zero callers in the workspace; there is no alias under the old spelling.

`hexagon_lines()` returns the outline as 6 standalone segments — adjacent-vertex pairs plus an explicit closing segment from the last vertex back to the first — 24 floats, `LINES`-ready.

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | Not a direct dependency — cross-referenced to make explicit that these generators take no coordinate-type parameter at all; positioning is entirely the caller's responsibility |

### Sources

| File | Relationship |
|------|--------------|
| `src/geometry.rs` | `hexagon_vertices`, `triangles_from_vertices`, `hexagon_triangles`, `hexagon_triangles_with_transform`, `hexagon_lines`, and the module-doc independent-primitive constraint |

### Tests

`tests/integration/geometry_tests.rs` pins the generators' contracts: vertex count/order/radius, triangle count and the fill's summed area against the analytic unit-hexagon area, outline segment count and closure, the transform variant against manual per-vertex transformation, and `from_iter`'s per-cell replication.
