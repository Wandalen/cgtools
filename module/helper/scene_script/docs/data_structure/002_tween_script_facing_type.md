# Data Structure: Tween Script-Facing Type

### Scope

- **Purpose**: Document the shape a script sees when it holds a `Tween` value — deliberately opaque, in contrast to the field-transparent vector types.
- **Responsibility**: Document why no fields are exposed and what that implies for how a script must interact with the value.
- **In Scope**: The `"Tween"` type exactly as `Engine::register_type_with_name` + `register_fn` expose it.
- **Out of Scope**: `animation::Tween<T>`'s actual Rust definition and full API — owned by `animation`, not re-documented here.

### Abstract

A script sees `Tween` as an opaque handle with no readable fields at all — every access goes through a method (`.update`, `.value`, `.is_completed`), unlike `F32x2`/`F64x2` ([`data_structure/001`](001_f32x2_f64x2_script_facing_vector_types.md)), which expose `.x`/`.y` directly. This is a deliberate consequence of what the type represents: a tween carries internal progress state (elapsed time) that must advance monotonically through `.update`, so exposing raw fields would let a script corrupt that state directly rather than only ever moving it forward through the registered method.

### Structure

```
Tween { }   // no registered fields or getters
```

Confirmed directly against the source: `tween_binding.rs` calls `register_fn` for every operation and never calls `register_get` — the type has zero script-visible fields. Internally, `Tween` is generic over its element vector type (`Tween<F32x2>` and `Tween<F64x2>` are distinct Rust types), but both are registered under the single Rhai type name `"Tween"` — a script never sees the element type as a separate name the way it does for the vector types themselves; the distinction only resurfaces indirectly, through whichever vector type a given `Tween` instance's `.value()`/`.update()` returns.

### Operations

Full call signatures and error behavior live in [`api/001`](../api/001_rhai_scripting_surface.md); this section states only what shape each operation consumes/produces:

- **Construction**: `tween(start, end, duration)` takes two vectors of the same type (either both `F32x2` or both `F64x2`) and a `float` duration, producing an opaque `Tween` value. The easing curve is always Linear — not a parameter (see [`pitfall/006`](../pitfall/006_only_linear_easing_is_exposed_to_scripts.md)).
- **Mutation**: `.update(delta_time)` is the only operation that changes a `Tween`'s internal state; it also returns the freshly-computed value.
- **Read-only access**: `.value()` and `.is_completed()` observe current state without changing it.

### Pitfalls

| File | Relationship |
|------|--------------|
| [006_only_linear_easing_is_exposed_to_scripts.md](../pitfall/006_only_linear_easing_is_exposed_to_scripts.md) | The easing curve this shape's constructor always uses, with no script-facing way to change it |

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
| `src/tween_binding.rs` | `tween_f32x2_register`, `tween_f64x2_register` — the registration that produces this shape |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `tween_f32x2_updates_toward_end_value`, `tween_f64x2_updates_toward_end_value` |
