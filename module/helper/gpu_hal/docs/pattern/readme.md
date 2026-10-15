# Pattern Doc Definition

A **pattern** here is a reusable design rule this crate itself is built on — distinct from the ecosystem-wide patterns in the workspace root's `docs/pattern/`, this one is scoped to this crate's own internal design. In `gpu_hal`, that's the enum-per-backend dispatch architecture shared by every handle type — a stable reference for the crate's core approach, kept distinct from any single feature. This collection holds one instance per pattern; the table below is the index into it.

### Scope

- **Purpose**: `gpu_hal`'s core architectural approach needs a stable reference distinct from any single feature.
- **Responsibility**: Document confirmed architectural patterns underlying the crate's public API.
- **In Scope**: The enum-per-backend dispatch architecture shared by every handle type.
- **Out of Scope**: Per-feature API surface (see `feature/`); the workspace-level layering pattern this one instantiates (see `../../../../docs/pattern/002_strict_layering_one_step_drilldown.md`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Enum-Per-Backend Dispatch with One-Step Drill-Down](001_enum_per_backend_dispatch_one_step_drilldown.md) | Backend-tagged enum per handle type, public non-panicking drill-down plus internal panicking dispatch | ✅ |
