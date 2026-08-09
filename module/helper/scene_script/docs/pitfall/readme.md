# Pitfall Doc Definition

### Scope

- **Purpose**: Navigational hub for `scene_script`'s known traps — non-obvious ways to get bitten by the current implementation.
- **Responsibility**: Document each trap, its concrete failure mode, and mitigation.
- **In Scope**: Rhai-language scoping surprises; gaps between what `top_level_lint` checks and what the top-level bindings convention actually promises.
- **Out of Scope**: The convention's own statement and enforcement mechanism (see `invariant/001`, which these pitfalls document the limits of).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Script Functions Can't See Outer-Scope Bindings](001_functions_cannot_see_outer_scope_bindings.md) | A `fn` body sees only its own parameters and locals, never top-level `let`/`const` | ✅ |
| 002 | [Checker Enforcement Is Structural, Not Semantic](002_checker_is_structural_not_semantic.md) | A side-effecting call can hide inside a `let` initializer or a larger expression, undetected | ✅ |
