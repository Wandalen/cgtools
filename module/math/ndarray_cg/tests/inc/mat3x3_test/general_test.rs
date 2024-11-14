use super::*;
use the_module::
{ 
  Ix2,
  RawSliceMut,
  ScalarMut,
  ConstLayout,
  IndexingMut,
  Mat3,
  mat
};

fn test_determinant_generic< Descriptor : mat::Descriptor > ()
where 
  Mat3< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let exp = 0.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
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
  Mat3< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 > +
      PartialEq
{
  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let got = mat.inverse();
  assert!( got.is_none() );

  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, -1.0, 2.0, 4.0, 0.0, 6.0, 0.0, 1.0, -1.0 ] );
  let exp = Mat3::< f32, Descriptor >::from_row_major( [ 3.0, -0.5, 3.0, -2.0, 0.5, -1.0, -2.0, 0.5, -2.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let exp = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );
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