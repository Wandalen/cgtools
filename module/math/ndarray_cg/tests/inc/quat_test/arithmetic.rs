use approx::assert_abs_diff_eq;

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

  let exp = QuatF64::from( [ 0.4242640687119285, 0.5342584568965025, 0.10999438818457405, 0.7228202652129152 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.013375757175498215, 0.010031817881623634, -0.006687878587749038, 0.999837848868489 ] );
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
  let exp = QuatF64::from( [ 0.479425538604203, 0.0, 0.0, 0.8775825618903728 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_x( -1.0 );
  let exp = QuatF64::from( [ -0.479425538604203, 0.0, 0.0, 0.8775825618903728 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_x( 256.0 );
  let exp = QuatF64::from( [ 0.7210377105017316, -0.0, 0.0, -0.6928958219201651 ] );
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
  let exp = QuatF64::from( [ 0.0, 0.479425538604203, 0.0, 0.8775825618903728 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_y( -1.0 );
  let exp = QuatF64::from( [ 0.0, -0.479425538604203, 0.0, 0.8775825618903728 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_y( 256.0 );
  let exp = QuatF64::from( [ 0.0, 0.7210377105017316, 0.0, -0.6928958219201651 ] );
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
  let exp = QuatF64::from( [ 0.0, 0.0, 0.479425538604203, 0.8775825618903728 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_z( -1.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, -0.479425538604203, 0.8775825618903728 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_angle_z( 256.0 );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.7210377105017316, -0.6928958219201651 ] );
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
  let exp = QuatF64::from( [ 0.7549338012644525, -0.2061492260268777, 0.5015090964037221, -0.3688713577132898 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_euler_xyz( [ 0.0, 0.0, 0.0 ] );
  let exp = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  assert_abs_diff_eq!( q, exp );

  let q = QuatF64::from_euler_xyz( [ -23.0, 123.0, 0.53 ] );
  let exp = QuatF64::from( [ 0.0769801414111575, -0.5074489930731315, -0.7909288495020033, 0.3331683242489008 ] );
  assert_abs_diff_eq!( q, exp );
}