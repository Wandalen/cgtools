//! Indices.

/// Internal namespace.
mod private
{
  use crate::*;

  /// A vector structure.
  #[ derive( Clone, Copy, PartialEq, PartialOrd, Hash, Debug ) ]
  pub struct Vector< E, const LEN : usize >( pub [ E; LEN ] )
  where E : MatEl;

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

}

crate::mod_interface!
{

  /// General trait implementation for the vector type
  layer general;
  /// General arithmetics for the vector type
  layer arithmetics;

  /// Overloading of operators, like index, sub, div, etc.
  layer operator;

  // /// Conversions from `Array` type to `Vector`
  // layer array;
  /// Functionality related to 2D vectors
  layer vec2;
  /// Functionality related to 3D vectors
  layer vec3;
  /// Functionality related to 4D vectors
  layer vec4;

  reuse ::mdmath_core::vector;

  exposed use
  {

    Vector,

    F32x1,
    F32x2,
    F32x3,
    F32x4,
    F64x1,
    F64x2,
    F64x3,
    F64x4,

    I32x1,
    I32x2,
    I32x3,
    I32x4,
    I64x1,
    I64x2,
    I64x3,
    I64x4,

    U32x1,
    U32x2,
    U32x3,
    U32x4,
    U64x1,
    U64x2,
    U64x3,
    U64x4,

  };

}
