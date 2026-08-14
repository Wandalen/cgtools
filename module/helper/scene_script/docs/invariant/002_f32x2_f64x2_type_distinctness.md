# Invariant: F32x2/F64x2 Type Distinctness

A script value bound to one registered vector type (any of `"F32x1"`, `"F32x2"`, `"F32x3"`, `"F32x4"`, `"F64x1"`, `"F64x2"`, `"F64x3"`, `"F64x4"`) never implicitly converts to or from another — every registered vector type is always distinct and non-interchangeable, whether the mismatch is in precision (`F32x2` vs `F64x2`), arity (`F32x2` vs `F32x3`), or both. The title and file name retain the original `F32x2`/`F64x2` pair the invariant was first documented against; the statement itself has always generalized to whatever set of vector types is currently registered.

### Scope

- **Purpose**: Pin that no two registered vector types — differing in precision, arity, or both — ever silently mix across the script boundary.
- **Responsibility**: State the property, its (host-provided, not crate-added) enforcement mechanism, and what a violation would undermine.
- **In Scope**: Cross-type calls or evaluations between any two of the eight registered vector types.
- **Out of Scope**: The boundary cast within a single precision's own construction (see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md)).

### Invariant Statement

For any script value bound to one registered vector type, passing it where a different registered vector type's function parameter is expected, or evaluating a script producing it as that other type via `Engine::eval::<T>`, always fails with a type-mismatch error — never a silent promotion, narrowing, widening, or reinterpretation. This holds for every pair among the eight registered types, symmetrically — not only the original `F32x2`/`F64x2` pair the invariant was first pinned against, and not only same-arity pairs: `F32x2` and `F32x3` are exactly as non-interchangeable as `F32x2` and `F64x2`.

### Enforcement Mechanism

This is not something `scene_script` implements directly — it is Rhai's own dynamic dispatch, which matches registered function signatures and requested evaluation types by exact registered type identity. `scene_script` relies on this rather than adding any conversion function between any two vector types: no such function is registered anywhere in `vector_binding.rs`, for any pair. Confirmed directly for the original pair: `f32x2_and_f64x2_are_distinct_types_not_interchangeable` (`tests/engine_test.rs`) asserts that `engine.eval::<F64x2>("f32x2(1.0, 2.0)")` returns an error whose message contains `"type"`. No dedicated regression test repeats this check for the other 7 types or for cross-arity pairs (e.g. `F32x2` vs `F32x3`) — the mechanism is Rhai's own dispatch, identical regardless of which two registered types are involved, so the single confirmed pair stands for the general case rather than each pair needing its own test.

### Violation Consequences

If this ever stopped holding — for example, through a future accidental cross-registration of an arithmetic operator taking one type and returning another — a script mixing types could silently receive a wrongly-truncated, -widened, or arity-mismatched result instead of a clear error at the call site. That would undermine the Naming Convention's implicit promise (crate [`readme.md`](../../readme.md)) that choosing a type name is the same act as choosing a precision and arity — the two would no longer be reliably linked.

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | The registration shape that produces this distinctness as a side effect of registering each precision/arity combination separately |

### Types

| File | Relationship |
|------|--------------|
| [../type/001_f32x2_script_facing_vector_value.md](../type/001_f32x2_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/002_f64x2_script_facing_vector_value.md](../type/002_f64x2_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/003_f32x1_script_facing_vector_value.md](../type/003_f32x1_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/004_f64x1_script_facing_vector_value.md](../type/004_f64x1_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/005_f32x3_script_facing_vector_value.md](../type/005_f32x3_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/006_f64x3_script_facing_vector_value.md](../type/006_f64x3_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/007_f32x4_script_facing_vector_value.md](../type/007_f32x4_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |
| [../type/008_f64x4_script_facing_vector_value.md](../type/008_f64x4_script_facing_vector_value.md) | One of the eight values this invariant keeps separate |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x1_register`, `f32x2_register`, `f32x3_register`, `f32x4_register`, `f64x1_register`, `f64x2_register`, `f64x3_register`, `f64x4_register` — no cross-type conversion is ever registered between any pair |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x2_and_f64x2_are_distinct_types_not_interchangeable` — the sole dedicated regression test; stands for the general mechanism per the Enforcement Mechanism section above |
