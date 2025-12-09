
# i32 and u32 Vectors
Cgtools provides access to vector types listed below, but math operations are available only for `f32` and `f64`.
Math operations need to be implemented for `i32`, `i64`, `u32`, `u64`
```rust
/// A 1-dimensional vector of `f32`s.
  pub type F32x1 = Vector< f32, 1 >;
  /// A 2-dimensional vector of `f32`s.
  pub type F32x2 = Vector< f32, 2 >;
  /// A 3-dimensional vector of `f32`s.
  pub type F32x3 = Vector< f32, 3 >;
  /// A 4-dimensional vector of `f32`s.
  pub type F32x4 = Vector< f32, 4 >;
  /// A 1-dimensional vector of `f64`s.
  pub type F64x1 = Vector< f64, 1 >;
  /// A 2-dimensional vector of `f64`s.
  pub type F64x2 = Vector< f64, 2 >;
  /// A 3-dimensional vector of `f64`s.
  pub type F64x3 = Vector< f64, 3 >;
  /// A 4-dimensional vector of `f64`s.
  pub type F64x4 = Vector< f64, 4 >;

  /// A 1-dimensional vector of `i32`s.
  pub type I32x1 = Vector< i32, 1 >;
  /// A 2-dimensional vector of `i32`s.
  pub type I32x2 = Vector< i32, 2 >;
  /// A 3-dimensional vector of `i32`s.
  pub type I32x3 = Vector< i32, 3 >;
  /// A 4-dimensional vector of `i32`s.
  pub type I32x4 = Vector< i32, 4 >;
  /// A 1-dimensional vector of `i64`s.
  pub type I64x1 = Vector< i64, 1 >;
  /// A 2-dimensional vector of `i64`s.
  pub type I64x2 = Vector< i64, 2 >;
  /// A 3-dimensional vector of `i64`s.
  pub type I64x3 = Vector< i64, 3 >;
  /// A 4-dimensional vector of `i64`s.
  pub type I64x4 = Vector< i64, 4 >;

  /// A 1-dimensional vector of `u32`s.
  pub type U32x1 = Vector< u32, 1 >;
  /// A 2-dimensional vector of `u32`s.
  pub type U32x2 = Vector< u32, 2 >;
  /// A 3-dimensional vector of `u32`s.
  pub type U32x3 = Vector< u32, 3 >;
  /// A 4-dimensional vector of `u32`s.
  pub type U32x4 = Vector< u32, 4 >;
  /// A 1-dimensional vector of `u64`s.
  pub type U64x1 = Vector< u64, 1 >;
  /// A 2-dimensional vector of `u64`s.
  pub type U64x2 = Vector< u64, 2 >;
  /// A 3-dimensional vector of `u64`s.
  pub type U64x3 = Vector< u64, 3 >;
  /// A 4-dimensional vector of `u64`s.
  pub type U64x4 = Vector< u64, 4 >;
```

# GLTF loader
The loader, at the end of proccessing the gltf file, needs to compute everything the scene needs - bounding box of each node, world matrices, etc.