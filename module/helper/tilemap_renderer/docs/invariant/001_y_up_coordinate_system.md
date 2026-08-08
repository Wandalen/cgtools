# Invariant: Y-Up Coordinate System

### Scope

- **Purpose**: Guarantee that callers author transforms and rotations against one coordinate convention regardless of which backend renders them.
- **Responsibility**: Document the Y-up contract every backend must present, and how each backend currently satisfies it.
- **In Scope**: Position, rotation-direction, and scale conventions for `Transform` as observed through any `Backend` implementation.
- **Out of Scope**: The `Transform::depth` ordering contract (see [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md), which is WebGL2-specific rather than cross-backend).

### Invariant Statement

Every backend presents a **Y-up** coordinate system to callers: `(0, 0)` is the bottom-left corner, positive Y points up, and positive rotation is counter-clockwise (CCW) — regardless of the native convention of the underlying rendering technology. A caller building a command stream never needs to branch on which backend will process it to get consistent placement, rotation direction, or scale sign.

### Enforcement Mechanism

- **WebGL2 adapter**: matches Y-up natively (OpenGL's own convention), so no conversion is applied.
- **SVG adapter**: SVG's native convention is Y-down (`(0, 0)` at the top-left), so `src/adapters/svg.rs` converts at emission time for every positioned element: position Y is flipped (`height - y`), rotation is negated (CCW in Y-up becomes CW in SVG's Y-down space), and scale Y is negated — emitted as `scale(1,-1)` even at identity scale so the flip is always present. Viewport pan/zoom is composed into the same conversion via a single top-level `<g transform="scale(s) translate(ox,-oy)">` wrapper (see the `transform_to_svg` / local-transform split in source) so batch-instance transforms drawn inside that wrapper use raw, unflipped local coordinates rather than double-converting.
- Verified directly in source: the conversion logic sits in `src/adapters/svg.rs` (`transform_to_svg`, the "Y-up (0,0 = bottom-left) → SVG Y-down (0,0 = top-left)" comment block, and the drop-shadow direction negation), and is covered by dedicated unit tests — `transform_y_up_bottom_left_origin`, `transform_y_up_top_right`, `transform_y_up_center`, `transform_identity_scale_emits_y_flip`, `local_transform_no_y_flip`, and `effect_drop_shadow_y_flipped` — all inline in `src/adapters/svg.rs`.

### Violation Consequences

A backend (or an SVG code path) that omits the conversion would render mirrored or vertically-flipped content with inverted rotation direction, with no compiler or runtime error — the output would simply place and rotate things wrong. Because SVG is the only shipped adapter that needs an active conversion, this is the concrete risk surface for any future SVG code path that positions or rotates an element without going through `transform_to_svg` (or the local-transform variant for wrapper-relative instances); the existing transform tests are the primary guard against regressing it silently.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md) | Actively converts Y-up to SVG's native Y-down at emission time |
| [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) | Satisfies Y-up natively; no conversion needed |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/svg.rs` | `transform_to_svg` and the local (wrapper-relative) transform path — the sole active conversion |
| `src/types.rs` | `Transform` type doc comment states the Y-up contract that backends must honor |

### Tests

| File | Relationship |
|------|--------------|
| `src/adapters/svg.rs` (inline `#[cfg(test)]`) | `transform_y_up_bottom_left_origin`, `transform_y_up_top_right`, `transform_y_up_center`, `transform_identity_scale_emits_y_flip`, `local_transform_no_y_flip`, `effect_drop_shadow_y_flipped` |
