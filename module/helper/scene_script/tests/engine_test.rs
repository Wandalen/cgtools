//! Smoke tests for the `scene_script` Rhai engine builder.

use ndarray_cg::{ F32x1, F32x2, F32x3, F32x4, F64x1, F64x2, F64x3, F64x4 };
use scene_script::engine_build;

#[ test ]
fn f32x1_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F32x1 = engine.eval( "f32x1(1.0) + f32x1(3.0)" ).unwrap();
  assert_eq!( result, F32x1::new( 4.0 ) );
}

#[ test ]
fn f64x1_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F64x1 = engine.eval( "f64x1(1.0) + f64x1(3.0)" ).unwrap();
  assert_eq!( result, F64x1::new( 4.0 ) );
}

#[ test ]
fn f32x2_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F32x2 = engine.eval( "f32x2(1.0, 2.0) + f32x2(3.0, 4.0)" ).unwrap();
  assert_eq!( result, F32x2::new( 4.0, 6.0 ) );
}

#[ test ]
fn f64x2_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F64x2 = engine.eval( "f64x2(1.0, 2.0) + f64x2(3.0, 4.0)" ).unwrap();
  assert_eq!( result, F64x2::new( 4.0, 6.0 ) );
}

#[ test ]
fn tween_f32x1_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F32x1 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F32x1::new( 10.0 ) );
}

#[ test ]
fn tween_f64x1_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F64x1 = engine.eval
  (
    "let t = tween( f64x1(0.0), f64x1(10.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F64x1::new( 10.0 ) );
}

#[ test ]
fn tween_f32x2_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F32x2 = engine.eval
  (
    "let t = tween( f32x2(0.0, 0.0), f32x2(10.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F32x2::new( 10.0, 0.0 ) );
}

#[ test ]
fn tween_f64x2_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F64x2 = engine.eval
  (
    "let t = tween( f64x2(0.0, 0.0), f64x2(10.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F64x2::new( 10.0, 0.0 ) );
}

#[ test ]
fn f32x3_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F32x3 = engine.eval( "f32x3(1.0, 2.0, 3.0) + f32x3(4.0, 5.0, 6.0)" ).unwrap();
  assert_eq!( result, F32x3::new( 5.0, 7.0, 9.0 ) );
}

#[ test ]
fn f64x3_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F64x3 = engine.eval( "f64x3(1.0, 2.0, 3.0) + f64x3(4.0, 5.0, 6.0)" ).unwrap();
  assert_eq!( result, F64x3::new( 5.0, 7.0, 9.0 ) );
}

#[ test ]
fn f32x4_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F32x4 = engine.eval( "f32x4(1.0, 2.0, 3.0, 4.0) + f32x4(5.0, 6.0, 7.0, 8.0)" ).unwrap();
  assert_eq!( result, F32x4::new( 6.0, 8.0, 10.0, 12.0 ) );
}

#[ test ]
fn f64x4_arithmetic_roundtrip()
{
  let engine = engine_build();
  let result : F64x4 = engine.eval( "f64x4(1.0, 2.0, 3.0, 4.0) + f64x4(5.0, 6.0, 7.0, 8.0)" ).unwrap();
  assert_eq!( result, F64x4::new( 6.0, 8.0, 10.0, 12.0 ) );
}

#[ test ]
fn tween_f32x3_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F32x3 = engine.eval
  (
    "let t = tween( f32x3(0.0, 0.0, 0.0), f32x3(10.0, 0.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F32x3::new( 10.0, 0.0, 0.0 ) );
}

#[ test ]
fn tween_f64x3_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F64x3 = engine.eval
  (
    "let t = tween( f64x3(0.0, 0.0, 0.0), f64x3(10.0, 0.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F64x3::new( 10.0, 0.0, 0.0 ) );
}

#[ test ]
fn tween_f32x4_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F32x4 = engine.eval
  (
    "let t = tween( f32x4(0.0, 0.0, 0.0, 0.0), f32x4(10.0, 0.0, 0.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F32x4::new( 10.0, 0.0, 0.0, 0.0 ) );
}

#[ test ]
fn tween_f64x4_updates_toward_end_value()
{
  let engine = engine_build();
  let value : F64x4 = engine.eval
  (
    "let t = tween( f64x4(0.0, 0.0, 0.0, 0.0), f64x4(10.0, 0.0, 0.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F64x4::new( 10.0, 0.0, 0.0, 0.0 ) );
}

#[ test ]
fn f32x2_and_f64x2_are_distinct_types_not_interchangeable()
{
  let engine = engine_build();
  let err = engine.eval::< F64x2 >( "f32x2(1.0, 2.0)" ).unwrap_err();
  assert!( err.to_string().contains( "type" ), "expected a type-mismatch error, got: {err}" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertion checks exact expected value; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
fn vector_dot_product_computes_scalar()
{
  let engine = engine_build();
  let result : f64 = engine.eval( "f32x2(3.0, 4.0).dot( f32x2(2.0, 1.0) )" ).unwrap();
  assert_eq!( result, 10.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
fn vector_magnitude_and_magnitude_squared_match_pythagorean_length()
{
  let engine = engine_build();
  let mag : f64 = engine.eval( "f32x2(3.0, 4.0).mag()" ).unwrap();
  let mag2 : f64 = engine.eval( "f32x2(3.0, 4.0).mag2()" ).unwrap();

  assert_eq!( mag, 5.0 );
  assert_eq!( mag2, 25.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "the mutation-check assertion compares against the exact input literal, never a computed result; epsilon comparison would weaken it" ) ]
fn vector_normalize_returns_new_unit_length_copy_without_mutating_original()
{
  let engine = engine_build();
  let normalized_mag : f64 = engine.eval( "f32x2(3.0, 4.0).normalize().mag()" ).unwrap();
  let original_unchanged : f64 = engine.eval
  (
    "let a = f32x2(3.0, 4.0); a.normalize(); a.mag()"
  ).unwrap();

  assert!( ( normalized_mag - 1.0 ).abs() < 1e-6, "expected unit length, got {normalized_mag}" );
  assert_eq!( original_unchanged, 5.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertion checks exact expected value; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
fn vector_distance_computes_separation_between_two_points()
{
  let engine = engine_build();
  let result : f64 = engine.eval( "f32x2(0.0, 0.0).distance( f32x2(3.0, 4.0) )" ).unwrap();
  assert_eq!( result, 5.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertion checks exact expected value; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
fn vector_distance_squared_computes_squared_separation()
{
  let engine = engine_build();
  let result : f64 = engine.eval( "f32x2(0.0, 0.0).distance_squared( f32x2(3.0, 4.0) )" ).unwrap();
  assert_eq!( result, 25.0 );
}

#[ test ]
fn vector_min_and_max_take_componentwise_extremes()
{
  let engine = engine_build();
  let min_result : F32x2 = engine.eval( "f32x2(1.0, 8.0).min( f32x2(5.0, 2.0) )" ).unwrap();
  let max_result : F32x2 = engine.eval( "f32x2(1.0, 8.0).max( f32x2(5.0, 2.0) )" ).unwrap();

  assert_eq!( min_result, F32x2::new( 1.0, 2.0 ) );
  assert_eq!( max_result, F32x2::new( 5.0, 8.0 ) );
}

#[ test ]
fn vector_unary_negation_negates_all_components()
{
  let engine = engine_build();
  let result : F32x2 = engine.eval( "-f32x2(3.0, -4.0)" ).unwrap();
  assert_eq!( result, F32x2::new( -3.0, 4.0 ) );
}

#[ test ]
fn vector_cross_product_computes_orthogonal_vector()
{
  let engine = engine_build();
  let result : F32x3 = engine.eval( "f32x3(1.0, 0.0, 0.0).cross( f32x3(0.0, 1.0, 0.0) )" ).unwrap();
  assert_eq!( result, F32x3::new( 0.0, 0.0, 1.0 ) );
}

#[ test ]
fn vector_truncate_drops_w_component()
{
  let engine = engine_build();
  let result : F32x3 = engine.eval( "f32x4(1.0, 2.0, 3.0, 4.0).truncate()" ).unwrap();
  assert_eq!( result, F32x3::new( 1.0, 2.0, 3.0 ) );
}

#[ test ]
fn vector_to_homogenous_appends_w_component()
{
  let engine = engine_build();
  let result : F32x4 = engine.eval( "f32x3(1.0, 2.0, 3.0).to_homogenous()" ).unwrap();
  assert_eq!( result, F32x4::new( 1.0, 2.0, 3.0, 1.0 ) );
}

#[ test ]
fn vector_f32x4_from_two_f32x2_concatenates_components()
{
  let engine = engine_build();
  let result : F32x4 = engine.eval( "f32x4( f32x2(1.0, 2.0), f32x2(3.0, 4.0) )" ).unwrap();
  assert_eq!( result, F32x4::new( 1.0, 2.0, 3.0, 4.0 ) );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
fn vector_f64_universal_math_uses_native_precision_without_casting()
{
  let engine = engine_build();
  let dot_result : f64 = engine.eval( "f64x2(3.0, 4.0).dot( f64x2(3.0, 4.0) )" ).unwrap();
  let mag_result : f64 = engine.eval( "f64x2(3.0, 4.0).mag()" ).unwrap();

  assert_eq!( dot_result, 25.0 );
  assert_eq!( mag_result, 5.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "0.25 is exactly representable in binary floating point and the division producing it has no rounding error; epsilon comparison would weaken the assertion" ) ]
fn tween_progress_reports_fraction_of_duration_elapsed()
{
  let engine = engine_build();
  let progress : f64 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 10.0 ); t.update( 2.5 ); t.progress()"
  ).unwrap();

  assert_eq!( progress, 0.25 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
fn tween_builder_methods_configure_duration_and_delay()
{
  let engine = engine_build();
  let duration : f64 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 2.0 ).with_delay( 0.5 ).with_duration( 20.0 ); t.duration()"
  ).unwrap();
  let delay : f64 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 2.0 ).with_delay( 0.5 ).with_duration( 20.0 ); t.delay()"
  ).unwrap();

  assert_eq!( duration, 20.0 );
  assert_eq!( delay, 0.5 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertion checks exact expected value; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
fn tween_time_accumulates_elapsed_delta_time()
{
  let engine = engine_build();
  let time : f64 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 100.0 ); t.update( 3.0 ); t.update( 4.0 ); t.time()"
  ).unwrap();

  assert_eq!( time, 7.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertion checks exact expected value; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
fn tween_pause_halts_further_progress_until_resumed()
{
  let engine = engine_build();
  let time : f64 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 10.0 ); t.update( 5.0 ); t.pause(); t.update( 3.0 ); t.resume(); t.update( 2.0 ); t.time()"
  ).unwrap();

  assert_eq!( time, 7.0 );
}

#[ test ]
fn tween_reset_returns_to_start_value()
{
  let engine = engine_build();
  let value : F32x1 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 10.0 ); t.update( 5.0 ); t.reset(); t.value()"
  ).unwrap();

  assert_eq!( value, F32x1::new( 0.0 ) );
}

#[ test ]
fn tween_current_repeat_increments_after_each_repeat_cycle()
{
  let engine = engine_build();
  let repeat_count : i64 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 10.0 ).with_repeat( 5 ); t.update( 10.0 ); t.update( 10.0 ); t.update( 10.0 ); t.current_repeat()"
  ).unwrap();

  assert_eq!( repeat_count, 3 );
}

#[ test ]
fn tween_with_yoyo_reverses_direction_on_alternate_repeats()
{
  let engine = engine_build();
  let value : F32x1 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 10.0 ).with_repeat( 5 ).with_yoyo( true ); t.update( 10.0 ); t.update( 2.5 ); t.value()"
  ).unwrap();

  assert_eq!( value, F32x1::new( 7.5 ) );
}

#[ test ]
fn tween_state_reports_animation_lifecycle_stage()
{
  let engine = engine_build();
  let pending : String = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 1.0 ); t.state()"
  ).unwrap();
  let completed : String = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 1.0 ); t.update( 2.0 ); t.state()"
  ).unwrap();

  assert_eq!( pending, "Pending" );
  assert_eq!( completed, "Completed" );
}

#[ test ]
fn tween_with_easing_selector_accepts_named_curve()
{
  let engine = engine_build();
  let value : F32x1 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 1.0, \"EaseInOutQuad\" ); t.update( 1.0 ); t.value()"
  ).unwrap();

  assert_eq!( value, F32x1::new( 10.0 ) );
}

#[ test ]
fn tween_with_easing_selector_rejects_unknown_curve_name()
{
  let engine = engine_build();
  let err = engine.eval::< F32x1 >
  (
    "tween( f32x1(0.0), f32x1(10.0), 1.0, \"NotARealCurve\" )"
  ).unwrap_err();

  assert!
  (
    err.to_string().contains( "unknown easing curve name" ),
    "expected an unknown-curve error, got: {err}"
  );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "assertion checks a value derived from CubicHermite's exact polynomial coefficients at t=0.5; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
fn tween_with_cubic_hermite_tangents_deviates_from_linear_interpolation()
{
  let engine = engine_build();
  let value : F32x1 = engine.eval
  (
    "let t = tween( f32x1(0.0), f32x1(10.0), 10.0, f32x1(4.0), f32x1(0.0) ); t.update( 5.0 ); t.value()"
  ).unwrap();

  // At the halfway point a zero-tangent Hermite curve would match plain linear
  // interpolation exactly (5.0); the non-zero m1 tangent pulls the result to
  // 5.5, proving the tangents actually reach the interpolation math rather
  // than being silently ignored.
  assert_eq!( value, F32x1::new( 5.5 ) );
}
