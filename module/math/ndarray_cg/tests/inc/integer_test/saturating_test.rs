//! Saturating / wrapping / checked integer arithmetic on `Vector` and `Mat`.

use super::*;

#[ test ]
fn vector_saturating_add()
{
  use the_module::I32x3;
  let a = I32x3::from_array( [ i32::MAX, 0, -5 ] );
  let b = I32x3::from_array( [ 1, 10, -10 ] );
  let r = a.saturating_add( b );
  assert_eq!( r, I32x3::from_array( [ i32::MAX, 10, -15 ] ) );
}

#[ test ]
fn vector_saturating_sub()
{
  use the_module::U32x3;
  let a = U32x3::from_array( [ 0, 100, 50 ] );
  let b = U32x3::from_array( [ 1, 30, 100 ] );
  let r = a.saturating_sub( b );
  assert_eq!( r, U32x3::from_array( [ 0, 70, 0 ] ) );
}

#[ test ]
fn vector_wrapping_add()
{
  use the_module::U32x3;
  let a = U32x3::from_array( [ u32::MAX, 5, 10 ] );
  let b = U32x3::from_array( [ 1, 5, 0 ] );
  let r = a.wrapping_add( b );
  assert_eq!( r, U32x3::from_array( [ 0, 10, 10 ] ) );
}

#[ test ]
fn vector_wrapping_mul()
{
  use the_module::I32x2;
  let a = I32x2::from_array( [ i32::MAX, 3 ] );
  let b = I32x2::from_array( [ 2, 4 ] );
  let r = a.wrapping_mul( b );
  // i32::MAX * 2 wraps; second component is the usual 12.
  assert_eq!( r[ 1 ], 12 );
  assert_eq!( r[ 0 ], i32::MAX.wrapping_mul( 2 ) );
}

#[ test ]
fn vector_checked_overflow()
{
  use the_module::I32x3;
  let a = I32x3::from_array( [ i32::MAX, 0, 0 ] );
  let b = I32x3::from_array( [ 1, 0, 0 ] );
  assert_eq!( a.checked_add( b ), None );

  let c = I32x3::from_array( [ 1, 2, 3 ] );
  let d = I32x3::from_array( [ 4, 5, 6 ] );
  assert_eq!( c.checked_add( d ), Some( I32x3::from_array( [ 5, 7, 9 ] ) ) );
}

#[ test ]
fn matrix_saturating_add()
{
  use the_module::I32x2x2;
  let a = I32x2x2::default().set_raw( [ i32::MAX, 0, 0, 0 ] );
  let b = I32x2x2::default().set_raw( [ 1, 0, 0, 0 ] );
  let r = a.saturating_add( b );
  assert_eq!( r.raw_slice(), &[ i32::MAX, 0, 0, 0 ] );
}

#[ test ]
fn matrix_checked_overflow()
{
  use the_module::I32x2x2;
  let a = I32x2x2::default().set_raw( [ i32::MAX, 0, 0, 0 ] );
  let b = I32x2x2::default().set_raw( [ 1, 0, 0, 0 ] );
  assert_eq!( a.checked_add( b ), None );

  let c = I32x2x2::default().set_raw( [ 1, 2, 3, 4 ] );
  let d = I32x2x2::default().set_raw( [ 5, 6, 7, 8 ] );
  let r = c.checked_add( d ).unwrap();
  assert_eq!( r.raw_slice(), &[ 6, 8, 10, 12 ] );
}

#[ test ]
fn matrix_wrapping_mul()
{
  use the_module::U32x2x2;
  let a = U32x2x2::default().set_raw( [ u32::MAX, 1, 1, 1 ] );
  let b = U32x2x2::default().set_raw( [ 2, 1, 1, 1 ] );
  let r = a.wrapping_mul( b );
  assert_eq!( r.raw_slice()[ 0 ], u32::MAX.wrapping_mul( 2 ) );
}
