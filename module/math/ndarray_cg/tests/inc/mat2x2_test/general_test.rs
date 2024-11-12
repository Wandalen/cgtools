use super::*;

#[ test ]
fn test_determinant()
{
  let mat = the_module::F32x2x2::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );
  let exp = -2.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = the_module::F32x2x2::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let exp = 1.0;
  let got = mat.determinant();
  assert_eq!( got, exp );
}

#[ test ]
fn test_inverse()
{
  let mat = the_module::F32x2x2::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );
  let exp = the_module::F32x2x2::from_row_major( [ -2.0, 1.0, 1.5, -0.5 ] );

  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = the_module::F32x2x2::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let exp = the_module::F32x2x2::from_row_major( [ 1.0, 0.0, 0.0, 1.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = the_module::F32x2x2::from_row_major( [ 1.0, 1.0, 1.0, 1.0 ] );
  let got = mat.inverse();
  assert!( got.is_none() );
}