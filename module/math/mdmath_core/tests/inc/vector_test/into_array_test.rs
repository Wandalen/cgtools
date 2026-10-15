//! Coverage for the `IntoArray` trait across its implementors: unit and arity 1-4 tuples,
//! fixed arrays, slices (including the loud length-mismatch contract), the `&T`/`&mut T`
//! forwarding impls, and the `as_array` clone path. Resolves the `cover by test` marker on
//! `IntoArray`'s exposure in `src/vector/mod.rs`.

use super::*;

#[ test ]
fn test_into_array_tuples()
{
  use the_module::IntoArray;

  let got : [ i32 ; 0 ] = ().into_array();
  let expected : [ i32 ; 0 ] = [];
  assert_eq!( got, expected );

  let got : [ i32 ; 1 ] = ( 1, ).into_array();
  assert_eq!( got, [ 1 ] );

  let got : [ i32 ; 2 ] = ( 1, 2 ).into_array();
  assert_eq!( got, [ 1, 2 ] );

  let got : [ i32 ; 3 ] = ( 1, 2, 3 ).into_array();
  assert_eq!( got, [ 1, 2, 3 ] );

  let got : [ i32 ; 4 ] = ( 1, 2, 3, 4 ).into_array();
  assert_eq!( got, [ 1, 2, 3, 4 ] );
}

#[ test ]
fn test_into_array_array_identity()
{
  use the_module::IntoArray;

  let src : [ u8 ; 3 ] = [ 7, 8, 9 ];
  let got : [ u8 ; 3 ] = src.into_array();
  assert_eq!( got, [ 7, 8, 9 ] );
}

#[ test ]
fn test_into_array_slice()
{
  use the_module::IntoArray;

  let data = vec![ 10, 20, 30 ];
  let slice : &[ i32 ] = &data;
  let got : [ i32 ; 3 ] = slice.into_array();
  assert_eq!( got, [ 10, 20, 30 ] );
}

/// Slice length must match the requested arity exactly — the mismatch fails loudly instead of
/// truncating or zero-filling.
#[ test ]
#[ should_panic( expected = "Slice length does not match array length" ) ]
fn test_into_array_slice_length_mismatch_panics()
{
  use the_module::IntoArray;

  let data = vec![ 10, 20, 30 ];
  let slice : &[ i32 ] = &data;
  let _got : [ i32 ; 2 ] = slice.into_array();
}

/// The `&T`/`&mut T` forwarding impls clone the collection instead of consuming it — the
/// original must remain intact and unchanged after the call.
#[ test ]
fn test_into_array_reference_forwarding()
{
  use the_module::IntoArray;

  let tuple = ( 5, 6 );
  let tuple_ref = &tuple;
  let got : [ i32 ; 2 ] = tuple_ref.into_array();
  assert_eq!( got, [ 5, 6 ] );
  assert_eq!( tuple, ( 5, 6 ) );

  let mut tuple = ( 7, 8 );
  let tuple_mut = &mut tuple;
  let got : [ i32 ; 2 ] = tuple_mut.into_array();
  assert_eq!( got, [ 7, 8 ] );
  assert_eq!( tuple, ( 7, 8 ) );
}

#[ test ]
fn test_as_array_does_not_consume()
{
  use the_module::IntoArray;

  let tuple = ( 1, 2, 3 );
  let got : [ i32 ; 3 ] = tuple.as_array();
  assert_eq!( got, [ 1, 2, 3 ] );
  assert_eq!( tuple, ( 1, 2, 3 ) );
}
