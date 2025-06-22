use approx::assert_abs_diff_eq;

use super::*;

#[ test ]
fn test_multiply()
{
  use the_module::
  {
    QuatF32,
  };

  let q1 = QuatF32::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF32::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp = QuatF32::from( [ -13.0, 42.0, 31.0, 34.0 ] );
  assert_eq!( q2 * q1, exp, "Quaternion * Quaternion multiplication mismatch" );

  let exp = QuatF32::from( [ -7.0, 6.0, 53.0, 34.0 ] );
  assert_eq!( q1 * q2, exp, "Quaternion * Quaternion multiplication mismatch" );

  let exp =  QuatF32::from( [ 5.0, 10.0, 15.0, 20.0 ] );
  assert_eq!( q1 * 5.0, exp, "Quaternion * Scalar multiplication mismatch" );

  let mut q1 = QuatF32::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = QuatF32::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp =  QuatF32::from( [ -7.0, 6.0, 53.0, 34.0 ] );

  q1 *= q2;
  assert_eq!( q1, exp, "Quaternion *= Quaternion multiplication mismatch" );
}

#[ test ]
fn test_devide()
{
  use the_module::
  {
    QuatF32,
  };

  let q1 = QuatF32::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF32::from( [ -5.0, 1.0, 3.0, 10.0 ] ).normalize();

  let exp = QuatF32::from( [ 0.4242640687119285, 0.5342584568965025, 0.10999438818457405, 0.7228202652129152 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );

  let q1 = QuatF32::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF32::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF32::from( [ 0.013375757175498215, 0.010031817881623634, -0.006687878587749038, 0.999837848868489 ] );
  assert_abs_diff_eq!( q1 / q2, exp, );

}