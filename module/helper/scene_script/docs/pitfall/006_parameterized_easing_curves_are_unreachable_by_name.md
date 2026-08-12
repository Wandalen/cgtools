# Pitfall: Parameterized Easing Curves Are Unreachable by Name

### Scope

- **Purpose**: Warn that a script's easing-curve selector only reaches curves that build with zero arguments — `CubicHermite` and `Squad`, which both need constructor arguments, cannot be named from a script no matter how the name is spelled.
- **Responsibility**: Document the concrete limitation, its evidence, and today's host-side workaround.
- **In Scope**: `easing_from_name`'s fixed match arms in `tween_binding.rs`.
- **Out of Scope**: Whether extending the selector to accept constructor arguments is deliberate scope or an unfinished surface — an open question this pitfall does not resolve either way (see [`feature/001`](../feature/001_rhai_scene_scripting.md)'s Design section).

### Trap

`animation::easing` exposes more than the 25 curves `easing_from_name` recognizes (`"Linear"` plus 24 CSS-style presets from `easing::cubic::bezier`) — `easing::cubic::hermite::CubicHermite` and `easing::squad::Squad` are both real, working `EasingFunction` implementors in the same crate. A reader who knows the host supports them might reasonably expect `tween(start, end, duration, "CubicHermite")` or `tween(start, end, duration, "Squad")` to work the same way `"EaseInOutQuad"` does. Neither name is recognized.

### Failure

`tween(start, end, duration, "CubicHermite")` and `tween(start, end, duration, "Squad")` both fail the same way any other unrecognized name does: a script-catchable runtime error whose message contains `"unknown easing curve name"` (see `tween_with_easing_selector_rejects_unknown_curve_name`, `tests/engine_test.rs`, for the general case this specific pair falls into). This is a loud, catchable failure, not a silent wrong-behavior trap — the risk here is a script author assuming the name merely needs the right spelling and retrying variants, rather than recognizing the curve is structurally unreachable through this constructor at all.

### Why These Two Specifically

Every curve `easing_from_name` *does* recognize implements `EasingBuilder`, whose `build()` takes zero arguments — `Linear::build()`, `EaseInOutQuad::build()`, and so on all produce a ready-to-use curve from nothing but their own type. `CubicHermite<T>` and `Squad<E>` implement `EasingFunction` directly but not `EasingBuilder`: `CubicHermite::new(m1: T, m2: T)` needs two tangent vectors, and `Squad::new(in_tangent: Quat<E>, out_tangent: Quat<E>)` needs two tangent quaternions. A bare string can select between zero-argument presets; it cannot supply the vectors or quaternions these two constructors require. This is *why* the easing-selector work completed this session — extending `tween(...)`'s 3-arg form with a 4-arg named-curve overload — could close the gap for all 25 zero-argument curves but structurally cannot reach these two without a larger change.

### Mitigation

Reaching `CubicHermite` or `Squad` from a script would need a richer script-facing constructor shape than a name string — e.g. a `tween(start, end, duration, "CubicHermite", m1, m2)` overload accepting the tangent vectors as additional arguments (and an equivalent quaternion-accepting form for `Squad`, which also first needs a script-facing quaternion type — none is registered today). Until then, either curve requires a host-side workaround: construct the `Tween` in Rust with the desired `CubicHermite`/`Squad` instance and drive it from host code, passing only the resulting values into the script rather than letting the script construct the tween itself.

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | The registration technique the completed easing-selector work already extended; a further extension for parameterized curves would need a new argument-passing shape, not just this pattern repeated |

### Data Structures

| File | Relationship |
|------|--------------|
| [../data_structure/001_tween_script_facing_type.md](../data_structure/001_tween_script_facing_type.md) | The opaque shape whose 4-arg constructor overload can select any zero-argument curve but not these two |

### Sources

| File | Relationship |
|------|--------------|
| `src/tween_binding.rs` | `easing_from_name`'s match arms — `"Linear"` plus the 24 `cubic::bezier` preset names, and no others |

### Tests

No dedicated regression test names `CubicHermite`/`Squad` specifically — `tween_with_easing_selector_rejects_unknown_curve_name` (`tests/engine_test.rs`) pins the general unrecognized-name error path both would fall into, using an arbitrary made-up name rather than either of these two real-but-unreachable curves.
