use super::*;

fn test_multiply< D : the_module::mat::Descriptor >()
where

{
  use the_module::
  {
    Quat,
  };

  let q1 = Quat::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  let q2 = Quat::from( [ -5.0, 1.0, 3.0, 10.0 ] );

  let exp = Quat::from( [ -12.0, 9.0, 24.0, -57.0 ] );
  assert_eq!( q2 * q1, exp, "QuaternionxQuaternion multiplication mismatch" );

  let exp =  Quat::from( [ 5.0, 10.0, 15.0, 20.0 ] );
  assert_eq!( q1 * 5.0, exp, "QuaternionxScalar multiplication mismatch" );
}