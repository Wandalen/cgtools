use super::*;

fn test_iter_unstable_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : std::ops::MulAssign< f32 > + PartialOrd + PartialEq< f32 > + std::fmt::Debug + Copy,
{
  use the_module::{ Mat, RawSliceMut, IndexingMut, IndexingRef };
  let mut mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  for value in mat.iter_unstable_mut()
  {
    *value *= 2.0;
  }
  let mut iter : Vec<_> = mat.iter_unstable().collect();
  let mut exp = vec![ &2.0, &4.0, &6.0, &8.0, &10.0, &12.0 ];
  iter.sort_by( | a, b | a.partial_cmp( b ).unwrap() );
  exp.sort_by( | a, b | a.partial_cmp( b ).unwrap() );
  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_unstable_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_unstable_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_unstable_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_unstable_mut_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_indexed_unstable_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 > + the_module::IndexingRef< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::{ nd, Mat, RawSliceMut, IndexingMut };

  let mut mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );

  for ( _, value ) in mat.iter_indexed_unstable_mut()
  {
    *value *= 2.0;
  }
  let mut iter : Vec<_> = mat.iter_indexed_unstable().map( | ( idx, &val ) | ( idx, val ) ).collect();
  let mut exp = 
  vec!
  [
    ( nd::Ix2( 0, 0 ), 2.0 ),
    ( nd::Ix2( 0, 1 ), 4.0 ),
    ( nd::Ix2( 0, 2 ), 6.0 ),
    ( nd::Ix2( 1, 0 ), 8.0 ),
    ( nd::Ix2( 1, 1 ), 10.0 ),
    ( nd::Ix2( 1, 2 ), 12.0 ),
  ];
  iter.sort_by( | a, b | a.1.partial_cmp( &b.1 ).unwrap() );
  exp.sort_by( | a, b | a.1.partial_cmp( &b.1 ).unwrap() );
  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_indexed_unstable_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_indexed_unstable_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_indexed_unstable_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_indexed_unstable_mut_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_lsfirst_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 >,
{
  use the_module::{ Mat, RawSliceMut, IndexingMut };
  let mut mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  for value in mat.iter_lsfirst_mut()
  {
    *value *= 2.0;
  }
  let iter : Vec< f32 > = mat.iter_lsfirst().copied().collect(); // Convert references to values
  let exp : Vec< f32 > = vec![ 2.0, 4.0, 6.0, 8.0, 10.0, 12.0 ];
  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_lsfirst_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_lsfirst_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_lsfirst_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_lsfirst_mut_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_msfirst_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 >,
{
  use the_module::{ Mat, RawSliceMut, IndexingMut };
  let mut mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  for value in mat.iter_msfirst_mut()
  {
    *value *= 2.0;
  }
  let iter : Vec< f32 > = mat.iter_msfirst().copied().collect(); // Convert references to values
  let exp : Vec< f32 > =  vec![ 2.0, 8.0, 4.0, 10.0, 6.0, 12.0 ];
  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_msfirst_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_msfirst_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_msfirst_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_msfirst_mut_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_indexed_lsfirst_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::{ nd, Mat, RawSliceMut, IndexingMut };
  let mut mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  for ( _, value ) in mat.iter_indexed_lsfirst_mut()
  {
    *value *= 2.0;
  }
  let indexed_iter : Vec<_> = mat.iter_indexed_lsfirst().map( | ( idx, &val ) | ( idx, val ) ).collect();
  let exp = 
  vec!
  [
    ( nd::Ix2( 0, 0 ), 2.0 ),
    ( nd::Ix2( 0, 1 ), 4.0 ),
    ( nd::Ix2( 0, 2 ), 6.0 ),
    ( nd::Ix2( 1, 0 ), 8.0 ),
    ( nd::Ix2( 1, 1 ), 10.0 ),
    ( nd::Ix2( 1, 2 ), 12.0 ),
  ];
  assert_eq!( indexed_iter, exp, "Expected {:?}, got {:?}", exp, indexed_iter );
}

#[ test ]
fn test_iter_indexed_lsfirst_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_indexed_lsfirst_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_indexed_lsfirst_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_indexed_lsfirst_mut_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_indexed_msfirst_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::RawSliceMut< Scalar = f32 > + the_module::IndexingMut< Scalar = f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::{ nd, Mat, RawSliceMut, IndexingMut };
  let mut mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  for ( _, value ) in mat.iter_indexed_msfirst_mut()
  {
    *value *= 2.0;
  }
  let indexed_iter : Vec<_> = mat.iter_indexed_msfirst().map( | ( idx, &val ) | ( idx, val ) ).collect();
  let exp = 
  vec!
  [
    ( nd::Ix2( 0, 0 ), 2.0 ),
    ( nd::Ix2( 1, 0 ), 8.0 ),
    ( nd::Ix2( 0, 1 ), 4.0 ),
    ( nd::Ix2( 1, 1 ), 10.0 ),
    ( nd::Ix2( 0, 2 ), 6.0 ),
    ( nd::Ix2( 1, 2 ), 12.0 ),
  ];
  assert_eq!( indexed_iter, exp, "Expected {:?}, got {:?}", exp, indexed_iter );
}

#[ test ]
fn test_iter_indexed_msfirst_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_indexed_msfirst_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_indexed_msfirst_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_indexed_msfirst_mut_generic::< DescriptorOrderColumnMajor >();
}
