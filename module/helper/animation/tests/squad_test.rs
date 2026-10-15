//! Integration tests for the `Squad` quaternion-interpolation easing function.
#[ cfg( test ) ]
mod tests
{
  use animation::easing::{ base::EasingFunction, Squad };
  use mingl::{ QuatF32, F32x3 };
  use std::f32::consts::PI;

  fn assert_f_eq( first : f32, second : f32, eps : f32 )
  {
    assert!( second - eps < first && first < second + eps, "{first} != {second} (eps {eps})" );
  }

  fn assert_quat_eq( first : QuatF32, second : QuatF32, eps : f32 )
  {
    assert_f_eq( first.x(), second.x(), eps );
    assert_f_eq( first.y(), second.y(), eps );
    assert_f_eq( first.z(), second.z(), eps );
    assert_f_eq( first.w(), second.w(), eps );
  }

  #[ test ]
  fn test_squad_boundaries()
  {
    let start = QuatF32::from_axis_angle( F32x3::new( 0.0, 1.0, 0.0 ), PI / 4.0 );
    let end = QuatF32::from_axis_angle( F32x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 );
    let out_tangent = QuatF32::from_axis_angle( F32x3::new( 1.0, 0.0, 0.0 ), PI / 6.0 );
    let in_tangent = QuatF32::from_axis_angle( F32x3::new( 1.0, 0.0, 0.0 ), PI / 3.0 );

    let squad = Squad::new( in_tangent, out_tangent );

    assert_quat_eq( squad.apply( start, end, 0.0 ), start, 0.0001 );
    assert_quat_eq( squad.apply( start, end, 1.0 ), end, 0.0001 );
  }

  // test_kind: bug_reproducer(BUG-149)
  /// ## Root Cause
  /// `Squad::apply` inserted an extraneous 1/3-blend step (`b_start = start.slerp(out_tangent,
  /// 1/3)`, `b_end = end.slerp(in_tangent, 1/3)`) before the second slerp, instead of slerping
  /// `out_tangent`/`in_tangent` directly against each other. The correct SQUAD formula
  /// (Shoemake's `Definition 17`, confirmed independently via this crate's own cited ROBOOP C++
  /// reference and MIT-thesis sources) is
  /// `Slerp( Slerp(start,end,t), Slerp(out_tangent,in_tangent,t), 2t(1-t) )` -- the pre-computed
  /// tangent quaternions are used directly in the inner slerp, with no further blending toward
  /// the endpoints.
  /// ## Why Not Caught
  /// `Squad` had zero test coverage of any kind prior to this fix.
  /// ## Fix Applied
  /// Replaced the `b_start`/`b_end` intermediate-blend computation with a direct
  /// `self.out_tangent.slerp( &self.in_tangent, time_e )` call. See `easing/squad.rs`.
  /// ## Prevention
  /// Added `test_squad_boundaries` (above) and this test. The boundary test alone is
  /// insufficient -- see Pitfall -- so this test additionally pins a mid-curve value against a
  /// reference independently composed via the confirmed-correct formula.
  /// ## Pitfall
  /// The outer `2t(1-t)` coefficient is exactly `0` at `time == 0.0` and `time == 1.0`, so
  /// `apply` returns precisely `start`/`end` at both boundaries under BOTH the buggy and fixed
  /// formula -- a boundary-only test (the style already used elsewhere in this crate, e.g.
  /// `test_cubic_boundaries_and_properties`) can never catch this class of defect; only a pinned
  /// mid-curve value can.
  #[ test ]
  fn test_squad_matches_reference_formula_mid_curve()
  {
    let start = QuatF32::default();
    let end = QuatF32::from_axis_angle( F32x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 );
    let out_tangent = QuatF32::from_axis_angle( F32x3::new( 1.0, 0.0, 0.0 ), PI / 6.0 );
    let in_tangent = QuatF32::from_axis_angle( F32x3::new( 0.0, 1.0, 0.0 ), PI / 3.0 );
    let time = 0.4_f64;

    let squad = Squad::new( in_tangent, out_tangent );
    let actual = squad.apply( start, end, time );

    let time_e = time as f32;
    let slerp1 = start.slerp( &end, time_e );
    let slerp2 = out_tangent.slerp( &in_tangent, time_e );
    let coeff = ( 2.0 * time * ( 1.0 - time ) ) as f32;
    let expected = slerp1.slerp( &slerp2, coeff );

    assert_quat_eq( actual, expected, 0.0001 );
  }
}
