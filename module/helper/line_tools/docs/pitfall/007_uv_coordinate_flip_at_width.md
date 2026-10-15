# Pitfall: UV Coordinate Flip at Width

### Scope

- **Purpose**: Record that join/cap UV coordinates shrink and flip sign as line width grows.
- **Responsibility**: Document the trap, its observable failure, and the current verification status.
- **In Scope**: UV computation in the 2D join/cap shaders.
- **Out of Scope**: The overlap-related pitfalls (see pitfall/001, pitfall/002, pitfall/003).

### Trap

Assuming UV coordinates used for join/cap texturing scale linearly and keep a consistent sign as line width increases.

### Failure

As line width grows, the UV coordinates computed for join and cap geometry shrink and flip sign — see the `uvLeft`/`uvRight`/`vUv` computation in `join_miter.vert` for the 2D miter join.

### Mitigation

The pre-migration specification recorded this case as resolved; the fix lives in the UV computation of the relevant join/cap shaders (`src/d2/shaders/`). The specific corrective logic was not independently re-derived during this migration.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | Feature whose join/cap UVs are affected |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/shaders/join_miter.vert` | Contains the `uvLeft`/`uvRight`/`vUv` computation |
| `src/joins.rs` | Join geometry/UV generation for all join kinds |
