# Pitfall: Close-Points Segment Overlap

### Scope

- **Purpose**: Record that segments overlap heavily when points sit closer together than the line width.
- **Responsibility**: Document the trap, its observable failure, and its (absent) mitigation.
- **In Scope**: Point-spacing vs. line-width overlap in both `d2::Line` and `d3::Line`.
- **Out of Scope**: The small-angle overlap case, which is narrower and partially mitigated (see pitfall/002).

### Trap

Assuming any point spacing renders correctly regardless of the configured line width.

### Failure

When consecutive points are closer together than the line width, segments overlap heavily — more severely than, and independently of, the small-angle case in [pitfall/002](002_small_angle_segment_overlap.md).

### Mitigation

None currently implemented. Keep point spacing proportional to (or larger than) the configured line width, or simplify/resample input polylines before feeding them to `d2::Line` or `d3::Line`.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | 2D lines affected by this trap |
| [feature/002_3d_line_rendering.md](../feature/002_3d_line_rendering.md) | 3D lines affected by this trap |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/line.rs` | 2D point storage — no spacing validation |
| `src/d3/line.rs` | 3D point storage — no spacing validation |
