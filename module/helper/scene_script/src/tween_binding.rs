mod private
{
  use ndarray_cg::{ F32x2, F64x2 };
  use animation::{ Tween, AnimatablePlayer, easing::base::{ EasingBuilder, Linear } };
  use rhai::Engine;

  /// Registers `animation::Tween< F32x2 >` into `engine` as Rhai type
  /// `"Tween"`: constructor `tween( start, end, duration )` (linear easing),
  /// `.update( delta_time )` advances and returns the current `F32x2`
  /// value, `.value()` peeks without advancing, `.is_completed()` reports
  /// state.
  ///
  /// `F32x2` satisfies `Tween`'s `Animatable` bound for free: `mingl::Vector`
  /// re-exports `ndarray_cg::Vector` verbatim, and `animation` already
  /// implements `Animatable` for `mingl::Vector< E, N >` where
  /// `E: MatEl + Animatable` — no new impl is needed here. See
  /// [`register_tween_f64x2`] for the `f64`-element sibling.
  ///
  /// Both share the Rhai type name `"Tween"` and constructor name `"tween"`
  /// — Rhai overloads `tween(...)` by the actual argument types (`F32x2` vs
  /// `F64x2`), so a script gets the right one from whichever vector type it
  /// passes in, without picking a differently-named constructor.
  #[ inline ]
  pub fn register_tween_f32x2( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F32x2 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F32x2, end : F32x2, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F32x2 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x2 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x2 > | t.is_completed() );
  }

  /// Registers `animation::Tween< F64x2 >` into `engine`, mirroring
  /// [`register_tween_f32x2`] exactly but over `F64x2`. `F64x2` satisfies
  /// `Animatable` via the same blanket `mingl::Vector< E, N >` impl (`f64`
  /// implements both `MatEl` — any `Copy + Default` type does — and
  /// `Animatable` directly), so no new impl is needed here either.
  #[ inline ]
  pub fn register_tween_f64x2( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F64x2 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F64x2, end : F64x2, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F64x2 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x2 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x2 > | t.is_completed() );
  }
}

crate::mod_interface!
{
  orphan use
  {
    register_tween_f32x2,
    register_tween_f64x2,
  };
}
