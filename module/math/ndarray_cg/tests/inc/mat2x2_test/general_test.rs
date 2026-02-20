use super::*;
use the_module::
{ 
  Ix2,
  RawSliceMut,
  ScalarMut,
  RawSlice,
  ConstLayout,
  IndexingMut,
  Mat2,
  Mat3,
  mat
};

fn test_determinant_generic< Descriptor : mat::Descriptor >()
where 
  Mat2< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );
  let exp = -2.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let exp = 1.0;
  let got = mat.determinant();
  assert_eq!( got, exp );
}

#[ test ]
fn test_determinant_row_major()
{
  test_determinant_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_determinant_column_major()
{
  test_determinant_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_inverse_generic< Descriptor : mat::Descriptor >()
where 
Mat2< f32, Descriptor > : 
    RawSliceMut< Scalar = f32 > +
    ScalarMut< Scalar = f32, Index = Ix2 > + 
    ConstLayout< Index = Ix2 > + 
    IndexingMut< Scalar = f32, Index = Ix2 > +
    PartialEq
{
  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );
  let exp = Mat2::< f32, Descriptor >::from_row_major( [ -2.0, 1.0, 1.5, -0.5 ] );

  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let exp = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 1.0, 1.0, 1.0 ] );
  let got = mat.inverse();
  assert!( got.is_none() );
}

#[ test ]
fn test_inverse_row_major()
{
  test_inverse_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_inverse_column_major()
{
  test_inverse_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_identity_generic< Descriptor : mat::Descriptor >()
where 
  Mat2< f32, Descriptor > : 
    RawSlice< Scalar = f32 > +
    RawSliceMut< Scalar = f32 >
{
  let exp = &[ 1.0, 0.0, 0.0, 1.0 ];

  let mat = the_module::mat2x2::identity::< f32 >();
  assert_eq!( mat.raw_slice(), exp );

  let mat = Mat2::< f32, Descriptor >::identity();
  assert_eq!( mat.raw_slice(), exp );
}

#[ test ]
fn test_identity_row_major()
{
  test_identity_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_identity_column_major()
{
  test_identity_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_to_homogenous_generic< Descriptor : mat::Descriptor >()
where
  Mat3< f32, Descriptor > :
    RawSliceMut< Scalar = f32 >,
  Mat2< f32, Descriptor > : 
    RawSlice< Scalar = f32 > +
    RawSliceMut< Scalar = f32 >
{
  let exp = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ];
  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let mat = mat.to_homogenous();
  assert_eq!( mat.to_array(), exp );

  let exp = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 1.0 ] );
  let mat = Mat2::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );
  let mat = mat.to_homogenous();
  assert_eq!( mat.to_array(), exp.to_array() );
}

#[ test ]
fn test_to_homogenous_row_major()
{
  test_to_homogenous_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_to_homogenous_column_major()
{
  test_to_homogenous_generic::< mat::DescriptorOrderColumnMajor >();
}