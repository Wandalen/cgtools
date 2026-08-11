//! Smoke tests for the `scene_script` Rhai engine builder.

use ndarray_cg::{ F32x2, F64x2 };
use scene_script::build_engine;

#[ test ]
fn f32x2_arithmetic_roundtrip()
{
  let engine = build_engine();
  let result : F32x2 = engine.eval( "f32x2(1.0, 2.0) + f32x2(3.0, 4.0)" ).unwrap();
  assert_eq!( result, F32x2::new( 4.0, 6.0 ) );
}

#[ test ]
fn f64x2_arithmetic_roundtrip()
{
  let engine = build_engine();
  let result : F64x2 = engine.eval( "f64x2(1.0, 2.0) + f64x2(3.0, 4.0)" ).unwrap();
  assert_eq!( result, F64x2::new( 4.0, 6.0 ) );
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

#[ test ]
fn tween_f64x2_updates_toward_end_value()
{
  let engine = build_engine();
  let value : F64x2 = engine.eval
  (
    "let t = tween( f64x2(0.0, 0.0), f64x2(10.0, 0.0), 1.0 ); t.update( 1.0 )"
  ).unwrap();

  assert_eq!( value, F64x2::new( 10.0, 0.0 ) );
}

#[ test ]
fn f32x2_and_f64x2_are_distinct_types_not_interchangeable()
{
  let engine = build_engine();
  let err = engine.eval::< F64x2 >( "f32x2(1.0, 2.0)" ).unwrap_err();
  assert!( err.to_string().contains( "type" ), "expected a type-mismatch error, got: {err}" );
}
