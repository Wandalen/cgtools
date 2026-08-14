# Type: F32x1 (Script-Facing Vector Value)

- **domain**: vector
- **ddd**: value_object

### Scope

- **Purpose**: Define `F32x1` as a Rhai-registered Domain Type — a single-precision 1-component vector value a script constructs, reads, and combines — distinct from `F64x1` ([`type/004`](004_f64x1_script_facing_vector_value.md)) despite sharing the same shape.
- **Responsibility**: State the type's domain meaning, its construction/validation rules, and its relationships to its double-precision counterpart and to `Tween`.
- **In Scope**: The Rhai-registered projection named `"F32x1"` — the script-visible type, not the Rust struct behind it.
- **Out of Scope**: `ndarray_cg::F32x1`'s own Rust definition, which `scene_script` does not own (see [`pattern/001`](../pattern/001_manual_customtype_registration_for_foreign_types.md) for why registration is manual rather than a trait derive); full call signatures and error behavior (see [`api/001`](../api/001_rhai_scripting_surface.md)).

### Definition

`F32x1` is the minimal member of `ndarray_cg`'s `{Element}x{Arity}` family: a 1-component vector held at single (`f32`) precision. A script constructs one via `f32x1(x)` and reads its component via `.x` — read-only; no operation registered anywhere mutates an existing `F32x1` in place. Every operation that produces an `F32x1` (`f32x1(...)`, `+`, `-`, `*`, unary `-`, `normalize`, `min`, `max`) produces a *new* value; there is no way to change one in place — this includes `normalize`, despite its name suggesting in-place mutation. Identity is purely structural: two `F32x1` values built from the same `x` are indistinguishable to a script, with no separate identity beyond their own component.

At arity 1, `F32x1` carries no arithmetic behavior a raw `f32` scalar wouldn't already have — its purpose is uniformity, not new capability: it lets the vector family's registration pattern ([`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md)) and a script's mental model extend down to a single component without a special case. The domain meaning is otherwise deliberately unconstrained, same as every other arity: `scene_script` registers no interpretation beyond "a 1D single-precision vector."

### Validation

No construction is ever rejected. `f32x1(x)` accepts any value Rhai can supply as `FLOAT` (Rhai's own `f64`) — including `NaN` and infinities — and always succeeds; there is no rejection rule of any kind. The only transformation applied is precision narrowing: `x` is cast `as f32` at construction (`src/vector_binding.rs`), which can silently lose precision for an input that doesn't round-trip exactly through `f32` — see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md) for the concrete failure mode this causes.

### Relationships

- [`type/004`](004_f64x1_script_facing_vector_value.md) — `F64x1` is the double-precision counterpart, sharing the same shape and operation set. The two never implicitly convert (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)); a script commits to one precision at construction.
- `Tween<F32x1>` — `tween(start, end, duration)` accepts two `F32x1` values as its `start`/`end` arguments, producing a `Tween` that interpolates between them (see [`data_structure/001`](../data_structure/001_tween_script_facing_type.md)).
- [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the registration technique that keeps `F32x1` and `F64x1` distinct, non-interchangeable, side-by-side names, and how the same technique extended the pre-existing `F32x2`/`F64x2` pair down to this arity.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | `F32x1` never implicitly converts to/from `F64x1` |
| [../invariant/003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | Why the registered name is exactly `"F32x1"` / `"f32x1"` |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/004_f32_boundary_cast_truncates_precision.md](../pitfall/004_f32_boundary_cast_truncates_precision.md) | The precision-narrowing consequence of this type's `f32` element width |

### APIs

| File | Relationship |
|------|--------------|
| [../api/001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | Full operational contract (signatures, error handling) for every operation this type participates in |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x1_register` — constructor, `.x` getter, `+`/`-` (binary)/`*` operators, unary `-` negation, `dot`/`mag`/`mag2`/`normalize`/`distance`/`distance_squared`/`min`/`max`, `to_string` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x1_arithmetic_roundtrip`; `F32x1` is also the representative type for every new `Tween` operation's test: `tween_progress_reports_fraction_of_duration_elapsed`, `tween_builder_methods_configure_duration_and_delay`, `tween_time_accumulates_elapsed_delta_time`, `tween_pause_halts_further_progress_until_resumed`, `tween_reset_returns_to_start_value`, `tween_current_repeat_increments_after_each_repeat_cycle`, `tween_with_yoyo_reverses_direction_on_alternate_repeats`, `tween_state_reports_animation_lifecycle_stage`, `tween_with_easing_selector_accepts_named_curve`, `tween_with_easing_selector_rejects_unknown_curve_name`, `tween_with_cubic_hermite_tangents_deviates_from_linear_interpolation` |
