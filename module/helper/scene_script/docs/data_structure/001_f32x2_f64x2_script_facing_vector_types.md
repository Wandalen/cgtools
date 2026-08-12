# Data Structure: F32x2 / F64x2 Script-Facing Vector Types

### Scope

- **Purpose**: Document the shape a script actually sees when it holds an `F32x2` or `F64x2` value — a Rhai-side projection, not the originating Rust struct.
- **Responsibility**: Document script-visible fields, mutability, and the relationship between the two precision variants.
- **In Scope**: The `"F32x2"`/`"F64x2"` types exactly as `Engine::register_type_with_name` + `register_get` expose them.
- **Out of Scope**: `ndarray_cg::F32x2`/`F64x2`'s actual Rust struct definition and full API — owned by `ndarray_cg`, not re-documented here.

### Abstract

A script sees `F32x2` and `F64x2` as two structurally identical, opaque value types differing only in the element precision each was built from. Both are 2-component vectors exposing their `x`/`y` components as script-level `float` (Rhai's `f64`) regardless of the underlying Rust precision — `F32x2`'s components are `f32` internally and widen to `f64` when read; `F64x2`'s are `f64` internally and cross unchanged. Neither type carries any domain meaning beyond "two numbers" — no unit-length, no coordinate-system binding, no other constraint is enforced on construction or afterward.

### Structure

```
F32x2 { x: float (read-only), y: float (read-only) }   // Rust-side element type: f32; x/y widen to f64 crossing into script
F64x2 { x: float (read-only), y: float (read-only) }   // Rust-side element type: f64; x/y cross unchanged
```

Both fields are independent — there is no relationship or ordering constraint between `x` and `y` beyond both being present. No setters are registered for either field (`vector_binding.rs` calls `register_get` but never `register_get_set` or `register_set`): a script cannot mutate a vector's components in place once constructed. The only way to obtain a different value is to construct a new one, via the `f32x2`/`f64x2` constructors or the registered arithmetic operators.

### Operations

Full call signatures and error behavior live in [`api/001`](../api/001_rhai_scripting_surface.md); this section states only what shape each operation consumes/produces:

- **Construction**: `f32x2(x, y)` / `f64x2(x, y)` each produce a new value of their own type — never the other.
- **Arithmetic**: `+`/`-` combine two values of the *same* type into a new value of that type; `*` combines one vector and one scalar (`float`), either operand order, into a new value of the vector's type. There is no registered operation combining an `F32x2` with an `F64x2` directly (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)).
- **Display**: `to_string` renders either type as `"F32x2(x, y)"` / `"F64x2(x, y)"`.

### Invariants

| File | Relationship |
|------|--------------|
| [002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | States that these two shapes never implicitly convert into one another |

### Pitfalls

| File | Relationship |
|------|--------------|
| [004_f32_boundary_cast_truncates_precision.md](../pitfall/004_f32_boundary_cast_truncates_precision.md) | `F32x2`'s construction narrows a script's `f64` literal — a consequence of this shape's Rust-side element type |

### APIs

| File | Relationship |
|------|--------------|
| [001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | Full operational contract (signatures, error handling) for the operations summarized above |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x2_register`, `f64x2_register` — the registration that produces this shape |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x2_arithmetic_roundtrip`, `f64x2_arithmetic_roundtrip`, `f32x2_and_f64x2_are_distinct_types_not_interchangeable` |
