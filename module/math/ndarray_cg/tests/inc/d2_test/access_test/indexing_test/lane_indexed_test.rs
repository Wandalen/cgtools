use super::*;

use super::*;

fn test_valid_row_indexed_iteration_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ nd, Mat, IndexingRef, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp : Vec<( nd::Ix2, f32 )> = vec![];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 0 ), 1.0 )];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 0 ), 1.0 ), ( nd::Ix2( 0, 1 ), 2.0 )];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 1 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 1, 0 ), 3.0 ), ( nd::Ix2( 1, 1 ), 4.0 )];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 0 ), 1.0 ), ( nd::Ix2( 0, 1 ), 2.0 ), ( nd::Ix2( 0, 2 ), 3.0 )];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 1 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 1, 0 ), 4.0 ), ( nd::Ix2( 1, 1 ), 5.0 ), ( nd::Ix2( 1, 2 ), 6.0 )];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec<_> = mat.lane_indexed_iter( 0, 2 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 2, 0 ), 7.0 ), ( nd::Ix2( 2, 1 ), 8.0 ), ( nd::Ix2( 2, 2 ), 9.0 )];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_indexed_iteration_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_indexed_iteration_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_indexed_iteration_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_indexed_iteration_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_indexed_iteration_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ nd, Mat, IndexingRef, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp : Vec<( nd::Ix2, f32 )> = vec![];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 0 ), 1.0 )];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 0 ), 1.0 ), ( nd::Ix2( 1, 0 ), 3.0 )];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 1 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 1 ), 2.0 ), ( nd::Ix2( 1, 1 ), 4.0 )];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 0 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 0 ), 1.0 ), ( nd::Ix2( 1, 0 ), 4.0 ), ( nd::Ix2( 2, 0 ), 7.0 )];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 1 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 1 ), 2.0 ), ( nd::Ix2( 1, 1 ), 5.0 ), ( nd::Ix2( 2, 1 ), 8.0 )];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec<_> = mat.lane_indexed_iter( 1, 2 ).map( | ( idx, &val ) | ( nd::Ix2( idx[ 0 ], idx[ 1 ] ), val ) ).collect();
  let exp = vec![( nd::Ix2( 0, 2 ), 3.0 ), ( nd::Ix2( 1, 2 ), 6.0 ), ( nd::Ix2( 2, 2 ), 9.0 )];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_indexed_iteration_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_indexed_iteration_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_indexed_iteration_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_indexed_iteration_generic::< DescriptorOrderColumnMajor >();
}

fn test_invalid_dimension_indexed_generic<D: the_module::mat::Descriptor + std::panic::RefUnwindSafe>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use std::panic;
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let result = panic::catch_unwind( ||
  {
    let _ = mat.lane_indexed_iter( 2, 0 ).collect::<Vec<_>>();
  });

  assert!( result.is_err(), "Expected panic, but no panic occurred" );
}

#[test]
fn test_invalid_dimension_indexed_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_invalid_dimension_indexed_generic::<DescriptorOrderRowMajor>();
}

#[test]
fn test_invalid_dimension_indexed_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_invalid_dimension_indexed_generic::<DescriptorOrderColumnMajor>();
}

fn test_negative_lane_index_indexed_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_indexed_iter( 0, usize::MAX ).collect();
}

#[test]
#[should_panic]
fn test_negative_lane_index_indexed_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_negative_lane_index_indexed_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_negative_lane_index_indexed_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_negative_lane_index_indexed_generic::<DescriptorOrderColumnMajor>();
}

fn test_out_of_bounds_lane_index_indexed_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_indexed_iter( 0, 2 ).collect();
  println!( "{_collected:?}" );
}

#[test]
#[should_panic]
fn test_out_of_bounds_lane_index_indexed_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_out_of_bounds_lane_index_indexed_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_out_of_bounds_lane_index_indexed_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_out_of_bounds_lane_index_indexed_generic::<DescriptorOrderColumnMajor>();
}

fn test_lane_iter_indexed_mut_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<3, 3, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingMut<Scalar = f32>,
{
  use the_module::{ Mat, RawSliceMut, IndexingMut };

  let mut mat = Mat::<3, 3, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ]);

  // Modify the second column
  for ( index, value ) in mat.lane_iter_indexed_mut( 1, 1 )
  {
    *value += 10.0;
    println!( "Modified index {:?} to value {}", index, *value );
  }

  let expected = Mat::<3, 3, f32, D>::default().set([ 1.0, 12.0, 3.0, 4.0, 15.0, 6.0, 7.0, 18.0, 9.0 ]);
  assert_eq!( mat.raw_slice(), expected.raw_slice(), "Column modification failed" );
}

#[test]
fn test_lane_iter_indexed_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_indexed_mut_generic::<DescriptorOrderRowMajor>();
}

#[test]
fn test_lane_iter_indexed_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_indexed_mut_generic::<DescriptorOrderColumnMajor>();
}