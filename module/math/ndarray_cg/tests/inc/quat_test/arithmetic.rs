use super::*;

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

fn test_multiply()
where

{
  use the_module::
  {
    Quat,
  };

  let q1 = Quat::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = Quat::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp = Quat::from( [ 0.2, 0.252, 0.052, 0.341 ] );
  assert_eq!( q1 / q2, exp, "Quaternion / Quaternion division mismatch" );

  let mut q1 = Quat::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = Quat::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  q1 /= q2;

  let exp = Quat::from( [ 0.2, 0.252, 0.052, 0.341 ] );
  assert_eq!( q1, exp, "Quaternion /= Quaternion division mismatch" );

}