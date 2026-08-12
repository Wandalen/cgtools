mod private
{
  use crate::{ f32x2_register, f64x2_register, tween_f32x2_register, tween_f64x2_register };

  /// Builds a `rhai::Engine` with both `F32x2`/`Tween< F32x2 >` and
  /// `F64x2`/`Tween< F64x2 >` registered, so a script can pick whichever
  /// float precision it needs.
  #[ inline ]
  #[ must_use ]
  pub fn engine_build() -> rhai::Engine
  {
    let mut engine = rhai::Engine::new();
    f32x2_register( &mut engine );
    f64x2_register( &mut engine );
    tween_f32x2_register( &mut engine );
    tween_f64x2_register( &mut engine );
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
