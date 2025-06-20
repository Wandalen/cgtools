use super::*;

fn test_slerp()
{
  use the_module::
  {
    QuatF32,
  };

  let q1 = QuatF32::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF32::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();

  let exp = QuatF32::from( [ 1.0, 2.0, 3.0, 4.0 ] );
  assert_eq!( q1.slerp( &q2, 0.0 ), "Quaternion slerp 0.0 mismatch" );

  let exp =  QuatF32::from( [ -5.0, 6.0, 1.0, 3.0 ] );
  assert_eq!( q1.slerp( &q2, 1.0 ), exp, "Quaternion slerp 1.0 mismatch" );

  let exp = QuatF32::from( [ -0.07189765816207114, 0.5401439887921695, 0.46824633063009835, 0.6955721184564136 ] );
  assert_eq!( q1.slerp( &q2, 0.3 ), exp, "Quaternion slerp 0.3 mismatch" );

  let exp = QuatF32::from( [ -0.23905007006563106, 0.626821944003501, 0.38777187393787, 0.6321252156811978 ] );
  assert_eq!( q1.slerp( &q2, 0.5 ), exp, "Quaternion slerp 0.5 mismatch" );

  let exp = QuatF32::from( [ -0.5332219143045811, 0.7111022022191557, 0.17788028791457453, 0.4223347620973825 ] );
  assert_eq!( q1.slerp( &q2, 0.9 ), exp, "Quaternion slerp 0.9 mismatch" );

}