# Pitfall: Zero-Length Segment Break

### Scope

- **Purpose**: Record why coincident neighbouring points must never reach line geometry generation.
- **Responsibility**: Document the trap, its observable failure, and the asymmetry between the 2D and 3D mitigations.
- **In Scope**: Duplicate-point handling in `d2::Line::point_add` and `d3::Line`'s point-adding methods.
- **Out of Scope**: General close-but-not-coincident point spacing (see pitfall/003).

### Trap

Assuming every consecutive pair of points added to a line defines a valid, non-zero-length segment.

### Failure

Two coincident (or near-coincident) neighbouring points produce a zero-length direction vector, which breaks downstream tangent/normal computation and produces degenerate, undrawable geometry at that point.

### Mitigation

`d2::Line::point_add` (`src/d2/line.rs`) compares each new point against the last point with an `EPSILON` of `1e-8` on both axes and silently discards the new point if it falls within that tolerance, so coincident points never reach the mesh. `d3::Line`'s point-adding methods (`src/d3/line.rs`) contain no equivalent guard as of this migration — coincident points passed to `d3::Line` are not known to be deduplicated.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | 2D lines — mitigated via the `EPSILON` dedup in `point_add` |
| [feature/002_3d_line_rendering.md](../feature/002_3d_line_rendering.md) | 3D lines — no equivalent guard found |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/line.rs` | Contains the `EPSILON` dedup guard in `point_add` |
| `src/d3/line.rs` | No equivalent guard |
