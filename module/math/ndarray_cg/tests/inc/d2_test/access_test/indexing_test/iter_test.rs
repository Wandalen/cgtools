use super::*;
// use super::hset;
use test_tools::hset; // xxx : remove it later
use the_module::nd::Dim;

fn test_iter_unstable_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::IndexingRef< Scalar = f32 > + the_module::RawSliceMut,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : PartialEq< f32 >,
{
  use the_module::{ Mat, RawSliceMut, IndexingRef };

  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let mut iter : Vec<_> = mat.iter_unstable().copied().collect();
  let mut exp : Vec<_> = vec![ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ];

  iter.sort_by( | a, b | a.partial_cmp( b ).unwrap() );
  exp.sort_by( | a, b | a.partial_cmp( b ).unwrap() );

  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_unstable_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_unstable_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_unstable_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_unstable_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_indexed_unstable_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::IndexingRef< Scalar = f32 > + the_module::RawSliceMut,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : PartialEq< f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::{ nd, Mat, RawSliceMut, IndexingRef };

  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let mut iter : Vec<_> = mat.iter_indexed_unstable().map( | ( idx, &val ) | ( idx, val ) ).collect();
  let mut exp = 
  vec!
  [
    ( nd::Ix2( 0, 0 ), 1.0 ),
    ( nd::Ix2( 0, 1 ), 2.0 ),
    ( nd::Ix2( 0, 2 ), 3.0 ),
    ( nd::Ix2( 1, 0 ), 4.0 ),
    ( nd::Ix2( 1, 1 ), 5.0 ),
    ( nd::Ix2( 1, 2 ), 6.0 ),
  ];

  iter.sort_by( | a, b | a.1.partial_cmp( &b.1 ).unwrap() );
  exp.sort_by( | a, b | a.1.partial_cmp( &b.1 ).unwrap() );

  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_indexed_unstable_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_indexed_unstable_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_indexed_unstable_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_indexed_unstable_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_lsfirst_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::IndexingRef< Scalar = f32 > + the_module::RawSliceMut,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : PartialEq< f32 >,
{
  use the_module::
  {
    Mat,
    RawSliceMut,
    IndexingRef,
  };
  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let iter : Vec< _ > = mat.iter_lsfirst().copied().collect();
  let exp = vec![ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ];
  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_lsfirst_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_lsfirst_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_lsfirst_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_lsfirst_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_msfirst_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::IndexingRef< Scalar = f32 > + the_module::RawSliceMut,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : PartialEq< f32 >,
{
  use the_module::
  {
    Mat,
    RawSliceMut,
    IndexingRef,
  };
  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let iter : Vec< _ > = mat.iter_msfirst().copied().collect();
  let exp = vec![ 1.0, 4.0, 2.0, 5.0, 3.0, 6.0 ];
  assert_eq!( iter, exp, "Expected {:?}, got {:?}", exp, iter );
}

#[ test ]
fn test_iter_msfirst_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_msfirst_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_msfirst_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_msfirst_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_indexed_lsfirst_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::IndexingRef< Scalar = f32 > + the_module::RawSliceMut,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : PartialEq< f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::
  {
    nd,
    Mat,
    RawSliceMut,
    IndexingRef,
  };
  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let indexed_iter : Vec< _ > = mat.iter_indexed_lsfirst().map( | ( idx, &val ) | ( idx, val ) ).collect();
  let exp =
  vec!
  [
    ( nd::Ix2( 0, 0 ), 1.0 ),
    ( nd::Ix2( 0, 1 ), 2.0 ),
    ( nd::Ix2( 0, 2 ), 3.0 ),
    ( nd::Ix2( 1, 0 ), 4.0 ),
    ( nd::Ix2( 1, 1 ), 5.0 ),
    ( nd::Ix2( 1, 2 ), 6.0 ),
  ];
  assert_eq!( &indexed_iter, &exp, "Expected {:?}, got {:?}", exp, indexed_iter );
}

#[ test ]
fn test_iter_indexed_lsfirst_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_indexed_lsfirst_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_indexed_lsfirst_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_indexed_lsfirst_generic::< DescriptorOrderColumnMajor >();
}

fn test_iter_indexed_msfirst_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 3, f32, D > : Default + the_module::IndexingRef< Scalar = f32 > + the_module::RawSliceMut,
  < the_module::Mat< 2, 3, f32, D > as the_module::Collection >::Scalar : PartialEq< f32 >,
  the_module::Mat< 2, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::
  {
    nd,
    Mat,
    RawSliceMut,
    IndexingRef,
  };
  let mat = Mat::< 2, 3, f32, D >::default().set( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let indexed_iter : Vec< _ > = mat.iter_indexed_msfirst().map( | ( idx, &val ) | ( idx, val ) ).collect();
  let exp = 
  vec!
  [
    ( nd::Ix2( 0, 0 ), 1.0 ),
    ( nd::Ix2( 1, 0 ), 4.0 ),
    ( nd::Ix2( 0, 1 ), 2.0 ),
    ( nd::Ix2( 1, 1 ), 5.0 ),
    ( nd::Ix2( 0, 2 ), 3.0 ),
    ( nd::Ix2( 1, 2 ), 6.0 ),
  ];
  assert_eq!( &indexed_iter, &exp, "Expected {:?}, got {:?}", exp, indexed_iter );
}

#[ test ]
fn test_iter_indexed_msfirst_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_iter_indexed_msfirst_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_iter_indexed_msfirst_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_iter_indexed_msfirst_generic::< DescriptorOrderColumnMajor >();
}
