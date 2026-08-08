# Pattern Doc Definition

### Scope

- **Purpose**: `minwebgpu`'s core architectural approach needs a stable reference distinct from any single feature.
- **Responsibility**: Document confirmed architectural patterns underlying the crate's public API.
- **In Scope**: The facade-over-descriptor-builders architecture shared by every module.
- **Out of Scope**: Per-feature API surface (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Facade Over Descriptor Builders](001_facade_over_descriptor_builders.md) | Safe Rust facade + descriptor builders + explicit device/queue passing over raw `web-sys` | ✅ |
