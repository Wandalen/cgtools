# Render Stack Doc Definition

A **render stack** (`d2`, `tile`, `d3`) is a vertical slice through the layer ladder — the vocabulary and invariants one rendering domain commits to, as defined by [the invariant-defined-stack pattern](../pattern/001_invariant_defined_stack.md). This collection holds one **instance** — one identity card — per stack: its invariant table, renounced capabilities, member crates, and which layers it occupies. The table below is the index into those three cards.

### Scope

- **Purpose**: One instance per render stack — the stack's identity card: its invariant table, renounced capabilities, member crates, and layer occupancy.
- **Responsibility**: Keep each stack's living invariant table in one place, linking every invariant to the crate-level instance that pins its enforcement.
- **In Scope**: The `d2`, `tile`, and `d3` stacks adopted by [ADR-001](../adr/001_multi_stack_rendering_architecture.md).
- **Out of Scope**: The rules for what a stack *is* and when one is founded (see [../pattern/001_invariant_defined_stack.md](../pattern/001_invariant_defined_stack.md)); the horizontal layer structure inside stacks (see [../layer/readme.md](../layer/readme.md)); per-invariant statements and enforcement (see each crate's own `docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [d2](001_d2.md) | Planar rendering with declarative export — submission-order layering, vector-representable commands | ✅ |
| 002 | [tile](002_tile.md) | Extension of d2 — lattice addresses, command-only compilation, determinism | ✅ |
| 003 | [d3](003_d3.md) | Sibling of d2 — depth-resolved visibility, PBR materials, HDR light transport | ✅ |
