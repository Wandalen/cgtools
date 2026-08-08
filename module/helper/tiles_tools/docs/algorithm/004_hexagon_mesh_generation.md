# Algorithm: Hexagon Mesh Generation

### Scope

- **Purpose**: Document `hexagon_triangles`/`hexagon_triangles_with_tranform`/`hexagon_lines`/`hexagon_vertices`, the flat 2D vertex-buffer generators for rendering a single hexagon.
- **Responsibility**: Document the fan-triangulation actually performed, disclosing that it contradicts an in-source comment claiming otherwise, and the real (misspelled) public spelling of the transform variant.
- **In Scope**: `hexagon_vertices`, `hexagon_triangles`, `hexagon_triangles_with_tranform`, `hexagon_lines`.
- **Out of Scope**: The coordinate systems these meshes are positioned by (see `type/001`); any renderer that consumes the returned `Vec<f32>` (`tiles_tools` has none of its own).

### Abstract

`src/geometry.rs` generates flat `Vec<f32>` position buffers for a single unit-radius hexagon centered at the origin: 6 vertices (`hexagon_vertices`), a triangulated fill (`hexagon_triangles`/`hexagon_triangles_with_tranform`), and an outline (`hexagon_lines`). These are pure geometry generators with no dependency on any grid coordinate type — a caller positions the returned mesh in world space by translating/scaling it per-cell using whichever coordinate system's `to_pixel` conversion it's using (see `algorithm/005`).

### Algorithm

`hexagon_vertices()` (`src/geometry.rs:168+`) returns the 6 unit-hexagon corner points.

`hexagon_triangles()` (`src/geometry.rs:59-79`) triangulates by holding `points[0]` fixed and walking the remaining 5 points pairwise via `.windows(2)`, emitting one triangle per window — `(first, window[0], window[1])` for each of the 4 windows. This is a **fan triangulation**: one shared central vertex (`points[0]`), fanning out across the remaining edge pairs. This directly contradicts the module's own leading comment, `// aaa : no fans or loops` (`src/geometry.rs:11`) — the shipped implementation is exactly the pattern that comment says to avoid. The output is still geometrically correct (4 triangles correctly tile the hexagon, matching `hexagon_triangles_with_tranform`'s own doc comment: *"The hexagon is divided into 4 triangles"*), so this is a discipline/consistency note for anyone extending the module under the stated no-fans constraint, not a rendering defect.

`hexagon_triangles_with_tranform(transform: F32x3x3)` (`src/geometry.rs:86+`) applies the same fan triangulation after transforming each vertex by the supplied 3×3 matrix. **Its name is the real, callable public API spelling** — `tranform`, missing the middle `s` in "transform" — verified directly against the function signature in source, not a doc-comment-only typo. Any caller of this function must match the misspelling exactly; there is no correctly-spelled alias.

`hexagon_lines()` (`src/geometry.rs:131+`) returns the 6-edge outline as line-segment position pairs, independent of the triangulation functions above.

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | Not a direct dependency — cross-referenced to make explicit that these generators take no coordinate-type parameter at all; positioning is entirely the caller's responsibility |

### Sources

| File | Relationship |
|------|--------------|
| `src/geometry.rs` | `hexagon_vertices`, `hexagon_triangles`, `hexagon_triangles_with_tranform`, `hexagon_lines`, the `// aaa : no fans or loops` comment |

### Tests

No dedicated regression test currently pins the fan-triangulation-vs-comment inconsistency, or the `hexagon_triangles_with_tranform` spelling, as intentional.
