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
