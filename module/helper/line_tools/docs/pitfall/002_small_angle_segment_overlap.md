# Pitfall: Small-Angle Segment Overlap

### Scope

- **Purpose**: Record how the miter join keeps its offset corner within its own segment pair at small angles.
- **Responsibility**: Document the trap, its observable failure, and the verified mitigation.
- **In Scope**: The miter join's overlap clamp in `join_miter.vert`.
- **Out of Scope**: The non-neighbour side effect of this clamp (see pitfall/005).

### Trap

Assuming the miter join's offset corner geometry always stays within the two segments it connects, regardless of the angle between them.

### Failure

At a small join angle combined with a large line width, the naively-computed offset corner (`offsetPoint`) extends past the adjacent segment, producing visible overlap.

### Mitigation

`join_miter.vert` detects this case explicitly — `dot( offsetPoint - intersectionPoint, normal * sigma ) < 0.0` — and re-projects `offsetPoint` onto the adjacent segment (`leftBottomCornerA + AB`), clamping the corner back within bounds. This clamp is itself the cause of [pitfall/005](005_non_neighbor_segment_overlap.md).

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | Feature whose miter join contains this clamp |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/shaders/join_miter.vert` | Contains the overlap-detection clamp |
| `src/joins.rs` | `Join::Miter` geometry generation |
