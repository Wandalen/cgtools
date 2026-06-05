//! Integer-specific traits: Eq, Ord, hashing, and identity/determinant/
//! transpose semantics.

use super::*;

#[ test ]
fn vector_eq_ord_hash()
{
  use std::collections::BTreeMap;
  use the_module::I32x3;

  let a = I32x3::from_array( [ 1, 2, 3 ] );
  let b = I32x3::from_array( [ 1, 2, 3 ] );
  let c = I32x3::from_array( [ 1, 2, 4 ] );

  assert_eq!( a, b );
  assert!( a < c );
  assert!( c > a );

  // Ord lets us use as BTreeMap key.
  let mut m = BTreeMap::new();
  m.insert( a, "a" );
  m.insert( c, "c" );
  assert_eq!( m.get( &b ), Some( &"a" ) );
}

#[ test ]
fn mat_eq_ord()
{
  use the_module::I32x2x2;
  let a = I32x2x2::default().set_raw( [ 1, 2, 3, 4 ] );
  let b = I32x2x2::default().set_raw( [ 1, 2, 3, 4 ] );
  let c = I32x2x2::default().set_raw( [ 1, 2, 3, 5 ] );

  assert_eq!( a, b );
  assert!( a < c );
}

#[ test ]
fn integer_identity_and_determinant()
{
  use the_module::{ I32x2x2, I32x3x3, I32x4x4 };

  let id2 = I32x2x2::identity();
  assert_eq!( id2.determinant(), 1 );

  let id3 = I32x3x3::identity();
  assert_eq!( id3.determinant(), 1 );

  let id4 = I32x4x4::identity();
  assert_eq!( id4.determinant(), 1 );

  let m = I32x2x2::default().set_raw( [ 2, 3, 5, 7 ] );
  assert_eq!( m.determinant(), 2 * 7 - 3 * 5 );
}

#[ test ]
fn integer_transpose()
{
  use the_module::I32x2x2;
  let m = I32x2x2::default().set_raw( [ 1, 2, 3, 4 ] );
  let t = m.transpose();
  // For column-major storage, transpose moves rows<->cols.
  let r = t.transpose();
  assert_eq!( r.raw_slice(), m.raw_slice() );
}

#[ test ]
fn integer_from_row_column_major()
{
  use the_module::{ I32x2x2, mat::{ DescriptorOrderColumnMajor, DescriptorOrderRowMajor } };
  let row : the_module::Mat< 2, 2, i32, DescriptorOrderRowMajor > =
    the_module::Mat::< 2, 2, i32, DescriptorOrderRowMajor >::from_row_major( [ 1, 2, 3, 4 ] );
  assert_eq!( row.raw_slice(), &[ 1, 2, 3, 4 ] );

  let col : I32x2x2 = I32x2x2::from_column_major( [ 1, 2, 3, 4 ] );
  let _ = col; // smoke
  let col2 : the_module::Mat< 2, 2, i32, DescriptorOrderColumnMajor > =
    the_module::Mat::< 2, 2, i32, DescriptorOrderColumnMajor >::from_column_major( [ 1, 2, 3, 4 ] );
  assert_eq!( col2.raw_slice(), &[ 1, 2, 3, 4 ] );
}
