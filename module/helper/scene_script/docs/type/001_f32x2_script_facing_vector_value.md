# Type: F32x2 (Script-Facing Vector Value)

- **domain**: vector
- **ddd**: value_object

### Scope

- **Purpose**: Define `F32x2` as a Rhai-registered Domain Type — a single-precision 2D vector value a script constructs, reads, and combines — distinct from `F64x2` ([`type/002`](002_f64x2_script_facing_vector_value.md)) despite sharing the same shape.
- **Responsibility**: State the type's domain meaning, its construction/validation rules, and its relationships to its double-precision counterpart and to `Tween`.
- **In Scope**: The Rhai-registered projection named `"F32x2"` — the script-visible type, not the Rust struct behind it.
- **Out of Scope**: `ndarray_cg::F32x2`'s own Rust definition, which `scene_script` does not own (see [`pattern/001`](../pattern/001_manual_customtype_registration_for_foreign_types.md) for why registration is manual rather than a trait derive); full call signatures and error behavior (see [`api/001`](../api/001_rhai_scripting_surface.md)).

### Definition

`F32x2` is a 2-component vector value, each component held at single (`f32`) precision. A script constructs one via `f32x2(x, y)` and reads its components via `.x`/`.y` — read-only; no operation registered anywhere mutates an existing `F32x2` in place. Every operation that produces an `F32x2` (`f32x2(...)`, `+`, `-`, `*`) produces a *new* value; there is no way to change one in place. Identity is purely structural: two `F32x2` values built from the same `(x, y)` are indistinguishable to a script, with no separate identity beyond their own components — this is what makes it a value object rather than a DDD entity, which would carry identity and lifecycle independent of its current attribute values.

The domain meaning is deliberately unconstrained: `scene_script` registers no interpretation beyond "a 2D single-precision vector." Whether a given `F32x2` stands for a position, a velocity, or an offset is a convention the calling script and host establish between themselves — this type carries no tag or field recording which.

### Validation

No construction is ever rejected. `f32x2(x, y)` accepts any two values Rhai can supply as `FLOAT` (Rhai's own `f64`) — including `NaN` and infinities — and always succeeds; there is no rejection rule of any kind. The only transformation applied is precision narrowing: both `x` and `y` are cast `as f32` at construction (`src/vector_binding.rs`), which can silently lose precision for an input that doesn't round-trip exactly through `f32` — see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md) for the concrete failure mode this causes. The type accepts its entire input domain and narrows silently rather than validating and rejecting.

### Relationships

- [`type/002`](002_f64x2_script_facing_vector_value.md) — `F64x2` is the double-precision counterpart, sharing the same shape and operation set. The two never implicitly convert (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)); a script commits to one precision at construction.
- `Tween<F32x2>` — `tween(start, end, duration)` accepts two `F32x2` values (never one `F32x2` and one `F64x2`) as its `start`/`end` arguments, producing a `Tween` that interpolates between them (see [`data_structure/001`](../data_structure/001_tween_script_facing_type.md)).
- [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the registration technique that keeps `F32x2` and `F64x2` distinct, non-interchangeable, side-by-side names.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | `F32x2` never implicitly converts to/from `F64x2` |
| [../invariant/003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | Why the registered name is exactly `"F32x2"` / `"f32x2"` |

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
| `src/vector_binding.rs` | `f32x2_register` — constructor, `.x`/`.y` getters, `+`/`-`/`*` operators, `to_string` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x2_arithmetic_roundtrip`, `f32x2_and_f64x2_are_distinct_types_not_interchangeable` |
