//! Scalar conversions: `cast`, `cast_as`, lossless `From`/`Into` between
//! vector and matrix scalar variants.

use super::*;

#[ test ]
fn vector_lossless_into_widening()
{
  use the_module::{ I32x3, F64x3 };

  let i = I32x3::from_array( [ 1, 2, 3 ] );
  // Lossless: i32 -> f64 has `From`, so `.into()` works.
  let f : F64x3 = i.into();
  assert_eq!( f, F64x3::from_array( [ 1.0, 2.0, 3.0 ] ) );
}

#[ test ]
fn vector_lossless_into_remaining_pairs()
{
  use the_module::{ I32x3, U32x3, F32x3, I64x3, U64x3, F64x3 };

  // i32 -> i64
  let a : I64x3 = I32x3::from_array( [ 1, -2, 3 ] ).into();
  assert_eq!( a, I64x3::from_array( [ 1i64, -2, 3 ] ) );

  // u32 -> u64
  let b : U64x3 = U32x3::from_array( [ 1, 2, 3 ] ).into();
  assert_eq!( b, U64x3::from_array( [ 1u64, 2, 3 ] ) );

  // u32 -> i64 (the full u32 range fits in i64)
  let c : I64x3 = U32x3::from_array( [ 1, 2, u32::MAX ] ).into();
  assert_eq!( c, I64x3::from_array( [ 1i64, 2, i64::from( u32::MAX ) ] ) );

  // u32 -> f64
  let d : F64x3 = U32x3::from_array( [ 1, 2, 3 ] ).into();
  assert_eq!( d, F64x3::from_array( [ 1.0, 2.0, 3.0 ] ) );

  // f32 -> f64
  let e : F64x3 = F32x3::from_array( [ 1.5, -2.5, 3.0 ] ).into();
  assert_eq!( e, F64x3::from_array( [ 1.5, -2.5, 3.0 ] ) );
}

#[ test ]
fn vector_lossy_cast_as()
{
  use the_module::{ F64x3, I32x3 };

  let f = F64x3::from_array( [ 1.7, 2.3, -3.9 ] );
  // `as i32` truncates toward zero.
  let i : I32x3 = f.cast_as();
  assert_eq!( i, I32x3::from_array( [ 1, 2, -3 ] ) );
}

#[ test ]
fn vector_explicit_cast_method()
{
  use the_module::{ I32x3, I64x3 };

  let a = I32x3::from_array( [ 1, 2, 3 ] );
  let b : I64x3 = a.cast::< i64 >();
  assert_eq!( b, I64x3::from_array( [ 1i64, 2, 3 ] ) );
}

#[ test ]
fn matrix_lossless_into_widening()
{
  use the_module::{ I32x2x2, F64x2x2 };

  let i = I32x2x2::default().set_raw( [ 1, 2, 3, 4 ] );
  let f : F64x2x2 = i.into();
  assert_eq!( f.raw_slice(), &[ 1.0, 2.0, 3.0, 4.0 ] );
}

#[ test ]
fn matrix_lossless_into_remaining_pairs()
{
  use the_module::{ I32x2x2, U32x2x2, F32x2x2, I64x2x2, U64x2x2, F64x2x2 };

  // i32 -> i64
  let a : I64x2x2 = I32x2x2::default().set_raw( [ 1, -2, 3, -4 ] ).into();
  assert_eq!( a.raw_slice(), &[ 1i64, -2, 3, -4 ] );

  // u32 -> u64
  let b : U64x2x2 = U32x2x2::default().set_raw( [ 1, 2, 3, 4 ] ).into();
  assert_eq!( b.raw_slice(), &[ 1u64, 2, 3, 4 ] );

  // u32 -> i64 (the full u32 range fits in i64)
  let c : I64x2x2 = U32x2x2::default().set_raw( [ 1, 2, 3, u32::MAX ] ).into();
  assert_eq!( c.raw_slice(), &[ 1i64, 2, 3, i64::from( u32::MAX ) ] );

  // u32 -> f64
  let d : F64x2x2 = U32x2x2::default().set_raw( [ 1, 2, 3, 4 ] ).into();
  assert_eq!( d.raw_slice(), &[ 1.0, 2.0, 3.0, 4.0 ] );

  // f32 -> f64
  let e : F64x2x2 = F32x2x2::default().set_raw( [ 1.5, -2.5, 3.0, -4.0 ] ).into();
  assert_eq!( e.raw_slice(), &[ 1.5, -2.5, 3.0, -4.0 ] );
}

#[ test ]
fn matrix_lossy_cast_as()
{
  use the_module::{ F64x2x2, I32x2x2 };

  let f = F64x2x2::default().set_raw( [ 1.9, -2.1, 3.5, 4.0 ] );
  let i : I32x2x2 = f.cast_as();
  assert_eq!( i.raw_slice(), &[ 1, -2, 3, 4 ] );
}
