# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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

## [0.1.0] - 2026-01-08

### Added

- Material rendering properties: Materials can now control face culling,
front face winding order, depth testing,
depth comparison function, and depth write mask
- New enums: `CullMode`, `FrontFace` for material property configuration
- Material trait methods: `get_cull_mode()`, `get_front_face()`,
`is_depth_test_enabled()`, `is_depth_write_enabled()`, `get_depth_func()`
- Renderer now respects per-material rendering settings
during opaque and transparent passes

[0.1.0]: https://github.com/Wandalen/cgtools/releases/tag/renderer-v0.1.0
