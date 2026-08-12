# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `scene_script`, this collection is the navigational hub for the implementation's known traps — non-obvious ways its current behavior can bite a consumer — recording each one's concrete failure mode and how to mitigate it. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s known traps — non-obvious ways to get bitten by the current implementation.
- **Responsibility**: Document each trap, its concrete failure mode, and mitigation.
- **In Scope**: Rhai-language scoping surprises; gaps between what `top_level_lint` checks and what the top-level bindings convention actually promises; traps a consumer hits deserializing a script's returned value via `rhai::serde`; silent precision loss at the `f32` construction boundary; optimizer folding that can hide a construct from the checker; the current scope's easing-curve limitation.
- **Out of Scope**: The convention's own statement and enforcement mechanism (see `invariant/001`, which these pitfalls document the limits of).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Script Functions Can't See Outer-Scope Bindings](001_functions_cannot_see_outer_scope_bindings.md) | A `fn` body sees only its own parameters and locals, never top-level `let`/`const` | ✅ |
| 002 | [Checker Enforcement Is Structural, Not Semantic](002_checker_is_structural_not_semantic.md) | A side-effecting call can hide inside a `let` initializer or a larger expression, undetected | ✅ |
| 003 | [`rhai::serde`'s Bridge Requires the Exact `FLOAT` Type](003_rhai_serde_bridge_requires_exact_float_type.md) | An `f32` deserialize-target field fails outright against Rhai's `f64` `FLOAT` — it never narrows | ✅ |
| 004 | [f32 Boundary Cast Silently Truncates Precision](004_f32_boundary_cast_truncates_precision.md) | `f32x2(...)` silently narrows any `f64` literal beyond `f32` precision | ✅ |
| 005 | [`OptimizationLevel::Simple` Folds Trivial Top-Level Constructs](005_optimization_level_simple_folds_trivial_top_level_constructs.md) | A trivial `if`/`let` can be optimized away before the checker ever sees it | ✅ |
| 006 | [Only Linear Easing Is Exposed to Scripts](006_only_linear_easing_is_exposed_to_scripts.md) | `tween(...)` is always Linear-eased; no script-facing way to select another curve | ✅ |
