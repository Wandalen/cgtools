# Pitfall Doc Definition

### Scope

- **Purpose**: Navigational hub for confirmed traps consumers of `renderer` hit in practice.
- **Responsibility**: Document each trap, its concrete failure mode, and its mitigation.
- **In Scope**: Traps confirmed against the current source.
- **Out of Scope**: Guaranteed behavior (see `invariant/`); subsystem design (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Requires EXT_color_buffer_float](001_requires_ext_color_buffer_float.md) | The crate renders into `RGBA16F` targets but never enables the WebGL2 extension that makes them color-renderable — callers must | ✅ |
