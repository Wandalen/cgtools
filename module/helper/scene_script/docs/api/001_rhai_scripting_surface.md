# API: Rhai Scripting Surface

### Scope

- **Purpose**: Enumerate everything a script can call once `engine_build()` has registered `scene_script`'s bindings.
- **Responsibility**: Document available operations with conceptual signatures, error conditions, and compatibility guarantees.
- **In Scope**: Constructors, methods, operators, and property getters registered by `vector_binding.rs` and `tween_binding.rs`.
- **Out of Scope**: The script-visible shape/fields of the involved types (see the [`type/`](../type/readme.md) collection for all 8 registered vector types, and [`data_structure/001`](../data_structure/001_tween_script_facing_type.md) for `Tween`); the Rust-level registration functions' own rustdoc (see crate [`readme.md`](../../readme.md)).

### Abstract

`scene_script` curates a deliberately small vocabulary for scripts: the full `{F32,F64}x{1,2,3,4}` vector family — two float precisions across four arities, eight types in total — and a tween type that interpolates between two same-type vector values over time. Nothing outside this registered surface is reachable from a script — there is no reflection, no dynamic access to arbitrary host types, and no way to call a Rust function that was not explicitly registered via `Engine::register_fn`/`register_get`. This is the complete, exhaustive list of what a script can do with the engine `engine_build()` returns.

### Operations

The table groups rows by element type across arities rather than enumerating all 8 constructors/getters individually — differing only in argument/getter count, every arity within a precision shares identical behavior, error handling, and boundary-cast treatment.

| Operation | Conceptual Signature | Behavior |
|-----------|----------------------|----------|
| `f32x1`, `f32x2`, `f32x3`, `f32x4` | `(x[, y[, z[, w]]]) -> F32x{1,2,3,4}` | Constructs the matching-arity `f32`-element vector. Every argument arrives as Rhai's `f64` and is cast to `f32` at the boundary (see [`pitfall/004`](../pitfall/004_f32_boundary_cast_truncates_precision.md)). |
| `f64x1`, `f64x2`, `f64x3`, `f64x4` | `(x[, y[, z[, w]]]) -> F64x{1,2,3,4}` | Constructs the matching-arity `f64`-element vector. No precision cast — Rhai's `f64` matches the element type exactly. |
| `f32x4`, `f64x4` (2-arg overload) | `(xy: F32x2/F64x2, zw: F32x2/F64x2) -> F32x4/F64x4` | Concatenates two same-precision 2-component vectors' components — `(xy.x, xy.y, zw.x, zw.y)`, not a geometric combination. Coexists with the arity-4 scalar constructor above; Rhai dispatches by argument type/count. |
| `.x` (all arities); `.y` (arity ≥2); `.z` (arity ≥3); `.w` (arity 4 only) | `(vector) -> float` | Read-only property getters, one per component the arity actually has. No corresponding setters are registered — a script cannot mutate a vector's components in place. |
| `+`, `-` (binary) | `(vector, vector) -> vector` | Component-wise add/subtract, for any vector type against its own type only — never across precision or arity (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)). |
| `-` (unary) | `(vector) -> vector` | Component-wise negation. Registered under the same `"-"` name as binary subtract; Rhai dispatches by arity (1 argument vs 2). |
| `*` | `(vector, float) -> vector` or `(float, vector) -> vector` | Scalar multiply, registered both operand orders. |
| `dot` | `(vector, vector) -> float` | Dot product, same type only. |
| `mag`, `mag2` | `(vector) -> float` | Euclidean magnitude and squared magnitude (`mag2` avoids the square root). |
| `normalize` | `(vector) -> vector` | Returns a new unit-length copy. Despite the name, this does not mutate the receiver — every vector operation is non-mutating, same as `+`/`-`/`*` above. |
| `distance` | `(vector, vector) -> float` | Euclidean distance between two same-type vectors — equivalent to `(b - a).mag()`. |
| `distance_squared` | `(vector, vector) -> float` | Squared Euclidean distance between two same-type vectors — equivalent to `(b - a).mag2()`; skips the square root `distance` pays for, cheaper when only relative comparisons matter. |
| `min`, `max` | `(vector, vector) -> vector` | Component-wise minimum/maximum. |
| `cross` | `(F32x3/F64x3, F32x3/F64x3) -> F32x3/F64x3` | 3-component cross product, right-handed. Registered only on the two arity-3 types — no arity-2 or arity-4 analog exists. |
| `to_homogenous` | `(F32x3/F64x3) -> F32x4/F64x4` | Appends `w = 1.0`, producing a homogeneous coordinate for matrix/transform math. Registered only on the two arity-3 types. |
| `truncate` | `(F32x4/F64x4) -> F32x3/F64x3` | Drops the `w` component, same precision. Registered only on the two arity-4 types. |
| `to_string` | `(vector) -> string` | Renders as `"F32x1(x)"`, `"F32x2(x, y)"`, `"F32x3(x, y, z)"`, `"F32x4(x, y, z, w)"`, and the `F64x*` equivalents. |
| `tween` (3-arg) | `(start: vector, end: vector, duration: float) -> Tween` | Constructs a `Tween` with Linear easing. Overloaded on the vector argument type across all 8 registered types: passing two same-type vectors yields a `Tween` over that type — a script never names the precision or arity separately from the vectors it passes, and `start`/`end` must be the same type (never, e.g., one `F32x2` and one `F32x3`). |
| `tween` (4-arg) | `(start: vector, end: vector, duration: float, easing: string) -> Tween`, fallible | Same as the 3-arg form, but names an easing curve instead of defaulting to Linear — `"Linear"` or one of 24 CSS-style presets (`"EaseInSine"` .. `"EaseInOutBack"`; see [`pitfall/006`](../pitfall/006_parameterized_easing_curves_are_unreachable_by_name.md) for exactly which curves, and which remain unreachable). An unrecognized name is a script-catchable runtime error whose message contains `"unknown easing curve name"` — never a silent fallback to Linear. |
| `tween` (5-arg) | `(start: vector, end: vector, duration: float, m1: vector, m2: vector) -> Tween` | Builds a `Tween` eased along a `CubicHermite` curve constructed directly from two tangent vectors (`m1`, `m2`, same type as `start`/`end`) — resolved by arity like every other `tween` overload, this bypasses the named-easing selector entirely, since `CubicHermite` cannot be named through it (see [`pitfall/006`](../pitfall/006_parameterized_easing_curves_are_unreachable_by_name.md)). |
| `.update` (on `Tween`) | `(tween, delta_time: float) -> vector` | Advances the tween's internal elapsed time by `delta_time` and returns the resulting interpolated value. Mutates the tween. |
| `.value` (on `Tween`) | `(tween) -> vector` | Reads the current interpolated value without advancing time. |
| `.is_completed` (on `Tween`) | `(tween) -> bool` | Reports whether the tween has reached its end value. |
| `.progress` (on `Tween`) | `(tween) -> float` | Fraction of duration elapsed, clamped to `0.0..=1.0`; reads `0.0` while still `Pending` (delay not yet elapsed). |
| `.duration`, `.delay` (on `Tween`) | `(tween) -> float` | Read back the tween's configured duration/delay. |
| `.time` (on `Tween`) | `(tween) -> float` | Total elapsed time since construction (or since the last `.reset()`), including any repeat cycles. |
| `.current_repeat` (on `Tween`) | `(tween) -> int` | Number of repeat cycles completed so far. |
| `.state` (on `Tween`) | `(tween) -> string` | The tween's lifecycle stage, `Debug`-formatted from the underlying Rust enum: `"Pending"`, `"Running"`, `"Paused"`, or `"Completed"`. |
| `.pause`, `.resume` (on `Tween`) | `(tween) -> ()` | Halt/resume progress — a paused tween ignores further `.update()` calls until resumed. Mutates the tween. |
| `.reset` (on `Tween`) | `(tween) -> ()` | Returns the tween to its start value and zeroes elapsed time and repeat count. Mutates the tween. |
| `.with_delay`, `.with_duration`, `.with_repeat`, `.with_yoyo` (on `Tween`) | `(tween, value) -> Tween` | Consuming builder methods — each takes the tween by value and returns a modified copy for chaining (`t.with_delay(0.5).with_duration(2.0)`) rather than mutating in place. `.with_repeat` takes an `int` repeat count; `.with_yoyo` takes a `bool`. |

### Error Handling

Every operation above is a strongly-typed Rhai function registration — calling one with the wrong argument types (e.g. passing an `F32x2` where an `F64x2` is expected, or an `F32x2` where an `F32x3` is expected) fails at the call site with a Rhai type/function-resolution error rather than an implicit conversion (confirmed for the original `F32x2`/`F64x2` pair by `f32x2_and_f64x2_are_distinct_types_not_interchangeable`, `tests/engine_test.rs`; the mechanism is identical for every other pair per [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)'s Enforcement Mechanism). Calling an unregistered name (anything not in the Operations table) fails with Rhai's own "Function not found" error. No operation registered here can itself raise a custom `scene_script` error type — every failure a script observes originates from Rhai's own dispatch machinery, not from application-level error values.

### Compatibility Guarantees

Every constructor and type name mirrors its Rust identifier exactly (see [`invariant/003`](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md)) — a naming convention enforced by manual review only, not tooling. The crate is pre-1.0 (`0.1.0` in `Cargo.toml`); no formal deprecation policy or versioning strategy for this scripting surface exists yet, and none is asserted here beyond the crate's own Cargo semver.

### Types

| File | Relationship |
|------|--------------|
| [001_f32x2_script_facing_vector_value.md](../type/001_f32x2_script_facing_vector_value.md) | The value `f32x2` constructs and `+`/`-`/`*` operate on |
| [002_f64x2_script_facing_vector_value.md](../type/002_f64x2_script_facing_vector_value.md) | The value `f64x2` constructs and `+`/`-`/`*` operate on |
| [003_f32x1_script_facing_vector_value.md](../type/003_f32x1_script_facing_vector_value.md) | The value `f32x1` constructs and `+`/`-`/`*` operate on |
| [004_f64x1_script_facing_vector_value.md](../type/004_f64x1_script_facing_vector_value.md) | The value `f64x1` constructs and `+`/`-`/`*` operate on |
| [005_f32x3_script_facing_vector_value.md](../type/005_f32x3_script_facing_vector_value.md) | The value `f32x3` constructs and `+`/`-`/`*` operate on |
| [006_f64x3_script_facing_vector_value.md](../type/006_f64x3_script_facing_vector_value.md) | The value `f64x3` constructs and `+`/`-`/`*` operate on |
| [007_f32x4_script_facing_vector_value.md](../type/007_f32x4_script_facing_vector_value.md) | The value `f32x4` constructs and `+`/`-`/`*` operate on |
| [008_f64x4_script_facing_vector_value.md](../type/008_f64x4_script_facing_vector_value.md) | The value `f64x4` constructs and `+`/`-`/`*` operate on |

### Data Structures

| File | Relationship |
|------|--------------|
| [001_tween_script_facing_type.md](../data_structure/001_tween_script_facing_type.md) | The shape `tween` constructs and `.update`/`.value`/`.is_completed` operate on |

### Patterns

| File | Relationship |
|------|--------------|
| [002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | Why `f32x2`/`f64x2` and their operators exist as separate, non-interchangeable registrations |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x1_register`, `f32x2_register`, `f32x3_register`, `f32x4_register`, `f64x1_register`, `f64x2_register`, `f64x3_register`, `f64x4_register` — every vector operation in the table above |
| `src/tween_binding.rs` | `tween_f32x1_register`, `tween_f32x2_register`, `tween_f32x3_register`, `tween_f32x4_register`, `tween_f64x1_register`, `tween_f64x2_register`, `tween_f64x3_register`, `tween_f64x4_register` — every tween operation in the table above |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | Exercises every operation: arithmetic roundtrips, tween update-toward-end-value, cross-type rejection |
