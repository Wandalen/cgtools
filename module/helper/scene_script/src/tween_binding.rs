mod private
{
  use ndarray_cg::{ F32x1, F32x2, F32x3, F32x4, F64x1, F64x2, F64x3, F64x4 };
  use animation::
  {
    Animatable, Tween, AnimatablePlayer,
    easing::
    {
      base::{ EasingBuilder, EasingFunction, Linear },
      cubic::
      {
        bezier::
        {
          EaseInSine, EaseOutSine, EaseInOutSine,
          EaseInQuad, EaseOutQuad, EaseInOutQuad,
          EaseInCubic, EaseOutCubic, EaseInOutCubic,
          EaseInQuart, EaseOutQuart, EaseInOutQuart,
          EaseInQuint, EaseOutQuint, EaseInOutQuint,
          EaseInExpo, EaseOutExpo, EaseInOutExpo,
          EaseInCirc, EaseOutCirc, EaseInOutCirc,
          EaseInBack, EaseOutBack, EaseInOutBack,
        },
        hermite::CubicHermite,
      },
    },
  };
  use rhai::{ Engine, EvalAltResult };

  /// Resolves a script-supplied easing-curve name to a boxed `EasingFunction`.
  /// Accepts `"Linear"` plus the 24 CSS-style presets from
  /// `animation::easing::cubic::bezier` (`"EaseInSine"` .. `"EaseInOutBack"`),
  /// spelled exactly as their Rust identifiers per this crate's naming
  /// convention (`docs/invariant/003`). An unrecognized name is a
  /// script-catchable runtime error, never a silent fallback to Linear.
  fn easing_from_name< A >( name : &str ) -> Result< Box< dyn EasingFunction< AnimatableType = A > >, Box< EvalAltResult > >
  where
    A : Animatable + 'static,
  {
    match name
    {
      "Linear" => Ok( Linear::< A >::build() ),
      "EaseInSine" => Ok( EaseInSine::< A >::build() ),
      "EaseOutSine" => Ok( EaseOutSine::< A >::build() ),
      "EaseInOutSine" => Ok( EaseInOutSine::< A >::build() ),
      "EaseInQuad" => Ok( EaseInQuad::< A >::build() ),
      "EaseOutQuad" => Ok( EaseOutQuad::< A >::build() ),
      "EaseInOutQuad" => Ok( EaseInOutQuad::< A >::build() ),
      "EaseInCubic" => Ok( EaseInCubic::< A >::build() ),
      "EaseOutCubic" => Ok( EaseOutCubic::< A >::build() ),
      "EaseInOutCubic" => Ok( EaseInOutCubic::< A >::build() ),
      "EaseInQuart" => Ok( EaseInQuart::< A >::build() ),
      "EaseOutQuart" => Ok( EaseOutQuart::< A >::build() ),
      "EaseInOutQuart" => Ok( EaseInOutQuart::< A >::build() ),
      "EaseInQuint" => Ok( EaseInQuint::< A >::build() ),
      "EaseOutQuint" => Ok( EaseOutQuint::< A >::build() ),
      "EaseInOutQuint" => Ok( EaseInOutQuint::< A >::build() ),
      "EaseInExpo" => Ok( EaseInExpo::< A >::build() ),
      "EaseOutExpo" => Ok( EaseOutExpo::< A >::build() ),
      "EaseInOutExpo" => Ok( EaseInOutExpo::< A >::build() ),
      "EaseInCirc" => Ok( EaseInCirc::< A >::build() ),
      "EaseOutCirc" => Ok( EaseOutCirc::< A >::build() ),
      "EaseInOutCirc" => Ok( EaseInOutCirc::< A >::build() ),
      "EaseInBack" => Ok( EaseInBack::< A >::build() ),
      "EaseOutBack" => Ok( EaseOutBack::< A >::build() ),
      "EaseInOutBack" => Ok( EaseInOutBack::< A >::build() ),
      _ => Err( format!( "unknown easing curve name: \"{name}\"" ).into() ),
    }
  }

  /// Registers `animation::Tween< F32x1 >` into `engine` as Rhai type
  /// `"Tween"`: constructor `tween( start, end, duration )` (linear easing),
  /// its 4-arg overload `tween( start, end, duration, easing )` picking a
  /// named curve via [`easing_from_name`], and its 5-arg overload
  /// `tween( start, end, duration, m1, m2 )` building a `CubicHermite` curve
  /// directly from two tangent values — `CubicHermite` has no zero-argument
  /// `EasingBuilder::build()`, so [`easing_from_name`]'s string dispatch can
  /// never reach it (see `docs/pitfall/006`); this overload bypasses name
  /// dispatch entirely, resolved by arity like every other `tween(...)`
  /// overload. `.update( delta_time )` advances and returns the current
  /// `F32x1` value, `.value()` peeks without advancing,
  /// `.is_completed()`/`.progress()`/`.duration()`/`.delay()`/`.time()`/
  /// `.current_repeat()`/`.state()` report playback state,
  /// `.pause()`/`.resume()`/`.reset()` control it, and
  /// `.with_delay()`/`.with_duration()`/`.with_repeat()`/`.with_yoyo()` each
  /// consume the tween and return a modified copy for chaining. See
  /// [`tween_f32x2_register`] for the shared-name rationale and
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
    .register_fn
    (
      "tween",
      | start : F32x1, end : F32x1, duration : f64, easing : &str | -> Result< Tween< F32x1 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F32x1 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F32x1, end : F32x1, duration : f64, m1 : F32x1, m2 : F32x1 | -> Tween< F32x1 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::< F32x1 >::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F32x1 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x1 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x1 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F32x1 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F32x1 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F32x1 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F32x1 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F32x1 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F32x1 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F32x1 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F32x1 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F32x1 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F32x1 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F32x1 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F32x1 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F32x1 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
  /// `F64x2`) and by arity (3-arg linear vs 4-arg named-easing), so a script
  /// gets the right one from whichever vector type and argument count it
  /// passes in, without picking a differently-named constructor. Every
  /// accessor/control/builder method described on [`tween_f32x1_register`]
  /// applies here too.
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
    .register_fn
    (
      "tween",
      | start : F32x2, end : F32x2, duration : f64, easing : &str | -> Result< Tween< F32x2 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F32x2 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F32x2, end : F32x2, duration : f64, m1 : F32x2, m2 : F32x2 | -> Tween< F32x2 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::< F32x2 >::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F32x2 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x2 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x2 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F32x2 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F32x2 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F32x2 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F32x2 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F32x2 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F32x2 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F32x2 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F32x2 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F32x2 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F32x2 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F32x2 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F32x2 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F32x2 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
    .register_fn
    (
      "tween",
      | start : F32x3, end : F32x3, duration : f64, easing : &str | -> Result< Tween< F32x3 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F32x3 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F32x3, end : F32x3, duration : f64, m1 : F32x3, m2 : F32x3 | -> Tween< F32x3 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F32x3 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x3 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x3 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F32x3 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F32x3 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F32x3 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F32x3 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F32x3 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F32x3 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F32x3 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F32x3 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F32x3 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F32x3 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F32x3 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F32x3 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F32x3 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
    .register_fn
    (
      "tween",
      | start : F32x4, end : F32x4, duration : f64, easing : &str | -> Result< Tween< F32x4 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F32x4 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F32x4, end : F32x4, duration : f64, m1 : F32x4, m2 : F32x4 | -> Tween< F32x4 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F32x4 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F32x4 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F32x4 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F32x4 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F32x4 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F32x4 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F32x4 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F32x4 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F32x4 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F32x4 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F32x4 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F32x4 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F32x4 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F32x4 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F32x4 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F32x4 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
    .register_fn
    (
      "tween",
      | start : F64x1, end : F64x1, duration : f64, easing : &str | -> Result< Tween< F64x1 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F64x1 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F64x1, end : F64x1, duration : f64, m1 : F64x1, m2 : F64x1 | -> Tween< F64x1 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F64x1 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x1 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x1 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F64x1 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F64x1 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F64x1 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F64x1 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F64x1 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F64x1 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F64x1 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F64x1 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F64x1 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F64x1 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F64x1 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F64x1 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F64x1 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
    .register_fn
    (
      "tween",
      | start : F64x2, end : F64x2, duration : f64, easing : &str | -> Result< Tween< F64x2 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F64x2 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F64x2, end : F64x2, duration : f64, m1 : F64x2, m2 : F64x2 | -> Tween< F64x2 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F64x2 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x2 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x2 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F64x2 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F64x2 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F64x2 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F64x2 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F64x2 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F64x2 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F64x2 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F64x2 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F64x2 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F64x2 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F64x2 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F64x2 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F64x2 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
    .register_fn
    (
      "tween",
      | start : F64x3, end : F64x3, duration : f64, easing : &str | -> Result< Tween< F64x3 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F64x3 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F64x3, end : F64x3, duration : f64, m1 : F64x3, m2 : F64x3 | -> Tween< F64x3 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F64x3 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x3 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x3 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F64x3 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F64x3 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F64x3 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F64x3 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F64x3 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F64x3 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F64x3 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F64x3 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F64x3 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F64x3 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F64x3 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F64x3 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F64x3 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
    .register_fn
    (
      "tween",
      | start : F64x4, end : F64x4, duration : f64, easing : &str | -> Result< Tween< F64x4 >, Box< EvalAltResult > >
      {
        Ok( Tween::new( start, end, duration, easing_from_name::< F64x4 >( easing )? ) )
      }
    )
    .register_fn
    (
      "tween",
      | start : F64x4, end : F64x4, duration : f64, m1 : F64x4, m2 : F64x4 | -> Tween< F64x4 >
      {
        Tween::new( start, end, duration, Box::new( CubicHermite::new( m1, m2 ) ) )
      }
    )
    .register_fn( "update", | t : &mut Tween< F64x4 >, delta_time : f64 | t.update( delta_time ) )
    .register_fn( "value", | t : &mut Tween< F64x4 > | t.value_get() )
    .register_fn( "is_completed", | t : &mut Tween< F64x4 > | t.is_completed() )
    .register_fn( "progress", | t : &mut Tween< F64x4 > | t.progress() )
    .register_fn( "duration", | t : &mut Tween< F64x4 > | t.duration_get() )
    .register_fn( "delay", | t : &mut Tween< F64x4 > | t.delay_get() )
    .register_fn( "time", | t : &mut Tween< F64x4 > | t.time() )
    .register_fn( "current_repeat", | t : &mut Tween< F64x4 > | i64::from( t.current_repeat() ) )
    .register_fn( "state", | t : &mut Tween< F64x4 > | format!( "{:?}", t.state() ) )
    .register_fn( "pause", | t : &mut Tween< F64x4 > | t.pause() )
    .register_fn( "resume", | t : &mut Tween< F64x4 > | t.resume() )
    .register_fn( "reset", | t : &mut Tween< F64x4 > | t.reset() )
    .register_fn( "with_delay", | t : Tween< F64x4 >, delay : f64 | t.with_delay( delay ) )
    .register_fn( "with_duration", | t : Tween< F64x4 >, duration : f64 | t.with_duration( duration ) )
    .register_fn( "with_repeat", | t : Tween< F64x4 >, count : i64 | t.with_repeat( count as i32 ) )
    .register_fn( "with_yoyo", | t : Tween< F64x4 >, yoyo : bool | t.with_yoyo( yoyo ) );
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
