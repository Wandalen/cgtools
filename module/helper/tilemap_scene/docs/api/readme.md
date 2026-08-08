# Api Doc Entity

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s programmatic runtime integration surface.
- **Responsibility**: Document the public operations a game uses to drive a loaded scene and render it.
- **In Scope**: `Scene` instance lifecycle and mutation, `Renderer::render`, `Camera`.
- **Out of Scope**: The declarative file format these operations act on (see `format/`); the internal rendering algorithm `render()` invokes (see `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Renderer Integration API](001_renderer_integration_api.md) | `Scene`/`Renderer`/`Camera` operations, with 3 corrections against the original specification's contract | ⚠️ |

Status ⚠️ marks an API doc that discloses divergences between the original specification's documented contract and the shipped implementation — see the file's own Error Handling section.
