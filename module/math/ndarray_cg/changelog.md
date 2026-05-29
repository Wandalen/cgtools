# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added
- `MatNum` trait — element bound for field-agnostic arithmetic (`Add`/`Sub`/`Mul`/`Div`/`Rem`, `Zero`/`One`, and the `*Assign` counterparts). Satisfied by every integer primitive and float, and used wherever an operation does not require `sqrt`, trig, or approximate equality.
- Integer matrix type aliases following the existing `F32xNxN` convention: `I32x2x2`/`I32x3x3`/`I32x4x4`, `I64xNxN`, `U32xNxN`, `U64xNxN`.
- `Eq` and `Ord` impls for `Vector<E, N>` and `Mat<R, C, E, D>` (gated on `E : Eq` / `E : Ord`), enabling integer vectors and matrices as `BTreeMap` / `BTreeSet` keys.
- Component-wise scalar conversions on `Vector` and `Mat`:
  - `cast::<T>()` via `T : From<E>` — reserved for lossless primitive conversions.
  - `cast_as::<T>()` via `num_traits::AsPrimitive` — `as`-style lossy / truncating conversions.
  - Concrete `From` impls for lossless primitive pairings (`i32 → i64 / f64`, `u32 → i64 / u64 / f64`, `f32 → f64`) so `.into()` resolves without ascription.
- Integer-only arithmetic helpers, dispatched per-element via `num_traits`:
  - `saturating_add` / `saturating_sub`
  - `wrapping_add` / `wrapping_sub` / `wrapping_mul`
  - `checked_add` / `checked_sub` / `checked_mul` (returning `Option<Self>`)
  - `IntegerScalar` marker trait (`MatEl + PrimInt`) for gating downstream integer-only code on a single bound.

### Changed
- Relaxed `E : nd::NdFloat` to `E : MatNum` (or `E : MatEl`) across the arithmetic and operator surfaces — matrix `+ - * /`, vector `+ - * / %`, scalar mul/div, `dot`, `mag2`, `transpose`, `determinant`, `identity`, `to_array` / `to_homogenous` / `truncate`, `from_cols`, `from_row_major` / `from_column_major`, and the `scale` / `shear` / `translation` constructors. `cross` and `distance_squared` are gated on `E : MatNum + num_traits::Signed` (signed integers and floats) because their intermediate subtractions can be negative. Float-only operations (`mag` / `normalize` / `distance`, rotation, perspective / look_at / orthographic, `inverse`, `decompose`, approx-eq, spherical conversions, quaternions) keep the `NdFloat` bound.
- Relaxed `IndexingRef` / `IndexingMut` / `ScalarRef` / `ScalarMut` impls for `Mat` from `nd::NdFloat` to `MatEl` — the access traits never relied on floating-point semantics.
- Upgraded `ndarray` dependency from 0.16 to 0.17. The prelude re-export now lists items explicitly (omitting `ArrayRef`, `LayoutRef`, `RawRef`, and the `ArrayRefN` aliases) to avoid colliding with the local `ArrayRef` trait used as a generic bound.

## [0.3.0] - 2024-08-08

### Added
- High-performance computer graphics mathematics library
- Built on top of ndarray for efficient array operations
- 2D and 3D vector types with comprehensive operations
- Matrix types (2x2, 3x3, 4x4) with transformation support
- Quaternion support for 3D rotations
- Homogeneous coordinate transformations
- Memory layout compatibility with graphics APIs
- Approximation utilities for floating-point comparisons
- Comprehensive arithmetic operation support

### Changed
- Enhanced performance optimizations for graphics computations
- Improved type safety and API ergonomics

[0.3.0]: https://github.com/Wandalen/cgtools/releases/tag/ndarray_cg-v0.3.0