# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Breaking Changes
- `web::file::load` no longer prepends `/static/` to its argument. The path is
  now used verbatim, except that bare filenames (no scheme, no leading `/`) are
  joined to the origin with a single `/`. Callers that previously relied on the
  implicit prefix must pass `"static/<path>"` explicitly. Absolute URLs
  (`http://`, `https://`, `//host/...`) and origin-absolute paths (`/foo`) are
  honored directly, enabling fetches from CDNs and non-trunk deployments.

### Added
- `web-sys` `console` feature is now activated (was previously relying on
  transitive activation that broke under `cargo test` on the host target).

## [0.2.0] - 2024-08-08

### Added
- Minimal graphics library with abstract rendering backend
- WebGL support with browser integration
- Abstract geometry and buffer management
- Camera orbit controls for 3D scenes
- Model loading capabilities (OBJ format support)
- Data type abstractions for graphics primitives
- Memory management utilities for graphics data
- WebAssembly-optimized implementation
- File handling and async operations for web environments
- Diagnostic tools for graphics debugging

### Changed
- Enhanced backend abstraction for multiple rendering targets
- Improved performance optimizations for web environments

[0.2.0]: https://github.com/Wandalen/cgtools/releases/tag/mingl-v0.2.0