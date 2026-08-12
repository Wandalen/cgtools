# Feature: Rhai Scene Scripting

### Scope

- **Purpose**: Single navigational hub for "script a 2D scene or its animation in Rhai" — the crate's whole capability, pointing to every artifact that implements it.
- **Responsibility**: Tie together both script forms, all four bindings, and every supporting doc instance without restating their content.
- **In Scope**: Everything reachable from `engine_build()` plus the two script-form patterns that use it.
- **Out of Scope**: Restating content already owned by another collection — this hub cross-references only.

### Design

`scene_script` embeds Rhai and exposes exactly one curated vocabulary slice today: two vector precisions (`F32x2`, `F64x2`) and Linear-eased tweens over either. A script can use this vocabulary in two styles, and the crate itself does not force either one: as a pure data document, returning a value built from vector arithmetic with no engine calls beyond construction (the script-as-data form), or as imperative glue that drives the host through loops, branches, and mutation confined inside a `main()` entry point (the script-as-glue form). `top_level_lint` constrains only a script's top-level *shape* — it says nothing about which of the two styles a shape-compliant script is actually written in.

The current scope is deliberately narrow: no color type, no vector arity beyond 2, no easing curve beyond Linear (see [`pitfall/006`](../pitfall/006_only_linear_easing_is_exposed_to_scripts.md)). Whether this narrowness is the finished design or a first slice of a larger intended surface is not stated anywhere in this workspace — no `roadmap.md` references `scene_script`, and no committed task exists to expand it. This is an open question, not a confirmed gap or a confirmed final scope: [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the mechanical seam for growing the surface if and when that is decided, but this document does not assert that decision has been made.

### Invariants

| File | Relationship |
|------|--------------|
| [001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) | The structural shape every script must satisfy |
| [002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | Non-interchangeability between the two vector precisions |
| [003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | The naming rule linking every registered name to its Rust identifier |

### Pitfalls

| File | Relationship |
|------|--------------|
| [001_functions_cannot_see_outer_scope_bindings.md](../pitfall/001_functions_cannot_see_outer_scope_bindings.md) | Rhai function-scoping surprise |
| [002_checker_is_structural_not_semantic.md](../pitfall/002_checker_is_structural_not_semantic.md) | Limits of the top-level shape check |
| [003_rhai_serde_bridge_requires_exact_float_type.md](../pitfall/003_rhai_serde_bridge_requires_exact_float_type.md) | Consumer-side deserialization trap |
| [004_f32_boundary_cast_truncates_precision.md](../pitfall/004_f32_boundary_cast_truncates_precision.md) | Silent precision loss constructing `F32x2` |
| [005_optimization_level_simple_folds_trivial_top_level_constructs.md](../pitfall/005_optimization_level_simple_folds_trivial_top_level_constructs.md) | Optimizer folding before the checker runs |
| [006_only_linear_easing_is_exposed_to_scripts.md](../pitfall/006_only_linear_easing_is_exposed_to_scripts.md) | The current scope's easing limitation |

### Algorithms

| File | Relationship |
|------|--------------|
| [001_top_level_statement_classification.md](../algorithm/001_top_level_statement_classification.md) | How the top-level shape check actually classifies each statement |

### APIs

| File | Relationship |
|------|--------------|
| [001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | Everything a script can call |

### Data Structures

| File | Relationship |
|------|--------------|
| [001_f32x2_f64x2_script_facing_vector_types.md](../data_structure/001_f32x2_f64x2_script_facing_vector_types.md) | Script-facing vector shape |
| [002_tween_script_facing_type.md](../data_structure/002_tween_script_facing_type.md) | Script-facing tween shape |

### Patterns

| File | Relationship |
|------|--------------|
| [001_manual_customtype_registration_for_foreign_types.md](../pattern/001_manual_customtype_registration_for_foreign_types.md) | How foreign types get exposed to Rhai at all |
| [002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | How the surface extends to new precisions/arities |
| [../../../../../docs/pattern/004_script_as_data.md](../../../../../docs/pattern/004_script_as_data.md) | The declarative script form (root-level, cross-crate) |
| [../../../../../docs/pattern/005_script_as_glue.md](../../../../../docs/pattern/005_script_as_glue.md) | The imperative script form (root-level, cross-crate) |

### Dependencies

| File | Relationship |
|------|--------------|
| [001_rhai_internals_feature.md](../dependency/001_rhai_internals_feature.md) | Why `rhai` and its `internals` feature were selected |

### Integrations

| File | Relationship |
|------|--------------|
| [001_rhai_engine_boundary.md](../integration/001_rhai_engine_boundary.md) | The runtime boundary between Rust and the embedded interpreter |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Crate entry point |
| `src/engine.rs` | `engine_build()` |
| `src/vector_binding.rs` | Vector bindings |
| `src/tween_binding.rs` | Tween bindings |
| `src/top_level_lint.rs` | Top-level shape checker |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | Binding smoke tests |
| `tests/example_convention_test.rs` | Checker edge cases and real example-script conformance |
