# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added
- **Compressed texture formats** (`texture::compressed`, behind the `constants` feature): `Format` names the targets a UASTC block can be transcoded into (ASTC 4×4, BC7, ETC2 RGBA8, and uncompressed RGBA8 as the fallback) and reports each one's `internal_format`, `extension_name`, `block_bytes` and `level_size`; `ColorSpace` makes the caller choose linear or sRGB explicitly, because uploading sRGB to a shader that already linearises decodes twice. `Support::query` enables and records which of the four WebGL compressed-texture extensions the context offers (the `*_EXTENSION` constants), minus formats a software driver advertises but only emulates, and `Support::best` picks the preferred format; `Support::advertised`, `Support::without_emulated` and `is_mesa_like_platform` expose the two halves of that check. The raw `compressedTexImage2D` internal-format constants (BPTC, S3TC, ETC2 / EAC, ASTC) are added to the `constants` set. Used by `renderer`'s `ktx2` feature.
- `context::from_canvas_2d_with_options( canvas, will_read_frequently )`: creates a 2D context with the `willReadFrequently` hint, for canvases read back with `getImageData` repeatedly. `from_canvas_2d` now delegates to it with `false`, the browser default, so its behaviour is unchanged.

## [0.2.0] - 2024-08-08

### Added
- Minimal WebGL toolkit for concise graphics programming
- Comprehensive shader management and compilation utilities
- Resource management for buffers, textures, and vertex arrays
- WebGL context creation and configuration
- Attribute and uniform handling with type safety
- Texture loading and management (2D and cube maps)
- Framebuffer operations and render-to-texture support
- Drawing utilities and primitive rendering
- File loading and async operations for web environments
- Diagnostic tools and debugging utilities
- Integration with browser logging and console output

### Changed
- Enhanced performance optimizations for WebGL operations
- Improved error handling and diagnostic capabilities

[0.2.0]: https://github.com/Wandalen/cgtools/releases/tag/minwebgl-v0.2.0