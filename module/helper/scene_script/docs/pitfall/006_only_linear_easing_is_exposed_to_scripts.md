# Pitfall: Only Linear Easing Is Exposed to Scripts

### Scope

- **Purpose**: Warn that a script cannot request any easing curve other than Linear, even though the underlying `animation` crate supports more.
- **Responsibility**: Document the concrete limitation, its evidence, and today's host-side workaround.
- **In Scope**: `tween_f32x2_register`/`tween_f64x2_register`'s hardcoded `Linear::build()`.
- **Out of Scope**: Whether this is deliberate scope or an unfinished surface — an open question this pitfall does not resolve either way (see [`feature/001`](../feature/001_rhai_scene_scripting.md)'s Design section).

### Trap

`animation::easing::base` exposes more than `Linear` — the import in `tween_binding.rs` itself is scoped as `easing::base::{ EasingBuilder, Linear }`, naming `Linear` as one member of a `base` family rather than the only easing available in the host crate. A reader who knows the host supports richer easing curves might reasonably expect a script-facing way to select one — e.g. an easing-name argument on `tween(...)`. No such parameter, overload, or alternate constructor exists anywhere in this crate.

### Failure

Every `tween(start, end, duration)` call, regardless of its arguments, produces a `Tween` that interpolates linearly and only linearly. There is no error, warning, or any other signal — a script simply gets Linear motion silently whenever something else might have been intended, and there is no way to detect this from script alone (no method reports which easing curve is in effect, because only one has ever been reachable).

### Mitigation

Today, non-linear easing requires a host-side workaround: pre-shape the interpolation curve in Rust before handing values to the script, or drive a tween's progress manually from host code rather than through the registered `tween(...)` constructor. Extending `tween_f32x2_register`/`tween_f64x2_register` to accept an easing selector is the structural fix, following the same registration technique already in use — see [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) for the seam this would extend.

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | The extension seam that would need to grow to expose additional easing curves |

### Data Structures

| File | Relationship |
|------|--------------|
| [../data_structure/001_tween_script_facing_type.md](../data_structure/001_tween_script_facing_type.md) | The opaque shape whose constructor carries this hardcoded choice |

### Sources

| File | Relationship |
|------|--------------|
| `src/tween_binding.rs` | `tween_f32x2_register`/`tween_f64x2_register`'s `Linear::build()` |

### Tests

No dedicated regression test pins this as a limitation — the existing tween tests (`tests/engine_test.rs`) exercise Linear behavior because it is the only behavior available, not because they specifically assert the absence of other easing curves.
