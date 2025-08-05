use ndarray_cg::approx::{ assert_abs_diff_eq, assert_relative_eq };

use super::*;

#[ test ]
fn test_slerp()
{
  use the_module::
  {
    QuatF64,
  };

  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();

  let exp = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  assert_abs_diff_eq!( q1.slerp( &q2, 0.0 ), exp );

  let exp =  QuatF64::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();
  assert_abs_diff_eq!( q1.slerp( &q2, 1.0 ), exp );

  let exp = QuatF64::from( [ -0.07189765816207114, 0.5401439887921695, 0.46824633063009835, 0.6955721184564136 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.3 ), exp );

  let exp = QuatF64::from( [ -0.23905007006563106, 0.626821944003501, 0.38777187393787, 0.6321252156811978 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.5 ), exp );

  let exp = QuatF64::from( [ -0.5332219143045811, 0.7111022022191557, 0.17788028791457453, 0.4223347620973825 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.9 ), exp );


  let q1 = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();
  let q2 = QuatF64::from( [ 0.9, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = QuatF64::from( [ 0.18080329575292692, 0.3652698894950247, 0.5479048342425371, 0.7305397789900494 ] );

  assert_abs_diff_eq!( q1.slerp( &q2, 0.1 ), exp );

  let exp = QuatF64::from( [ 0.17371392923604712, 0.36574411080923475, 0.548616166213852, 0.7314882216184695 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.5 ), exp );

  let exp = QuatF64::from( [ 0.1657276357739934, 0.3662549236088048, 0.549382385413207, 0.7325098472176096 ] );
  assert_abs_diff_eq!( q1.slerp( &q2, 0.95 ), exp );

}
