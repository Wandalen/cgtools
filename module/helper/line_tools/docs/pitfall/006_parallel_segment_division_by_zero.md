# Pitfall: Parallel-Segment Division by Zero

### Scope

- **Purpose**: Record the degenerate-tangent risk when neighbouring segments are parallel.
- **Responsibility**: Document the trap, its observable failure, and the current verification gap.
- **In Scope**: The miter join tangent computation in `join_miter.vert`.
- **Out of Scope**: The small-angle and close-points overlap cases (see pitfall/002, pitfall/003).

### Trap

Assuming the miter join's tangent computation — `normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) )` — is always well-defined.

### Failure

When two neighbouring segments are parallel and opposite (a near-180° turn), the two unit direction vectors summed in the tangent computation cancel toward zero; normalizing a zero (or near-zero) vector produces undefined/NaN geometry, breaking the line at that join.

### Mitigation

The pre-migration specification recorded this case as resolved. The join geometry is generated in `src/joins.rs` and `src/d2/shaders/join_miter.vert`; an explicit guard against a degenerate (parallel, zero-sum) tangent was not conclusively identified in `join_miter.vert` during this migration. Confirm against current shader source before relying on this case for parallel-heavy input geometry.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | Feature whose miter join tangent computation is at risk |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/shaders/join_miter.vert` | Contains the tangent computation |
| `src/joins.rs` | `Join::Miter` geometry generation |
