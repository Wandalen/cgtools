use super::*;

#[ test ]
fn assumptions()
{
  use the_module::nd::array;

  let data = array!
  [
    [ 1., 2., 3. ],
    [ 3., 5., 6. ],
  ];

  println!( "shape : {:?}", data.shape() );
  println!( "strides : {:?}", data.strides() );
  println!( "row : {:?}", data.row( 0 ) );
  println!( "[ 1 ][ 2 ] : {:?}", data[ [ 1, 2 ] ] ); // 6

  assert_eq!( [ 2, 3 ], data.shape() );
  assert_eq!( [ 3, 1 ], data.strides() );
  assert_eq!( 6., data[ [ 1, 2 ] ] );
}

fn test_valid_row_iteration_1x2_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 1, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 1x2 matrix
  let mat = Mat::< 1, 2, f32, D >::default().set( [ 1.0, 2.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect(); // Convert references to values
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_iteration_1x2_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_iteration_1x2_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_iteration_1x2_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_iteration_1x2_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_iteration_1x2_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 1, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 1x2 matrix
  let mat = Mat::< 1, 2, f32, D >::default().set( [ 1.0, 2.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_iteration_1x2_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_iteration_1x2_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_iteration_1x2_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_iteration_1x2_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_row_iteration_2x1_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 2x1 matrix
  let mat = Mat::< 2, 1, f32, D >::default().set( [ 1.0, 2.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 2.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_iteration_2x1_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_iteration_2x1_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_iteration_2x1_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_iteration_2x1_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_iteration_2x1_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 2x1 matrix
  let mat = Mat::< 2, 1, f32, D >::default().set( [ 1.0, 2.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_iteration_2x1_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_iteration_2x1_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_iteration_2x1_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_iteration_2x1_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_row_iteration_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 3.0, 4.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 0 ).copied().collect();
  let exp = vec![ 1.0, 2.0, 3.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 1 ).copied().collect();
  let exp = vec![ 4.0, 5.0, 6.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
  let row_iter : Vec< f32 > = mat.lane_iter( 0, 2 ).copied().collect();
  let exp = vec![ 7.0, 8.0, 9.0 ];
  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
}

#[ test ]
fn test_valid_row_iteration_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_row_iteration_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_row_iteration_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_row_iteration_generic::< DescriptorOrderColumnMajor >();
}

fn test_valid_column_iteration_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 0, 0, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 1, 1, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };
  // 0x0 matrix
  let mat = Mat::< 0, 0, f32, D >::default();
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp : Vec< f32 > = vec![];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 1x1 matrix
  let mat = Mat::< 1, 1, f32, D >::default().set( [ 1.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 2x2 matrix
  let mat = Mat::< 2, 2, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 3.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0, 4.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  // 3x3 matrix
  let mat = Mat::< 3, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 0 ).copied().collect();
  let exp = vec![ 1.0, 4.0, 7.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 1 ).copied().collect();
  let exp = vec![ 2.0, 5.0, 8.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
  let col_iter : Vec< f32 > = mat.lane_iter( 1, 2 ).copied().collect();
  let exp = vec![ 3.0, 6.0, 9.0 ];
  assert_eq!( col_iter, exp, "Expected {:?}, got {:?}", exp, col_iter );
}

#[ test ]
fn test_valid_column_iteration_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_valid_column_iteration_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_valid_column_iteration_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_valid_column_iteration_generic::< DescriptorOrderColumnMajor >();
}

fn test_invalid_dimension_generic<D: the_module::mat::Descriptor + std::panic::RefUnwindSafe>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use std::panic;
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let result = panic::catch_unwind( ||
  {
    let _ = mat.lane_iter( 2, 0 ).collect::<Vec<_>>();
  });

  assert!( result.is_err(), "Expected panic, but no panic occurred" );
}

#[test]
fn test_invalid_dimension_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_invalid_dimension_generic::<DescriptorOrderRowMajor>();
}

#[test]
fn test_invalid_dimension_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_invalid_dimension_generic::<DescriptorOrderColumnMajor>();
}

fn test_negative_lane_index_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter( 0, usize::MAX ).collect();
}

#[test]
#[should_panic]
fn test_negative_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_negative_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_negative_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_negative_lane_index_generic::<DescriptorOrderColumnMajor>();
}

fn test_out_of_bounds_lane_index_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<2, 2, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingRef<Scalar = f32>,
{
  use the_module::{ Mat, IndexingRef, RawSliceMut };

  let mat = Mat::<2, 2, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0 ]);
  let _collected: Vec<_> = mat.lane_iter( 0, 2 ).collect();
  println!( "{_collected:?}" )
}

#[test]
#[should_panic]
fn test_out_of_bounds_lane_index_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_out_of_bounds_lane_index_generic::<DescriptorOrderRowMajor>();
}

#[test]
#[should_panic]
fn test_out_of_bounds_lane_index_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_out_of_bounds_lane_index_generic::<DescriptorOrderColumnMajor>();
}

fn test_lane_iter_mut_generic<D: the_module::mat::Descriptor>()
where
  the_module::Mat<3, 3, f32, D>: Default + the_module::RawSliceMut<Scalar = f32> + the_module::IndexingMut<Scalar = f32>,
{
  use the_module::{ Mat, RawSliceMut, IndexingMut };

  let mut mat = Mat::<3, 3, f32, D>::default().set([ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ]);

  // Modify the first row
  for value in mat.lane_iter_mut( 0, 0 )
  {
    *value *= 2.0;
  }

  let expected = Mat::<3, 3, f32, D>::default().set([ 2.0, 4.0, 6.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ]);
  assert_eq!( mat.raw_slice(), expected.raw_slice(), "Row modification failed" );
}

#[test]
fn test_lane_iter_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_lane_iter_mut_generic::<DescriptorOrderRowMajor>();
}

#[test]
fn test_lane_iter_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_lane_iter_mut_generic::<DescriptorOrderColumnMajor>();
}
