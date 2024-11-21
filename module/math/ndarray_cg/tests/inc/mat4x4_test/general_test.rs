use super::*;

use the_module::
{ 
  Ix2,
  RawSliceMut,
  ScalarMut,
  ConstLayout,
  IndexingMut,
  Mat3,
  Mat4,
  mat
};

fn test_determinant_generic< Descriptor : mat::Descriptor >()
where 
  Mat4< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 >,
  Mat3< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 2.0, 3.0, 4.0, 
    5.0, 6.0, 7.0, 8.0, 
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0 
  ]);

  let exp = 0.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);
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
  Mat4< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 > +
      PartialEq,
  Mat3< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > + 
      ConstLayout< Index = Ix2 > + 
      IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = the_module::F32x4x4::from_row_major
  ([ 
    1.0, 2.0, 3.0, 4.0, 
    5.0, 6.0, 7.0, 8.0, 
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0 
  ]);
  let got = mat.inverse();
  assert!( got.is_none() );

  let mat = the_module::F32x4x4::from_row_major
  ([ 
    1.0, 3.0, 5.0, 9.0, 
    1.0, 3.0, 1.0, 7.0, 
    4.0, 3.0, 9.0, 7.0,
    5.0, 2.0, 0.0, 9.0 
  ]);

  let exp = the_module::F32x4x4::from_row_major
  ([ 
    -13.0 / 47.0,  2.0 / 47.0,   7.0 / 47.0,    6.0 / 47.0, 
    -5.0 / 8.0,    7.0 / 8.0,    1.0 / 4.0,    -1.0 / 4.0, 
    39.0 / 376.0, -53.0 / 376.0, 13.0 / 188.0, -9.0 / 188.0,
    55.0 / 188.0, -41.0 / 188.0, -13.0 / 94.0,  9.0 / 94.0 
  ]);

  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = the_module::F32x4x4::from_row_major
  ([ 
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);
  let got = mat.inverse().unwrap();
  assert_eq!( got, mat );
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