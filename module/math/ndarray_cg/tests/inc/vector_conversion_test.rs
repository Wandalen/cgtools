//! Coverage for generic `Vector` conversion surface (`src/vector/general.rs`): the `IntoVector`
//! blanket over `IntoArray` implementors, the `as_vector` clone path, and `TryFrom< &[ E ] >`
//! with its typed `VectorLengthMismatch` error.

use super::*;

#[ test ]
fn test_into_vector_from_tuples_and_arrays()
{
  use the_module::{ Vector, IntoVector };

  let got : Vector< i32, 2 > = ( 1, 2 ).into_vector();
  assert_eq!( got.to_array(), [ 1, 2 ] );

  let got : Vector< i32, 3 > = ( 1, 2, 3 ).into_vector();
  assert_eq!( got.to_array(), [ 1, 2, 3 ] );

  let got : Vector< f32, 4 > = [ 1.0, 2.0, 3.0, 4.0 ].into_vector();
  assert_eq!( got.to_array(), [ 1.0, 2.0, 3.0, 4.0 ] );
}

#[ test ]
fn test_as_vector_does_not_consume()
{
  use the_module::{ Vector, IntoVector };

  let src = ( 5, 6 );
  let got : Vector< i32, 2 > = src.as_vector();
  assert_eq!( got.to_array(), [ 5, 6 ] );
  assert_eq!( src, ( 5, 6 ) );
}

#[ test ]
fn test_try_from_slice_ok()
{
  use the_module::Vector;

  let data = [ 7, 8, 9 ];
  let got = Vector::< i32, 3 >::try_from( &data[ .. ] );
  assert_eq!( got.unwrap().to_array(), [ 7, 8, 9 ] );
}

/// The typed error must carry both the compile-time expected length and the actual slice
/// length, and render both in its `Display` output.
#[ test ]
fn test_try_from_slice_length_mismatch_typed_error()
{
  use the_module::{ Vector, VectorLengthMismatch };

  let data = [ 7, 8, 9 ];
  let got = Vector::< i32, 2 >::try_from( &data[ .. ] );
  let err = got.unwrap_err();
  assert_eq!( err, VectorLengthMismatch { expected : 2, actual : 3 } );
  let rendered = format!( "{}", err );
  assert!( rendered.contains( '3' ) && rendered.contains( '2' ), "Display must name both lengths, got: {rendered}" );

  // The error participates in std error handling ( boxable as dyn Error ).
  let _boxed : Box< dyn std::error::Error > = Box::new( err );
}
