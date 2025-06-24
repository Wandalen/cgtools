use super::*;

use approx::{assert_abs_diff_eq, assert_relative_eq};
use ndarray_cg::{IndexingRef, QuatF64};
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
  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 2.0, 3.0, 4.0, 
    5.0, 6.0, 7.0, 8.0, 
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0 
  ]);
  let got = mat.inverse();
  assert!( got.is_none() );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 3.0, 5.0, 9.0, 
    1.0, 3.0, 1.0, 7.0, 
    4.0, 3.0, 9.0, 7.0,
    5.0, 2.0, 0.0, 9.0 
  ]);

  let exp = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    -13.0 / 47.0,  2.0 / 47.0,   7.0 / 47.0,    6.0 / 47.0, 
    -5.0 / 8.0,    7.0 / 8.0,    1.0 / 4.0,    -1.0 / 4.0, 
    39.0 / 376.0, -53.0 / 376.0, 13.0 / 188.0, -9.0 / 188.0,
    55.0 / 188.0, -41.0 / 188.0, -13.0 / 94.0,  9.0 / 94.0 
  ]);

  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = Mat4::< f32, Descriptor >::from_row_major
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

fn test_truncate_generic< Descriptor : mat::Descriptor >()
where 
  Mat4< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 >,
  Mat3< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      IndexingRef< Scalar = f32, Index = Ix2 > +
      PartialEq
{
  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 2.0, 3.0, 4.0, 
    5.0, 6.0, 7.0, 8.0, 
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0 
  ]);

  let exp = Mat3::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 2.0, 3.0, 
    5.0, 6.0, 7.0, 
    9.0, 10.0, 11.0,
  ]);

  let got = mat.truncate();
  assert_eq!( got, exp );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);

  let exp = Mat3::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 0.0, 0.0, 
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);

  let got = mat.truncate();
  assert_eq!( got, exp );
}

#[ test ]
fn test_truncate_row_major()
{
  test_truncate_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_truncate_column_major()
{
  test_truncate_generic::< mat::DescriptorOrderColumnMajor >();
}


fn test_from_scale_rotation_translation_generic< Descriptor : mat::Descriptor >()
where 
  Descriptor : PartialEq,
  Mat4< f64, Descriptor > : 
      ScalarMut< Scalar = f64 > + 
      RawSliceMut< Scalar = f64 > +
      IndexingMut< Scalar = f64, Index = Ix2 >
{
  let s = [ 1.0, 2.0, 3.0 ];
  let r = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ).normalize();
  let t = [ 0.0, 0.0, 0.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([ 
    1.0, 0.0, 0.0, 0.0, 
    0.0, 2.0, 0.0, 0.0, 
    0.0, 0.0, 3.0, 0.0, 
    0.0, 0.0, 0.0, 1.0 
  ]);

  assert_abs_diff_eq!( got, exp );

  let s = [ 1.0, 1.0, 1.0 ];
  let r = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();
  let t = [ 0.0, 0.0, 0.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([ 
    0.7605633802816901, -0.14084507042253522, -0.6338028169014085, 0.0, 
    -0.4225352112676056, 0.6338028169014085, -0.6478873239436619, 0.0, 
    0.49295774647887325, 0.7605633802816901, 0.4225352112676056, 0.0, 
    0.0, 0.0, 0.0, 1.0 
  ]);

  assert_abs_diff_eq!( got, exp );

  let s = [ 1.0, 1.0, 1.0 ];
  let r = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ).normalize();
  let t = [ 1.0, -10.0, 30.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([ 
    1.0, 0.0, 0.0, 0.0, 
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0, 
    1.0, -10.0, 30.0, 1.0 
  ]);

  assert_abs_diff_eq!( got, exp );

  let s = [ 1.0, 2.0, 3.0 ];
  let r = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();
  let t = [ 1.0, -10.0, 30.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([ 
    0.7605633802816901, -0.14084507042253522, -0.6338028169014085, 0.0, 
    -0.8450704225352113, 1.2676056338028168, -1.2957746478873242, 
    0.0, 1.4788732394366197, 2.281690140845071, 1.2676056338028165, 0.0, 
    1.0, -10.0, 30.0, 1.0 
  ]);

  assert_abs_diff_eq!( got, exp );
}

#[ test ]
fn test_from_scale_rotation_translation_row_major()
{
  test_from_scale_rotation_translation_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_from_scale_rotation_translation_column_major()
{
  test_from_scale_rotation_translation_generic::< mat::DescriptorOrderColumnMajor >();
}