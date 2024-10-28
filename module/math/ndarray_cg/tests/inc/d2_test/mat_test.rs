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

#[ test ]
fn default()
{
  use the_module::{ Mat, IndexingRef, Zero };
  use the_module::mat::DescriptorOrderRowMajor;

  let mat : Mat::< 2, 2, f32, DescriptorOrderRowMajor > = Default::default();
  assert!( IndexingRef::iter_unstable( &mat ).all( | e | e.is_zero() ), "Matrix should not be zero after setting non-zero values" );
  // assert!( Default::is_zero( &mat ), "Matrix should be zero after calling set_zero()" );
  let mat = Mat::< 2, 2, f32, DescriptorOrderRowMajor >::default();
  // assert!( mat.is_zero(), "Matrix should be zero after calling set_zero()" );
  assert!( IndexingRef::iter_unstable( &mat ).all( | e | e.is_zero() ), "Matrix should not be zero after setting non-zero values" );

  let mut mat = Mat::< 2, 2, f32, DescriptorOrderRowMajor >::default().raw_set( [ 1.0, 2.0, 3.0, 4.0 ] );

  assert!( !IndexingRef::iter_unstable( &mat ).all( | e | e.is_zero() ), "Matrix should not be zero after setting non-zero values" );
  // assert!( !mat.is_zero(), "Matrix should not be zero after setting non-zero values" );
  // mat.set_zero();
  mat = Default::default();
  assert!( IndexingRef::iter_unstable( &mat ).all( | e | e.is_zero() ), "Matrix should not be zero after setting non-zero values" );
  // assert!( mat.is_zero(), "Matrix should be zero after calling set_zero()" );

}

#[ test ]
fn test_has_index_dim()
{
  use the_module::{ Mat, Indexable, Ix2 };
  use the_module::mat::DescriptorOrderRowMajor;

  // Test for 0x0 Matrix
  let mat_0x0 = Mat::< 0, 0, f32, DescriptorOrderRowMajor >::default();
  let expected_dim_0x0 = Ix2( 0, 0 );
  let dim_0x0 = mat_0x0.dim();
  assert_eq!( dim_0x0, expected_dim_0x0, "Dimension mismatch for 0x0 matrix" );

  // Test for 1x1 Matrix
  let mat_1x1 = Mat::< 1, 1, f32, DescriptorOrderRowMajor >::default();
  let expected_dim_1x1 = Ix2( 1, 1 );
  let dim_1x1 = mat_1x1.dim();
  assert_eq!( dim_1x1, expected_dim_1x1, "Dimension mismatch for 1x1 matrix" );

  // Test for 2x2 Matrix
  let mat_2x2 = Mat::< 2, 2, f32, DescriptorOrderRowMajor >::default();
  let expected_dim_2x2 = Ix2( 2, 2 );
  let dim_2x2 = mat_2x2.dim();
  assert_eq!( dim_2x2, expected_dim_2x2, "Dimension mismatch for 2x2 matrix" );

  // Test for 3x3 Matrix
  let mat_3x3 = Mat::< 3, 3, f32, DescriptorOrderRowMajor >::default();
  let expected_dim_3x3 = Ix2( 3, 3 );
  let dim_3x3 = mat_3x3.dim();
  assert_eq!( dim_3x3, expected_dim_3x3, "Dimension mismatch for 3x3 matrix" );

  // Test for 2x3 Matrix
  let mat_2x3 = Mat::< 2, 3, f32, DescriptorOrderRowMajor >::default();
  let expected_dim_2x3 = Ix2( 2, 3 );
  let dim_2x3 = mat_2x3.dim();
  assert_eq!( dim_2x3, expected_dim_2x3, "Dimension mismatch for 2x3 matrix" );
}
