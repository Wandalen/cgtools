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
fn matrix_lossy_cast_as()
{
  use the_module::{ F64x2x2, I32x2x2 };

  let f = F64x2x2::default().set_raw( [ 1.9, -2.1, 3.5, 4.0 ] );
  let i : I32x2x2 = f.cast_as();
  assert_eq!( i.raw_slice(), &[ 1, -2, 3, 4 ] );
}
