//! Verifies the `VectorDataType` descriptor invariants that `data_type.rs` documents
//! (formerly guarded only by a `verify` marker): for flat structures `nelements() == 1`;
//! for nested ( matrix-like ) structures `nelements()` is the inner-array ( row ) length and
//! `natoms()` is the total scalar count; `byte_size()` is scalar size times atom count.
//! Nested-array coverage exercises a non-`f32` primitive too, proving the readme's
//! all-supported-scalars claim.

use super::*;

#[ test ]
fn scalar_descriptor_is_flat_single_atom()
{
  use the_module::{ DataType, IntoVectorDataType };

  let desc = f32::into_vector_data_type();
  assert_eq!( desc.scalar(), DataType::F32 );
  assert_eq!( desc.natoms(), 1 );
  assert_eq!( desc.nelements(), 1 );
  assert_eq!( desc.byte_size(), 4 );
}

#[ test ]
fn flat_array_descriptor_has_nelements_one()
{
  use the_module::{ DataType, IntoVectorDataType };

  let desc = < [ f32 ; 3 ] >::into_vector_data_type();
  assert_eq!( desc.scalar(), DataType::F32 );
  assert_eq!( desc.natoms(), 3 );
  assert_eq!( desc.nelements(), 1 );
  assert_eq!( desc.byte_size(), 12 );

  let desc = < [ u16 ; 5 ] >::into_vector_data_type();
  assert_eq!( desc.scalar(), DataType::U16 );
  assert_eq!( desc.natoms(), 5 );
  assert_eq!( desc.nelements(), 1 );
  assert_eq!( desc.byte_size(), 10 );
}

#[ test ]
fn nested_array_descriptor_has_row_length_nelements()
{
  use the_module::{ DataType, IntoVectorDataType };

  // Matrix-like [ [ f32 ; 4 ] ; 3 ]: 3 rows of 4 scalars.
  let desc = < [ [ f32 ; 4 ] ; 3 ] >::into_vector_data_type();
  assert_eq!( desc.scalar(), DataType::F32 );
  assert_eq!( desc.natoms(), 12 );
  assert_eq!( desc.nelements(), 4 );
  assert_eq!( desc.byte_size(), 48 );

  // Non-f32 nested arrays are supported too ( readme's all-supported-scalars claim ).
  let desc = < [ [ u8 ; 2 ] ; 3 ] >::into_vector_data_type();
  assert_eq!( desc.scalar(), DataType::U8 );
  assert_eq!( desc.natoms(), 6 );
  assert_eq!( desc.nelements(), 2 );
  assert_eq!( desc.byte_size(), 6 );

  let desc = < [ [ i32 ; 3 ] ; 3 ] >::into_vector_data_type();
  assert_eq!( desc.scalar(), DataType::I32 );
  assert_eq!( desc.natoms(), 9 );
  assert_eq!( desc.nelements(), 3 );
  assert_eq!( desc.byte_size(), 36 );
}

#[ test ]
fn byte_size_matches_scalar_width()
{
  use the_module::DataType;

  assert_eq!( DataType::I8.byte_size(), 1 );
  assert_eq!( DataType::U8.byte_size(), 1 );
  assert_eq!( DataType::I16.byte_size(), 2 );
  assert_eq!( DataType::U16.byte_size(), 2 );
  assert_eq!( DataType::I32.byte_size(), 4 );
  assert_eq!( DataType::U32.byte_size(), 4 );
  assert_eq!( DataType::F32.byte_size(), 4 );
}
