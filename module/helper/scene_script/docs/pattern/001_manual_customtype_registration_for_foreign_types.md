# Pattern: Manual CustomType Registration for Foreign Types

### Scope

- **Purpose**: Name the technique used to expose a foreign Rust type to Rhai scripts without violating the orphan rule.
- **Responsibility**: Define the pattern's problem, solution, applicability, and trade-offs.
- **In Scope**: The `register_type_with_name` + `register_fn`/`register_get` technique.
- **Out of Scope**: The declarative/imperative script-form patterns this technique's output serves (see root [`docs/pattern/004`](../../../../../docs/pattern/004_script_as_data.md), [`005`](../../../../../docs/pattern/005_script_as_glue.md)); which precision/arity variants to register (see [`pattern/002`](002_dual_precision_side_by_side_registration.md)).

### Problem

Rhai's own `#[derive(CustomType)]` requires implementing its `CustomType` trait directly on the target type. `F32x2`/`F64x2` (from `ndarray_cg`) and `Tween<T>` (from `animation`) are all foreign to `scene_script` — neither the trait nor the types are defined in this crate — so Rust's orphan rule forbids that `impl` here. Without a workaround, none of these types could be exposed to scripts at all.

### Solution

Register each foreign type by hand instead of deriving:

- `Engine::register_type_with_name::<T>(name)` gives the type a script-visible name without requiring any trait to be implemented on `T`.
- `Engine::register_fn`/`register_get` attach constructors, methods, operators, and property getters individually, each defined as a free function or closure living inside `scene_script` — never as a trait method on the foreign type itself. This is what sidesteps the orphan rule: no trait is implemented on foreign types anywhere in this pattern, only ordinary functions are registered against a type parameter.

### Applicability

Applies to every type currently exposed to scripts — `F32x2`, `F64x2`, and `Tween<F32x2>`/`Tween<F64x2>` are all foreign, so all four registrations in `vector_binding.rs`/`tween_binding.rs` follow this shape. It would also apply to any future foreign type needing script exposure (a new math or animation primitive from a sibling crate). It is unnecessary overhead for a type `scene_script` itself owns — none exist today, since every script-facing type originates from `ndarray_cg` or `animation`.

### Consequences

- **More boilerplate per type**: every field, constructor, operator, and method must be registered individually; `#[derive(CustomType)]` would have generated much of this automatically had the type not been foreign.
- **No compiler-checked completeness**: omitting a `register_get` call for a field simply makes that field silently unreachable from scripts — there is no compile error the way a missing trait method would produce.
- **Works within the orphan-rule constraint at zero dependency cost**: no wrapper newtype, no upstream trait implementation request, no extra crate — the technique is entirely local to `scene_script`.

### Data Structures

| File | Relationship |
|------|--------------|
| [001_f32x2_f64x2_script_facing_vector_types.md](../data_structure/001_f32x2_f64x2_script_facing_vector_types.md) | The shape this pattern produces for the vector types |
| [002_tween_script_facing_type.md](../data_structure/002_tween_script_facing_type.md) | The shape this pattern produces for the tween type |

### APIs

| File | Relationship |
|------|--------------|
| [001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | The full set of operations registered using this technique |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x2_register`, `f64x2_register` |
| `src/tween_binding.rs` | `tween_f32x2_register`, `tween_f64x2_register` |
