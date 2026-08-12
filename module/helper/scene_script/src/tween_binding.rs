mod private
{
  use ndarray_cg::{ F32x1, F32x2, F32x3, F32x4, F64x1, F64x2, F64x3, F64x4 };
  use animation::{ Tween, AnimatablePlayer, easing::base::{ EasingBuilder, Linear } };
  use rhai::Engine;

  /// Registers `animation::Tween< F32x1 >` into `engine` as Rhai type
  /// `"Tween"`: constructor `tween( start, end, duration )` (linear easing),
  /// `.update( delta_time )` advances and returns the current `F32x1`
  /// value, `.value()` peeks without advancing, `.is_completed()` reports
  /// state. See [`tween_f32x2_register`] for the shared-name rationale and
  /// [`tween_f64x1_register`] for the `f64`-element sibling.
  #[ inline ]
  pub fn tween_f32x1_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F32x1 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F32x1, end : F32x1, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F32x1 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x1 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x1 > | t.is_completed() );
  }

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
  /// [`tween_f64x2_register`] for the `f64`-element sibling.
  ///
  /// Both share the Rhai type name `"Tween"` and constructor name `"tween"`
  /// — Rhai overloads `tween(...)` by the actual argument types (`F32x2` vs
  /// `F64x2`), so a script gets the right one from whichever vector type it
  /// passes in, without picking a differently-named constructor.
  #[ inline ]
  pub fn tween_f32x2_register( engine : &mut Engine )
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

  /// Registers `animation::Tween< F32x3 >` into `engine`, mirroring
  /// [`tween_f32x2_register`] exactly but over `F32x3`. See
  /// [`tween_f64x3_register`] for the `f64`-element sibling.
  #[ inline ]
  pub fn tween_f32x3_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F32x3 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F32x3, end : F32x3, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F32x3 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x3 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x3 > | t.is_completed() );
  }

  /// Registers `animation::Tween< F32x4 >` into `engine`, mirroring
  /// [`tween_f32x2_register`] exactly but over `F32x4`. See
  /// [`tween_f64x4_register`] for the `f64`-element sibling.
  #[ inline ]
  pub fn tween_f32x4_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F32x4 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F32x4, end : F32x4, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F32x4 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x4 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x4 > | t.is_completed() );
  }

  /// Registers `animation::Tween< F64x1 >` into `engine`, mirroring
  /// [`tween_f32x1_register`] exactly but over `F64x1`. `F64x1` satisfies
  /// `Animatable` via the same blanket `mingl::Vector< E, N >` impl (`f64`
  /// implements both `MatEl` — any `Copy + Default` type does — and
  /// `Animatable` directly), so no new impl is needed here either.
  #[ inline ]
  pub fn tween_f64x1_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F64x1 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F64x1, end : F64x1, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F64x1 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x1 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x1 > | t.is_completed() );
  }

  /// Registers `animation::Tween< F64x2 >` into `engine`, mirroring
  /// [`tween_f32x2_register`] exactly but over `F64x2`. `F64x2` satisfies
  /// `Animatable` via the same blanket `mingl::Vector< E, N >` impl (`f64`
  /// implements both `MatEl` — any `Copy + Default` type does — and
  /// `Animatable` directly), so no new impl is needed here either.
  #[ inline ]
  pub fn tween_f64x2_register( engine : &mut Engine )
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

  /// Registers `animation::Tween< F64x3 >` into `engine`, mirroring
  /// [`tween_f32x3_register`] exactly but over `F64x3`.
  #[ inline ]
  pub fn tween_f64x3_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F64x3 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F64x3, end : F64x3, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F64x3 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x3 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x3 > | t.is_completed() );
  }

  /// Registers `animation::Tween< F64x4 >` into `engine`, mirroring
  /// [`tween_f32x4_register`] exactly but over `F64x4`.
  #[ inline ]
  pub fn tween_f64x4_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< Tween< F64x4 > >( "Tween" )
    .register_fn
    (
      "tween",
      | start : F64x4, end : F64x4, duration : f64 | Tween::new( start, end, duration, Linear::build() )
    )
    .register_fn( "update", | t : &mut Tween< F64x4 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x4 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x4 > | t.is_completed() );
  }
}

crate::mod_interface!
{
  orphan use
  {
    tween_f32x1_register,
    tween_f32x2_register,
    tween_f32x3_register,
    tween_f32x4_register,
    tween_f64x1_register,
    tween_f64x2_register,
    tween_f64x3_register,
    tween_f64x4_register,
  };
}
