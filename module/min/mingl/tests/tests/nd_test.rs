#[ allow( unused_imports ) ]
#[ allow( clippy::used_underscore_binding ) ]
use super::*;

#[ test ]
fn basic()
{
  use the_module::math::nd;

  let trans_data : nd::Array< _, _ > = nd::array!
  [
    [
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
      [ 0.0, -0.2 ],
    ],
    [
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
      [ 0.0, -0.1 ],
    ],
    [
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
      [ 0.0, 0.0 ],
    ],
  ];

  // Transformation matrices
  let trans_data_array : [ f32 ; 18 ] =
  [

    1.0, 0.0,
    0.0, 1.0,
    0.0, -0.2,

    1.0, 0.0,
    0.0, 1.0,
    0.0, -0.1,

    1.0, 0.0,
    0.0, 1.0,
    0.0, 0.0,

  ];

  // You can use either flat array ( either static or dynamic )
  // or you can prefer nd::Array with it's flexible math.
  // The last one will save you much time on development and performance.
  assert_eq!( &trans_data_array[ .. ], trans_data.as_slice().unwrap() );

}