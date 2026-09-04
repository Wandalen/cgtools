# Pitfall: f32 Boundary Cast Silently Truncates Precision

### Scope

- **Purpose**: Warn that constructing any `f32`-element vector type (`F32x1`, `F32x2`, `F32x3`, `F32x4`) from a script silently narrows whatever `f64` literal the script provided.
- **Responsibility**: Document the concrete narrowing mechanism and the mitigation (pick the right type up front).
- **In Scope**: Every `f32x{1,2,3,4}_register`'s constructor cast (`x as f32`, and `y`/`z`/`w as f32` where the arity has them).
- **Out of Scope**: The type-distinctness invariant itself ([`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)) — this pitfall's failure mode does not violate that invariant; every type stays distinct, precision is merely lost within each `f32`-element type's own construction.

### Trap

Rhai's only numeric type is `f64` (`FLOAT`) — a script never has an `f32` literal to begin with; every numeric literal a script writes is `f64` from the moment it's parsed. Each `f32x{1,2,3,4}(...)` registration accepts every component as `f64` and casts each `as f32` internally (`vector_binding.rs`). A script author who writes a value with more significant digits than `f32` can represent sees it silently narrowed the instant it enters any of these four types — there is no warning, error, or any script-visible signal that narrowing happened; the constructor call simply succeeds with a slightly different value than the literal written.

### Failure

Any `f32x{1,2,3,4}(...)` call where a component carries precision beyond what `f32` can represent rounds silently to the nearest representable `f32` value. The identical literal passed to the matching `f64x{1,2,3,4}(...)` constructor instead retains full `f64` precision. A script that builds a value via an `f32x*` constructor and later compares it against an independently-computed `f64`-precision expectation (or against a value built via the matching `f64x*` constructor) can observe an unexpected mismatch purely from this narrowing — with nothing in the failure pointing back at the type choice as the cause. The failure mode is identical across all four arities; only the number of narrowed components differs.

### Mitigation

Use the matching `f64x{1,2,3,4}(...)` constructor whenever a script's numeric precision matters for correctness; treat every `f32x*(...)` constructor as the deliberately-lossy choice — appropriate only when the receiving Rust-side `f32`-element consumer genuinely needs `f32` (e.g. a GPU-bound buffer layout), never reached for out of habit or because it was the first constructor tried.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_rhai_scene_scripting.md](../feature/001_rhai_scene_scripting.md) | Navigational hub this pitfall's warning serves |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | Every type stays distinct even though this pitfall's precision loss happens within each `f32`-element type individually |

### Types

| File | Relationship |
|------|--------------|
| [../type/001_f32x2_script_facing_vector_value.md](../type/001_f32x2_script_facing_vector_value.md) | One of the four values whose Rust-side element type (`f32`) is the root cause of this narrowing |
| [../type/003_f32x1_script_facing_vector_value.md](../type/003_f32x1_script_facing_vector_value.md) | One of the four values whose Rust-side element type (`f32`) is the root cause of this narrowing |
| [../type/005_f32x3_script_facing_vector_value.md](../type/005_f32x3_script_facing_vector_value.md) | One of the four values whose Rust-side element type (`f32`) is the root cause of this narrowing |
| [../type/007_f32x4_script_facing_vector_value.md](../type/007_f32x4_script_facing_vector_value.md) | One of the four values whose Rust-side element type (`f32`) is the root cause of this narrowing |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x1_register`, `f32x2_register`, `f32x3_register`, `f32x4_register` — each one's `as f32` cast on every component |

### Tests

No dedicated regression test pins this narrowing for any arity — it is an inherent, intentional consequence of each `f32`-element type's Rust-side element type, not a claim this crate could regress on. `f32x2_arithmetic_roundtrip`, `f32x1_arithmetic_roundtrip`, `f32x3_arithmetic_roundtrip`, and `f32x4_arithmetic_roundtrip` (`tests/engine_test.rs`) all use only values exactly representable in `f32`, so none of them exercise the narrowing case.
