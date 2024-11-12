use super::*;

#[ test ]
fn test_determinant()
{
  let mat = the_module::F32x3x3::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let exp = 0.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = the_module::F32x3x3::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let exp = 1.0;
  let got = mat.determinant();
  assert_eq!( got, exp );
}

#[ test ]
fn test_inverse()
{
  let mat = the_module::F32x3x3::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let got = mat.inverse();
  assert!( got.is_none() );

  let mat = the_module::F32x3x3::from_row_major( [ 1.0, -1.0, 2.0, 4.0, 0.0, 6.0, 0.0, 1.0, -1.0 ] );
  let exp = the_module::F32x3x3::from_row_major( [ 3.0, -0.5, 3.0, -2.0, 0.5, -1.0, -2.0, 0.5, -2.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = the_module::F32x3x3::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let exp = the_module::F32x3x3::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );
}