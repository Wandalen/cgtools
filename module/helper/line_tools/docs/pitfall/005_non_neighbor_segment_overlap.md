# Pitfall: Non-Neighbor Segment Overlap

### Scope

- **Purpose**: Record the known side effect of the small-angle overlap clamp.
- **Responsibility**: Document the trap, its observable failure, and its (absent) mitigation.
- **In Scope**: Non-local geometric side effects of the `join_miter.vert` overlap clamp.
- **Out of Scope**: The small-angle clamp itself (see pitfall/002).

### Trap

Assuming the small-angle overlap clamp in `join_miter.vert` (see [pitfall/002](002_small_angle_segment_overlap.md)) affects only the join it corrects.

### Failure

The corner-case-2 clamp can produce unusual geometric overlap between segments that are not direct neighbours of the clamped join.

### Mitigation

None currently implemented. Open issue in the `join_miter.vert` clamp path.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | Feature affected by this non-local side effect |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/shaders/join_miter.vert` | Contains the clamp that causes this side effect |
