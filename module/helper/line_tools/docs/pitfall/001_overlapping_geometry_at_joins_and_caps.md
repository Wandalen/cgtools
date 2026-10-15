# Pitfall: Overlapping Geometry at Joins and Caps

### Scope

- **Purpose**: Record why enabling blending on 2D lines exposes visible seams.
- **Responsibility**: Document the trap, its observable failure, and the available mitigation.
- **In Scope**: The join/cap/body compositing overlap in `d2::Line` rendering.
- **Out of Scope**: The small-angle and close-points overlap cases (see pitfall/002, pitfall/003).

### Trap

Assuming joins, caps, and the segment body can be composited with alpha blending enabled without visible seams.

### Failure

`d2::Line` draws joins, caps, and the segment body as separate, independently-triangulated instanced geometry. Enabling alpha blending (as opposed to opaque or alpha-to-coverage rendering) exposes the overlap between these pieces as visible double-blended seams at every join and cap.

### Mitigation

Prefer `alpha_to_coverage_use( true )` (MSAA-based anti-aliasing) over blending. The overlap itself is an accepted trade-off of drawing caps, joins, and body as separate geometry — there is no code-level fix that eliminates it while blending is enabled.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_2d_line_rendering.md](../feature/001_2d_line_rendering.md) | Feature whose join/cap/body compositing causes this trap |

### Sources

| File | Relationship |
|------|--------------|
| `src/d2/line.rs` | Issues the separate join/cap/body draw calls |
| `src/d2/shaders/` | Body, join, and cap shaders composited per draw |
