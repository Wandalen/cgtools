mod private
{
  use crate::{ register_f32x2, register_f64x2, register_tween_f32x2, register_tween_f64x2 };

  /// Builds a `rhai::Engine` with both `F32x2`/`Tween< F32x2 >` and
  /// `F64x2`/`Tween< F64x2 >` registered, so a script can pick whichever
  /// float precision it needs.
  #[ inline ]
  #[ must_use ]
  pub fn build_engine() -> rhai::Engine
  {
    let mut engine = rhai::Engine::new();
    register_f32x2( &mut engine );
    register_f64x2( &mut engine );
    register_tween_f32x2( &mut engine );
    register_tween_f64x2( &mut engine );
    engine
  }
}

crate::mod_interface!
{
  orphan use
  {
    build_engine,
  };
}
