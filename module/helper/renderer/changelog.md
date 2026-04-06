# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed
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