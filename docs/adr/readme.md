# ADR Doc Definition

An **ADR** (Architecture Decision Record) captures one ecosystem-level choice — something that binds multiple crates at once — along with why it was made and what was considered instead. This collection holds one **instance** — one ADR file — per accepted decision; the table below is the index into them.

### Scope

- **Purpose**: Record ecosystem-level architecture decisions — choices that bind multiple crates at once and cannot be reconstructed from any single crate's source.
- **Responsibility**: Document each decision's context, the decision itself, the alternatives considered, and the consequences accepted.
- **In Scope**: Decisions spanning multiple crates of this workspace.
- **Out of Scope**: Single-crate decisions (see that crate's own `docs/`, e.g. `tiles_tools/docs/architectural_evaluation/`); investigations that have not yet produced a decision (see [../explorations/readme.md](../explorations/readme.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Multi-Stack Rendering Architecture](001_multi_stack_rendering_architecture.md) | Shared foundation + invariant-defined stacks (d2, tile, d3) as the ecosystem's shape | ✅ Accepted |
| 002 | [In-House GPU HAL](002_gpu_hal_in_house.md) | L1 is the in-house `gpu_hal` over the `min*` drivers — `wgpu` powers the native leg, not the abstraction | ✅ Accepted |
| 003 | [Extend L1 HAL Adoption to the d2 Stack](003_d2_stack_hal_adoption.md) | `tilemap_renderer` gains `gpu_hal`-backed WebGPU/native adapters plus a no-op adapter; L5→L3 wiring stays example-local | ✅ Accepted |
