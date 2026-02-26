# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- **Shader program caching**: Materials sharing identical shader source code now reuse a single compiled GPU program instead of compiling duplicates. Programs are keyed by full source text in a `shader_source_registry`.
- **Draw call grouping**: Opaque and transparent primitives are sorted by program UUID before drawing, minimizing GPU state switches (program binds).
- **Three-phase rendering pipeline**: The render loop is now split into (1) scene traversal & program compilation, (2) per-program uniform uploads (camera, lights, exposure), and (3) sorted draw calls.
- **Material `bind()` contract**: `bind()` is now the single method responsible for activating texture units, uploading texture data, and binding textures. Implementations must call `gl.active_texture()` before each texture bind. The `upload_textures()` trait method has been removed.
- **IBL texture safety**: IBL textures are rebound after every `material.bind()` call, preventing non-IBL materials from accidentally overwriting IBL texture units.
- **Dirty-flag pattern for `needs_update`**: `PbrMaterial::needs_update` is now `Cell<bool>` with interior mutability. The renderer calls `clear_needs_update()` after uploading uniforms, so `upload_on_state_change()` is skipped on subsequent frames unless the material is explicitly marked dirty via `set_needs_update()`.

### Fixed

- Fixed IBL texture corruption where `upload_textures()` could overwrite IBL texture units because `active_texture` was not reset after `ibl.bind()`.
- Fixed `light_map` texture not being bound in `PbrMaterial::bind()` (was missing from the bind list).
- Fixed texture unit state leak in custom materials (`GemMaterial`, `SurfaceMaterial`) — `upload()` is now called inside `bind()` with explicit `active_texture()` per unit.

### Removed

- Removed `upload_textures()` from the `Material` trait.
- Removed `base_shader_hash()` from the `Material` trait and `PbrMaterial`.
- Removed dead/commented-out rendering code from `renderer.rs`.
- **BREAKING**: Renamed `shadow::Light::light_size()` to `shadow::Light::size()`
- Upgraded shadow depth format from `DEPTH_COMPONENT24` to `DEPTH_COMPONENT32F`

### Added

- `cull_mode` field to `PbrMaterial` for fine-grained face culling control
- `Drop` implementation for `SwapFramebuffer` to prevent GPU memory leaks
- GSAA (Geometric Specular Anti-Aliasing) for improved specular highlights
- Reflection-space LOD bias for reduced IBL aliasing
- Dither noise (IGN) for HDR banding reduction
- Firefly suppression via selective Reinhard tonemapping
- `highp` precision qualifiers for IBL and shadow map samplers

## [0.1.0] - 2024-08-08

### Added

- Initial release of renderer crate
- 3D renderer for WebGL applications
- glTF model loading and processing support
- Scene graph management with hierarchical transforms
- Material system with PBR support
- Mesh rendering with vertex/index buffers
- Camera controls and projection management
- Post-processing pipeline with multiple effects
- Outline rendering (narrow and wide variants)
- Image-based lighting (IBL) support
- Texture and sampler management
- WebAssembly-optimized rendering pipeline

[0.1.0]: https://github.com/Wandalen/cgtools/releases/tag/renderer-v0.1.0
