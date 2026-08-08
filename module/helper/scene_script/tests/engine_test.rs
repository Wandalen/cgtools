//! Smoke tests for the `scene_script` Rhai engine builder.

use ndarray_cg::F32x2;
use scene_script::build_engine;

#[ test ]
fn f32x2_arithmetic_roundtrip()
{
  let engine = build_engine();
  let result : F32x2 = engine.eval( "f32x2(1.0, 2.0) + f32x2(3.0, 4.0)" ).unwrap();
  assert_eq!( result, F32x2::new( 4.0, 6.0 ) );
}

#[ test ]
fn tween_f32x2_updates_toward_end_value()
{
  let engine = build_engine();
  let value : F32x2 = engine.eval
  (
    "let t = tween( f32x2(0.0, 0.0), f32x2(10.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F32x2::new( 10.0, 0.0 ) );
}
