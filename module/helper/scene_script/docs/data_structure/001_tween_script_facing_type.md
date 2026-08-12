# Data Structure: Tween Script-Facing Type

### Scope

- **Purpose**: Document the shape a script sees when it holds a `Tween` value — deliberately opaque, in contrast to the field-transparent vector types.
- **Responsibility**: Document why no fields are exposed and what that implies for how a script must interact with the value.
- **In Scope**: The `"Tween"` type exactly as `Engine::register_type_with_name` + `register_fn` expose it.
- **Out of Scope**: `animation::Tween<T>`'s actual Rust definition and full API — owned by `animation`, not re-documented here.

### Abstract

A script sees `Tween` as an opaque handle with no readable fields at all — every access goes through a method (`.update`, `.value`, `.is_completed`), unlike the 8 registered vector types (e.g. [`type/001`](../type/001_f32x2_script_facing_vector_value.md), [`type/002`](../type/002_f64x2_script_facing_vector_value.md); see the [`type/`](../type/readme.md) collection for the full set), which expose their components (`.x` and, depending on arity, `.y`/`.z`/`.w`) directly. This is a deliberate consequence of what the type represents: a tween carries internal progress state (elapsed time) that must advance monotonically through `.update`, so exposing raw fields would let a script corrupt that state directly rather than only ever moving it forward through the registered method.

### Structure

```
Tween { }   // no registered fields or getters
```

Confirmed directly against the source: `tween_binding.rs` calls `register_fn` for every operation and never calls `register_get` — the type has zero script-visible fields. Internally, `Tween` is generic over its element vector type (`Tween<F32x1>`, `Tween<F32x2>`, `Tween<F32x3>`, `Tween<F32x4>`, `Tween<F64x1>`, `Tween<F64x2>`, `Tween<F64x3>`, `Tween<F64x4>` are 8 distinct Rust types), but all 8 are registered under the single Rhai type name `"Tween"` — a script never sees the element type as a separate name the way it does for the vector types themselves; the distinction only resurfaces indirectly, through whichever vector type a given `Tween` instance's `.value()`/`.update()` returns. This single-name registration is also why `Tween` is documented as one `data_structure/` instance rather than split 8 ways like the vector types: there is exactly one script-visible name to document, not eight.

### Operations

Full call signatures and error behavior live in [`api/001`](../api/001_rhai_scripting_surface.md); this section states only what shape each operation consumes/produces:

- **Construction**: `tween(start, end, duration)` takes two vectors of the same registered type (both arguments must be the identical one of the 8 — e.g. both `F32x2`, or both `F64x3`, never mixed precision or arity) and a `float` duration, producing an opaque `Tween` value eased Linearly. A 4-arg overload, `tween(start, end, duration, easing)`, takes a curve name instead of defaulting to Linear (see [`pitfall/006`](../pitfall/006_parameterized_easing_curves_are_unreachable_by_name.md) for exactly which names are accepted and which curves remain unreachable this way).
- **Mutation**: `.update(delta_time)` remains the only operation that advances interpolation, but it is no longer the only mutating operation — `.pause()`/`.resume()`/`.reset()` also change internal state (halting/resuming progress, or returning to the start value with elapsed time and repeat count zeroed).
- **Read-only access**: `.value()`, `.is_completed()`, `.progress()`, `.duration()`, `.delay()`, `.time()`, `.current_repeat()`, and `.state()` observe current state without changing it — the last returns the lifecycle stage (`"Pending"`/`"Running"`/`"Paused"`/`"Completed"`) as a string.
- **Builder-style reconfiguration**: `.with_delay(value)`, `.with_duration(value)`, `.with_repeat(count)`, and `.with_yoyo(enabled)` each consume the `Tween` by value and return a modified copy for chaining (`t.with_delay(0.5).with_duration(2.0)`) — same non-mutating-receiver shape as the vector types' own operations, not the `.update()`/`.pause()`/`.resume()`/`.reset()` in-place style.

### Pitfalls

| File | Relationship |
|------|--------------|
| [006_parameterized_easing_curves_are_unreachable_by_name.md](../pitfall/006_parameterized_easing_curves_are_unreachable_by_name.md) | The named-easing-curve subset this shape's constructor can select, and which curves remain unreachable |

### Patterns

| File | Relationship |
|------|--------------|
| [002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | How this shape stays a single script-visible name across two distinct Rust-generic instantiations |

### APIs

| File | Relationship |
|------|--------------|
| [001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | Full operational contract (signatures, error handling) for the operations summarized above |

### Sources

| File | Relationship |
|------|--------------|
| `src/tween_binding.rs` | `tween_f32x1_register`, `tween_f32x2_register`, `tween_f32x3_register`, `tween_f32x4_register`, `tween_f64x1_register`, `tween_f64x2_register`, `tween_f64x3_register`, `tween_f64x4_register` — the 8 registrations that produce this shape, one per vector type |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `tween_f32x1_updates_toward_end_value`, `tween_f32x2_updates_toward_end_value`, `tween_f32x3_updates_toward_end_value`, `tween_f32x4_updates_toward_end_value`, `tween_f64x1_updates_toward_end_value`, `tween_f64x2_updates_toward_end_value`, `tween_f64x3_updates_toward_end_value`, `tween_f64x4_updates_toward_end_value` — construction and `.update` roundtrip, one per vector type; `tween_progress_reports_fraction_of_duration_elapsed`, `tween_builder_methods_configure_duration_and_delay`, `tween_time_accumulates_elapsed_delta_time`, `tween_pause_halts_further_progress_until_resumed`, `tween_reset_returns_to_start_value`, `tween_current_repeat_increments_after_each_repeat_cycle`, `tween_with_yoyo_reverses_direction_on_alternate_repeats`, `tween_state_reports_animation_lifecycle_stage`, `tween_with_easing_selector_accepts_named_curve`, `tween_with_easing_selector_rejects_unknown_curve_name` — the new accessor/control/builder/easing-selector operations, all exercised via `F32x1` as the representative element type |
