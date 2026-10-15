# Pattern Doc Definition

A **pattern** here is a reusable design rule this crate itself is built on — distinct from the ecosystem-wide patterns in the workspace root's `docs/pattern/`, this one is scoped to this crate's own internal design. In `minwebgpu`, that's the facade-over-descriptor-builders architecture shared by every module — a stable reference for the crate's core approach, kept distinct from any single feature. This collection holds one instance per pattern; the table below is the index into them.

### Scope

- **Purpose**: `minwebgpu`'s core architectural approach needs a stable reference distinct from any single feature.
- **Responsibility**: Document confirmed architectural patterns underlying the crate's public API.
- **In Scope**: The facade-over-descriptor-builders architecture shared by every module.
- **Out of Scope**: Per-feature API surface (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Facade Over Descriptor Builders](001_facade_over_descriptor_builders.md) | Safe Rust facade + descriptor builders + explicit device/queue passing over raw `web-sys` | ✅ |
