# Invariant Doc Definition

### Scope

- **Purpose**: Navigational hub for `scene_script`'s correctness properties that must always hold.
- **Responsibility**: Document each invariant's precise statement, enforcement mechanism, and violation consequences.
- **In Scope**: The top-level bindings convention every compiled script's statement list must satisfy.
- **Out of Scope**: Semantic determinism of what a script's `main()` body actually does at runtime (an authorial discipline, not checked — see `pitfall/`); the schema-level bindings themselves (see crate `readme.md`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Top-Level Bindings Convention](001_top_level_bindings_convention.md) | A script's top level holds only declarative bindings and a single trailing entry-point call | ✅ |
