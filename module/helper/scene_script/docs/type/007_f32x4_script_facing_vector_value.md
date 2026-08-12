# Type: F32x4 (Script-Facing Vector Value)

- **domain**: vector
- **ddd**: value_object

### Scope

- **Purpose**: Define `F32x4` as a Rhai-registered Domain Type — a single-precision 4D vector value a script constructs, reads, and combines — distinct from `F64x4` ([`type/008`](008_f64x4_script_facing_vector_value.md)) despite sharing the same shape.
- **Responsibility**: State the type's domain meaning, its construction/validation rules, and its relationships to its double-precision counterpart and to `Tween`.
- **In Scope**: The Rhai-registered projection named `"F32x4"` — the script-visible type, not the Rust struct behind it.
- **Out of Scope**: `ndarray_cg::F32x4`'s own Rust definition, which `scene_script` does not own (see [`pattern/001`](../pattern/001_manual_customtype_registration_for_foreign_types.md) for why registration is manual rather than a trait derive); full call signatures and error behavior (see [`api/001`](../api/001_rhai_scripting_surface.md)).

### Definition

`F32x4` is a 4-component vector value, each component held at single (`f32`) precision. A script constructs one via `f32x4(x, y, z, w)`, or via a 2-arg overload `f32x4(xy, zw)` that concatenates two `F32x2` values' components, and reads its components via `.x`/`.y`/`.z`/`.w` — read-only; no operation registered anywhere mutates an existing `F32x4` in place. Every operation that produces an `F32x4` (either constructor overload, `+`, `-`, `*`, unary `-`, `normalize`, `min`, `max`) produces a *new* value; there is no way to change one in place. Identity is purely structural, same as [`type/001`](001_f32x2_script_facing_vector_value.md)'s `F32x2`.

The domain meaning is deliberately unconstrained, same as every other arity in this family: `scene_script` registers no interpretation beyond "a 4D single-precision vector."

### Validation

No construction is ever rejected. `f32x4(x, y, z, w)` accepts any four values Rhai can supply as `FLOAT` (Rhai's own `f64`) — including `NaN` and infinities — and always succeeds; there is no rejection rule of any kind. The only transformation applied is precision narrowing: `x`, `y`, `z`, and `w` are each cast `as f32` at construction (`src/vector_binding.rs`) — see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md) for the concrete failure mode this causes.

### Relationships

- [`type/008`](008_f64x4_script_facing_vector_value.md) — `F64x4` is the double-precision counterpart, sharing the same shape and operation set. The two never implicitly convert (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)); a script commits to one precision at construction.
- `Tween<F32x4>` — `tween(start, end, duration)` accepts two `F32x4` values as its `start`/`end` arguments, producing a `Tween` that interpolates between them (see [`data_structure/001`](../data_structure/001_tween_script_facing_type.md)).
- [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the registration technique that keeps `F32x4` and `F64x4` distinct, non-interchangeable, side-by-side names.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | `F32x4` never implicitly converts to/from `F64x4` |
| [../invariant/003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | Why the registered name is exactly `"F32x4"` / `"f32x4"` |

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
| `src/vector_binding.rs` | `f32x4_register` — constructor (plus 2-arg `f32x4(xy, zw)` overload), `.x`/`.y`/`.z`/`.w` getters, `+`/`-` (binary)/`*` operators, unary `-` negation, `dot`/`mag`/`mag2`/`normalize`/`distance`/`min`/`max`, `truncate` (arity-4 only), `to_string` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x4_arithmetic_roundtrip`, `tween_f32x4_updates_toward_end_value`, `vector_truncate_drops_w_component`, `vector_f32x4_from_two_f32x2_concatenates_components` — `F32x4` is the representative type for `truncate`'s and the 2-arg constructor overload's tests |
