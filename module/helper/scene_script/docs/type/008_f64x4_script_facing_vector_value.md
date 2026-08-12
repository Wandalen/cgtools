# Type: F64x4 (Script-Facing Vector Value)

- **domain**: vector
- **ddd**: value_object

### Scope

- **Purpose**: Define `F64x4` as a Rhai-registered Domain Type — a double-precision 4D vector value a script constructs, reads, and combines — distinct from `F32x4` ([`type/007`](007_f32x4_script_facing_vector_value.md)) despite sharing the same shape.
- **Responsibility**: State the type's domain meaning, its construction/validation rules, and its relationships to its single-precision counterpart and to `Tween`.
- **In Scope**: The Rhai-registered projection named `"F64x4"` — the script-visible type, not the Rust struct behind it.
- **Out of Scope**: `ndarray_cg::F64x4`'s own Rust definition, which `scene_script` does not own (see [`pattern/001`](../pattern/001_manual_customtype_registration_for_foreign_types.md) for why registration is manual rather than a trait derive); full call signatures and error behavior (see [`api/001`](../api/001_rhai_scripting_surface.md)); `Vector<f64, 4>`'s additional Rust-side methods (`truncate`, `From<(Vec2, Vec2)>`) — none of these are registered into Rhai.

### Definition

`F64x4` is a 4-component vector value, each component held at double (`f64`) precision. A script constructs one via `f64x4(x, y, z, w)` and reads its components via `.x`/`.y`/`.z`/`.w` — read-only; no operation registered anywhere mutates an existing `F64x4` in place. Every operation that produces an `F64x4` (`f64x4(...)`, `+`, `-`, `*`) produces a *new* value; there is no way to change one in place. Identity is purely structural, same as [`type/002`](002_f64x2_script_facing_vector_value.md)'s `F64x2`.

### Validation

No construction is ever rejected. `f64x4(x, y, z, w)` accepts any four values Rhai can supply as `FLOAT` (Rhai's own `f64`) — including `NaN` and infinities — and always succeeds; there is no rejection rule of any kind. Unlike `F32x4` ([`type/007`](007_f32x4_script_facing_vector_value.md)), **no precision narrowing occurs at all**: Rhai's `FLOAT` already is `f64`, so `x`/`y`/`z`/`w` pass through `src/vector_binding.rs`'s `f64x4_register` unchanged.

### Relationships

- [`type/007`](007_f32x4_script_facing_vector_value.md) — `F32x4` is the single-precision counterpart, sharing the same shape and operation set. The two never implicitly convert (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)); a script commits to one precision at construction.
- `Tween<F64x4>` — `tween(start, end, duration)` accepts two `F64x4` values as its `start`/`end` arguments, producing a `Tween` that interpolates between them (see [`data_structure/001`](../data_structure/001_tween_script_facing_type.md)).
- [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the registration technique that keeps `F32x4` and `F64x4` distinct, non-interchangeable, side-by-side names.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | `F64x4` never implicitly converts to/from `F32x4` |
| [../invariant/003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | Why the registered name is exactly `"F64x4"` / `"f64x4"` |

### APIs

| File | Relationship |
|------|--------------|
| [../api/001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | Full operational contract (signatures, error handling) for every operation this type participates in |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f64x4_register` — constructor, `.x`/`.y`/`.z`/`.w` getters, `+`/`-`/`*` operators, `to_string` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f64x4_arithmetic_roundtrip`, `tween_f64x4_updates_toward_end_value` |
