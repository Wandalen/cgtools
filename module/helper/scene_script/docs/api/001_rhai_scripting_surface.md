# API: Rhai Scripting Surface

### Scope

- **Purpose**: Enumerate everything a script can call once `engine_build()` has registered `scene_script`'s bindings.
- **Responsibility**: Document available operations with conceptual signatures, error conditions, and compatibility guarantees.
- **In Scope**: Constructors, methods, operators, and property getters registered by `vector_binding.rs` and `tween_binding.rs`.
- **Out of Scope**: The script-visible shape/fields of the involved types (see [`data_structure/001`](../data_structure/001_f32x2_f64x2_script_facing_vector_types.md), [`data_structure/002`](../data_structure/002_tween_script_facing_type.md)); the Rust-level registration functions' own rustdoc (see crate [`readme.md`](../../readme.md)).

### Abstract

`scene_script` curates a deliberately small vocabulary for scripts: two 2-component vector types at different float precisions, and a tween type that interpolates between two vector values over time. Nothing outside this registered surface is reachable from a script — there is no reflection, no dynamic access to arbitrary host types, and no way to call a Rust function that was not explicitly registered via `Engine::register_fn`/`register_get`. This is the complete, exhaustive list of what a script can do with the engine `engine_build()` returns.

### Operations

| Operation | Conceptual Signature | Behavior |
|-----------|----------------------|----------|
| `f32x2` | `(x, y) -> F32x2` | Constructs an `F32x2`. Both arguments arrive as Rhai's `f64` and are cast to `f32` at the boundary (see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md)). |
| `f64x2` | `(x, y) -> F64x2` | Constructs an `F64x2`. No precision cast — Rhai's `f64` matches `F64x2`'s element type exactly. |
| `.x`, `.y` (on `F32x2`/`F64x2`) | `(vector) -> float` | Read-only property getters. No corresponding setters are registered — a script cannot mutate a vector's components in place. |
| `+`, `-` | `(vector, vector) -> vector` | Component-wise add/subtract, for either vector type against its own type only (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)). |
| `*` | `(vector, float) -> vector` or `(float, vector) -> vector` | Scalar multiply, registered both operand orders. |
| `to_string` | `(vector) -> string` | Renders as `"F32x2(x, y)"` / `"F64x2(x, y)"`. |
| `tween` | `(start: vector, end: vector, duration: float) -> Tween` | Constructs a `Tween`, always Linear-eased (see [`pitfall/006`](../pitfall/006_only_linear_easing_is_exposed_to_scripts.md)). Overloaded on the vector argument type: passing `F32x2` values yields a `Tween` over `F32x2`, `F64x2` values yield one over `F64x2` — a script never names the precision separately from the vectors it passes. |
| `.update` (on `Tween`) | `(tween, delta_time: float) -> vector` | Advances the tween's internal elapsed time by `delta_time` and returns the resulting interpolated value. Mutates the tween. |
| `.value` (on `Tween`) | `(tween) -> vector` | Reads the current interpolated value without advancing time. |
| `.is_completed` (on `Tween`) | `(tween) -> bool` | Reports whether the tween has reached its end value. |

### Error Handling

Every operation above is a strongly-typed Rhai function registration — calling one with the wrong argument types (e.g. passing an `F32x2` where an `F64x2` is expected, or vice versa) fails at the call site with a Rhai type/function-resolution error rather than an implicit conversion (confirmed by `f32x2_and_f64x2_are_distinct_types_not_interchangeable`, `tests/engine_test.rs`). Calling an unregistered name (anything not in the Operations table) fails with Rhai's own "Function not found" error. No operation registered here can itself raise a custom `scene_script` error type — every failure a script observes originates from Rhai's own dispatch machinery, not from application-level error values.

### Compatibility Guarantees

Every constructor and type name mirrors its Rust identifier exactly (see [`invariant/003`](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md)) — a naming convention enforced by manual review only, not tooling. The crate is pre-1.0 (`0.1.0` in `Cargo.toml`); no formal deprecation policy or versioning strategy for this scripting surface exists yet, and none is asserted here beyond the crate's own Cargo semver.

### Data Structures

| File | Relationship |
|------|--------------|
| [001_f32x2_f64x2_script_facing_vector_types.md](../data_structure/001_f32x2_f64x2_script_facing_vector_types.md) | The shape `f32x2`/`f64x2` construct and `+`/`-`/`*` operate on |
| [002_tween_script_facing_type.md](../data_structure/002_tween_script_facing_type.md) | The shape `tween` constructs and `.update`/`.value`/`.is_completed` operate on |

### Patterns

| File | Relationship |
|------|--------------|
| [002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | Why `f32x2`/`f64x2` and their operators exist as separate, non-interchangeable registrations |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x2_register`, `f64x2_register` — every vector operation in the table above |
| `src/tween_binding.rs` | `tween_f32x2_register`, `tween_f64x2_register` — every tween operation in the table above |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | Exercises every operation: arithmetic roundtrips, tween update-toward-end-value, cross-type rejection |
