# Pattern: Dual-Precision Side-by-Side Registration

### Scope

- **Purpose**: Name the extension seam for adding a new precision or arity to the script-facing vector/tween surface.
- **Responsibility**: Define the pattern's problem, solution, applicability, and trade-offs.
- **In Scope**: The `F32x2`/`F64x2` side-by-side registration shape and how it generalizes.
- **Out of Scope**: The orphan-rule registration mechanics themselves (see [`pattern/001`](001_manual_customtype_registration_for_foreign_types.md)).

### Problem

Rhai's own numeric type (`FLOAT`) is `f64`-only, but host math types come in multiple precisions (`f32`, `f64`) and, per `ndarray_cg`'s own `{Element}x{Arity}` family, multiple arities. Picking a single precision or arity to expose to scripts would silently strand any host code built on the others — a script author needing `f32`-precision data (to match a downstream GPU buffer layout, for instance) would have no way to get it if only `F64x2` were registered, or vice versa; a script needing a 3- or 4-component vector would have no way to get one if only arity 2 were registered.

### Solution

Register every needed precision/arity combination under its own distinct Rhai type name and constructor name, always mirroring the Rust identifier exactly rather than a generic alias (`"F32x2"`/`f32x2`, `"F64x2"`/`f64x2`, and the same shape for every other arity — never a shared `"Vec2"`/`"Vec"`; see [`invariant/003`](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md)). Rhai resolves overloaded operators (`+`, `-`, `*`) and the shared `"tween"` constructor by each call's actual argument types, so all 8 variants coexist without ambiguity and without any script-facing precision- or arity-selection flag — a script simply calls `f32x2(..)`, `f64x3(..)`, or any of the other six constructors, and gets the matching precision and arity throughout everything built from it.

### Applicability

Applies whenever the script-facing surface grows: a new arity (originally anticipated for `F32x3`/`F64x3` here, and since carried out for all of `F32x1`/`F32x3`/`F32x4`/`F64x1`/`F64x3`/`F64x4` — the full `{F32,F64}x{1,2,3,4}` family from `ndarray_cg`'s own naming convention, per the crate [`readme.md`](../../readme.md)'s Naming Convention section, is now registered), or a new `Animatable` element type for `Tween`. Each addition is one more `_register()` function following the existing pair's exact shape, wired into `engine_build()` ([`src/engine.rs`](../../src/engine.rs)) alongside the other 15. Not applicable when only one precision or arity will ever be needed — the side-by-side duplication has no benefit without a second variant to coexist with.

### Consequences

- **Registration boilerplate scales linearly** with the number of precision × arity combinations exposed — there is no generic-over-precision-or-arity registration mechanism; each combination is written out by hand (see [`pattern/001`](001_manual_customtype_registration_for_foreign_types.md)). At the current 2 precisions × 4 arities, this is 8 vector registrations plus 8 `Tween` registrations.
- **Script authors must know their needed precision and arity up front**: there is no automatic promotion, narrowing, or arity conversion between any two registered types (see [`invariant/002`](../invariant/002_f32x2_f64x2_type_distinctness.md)) — choosing the wrong one is a construction-time decision, not something fixable later by a conversion call, because none is registered.
- **Keeps every registered name traceable 1:1 to a concrete Rust type**, which is what makes the Naming Convention enforceable by inspection even though nothing automated checks it (see [`invariant/003`](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md)).

### Invariants

| File | Relationship |
|------|--------------|
| [002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | The non-interchangeability this pattern's side-by-side registrations produce |
| [003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | The naming rule every new registration following this pattern must keep |

### Types

| File | Relationship |
|------|--------------|
| [001_f32x2_script_facing_vector_value.md](../type/001_f32x2_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [002_f64x2_script_facing_vector_value.md](../type/002_f64x2_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [003_f32x1_script_facing_vector_value.md](../type/003_f32x1_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [004_f64x1_script_facing_vector_value.md](../type/004_f64x1_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [005_f32x3_script_facing_vector_value.md](../type/005_f32x3_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [006_f64x3_script_facing_vector_value.md](../type/006_f64x3_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [007_f32x4_script_facing_vector_value.md](../type/007_f32x4_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |
| [008_f64x4_script_facing_vector_value.md](../type/008_f64x4_script_facing_vector_value.md) | One of the eight values this pattern currently produces side by side |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x1_register`, `f32x2_register`, `f32x3_register`, `f32x4_register`, `f64x1_register`, `f64x2_register`, `f64x3_register`, `f64x4_register` |
| `src/tween_binding.rs` | `tween_f32x1_register`, `tween_f32x2_register`, `tween_f32x3_register`, `tween_f32x4_register`, `tween_f64x1_register`, `tween_f64x2_register`, `tween_f64x3_register`, `tween_f64x4_register` |
| `src/engine.rs` | `engine_build()` — where every side-by-side registration is wired together |
