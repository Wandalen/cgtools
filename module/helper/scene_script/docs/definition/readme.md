# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `algorithm/` | Non-trivial procedures, stated as HOW rather than WHAT/WHERE | [algorithm/readme.md](../algorithm/readme.md) | 1 |
| `api/` | The crate's callable surface — operations, error handling, compatibility guarantees | [api/readme.md](../api/readme.md) | 1 |
| `data_structure/` | Script-facing value shapes — fields, mutability, invariants | [data_structure/readme.md](../data_structure/readme.md) | 2 |
| `dependency/` | Crate selection rationale and configuration for direct dependencies | [dependency/readme.md](../dependency/readme.md) | 1 |
| `feature/` | Navigational hub tying every doc instance to the one end-to-end capability | [feature/readme.md](../feature/readme.md) | 1 |
| `integration/` | The runtime boundary between this crate and the embedded Rhai interpreter | [integration/readme.md](../integration/readme.md) | 1 |
| `invariant/` | Correctness properties that must always hold, and their enforcement mechanisms | [invariant/readme.md](../invariant/readme.md) | 3 |
| `pattern/` | Recurring crate-local solution shapes — problem, solution, applicability, consequences | [pattern/readme.md](../pattern/readme.md) | 2 |
| `pitfall/` | Known traps in Rhai scoping, checker enforcement, precision, and easing scope, their failure modes, and mitigations | [pitfall/readme.md](../pitfall/readme.md) | 6 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|--------|-----|------|------|
| algorithm | 001 | Top-Level Statement Classification | [algorithm/001_top_level_statement_classification.md](../algorithm/001_top_level_statement_classification.md) |
| api | 001 | Rhai Scripting Surface | [api/001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) |
| data_structure | 001 | F32x2/F64x2 Script-Facing Vector Types | [data_structure/001_f32x2_f64x2_script_facing_vector_types.md](../data_structure/001_f32x2_f64x2_script_facing_vector_types.md) |
| data_structure | 002 | Tween Script-Facing Type | [data_structure/002_tween_script_facing_type.md](../data_structure/002_tween_script_facing_type.md) |
| dependency | 001 | rhai `internals` Feature | [dependency/001_rhai_internals_feature.md](../dependency/001_rhai_internals_feature.md) |
| feature | 001 | Rhai Scene Scripting | [feature/001_rhai_scene_scripting.md](../feature/001_rhai_scene_scripting.md) |
| integration | 001 | Rhai Engine Boundary | [integration/001_rhai_engine_boundary.md](../integration/001_rhai_engine_boundary.md) |
| invariant | 001 | Top-Level Bindings Convention | [invariant/001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) |
| invariant | 002 | F32x2/F64x2 Type Distinctness | [invariant/002_f32x2_f64x2_type_distinctness.md](../invariant/002_f32x2_f64x2_type_distinctness.md) |
| invariant | 003 | Rhai-Facing Names Mirror Rust Identifiers | [invariant/003_rhai_facing_names_mirror_rust_identifiers.md](../invariant/003_rhai_facing_names_mirror_rust_identifiers.md) |
| pattern | 001 | Manual CustomType Registration for Foreign Types | [pattern/001_manual_customtype_registration_for_foreign_types.md](../pattern/001_manual_customtype_registration_for_foreign_types.md) |
| pattern | 002 | Dual-Precision Side-by-Side Registration | [pattern/002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) |
| pitfall | 001 | Script Functions Can't See Outer-Scope Bindings | [pitfall/001_functions_cannot_see_outer_scope_bindings.md](../pitfall/001_functions_cannot_see_outer_scope_bindings.md) |
| pitfall | 002 | Checker Enforcement Is Structural, Not Semantic | [pitfall/002_checker_is_structural_not_semantic.md](../pitfall/002_checker_is_structural_not_semantic.md) |
| pitfall | 003 | `rhai::serde`'s Bridge Requires the Exact `FLOAT` Type | [pitfall/003_rhai_serde_bridge_requires_exact_float_type.md](../pitfall/003_rhai_serde_bridge_requires_exact_float_type.md) |
| pitfall | 004 | f32 Boundary Cast Silently Truncates Precision | [pitfall/004_f32_boundary_cast_truncates_precision.md](../pitfall/004_f32_boundary_cast_truncates_precision.md) |
| pitfall | 005 | `OptimizationLevel::Simple` Folds Trivial Top-Level Constructs | [pitfall/005_optimization_level_simple_folds_trivial_top_level_constructs.md](../pitfall/005_optimization_level_simple_folds_trivial_top_level_constructs.md) |
| pitfall | 006 | Only Linear Easing Is Exposed to Scripts | [pitfall/006_only_linear_easing_is_exposed_to_scripts.md](../pitfall/006_only_linear_easing_is_exposed_to_scripts.md) |
