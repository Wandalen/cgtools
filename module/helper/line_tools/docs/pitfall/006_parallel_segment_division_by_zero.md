# Pitfall: Parallel-Segment Division by Zero

### Scope

- **Purpose**: Record the degenerate-tangent risk when neighbouring segments are parallel and opposite, and its resolution (BUG-158).
- **Responsibility**: Document the trap, its observable failure, and the guard now in place.
- **In Scope**: The shared tangent computation duplicated (as identical copy-pasted GLSL) across `join_miter.vert`, `join_bevel.vert`, `join_round.vert`, `body.vert` and `body_terminal.vert`.
- **Out of Scope**: The small-angle and close-points overlap cases (see pitfall/002, pitfall/003).

### Trap

Assuming the tangent computation shared by all 5 `.vert` files — `normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) )` (`p2 - p1`/`p1 - p0` in `body.vert`'s own naming) — is always well-defined.

### Failure

When two neighbouring segments are parallel and opposite (a ~180° cusp), the two unit direction vectors summed in the tangent computation cancel toward zero; normalizing a zero (or near-zero) vector produces `NaN` in GLSL (0/0 on both components), corrupting the joint's/segment's geometry and propagating into `gl_Position`.

### Mitigation

**Resolved (BUG-158).** All 5 sites now guard the sum's squared length before normalizing (`dot( tangentSum, tangentSum ) > 1e-12 ? normalize( tangentSum ) : dirIn`), falling back to the incoming segment's own direction (already unit-length) when the sum collapses — see `join_miter.vert`'s `Fix(BUG-158)` comment for the full root cause, and `tests/webgl/join_tangent.rs` for a Rust-side regression port of the guard (this crate has no shader-execution test harness, so the GLSL itself isn't directly exercised by `cargo test`). The join geometry template (vertex positions, indices) is generated in `src/joins.rs`, which does not itself compute a tangent and was unaffected.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | Feature whose join/body tangent computation was at risk |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/shaders/join_miter.vert` | Contains the guarded tangent computation; full `Fix(BUG-158)` root cause comment |
| `src/d2/shaders/join_bevel.vert` | Same guard (cross-references join_miter.vert) |
| `src/d2/shaders/join_round.vert` | Same guard (cross-references join_miter.vert) |
| `src/d2/shaders/body.vert` | Same guard (cross-references join_miter.vert) |
| `src/d2/shaders/body_terminal.vert` | Same guard (cross-references join_miter.vert) |
| `tests/webgl/join_tangent.rs` | Rust-side regression port of the guard (BUG-158) |
| `src/joins.rs` | `Join` geometry template generation — no tangent computation, unaffected |
