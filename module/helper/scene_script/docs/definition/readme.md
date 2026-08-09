# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `invariant/` | Correctness properties that must always hold, and their enforcement mechanisms | [invariant/readme.md](../invariant/readme.md) | 1 |
| `pitfall/` | Known traps in Rhai scoping and checker enforcement, their failure modes, and mitigations | [pitfall/readme.md](../pitfall/readme.md) | 2 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|--------|-----|------|------|
| invariant | 001 | Top-Level Bindings Convention | [invariant/001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) |
| pitfall | 001 | Script Functions Can't See Outer-Scope Bindings | [pitfall/001_functions_cannot_see_outer_scope_bindings.md](../pitfall/001_functions_cannot_see_outer_scope_bindings.md) |
| pitfall | 002 | Checker Enforcement Is Structural, Not Semantic | [pitfall/002_checker_is_structural_not_semantic.md](../pitfall/002_checker_is_structural_not_semantic.md) |
