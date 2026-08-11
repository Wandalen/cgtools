# Feature: Shadow Mapping

Shadows via depth-from-light: the scene is rendered from the light's
viewpoint into a depth texture, then that occlusion is composed into the lit
image.

### Scope

- **Purpose**: Navigational hub for the shadow subsystem — map generation and its composition path.
- **Responsibility**: Describe the `ShadowMap` structure and where shadow occlusion enters the frame, linked to sources.
- **In Scope**: Shadow-map rendering and shadow-to-color composition.
- **Out of Scope**: The lights themselves and their capacity limits (see `../invariant/002`); the main frame pipeline the result composes into (see [001_pbr_rendering_core.md](001_pbr_rendering_core.md)).

### Design

**Map generation.** `src/webgl/shadow.rs` defines `ShadowMap`: a framebuffer
with a depth texture at a configurable square `resolution`, plus a dedicated
program — its documented job is "rendering depth from light's perspective".
Scene nodes are drawn through it to record, per light, the nearest occluder
depth.

**Composition.** The recorded occlusion is turned into shading terms on the
main image via the dedicated post pass
`src/webgl/post_processing/shadow_to_color.rs`, keeping shadow application a
pipeline step rather than a per-material concern.

**Fit with the pipeline.** Shadow maps are ordinary depth targets (not
`RGBA16F` color targets), so this subsystem does not itself depend on
`EXT_color_buffer_float`; it inherits the requirement only by living in the
same frame as the HDR pipeline.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_depth_buffer_visibility_with_oit.md](../invariant/001_depth_buffer_visibility_with_oit.md) | Same depth-buffer machinery, applied from the light's viewpoint instead of the camera's |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/post_processing/shadow_to_color.rs` | Composition of shadow occlusion into the lit image |
| `src/webgl/shadow.rs` | `ShadowMap`: framebuffer, depth texture, program, resolution |

### Tests

| File | Relationship |
|------|--------------|
| — | No dedicated shadow test exists in `tests/` yet; coverage is indirect through rendering examples |
