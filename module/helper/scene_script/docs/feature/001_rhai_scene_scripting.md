# Feature: Rhai Scene Scripting

### Scope

- **Purpose**: Single navigational hub for "script a 2D scene or its animation in Rhai" — the crate's whole capability, pointing to every artifact that implements it.
- **Responsibility**: Tie together both script forms, all four bindings, and every supporting doc instance without restating their content.
- **In Scope**: Everything reachable from `engine_build()` plus the two script-form patterns that use it.
- **Out of Scope**: Restating content already owned by another collection — this hub cross-references only.

### Design

`scene_script` embeds Rhai and exposes exactly one curated vocabulary slice today: the full `{F32,F64}x{1,2,3,4}` vector family (two precisions across four arities, eight types), the arity-generic vector math each type carries (`dot`/`mag`/`mag2`/`normalize`/`distance`/`distance_squared`/`min`/`max`/unary negation, plus `cross` and `to_homogenous` on the two arity-3 types and `truncate` on the two arity-4 types), and tweens over any of the eight vector types — Linear-eased by default, eased along any of 24 named CSS-style curves a script selects by name, or eased along a `CubicHermite` curve built directly from two tangent vectors. A script can use this vocabulary in two styles, and the crate itself does not force either one: as a pure data document, returning a value built from vector arithmetic with no engine calls beyond construction (the script-as-data form), or as imperative glue that drives the host through loops, branches, and mutation confined inside a `main()` entry point (the script-as-glue form). `top_level_lint` constrains only a script's top-level *shape* — it says nothing about which of the two styles a shape-compliant script is actually written in.

The current scope is deliberately narrow in what remains: no color type, and no script-facing quaternion type — which is why `Squad`, the one remaining tangent-parameterized easing curve, still cannot be constructed from a script at all (see [`pitfall/006`](../pitfall/006_parameterized_easing_curves_are_unreachable_by_name.md)). The vector arity limitation this paragraph once noted (arity capped at 2) no longer holds — all four arities `ndarray_cg`'s own naming family defines (1 through 4) are now registered for both precisions; the easing limitation this paragraph once noted (Linear only) no longer holds either — every zero-argument easing curve the host `animation` crate defines is now selectable by name; and the `CubicHermite` limitation this paragraph once noted (unreachable by any means) no longer holds either — a dedicated 5-arg overload now builds it directly from two tangent vectors, without going through the name-based selector at all. Whether the *remaining* narrowness (no color type, no `Squad` support) is the finished design or a further slice yet to expand is not stated anywhere in this workspace — no `roadmap.md` references `scene_script`, and no committed task exists to expand it further. This is an open question, not a confirmed gap or a confirmed final scope: [`pattern/002`](../pattern/002_dual_precision_side_by_side_registration.md) documents the mechanical seam that already carried the surface from arity 2 alone to the full family and from Linear-only to 25 named curves plus one direct-tangent curve, and would carry any future growth the same way.

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
| [001_tween_script_facing_type.md](../data_structure/001_tween_script_facing_type.md) | Script-facing tween shape |

### Dependencies

| File | Relationship |
|------|--------------|
| [001_rhai_internals_feature.md](../dependency/001_rhai_internals_feature.md) | Why `rhai` and its `internals` feature were selected |

### Integrations

| File | Relationship |
|------|--------------|
| [001_rhai_engine_boundary.md](../integration/001_rhai_engine_boundary.md) | The runtime boundary between Rust and the embedded interpreter |

### Invariants

| File | Relationship |
|------|--------------|
| [001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) | The structural shape every script must satisfy |
| [002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) | Non-interchangeability between the two vector precisions |
| [003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) | The naming rule linking every registered name to its Rust identifier |
| [004_script_as_data_purity.md](../invariant/004_script_as_data_purity.md) | The whole-AST no-call guarantee the script-as-data form requires |

### Patterns

| File | Relationship |
|------|--------------|
| [001_manual_customtype_registration_for_foreign_types.md](../pattern/001_manual_customtype_registration_for_foreign_types.md) | How foreign types get exposed to Rhai at all |
| [002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | How the surface extends to new precisions/arities |
| [../../../../../docs/pattern/004_script_as_data.md](../../../../../docs/pattern/004_script_as_data.md) | The declarative script form (root-level, cross-crate) |
| [../../../../../docs/pattern/005_script_as_glue.md](../../../../../docs/pattern/005_script_as_glue.md) | The imperative script form (root-level, cross-crate) |

### Pitfalls

| File | Relationship |
|------|--------------|
| [001_functions_cannot_see_outer_scope_bindings.md](../pitfall/001_functions_cannot_see_outer_scope_bindings.md) | Rhai function-scoping surprise |
| [002_checker_is_structural_not_semantic.md](../pitfall/002_checker_is_structural_not_semantic.md) | Limits of the top-level shape check |
| [003_rhai_serde_bridge_requires_exact_float_type.md](../pitfall/003_rhai_serde_bridge_requires_exact_float_type.md) | Consumer-side deserialization trap |
| [004_f32_boundary_cast_truncates_precision.md](../pitfall/004_f32_boundary_cast_truncates_precision.md) | Silent precision loss constructing any `F32x*` type |
| [005_optimization_level_simple_folds_trivial_top_level_constructs.md](../pitfall/005_optimization_level_simple_folds_trivial_top_level_constructs.md) | Optimizer folding before the checker runs |
| [006_parameterized_easing_curves_are_unreachable_by_name.md](../pitfall/006_parameterized_easing_curves_are_unreachable_by_name.md) | The remaining easing limitation now that named-curve selection covers every zero-argument preset and a direct-tangent overload reaches `CubicHermite` — `Squad` alone remains fully blocked |

### Types

| File | Relationship |
|------|--------------|
| [001_f32x2_script_facing_vector_value.md](../type/001_f32x2_script_facing_vector_value.md) | Script-facing single-precision 2D vector value |
| [002_f64x2_script_facing_vector_value.md](../type/002_f64x2_script_facing_vector_value.md) | Script-facing double-precision 2D vector value |
| [003_f32x1_script_facing_vector_value.md](../type/003_f32x1_script_facing_vector_value.md) | Script-facing single-precision 1D vector value |
| [004_f64x1_script_facing_vector_value.md](../type/004_f64x1_script_facing_vector_value.md) | Script-facing double-precision 1D vector value |
| [005_f32x3_script_facing_vector_value.md](../type/005_f32x3_script_facing_vector_value.md) | Script-facing single-precision 3D vector value |
| [006_f64x3_script_facing_vector_value.md](../type/006_f64x3_script_facing_vector_value.md) | Script-facing double-precision 3D vector value |
| [007_f32x4_script_facing_vector_value.md](../type/007_f32x4_script_facing_vector_value.md) | Script-facing single-precision 4D vector value |
| [008_f64x4_script_facing_vector_value.md](../type/008_f64x4_script_facing_vector_value.md) | Script-facing double-precision 4D vector value |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Crate entry point |
| `src/engine.rs` | `engine_build()` |
| `src/vector_binding.rs` | Vector bindings |
| `src/tween_binding.rs` | Tween bindings |
| `src/top_level_lint.rs` | Top-level shape checker |
| `src/purity_lint.rs` | Whole-AST purity checker |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | Binding smoke tests |
| `tests/example_convention_test.rs` | Checker edge cases and real example-script conformance |
| `tests/purity_lint_test.rs` | Whole-AST purity checker accept/reject cases |
