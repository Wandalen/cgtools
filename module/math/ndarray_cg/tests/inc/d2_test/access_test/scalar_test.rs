use super::*;

#[ test ]
fn assumptions()
{
  let align1 = std::mem::align_of::< [ [ u8 ; 3 ] ; 3 ] >();
  let align2 = std::mem::align_of::< [ u8 ; 9 ] >();
  println!( "align : {align1}" );
  assert_eq!( align1, align2, "Same alignment" );

  let size1 = std::mem::size_of::< [ [ u8 ; 3 ] ; 3 ] >();
  let size2 = std::mem::size_of::< [ u8 ; 9 ] >();
  println!( "size : {size1}" );
  assert_eq!( size1, size2, "Same size" );
}

fn test_scalar_ref_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::ScalarRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 2, 2, f32, D > : Default,
  the_module::Mat< 2, 2, f32, D > : the_module::ConstLayout,
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat, Ix2, ScalarRef };

  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );

  // Test scalar_ref for each element
  let scalar = mat.scalar_ref( Ix2( 0, 0 ) );
  let exp = &1.0;
  assert_eq!( scalar, exp, "Expected {:?}, got {:?}", exp, scalar );

  let scalar = mat.scalar_ref( Ix2( 0, 1 ) );
  let exp = &2.0;
  assert_eq!( scalar, exp, "Expected {:?}, got {:?}", exp, scalar );

  let scalar = mat.scalar_ref( Ix2( 1, 0 ) );
  let exp = &3.0;
  assert_eq!( scalar, exp, "Expected {:?}, got {:?}", exp, scalar );

  let scalar = mat.scalar_ref( Ix2( 1, 1 ) );
  let exp = &4.0;
  assert_eq!( scalar, exp, "Expected {:?}, got {:?}", exp, scalar );
}

#[ test ]
fn test_scalar_ref_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_scalar_ref_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_scalar_ref_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_scalar_ref_generic::< DescriptorOrderColumnMajor >();
}

fn test_scalar_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 3, 3, f32, D > : the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 3, 3, f32, D > : Default,
  the_module::Mat< 3, 3, f32, D > : the_module::ConstLayout,
  the_module::Mat< 3, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat, Ix2, ScalarMut, RawSliceMut };

  let mut mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );

  // Modify a specific element
  let index = Ix2( 2, 2 ); // Access the element at row 2, column 2
  let value = mat.scalar_mut( index );
  *value = 10.0;

  // Verify the modification
  let expected = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0 ] );
  assert_eq!( mat.raw_slice(), expected.raw_slice(), "Modification failed" );
}

#[ test ]
fn test_scalar_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_scalar_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_scalar_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_scalar_mut_generic::< DescriptorOrderColumnMajor >();
}
