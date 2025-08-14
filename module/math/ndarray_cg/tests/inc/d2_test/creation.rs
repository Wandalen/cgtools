use super::*;

fn test_from_2darray_flattened()
{
  let data = 
  [
    [ 0.0, 1.0 ],
    [ 2.0, 3.0 ]
  ];

  let mat = the_module::Mat2::from_column_major( data.as_flattened() );

  let slice = mat.raw_slice();
  let exp = &[ 0.0, 1.0, 2.0, 3.0 ];
  assert_eq!( slice, exp, "Raw slice mismatch. Expected {:?}, got {:?}", exp, slice );
}