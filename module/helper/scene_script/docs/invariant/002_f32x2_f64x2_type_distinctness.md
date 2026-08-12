# Invariant: F32x2/F64x2 Type Distinctness

A script value bound to Rhai type `"F32x2"` never implicitly converts to or from `"F64x2"` — the two are always distinct, non-interchangeable types.

### Scope

- **Purpose**: Pin that the two vector precisions never silently mix across the script boundary.
- **Responsibility**: State the property, its (host-provided, not crate-added) enforcement mechanism, and what a violation would undermine.
- **In Scope**: Cross-type calls or evaluations between `F32x2`-typed and `F64x2`-typed script values.
- **Out of Scope**: The boundary cast within a single precision's own construction (see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md)).

### Invariant Statement

For any script value bound to Rhai type `"F32x2"`, passing it where an `"F64x2"`-typed function parameter is expected, or evaluating a script producing it as `F64x2` via `Engine::eval::<F64x2>`, always fails with a type-mismatch error — never a silent promotion, narrowing, or reinterpretation. The same holds symmetrically for `"F64x2"` values used where `"F32x2"` is expected.

### Enforcement Mechanism

This is not something `scene_script` implements directly — it is Rhai's own dynamic dispatch, which matches registered function signatures and requested evaluation types by exact registered type identity. `scene_script` relies on this rather than adding any conversion function between `F32x2` and `F64x2`: no such function is registered anywhere in `vector_binding.rs`. Confirmed directly: `f32x2_and_f64x2_are_distinct_types_not_interchangeable` (`tests/engine_test.rs`) asserts that `engine.eval::<F64x2>("f32x2(1.0, 2.0)")` returns an error whose message contains `"type"`.

### Violation Consequences

If this ever stopped holding — for example, through a future accidental cross-registration of an arithmetic operator taking one type and returning the other — a script mixing precisions could silently receive a wrongly-precision-truncated or -widened result instead of a clear error at the call site. That would undermine the Naming Convention's implicit promise (crate [`readme.md`](../../readme.md)) that choosing a type name is the same act as choosing a precision — the two would no longer be reliably linked.

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | The registration shape that produces this distinctness as a side effect of registering each precision separately |

### Data Structures

| File | Relationship |
|------|--------------|
| [../data_structure/001_f32x2_f64x2_script_facing_vector_types.md](../data_structure/001_f32x2_f64x2_script_facing_vector_types.md) | The two shapes this invariant keeps separate |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x2_register`, `f64x2_register` — no cross-type conversion is ever registered |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x2_and_f64x2_are_distinct_types_not_interchangeable` |
