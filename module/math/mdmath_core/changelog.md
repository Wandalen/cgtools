# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added
- `Scalar` trait (exposed via the `float` layer) — `Copy + num_traits::Num + num_traits::NumAssign`. Blanket-implemented for every integer primitive and float; used as the element bound for arithmetic helpers that do not require float-specific semantics.

### Changed
- Relaxed `E : NdFloat` to `E : Scalar` on the field-agnostic vector helpers: `dot`, `mag2`, `cross` / `cross_mut`, and the `sum` / `sub` / `mul` / `div` family (including the `_mut` and `_scalar` variants). Float-specific helpers (`mag`, `normalize`, `normalize_to`, `project_on`, `angle`, `is_orthogonal`, `min`, `max`) keep their `NdFloat` bound.

## [0.3.0] - 2024-08-08

### Added
- Core multidimensional mathematics library
- Fundamental mathematical traits and types
- Modular feature system for selective functionality
- Index operations and array manipulation utilities
- N-dimensional array support through ndarray integration
- Floating-point arithmetic operations and utilities
- Approximation comparison tools for floating-point values
- General mathematical operations and transformations
- No-std compatibility for embedded environments

### Changed
- Enhanced modular architecture with optional features
- Improved performance and memory efficiency

[0.3.0]: https://github.com/Wandalen/cgtools/releases/tag/mdmath_core-v0.3.0