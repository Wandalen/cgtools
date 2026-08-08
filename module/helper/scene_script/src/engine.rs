mod private
{
  use crate::{ register_f32x2, register_tween_f32x2 };

  /// Builds a `rhai::Engine` with `F32x2` and `Tween< F32x2 >` registered.
  #[ inline ]
  #[ must_use ]
  pub fn build_engine() -> rhai::Engine
  {
    let mut engine = rhai::Engine::new();
    register_f32x2( &mut engine );
    register_tween_f32x2( &mut engine );
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
