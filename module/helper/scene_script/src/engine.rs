mod private
{
  use crate::
  {
    f32x1_register, f32x2_register, f32x3_register, f32x4_register,
    f64x1_register, f64x2_register, f64x3_register, f64x4_register,
    tween_f32x1_register, tween_f32x2_register, tween_f32x3_register, tween_f32x4_register,
    tween_f64x1_register, tween_f64x2_register, tween_f64x3_register, tween_f64x4_register,
  };

  /// Builds a `rhai::Engine` with the full `{F32,F64}x{1,2,3,4}` vector
  /// family and their `Tween` pairings registered, so a script can pick
  /// whichever element precision and arity it needs.
  #[ inline ]
  #[ must_use ]
  pub fn engine_build() -> rhai::Engine
  {
    let mut engine = rhai::Engine::new();
    f32x1_register( &mut engine );
    f32x2_register( &mut engine );
    f32x3_register( &mut engine );
    f32x4_register( &mut engine );
    f64x1_register( &mut engine );
    f64x2_register( &mut engine );
    f64x3_register( &mut engine );
    f64x4_register( &mut engine );
    tween_f32x1_register( &mut engine );
    tween_f32x2_register( &mut engine );
    tween_f32x3_register( &mut engine );
    tween_f32x4_register( &mut engine );
    tween_f64x1_register( &mut engine );
    tween_f64x2_register( &mut engine );
    tween_f64x3_register( &mut engine );
    tween_f64x4_register( &mut engine );
    engine
  }
}

crate::mod_interface!
{
  orphan use
  {
    engine_build,
  };
}
