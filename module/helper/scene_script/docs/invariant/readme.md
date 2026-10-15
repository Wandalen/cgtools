# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on. In `scene_script`, this collection is the navigational hub for the correctness properties that must always hold, tying each one to its enforcement mechanism and the consequences of violating it. This collection holds one instance per invariant, each pinned to where it is enforced in code; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s correctness properties that must always hold.
- **Responsibility**: Document each invariant's precise statement, enforcement mechanism, and violation consequences.
- **In Scope**: The top-level bindings convention every compiled script's statement list must satisfy.
- **Out of Scope**: Semantic determinism of what a script's `main()` body actually does at runtime (an authorial discipline, not checked — see `pitfall/`); the schema-level bindings themselves (see crate `readme.md`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Top-Level Bindings Convention](001_top_level_bindings_convention.md) | A script's top level holds only declarative bindings and a single trailing entry-point call | ✅ |
| 002 | [F32x2/F64x2 Type Distinctness](002_f32x2_f64x2_type_distinctness.md) | No registered vector type ever implicitly converts to or interchanges with another, by precision or arity | ✅ |
| 003 | [Rhai-Facing Names Mirror Rust Identifiers](003_rhai_facing_names_mirror_rust_identifiers.md) | Every registered name textually matches the Rust identifier it wraps | ✅ |
| 004 | [Script-As-Data Purity](004_script_as_data_purity.md) | No function or method call anywhere in the AST, top-level or nested | ✅ |
