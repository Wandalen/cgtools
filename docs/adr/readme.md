# ADR Doc Definition

### Scope

- **Purpose**: Record ecosystem-level architecture decisions — choices that bind multiple crates at once and cannot be reconstructed from any single crate's source.
- **Responsibility**: Document each decision's context, the decision itself, the alternatives considered, and the consequences accepted.
- **In Scope**: Decisions spanning multiple crates of this workspace.
- **Out of Scope**: Single-crate decisions (see that crate's own `docs/`, e.g. `tiles_tools/docs/architectural_evaluation/`); investigations that have not yet produced a decision (see [../explorations/readme.md](../explorations/readme.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Multi-Stack Rendering Architecture](001_multi_stack_rendering_architecture.md) | Shared foundation + invariant-defined stacks (d2, tile, d3) as the ecosystem's shape | ✅ Accepted |
