use ndarray_cg::{ F64x3, approx::assert_abs_diff_eq };

use super::*;

#[ test ]
fn test_multiply()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp = QuatF64::from( [ -13.0, 42.0, 31.0, 34.0 ] );
  assert_eq!( q2 * q1, exp, "Quaternion * Quaternion multiplication mismatch" );

  let exp = QuatF64::from( [ -7.0, 6.0, 53.0, 34.0 ] );
  assert_eq!( q1 * q2, exp, "Quaternion * Quaternion multiplication mismatch" );

  let exp =  QuatF64::from( [ 5.0, 10.0, 15.0, 20.0 ] );
  assert_eq!( q1 * 5.0, exp, "Quaternion * Scalar multiplication mismatch" );

  let mut q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp =  QuatF64::from( [ -7.0, 6.0, 53.0, 34.0 ] );

  q1 *= q2;
  assert_eq!( q1, exp, "Quaternion *= Quaternion multiplication mismatch" );
}

#[ test ]
fn test_devide()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ -5.0, 1.0, 3.0, 10.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.424_264_068_711_928_5, 0.534_258_456_896_502_5, 0.109_994_388_184_574_05, 0.722_820_265_212_915_2 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.013_375_757_175_498_215, 0.010_031_817_881_623_634, -0.006_687_878_587_749_038, 0.999_837_848_868_489 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );
}

#[ test ]
fn test_from_angle_x()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_angle_x( 1.0 );
  let exp = QuatF64::from( [ 0.479_425_538_604_203, 0.0, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_x( -1.0 );
  let exp = QuatF64::from( [ -0.479_425_538_604_203, 0.0, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_x( 256.0 );
  let exp = QuatF64::from( [ 0.721_037_710_501_731_6, -0.0, 0.0, -0.692_895_821_920_165_1 ] );
  assert_abs_diff_eq!( q, exp );
}

#[ test ]
fn test_from_angle_y()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_angle_y( 1.0 );
  let exp = QuatF64::from( [ 0.0, 0.479_425_538_604_203, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_y( -1.0 );
  let exp = QuatF64::from( [ 0.0, -0.479_425_538_604_203, 0.0, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_y( 256.0 );
  let exp = QuatF64::from( [ 0.0, 0.721_037_710_501_731_6, 0.0, -0.692_895_821_920_165_1 ] );
  assert_abs_diff_eq!( q, exp );
}

#[ test ]
fn test_from_angle_z()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_angle_z( 1.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.479_425_538_604_203, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_z( -1.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, -0.479_425_538_604_203, 0.877_582_561_890_372_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_z( 256.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.721_037_710_501_731_6, -0.692_895_821_920_165_1 ] );
  assert_abs_diff_eq!( q, exp );
}

#[ test ]
fn test_from_euler_xyz()
{
  use the_module::
  {
    QuatF64,
  };

  let q = QuatF64::from_euler_xyz( [ 1.0, 2.0, 3.0 ] );
  let exp = QuatF64::from( [ 0.754_933_801_264_452_5, -0.206_149_226_026_877_7, 0.501_509_096_403_722_1, -0.368_871_357_713_289_8 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_euler_xyz( [ 0.0, 0.0, 0.0 ] );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_euler_xyz( [ -23.0, 123.0, 0.53 ] );
  let exp = QuatF64::from( [ 0.076_980_141_411_157_5, -0.507_448_993_073_131_5, -0.790_928_849_502_003_3, 0.333_168_324_248_900_8 ] );
  assert_abs_diff_eq!( q, exp );
}

#[test]
fn test_to_euler_xyz()
{
  use ndarray_cg::QuatF64;

  let test_cases =
  [
    [ 1.0_f64.to_radians(), 2.0_f64.to_radians(), 3.0_f64.to_radians() ],
    [ 0.0, 0.0, 0.0 ],
    [ 0.01, 0.01, 0.01 ],
    [ 0.0, 90.0_f64.to_radians(), 0.0 ],
  ];

  for input in test_cases
  {
    let q_in = QuatF64::from_euler_xyz( input ).normalize();

    let result = q_in.to_euler_xyz();

    assert_abs_diff_eq!
    (
      result,
      F64x3::from_array( input ),
      epsilon = 1e-1
    );
  }
}

#[ test ]
fn test_to_euler_xyz_from_raw_quat()
{
  use ndarray_cg::QuatF64;

  let test_cases =
  [
    ( [ 0.009, 0.017, 0.026, 0.999 ], [ 1.0_f64.to_radians(), 2.0_f64.to_radians(), 3.0_f64.to_radians() ] ),
    ( [ 0.0, 0.0, 0.0, 1.0 ], [ 0.0, 0.0, 0.0 ] ),
    ( [ 0.0, 0.0, 0.0, 1.0 ], [ 0.01_f64.to_radians(), 0.01_f64.to_radians(), 0.01_f64.to_radians() ] ),
    ( [ 0.707, 0.0, 0.707, 0.0 ], [ 0.0, -90.0_f64.to_radians(), 0.0 ] ),
  ];

  for ( raw_quat, expected ) in test_cases
  {
    let q_in = QuatF64::from( raw_quat );

    let result = q_in.to_euler_xyz();

    println!("Expected: {:?}", [ expected[ 0 ].to_degrees(), expected[ 1 ].to_degrees(), expected[ 2 ].to_degrees() ]);

    assert_abs_diff_eq!
    (
      result,
      F64x3::from_array( expected ),
      epsilon = 1e-1
    );
  }
}
