use super::*;

#[ test ]
fn test_determinant()
{
  let mat = the_module::F32x4x4::from_row_major
  ([ 
    1.0, 2.0, 3.0, 4.0, 
    5.0, 6.0, 7.0, 8.0, 
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0 
  ]);

  let exp = 0.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = the_module::F32x4x4::from_row_major
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
fn test_inverse()
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