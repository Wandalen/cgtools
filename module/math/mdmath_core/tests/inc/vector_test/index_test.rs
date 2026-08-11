//! Vector-trait coverage (`Collection`, `ConstLength`, `IntoArray`, `ArrayRef`, `ArrayMut`)
//! for the ndarray index types `Ix0`-`Ix4`, exercising the impls in `src/vector/index/`.
//! `Ix4` support was added by resolving that module's `implement for 4` marker — the arity-4
//! assertions here are its regression guard.

use super::*;

#[ test ]
fn test_const_length_ix()
{
  use the_module::ConstLength;
  use ndarray::{ Ix0, Ix1, Ix2, Ix3, Ix4 };

  assert_eq!( < Ix0 as ConstLength >::LEN, 0 );
  assert_eq!( < Ix1 as ConstLength >::LEN, 1 );
  assert_eq!( < Ix2 as ConstLength >::LEN, 2 );
  assert_eq!( < Ix3 as ConstLength >::LEN, 3 );
  assert_eq!( < Ix4 as ConstLength >::LEN, 4 );
}

#[ test ]
fn test_collection_scalar_ix()
{
  use the_module::Collection;
  use ndarray::{ Ix0, Ix1, Ix2, Ix3, Ix4 };

  fn scalar_is_usize< T >( _ : &T )
  where
    T : Collection< Scalar = usize >,
  {
  }

  scalar_is_usize( &Ix0() );
  scalar_is_usize( &Ix1( 1 ) );
  scalar_is_usize( &Ix2( 1, 2 ) );
  scalar_is_usize( &Ix3( 1, 2, 3 ) );
  scalar_is_usize( &Ix4( 1, 2, 3, 4 ) );
}

#[ test ]
fn test_into_array_ix()
{
  use the_module::IntoArray;
  use ndarray::{ Ix0, Ix1, Ix2, Ix3, Ix4 };

  let got : [ usize ; 0 ] = Ix0().into_array();
  let expected : [ usize ; 0 ] = [];
  assert_eq!( got, expected );
  assert_eq!( Ix1( 1 ).into_array(), [ 1 ] );
  assert_eq!( Ix2( 1, 2 ).into_array(), [ 1, 2 ] );
  assert_eq!( Ix3( 1, 2, 3 ).into_array(), [ 1, 2, 3 ] );
  assert_eq!( Ix4( 1, 2, 3, 4 ).into_array(), [ 1, 2, 3, 4 ] );
}

#[ test ]
fn test_array_ref_ix()
{
  use the_module::ArrayRef;
  use ndarray::{ Ix0, Ix1, Ix2, Ix3, Ix4 };

  let dim = Ix0();
  let got : &[ usize ; 0 ] = dim.array_ref();
  let expected : [ usize ; 0 ] = [];
  assert_eq!( got, &expected );

  let dim = Ix1( 5 );
  let got : &[ usize ; 1 ] = dim.array_ref();
  assert_eq!( got, &[ 5 ] );

  let dim = Ix2( 5, 6 );
  let got : &[ usize ; 2 ] = dim.array_ref();
  assert_eq!( got, &[ 5, 6 ] );

  let dim = Ix3( 5, 6, 7 );
  let got : &[ usize ; 3 ] = dim.array_ref();
  assert_eq!( got, &[ 5, 6, 7 ] );

  let dim = Ix4( 9, 8, 7, 6 );
  let got : &[ usize ; 4 ] = dim.array_ref();
  assert_eq!( got, &[ 9, 8, 7, 6 ] );
}

#[ test ]
fn test_vector_mut_ix()
{
  use the_module::ArrayMut;
  use ndarray::{ Ix2, Ix4 };

  let mut dim = Ix2( 1, 2 );
  {
    let arr : &mut [ usize ; 2 ] = dim.vector_mut();
    arr[ 1 ] = 20;
  }
  assert_eq!( dim[ 0 ], 1 );
  assert_eq!( dim[ 1 ], 20 );

  let mut dim = Ix4( 1, 2, 3, 4 );
  {
    let arr : &mut [ usize ; 4 ] = dim.vector_mut();
    arr[ 0 ] = 10;
    arr[ 3 ] = 40;
  }
  // Mutations through the transmuted array view must be visible through Dim's own indexing.
  assert_eq!( dim[ 0 ], 10 );
  assert_eq!( dim[ 1 ], 2 );
  assert_eq!( dim[ 2 ], 3 );
  assert_eq!( dim[ 3 ], 40 );
}
