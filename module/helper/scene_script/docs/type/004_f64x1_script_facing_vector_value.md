# Type: F64x1 (Script-Facing Vector Value)

- **domain**: vector
- **ddd**: value_object

### Scope

- **Purpose**: Define `F64x1` as a Rhai-registered Domain Type — a double-precision 1-component vector value a script constructs, reads, and combines — distinct from `F32x1` ([`type/003`](003_f32x1_script_facing_vector_value.md)) despite sharing the same shape.
- **Responsibility**: State the type's domain meaning, its construction/validation rules, and its relationships to its single-precision counterpart and to `Tween`.
- **In Scope**: The Rhai-registered projection named `"F64x1"` — the script-visible type, not the Rust struct behind it.
- **Out of Scope**: `ndarray_cg::F64x1`'s own Rust definition, which `scene_script` does not own (see [`pattern/001`](../pattern/001_manual_customtype_registration_for_foreign_types.md) for why registration is manual rather than a trait derive); full call signatures and error behavior (see [`api/001`](../api/001_rhai_scripting_surface.md)).

### Definition

`F64x1` is the minimal member of `ndarray_cg`'s `{Element}x{Arity}` family: a 1-component vector held at double (`f64`) precision. A script constructs one via `f64x1(x)` and reads its component via `.x` — read-only; no operation registered anywhere mutates an existing `F64x1` in place. Every operation that produces an `F64x1` (`f64x1(...)`, `+`, `-`, `*`) produces a *new* value; there is no way to change one in place. Identity is purely structural: two `F64x1` values built from the same `x` are indistinguishable to a script.

At arity 1, `F64x1` carries no arithmetic behavior a raw `f64` scalar wouldn't already have — same rationale as [`type/003`](003_f32x1_script_facing_vector_value.md)'s `F32x1`: uniformity with the rest of the vector family, not new capability.

### Validation

No construction is ever rejected. `f64x1(x)` accepts any value Rhai can supply as `FLOAT` (Rhai's own `f64`) — including `NaN` and infinities — and always succeeds; there is no rejection rule of any kind. Unlike `F32x1` ([`type/003`](003_f32x1_script_facing_vector_value.md)), **no precision narrowing occurs at all**: Rhai's `FLOAT` already is `f64`, so `x` passes through `src/vector_binding.rs`'s `f64x1_register` unchanged — there is no boundary cast and no equivalent of [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md) for this type.

### Relationships

- [`type/003`](003_f32x1_script_facing_vector_value.md) — `F32x1` is the single-precision counterpart, sharing the same shape and operation set. The two never implicitly convert (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)); a script commits to one precision at construction.
- `Tween<F64x1>` — `tween(start, end, duration)` accepts two `F64x1` values as its `start`/`end` arguments, producing a `Tween` that interpolates between them (see [`data_structure/001`](../data_structure/001_tween_script_facing_type.md)).
- [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the registration technique that keeps `F32x1` and `F64x1` distinct, non-interchangeable, side-by-side names.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | `F64x1` never implicitly converts to/from `F32x1` |
| [../invariant/003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | Why the registered name is exactly `"F64x1"` / `"f64x1"` |

### APIs

| File | Relationship |
|------|--------------|
| [../api/001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | Full operational contract (signatures, error handling) for every operation this type participates in |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f64x1_register` — constructor, `.x` getter, `+`/`-` (binary)/`*` operators, unary `-` negation, `dot`/`mag`/`mag2`/`normalize`/`distance`/`min`/`max` (all native `f64`, no boundary cast), `to_string` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f64x1_arithmetic_roundtrip` |
