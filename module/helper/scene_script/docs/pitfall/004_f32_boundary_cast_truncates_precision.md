# Pitfall: f32 Boundary Cast Silently Truncates Precision

### Scope

- **Purpose**: Warn that constructing an `F32x2` from a script silently narrows whatever `f64` literal the script provided.
- **Responsibility**: Document the concrete narrowing mechanism and the mitigation (pick the right type up front).
- **In Scope**: `f32x2_register`'s constructor cast (`x as f32`, `y as f32`).
- **Out of Scope**: The `F32x2`/`F64x2` distinctness invariant itself ([`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)) — this pitfall's failure mode does not violate that invariant; the two types stay distinct, precision is merely lost within `F32x2`'s own construction.

### Trap

Rhai's only numeric type is `f64` (`FLOAT`) — a script never has an `f32` literal to begin with; every numeric literal a script writes is `f64` from the moment it's parsed. `f32x2(x, y)`'s registration accepts `x: f64, y: f64` and casts each `as f32` internally (`vector_binding.rs`). A script author who writes a value with more significant digits than `f32` can represent sees it silently narrowed the instant it enters an `F32x2` — there is no warning, error, or any script-visible signal that narrowing happened; the constructor call simply succeeds with a slightly different value than the literal written.

### Failure

Any `f32x2(x, y)` call where `x` or `y` carries precision beyond what `f32` can represent rounds silently to the nearest representable `f32` value. The identical literal passed to `f64x2(x, y)` instead retains full `f64` precision. A script that builds a value via `f32x2(...)` and later compares it against an independently-computed `f64`-precision expectation (or against a value built via `f64x2(...)`) can observe an unexpected mismatch purely from this narrowing — with nothing in the failure pointing back at the type choice as the cause.

### Mitigation

Use `f64x2(...)` whenever a script's numeric precision matters for correctness; treat `f32x2(...)` as the deliberately-lossy choice — appropriate only when the receiving Rust-side `F32x2` consumer genuinely needs `f32` (e.g. a GPU-bound buffer layout), never reached for out of habit or because it was the first constructor tried.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | The two types stay distinct even though this pitfall's precision loss happens within one of them |

### Types

| File | Relationship |
|------|--------------|
| [../type/001_f32x2_script_facing_vector_value.md](../type/001_f32x2_script_facing_vector_value.md) | The value whose Rust-side element type (`f32`) is the root cause of this narrowing |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x2_register`'s `x as f32, y as f32` cast |

### Tests

No dedicated regression test pins this narrowing — it is an inherent, intentional consequence of `F32x2`'s Rust-side element type, not a claim this crate could regress on. `f32x2_arithmetic_roundtrip` (`tests/engine_test.rs`) uses only values exactly representable in `f32`, so it does not exercise the narrowing case.
