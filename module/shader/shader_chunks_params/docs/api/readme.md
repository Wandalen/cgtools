# API Doc Definition

An **api** documents a public programmatic interface exposed to external callers. In `shader_chunks_params`, this collection is the navigational hub for the crate's entire public surface: the `//@ param:` grammar, the 5-kind taxonomy, and the `discover`/`discover_chunk`/`infer_range` functions. This collection holds one instance per distinct interface; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `shader_chunks_params`'s public API.
- **Responsibility**: Document the `//@ param:` grammar, the taxonomy types, and the discovery functions' behavior and panic contracts.
- **In Scope**: Everything exported via `mod_interface!` in `src/lib.rs`.
- **Out of Scope**: The range-inference heuristic's own rule table (see `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Tunable Parameter Taxonomy](001_tunable_parameter_taxonomy.md) | The `//@ param:` grammar, the 5-kind taxonomy, and every public type/function | ✅ |
