use super::*;
use approx::assert_abs_diff_eq;
use ndarray_cg::{IndexingRef, QuatF32, QuatF64};
use the_module::
{ 
  Ix2,
  RawSliceMut,
  ScalarMut,
  ConstLayout,
  IndexingMut,
  Mat3,
  Mat2,
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

fn test_truncate_generic< Descriptor : mat::Descriptor >()
where 
  Mat3< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > + 
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = f32, Index = Ix2 >,
  Mat2< f32, Descriptor > : 
      RawSliceMut< Scalar = f32 > +
      IndexingRef< Scalar = f32 > +
      PartialEq
{
  let mat = Mat3::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 2.0, 3.0, 
    4.0, 5.0, 6.0,
    7.0, 8.0, 9.0,
  ]);

  let exp = Mat2::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 2.0,
    4.0, 5.0,
  ]);

  let got = mat.truncate();
  assert_eq!( got, exp );

  let mat = Mat3::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,  
    0.0, 0.0, 1.0,
  ]);

  let exp = Mat2::< f32, Descriptor >::from_row_major
  ([ 
    1.0, 0.0,
    0.0, 1.0,
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

fn test_from_quat_generic< Descriptor : mat::Descriptor >()
where 
  Descriptor : PartialEq,
  Mat3< f64, Descriptor > : 
      RawSliceMut< Scalar = f64 > + 
      IndexingRef< Scalar = f64, Index = Ix2 > +
      PartialEq,
{
  let q = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = Mat3::< f64, Descriptor >::from_column_major
  ([ 
    0.13333333333333353, 0.9333333333333332, -0.33333333333333326, 
    -0.6666666666666666, 0.3333333333333335, 0.6666666666666665, 
    0.7333333333333332, 0.13333333333333336, 0.6666666666666667,
  ]);

  assert_eq!( Mat3::< f64, Descriptor >::from_quat( q ), exp, " Mat3 from Quat mismatch" );

  let q = QuatF64::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();

  let exp = Mat3::< f64, Descriptor >::from_column_major
  ([ 
    -0.042253521126760285, -0.76056338028169, -0.6478873239436618, 
    -0.9295774647887323, 0.267605633802817, -0.2535211267605634, 
    0.36619718309859145, 0.5915492957746478, -0.7183098591549293,
  ]);

  assert_eq!( Mat3::< f64, Descriptor >::from_quat( q ), exp, " Mat3 from Quat mismatch" );

   let q = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();

  let exp = Mat3::< f64, Descriptor >::from_column_major
  ([ 
    0.7605633802816901, -0.14084507042253522, -0.6338028169014085, 
    -0.4225352112676056, 0.6338028169014085, -0.6478873239436619, 
    0.49295774647887325, 0.7605633802816901, 0.4225352112676056,
  ]);

  assert_abs_diff_eq!( Mat3::< f64, Descriptor >::from_quat( q ), exp );
}

#[ test ]
fn test_from_quat_row_major()
{
  test_from_quat_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_from_quat_column_major()
{
  test_from_quat_generic::< mat::DescriptorOrderColumnMajor >();
}