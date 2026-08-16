//! Integration tests related to `EasingFunction` and `EasingBuilder`
//! traits and structs that implements them
#![ expect( clippy::float_cmp, reason = "assertions check deterministic easing-curve arithmetic against exact endpoint values" ) ]

#[ cfg( test ) ]
mod tests
{
  use animation::easing::
  {
    base::{ EasingFunction, EasingBuilder },
    Linear, Step
  };
  use animation::easing::cubic::bezier::
  {
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack
  };
  use animation::easing::cubic::hermite::CubicHermite;

  #[ test ]
  fn test_linear_function()
  {
    // Linear easing should return the input value directly
    assert_eq!( Linear::build().apply( 0.0, 1.0, 0.5 ), 0.5_f32 );
    assert_eq!( Linear::build().apply( 0.0, 1.0, 0.0 ), 0.0_f32 );
    assert_eq!( Linear::build().apply( 0.0, 1.0, 1.0 ), 1.0_f32 );
  }

  fn assert_f_eq( first : f64, second : f64, eps : f64 )
  {
    assert!( second - eps < first && first < second + eps );
  }

  #[ test ]
  fn test_step_function()
  {
    let eps = 0.001;
    // Step easing should progress in discrete steps
    let step_func = Step::new( 5.0 );
    assert_eq!( step_func.apply( 0.0, 1.0, 0.0 ), 0.0_f64 );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.01 ), 0.2, eps );
    assert_f_eq(  step_func.apply( 0.0, 1.0, 0.2 ), 0.2, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.21 ), 0.4, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.4 ), 0.4, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.81 ), 1.0, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 1.0 ), 1.0, eps );
  }

  #[ test ]
  fn test_cubic_boundaries_and_properties()
  {
    // A list of all cubic easing functions to test common properties
    let cubic_functions : Vec< Box< dyn EasingFunction< AnimatableType = f32 > > > = vec!
    [
      EaseInSine::build(),
      EaseOutSine::build(),
      EaseInOutSine::build(),
      EaseInQuad::build(),
      EaseOutQuad::build(),
      EaseInOutQuad::build(),
      EaseInCubic::build(),
      EaseOutCubic::build(),
      EaseInOutCubic::build(),
      EaseInQuart::build(),
      EaseOutQuart::build(),
      EaseInOutQuart::build(),
      EaseInQuint::build(),
      EaseOutQuint::build(),
      EaseInOutQuint::build(),
      EaseInExpo::build(),
      EaseOutExpo::build(),
      EaseInOutExpo::build(),
      EaseInCirc::build(),
      EaseOutCirc::build(),
      EaseInOutCirc::build(),
      EaseInBack::build(),
      EaseOutBack::build(),
      EaseInOutBack::build(),
    ];

    // All cubic functions should return 0.0 at t = 0.0 and 1.0 at t = 1.0
    for easing_function in cubic_functions
    {
      assert_eq!( easing_function.apply( 0.0, 1.0, 0.0 ), 0.0, "{easing_function:?} should start at 0.0" );
      assert_eq!( easing_function.apply( 0.0, 1.0, 1.0 ), 1.0, "{easing_function:?} should end at 1.0" );
    }
  }

  #[ test ]
  fn test_back_easing_overshoot()
  {
    // Back easing functions should have values outside the [ 0.0, 1.0 ] range
    assert!( EaseInBack::build().apply( 0.0, 1.0, 0.1 ) < 0.0 );
    assert!( EaseOutBack::build().apply( 0.0, 1.0, 0.9 ) > 1.0 );
    assert!( EaseInOutBack::build().apply( 0.0, 1.0, 0.1 ) < 0.0 );
    assert!( EaseInOutBack::build().apply( 0.0, 1.0, 0.9 ) > 1.0 );
  }

  #[ test ]
  fn test_specific_easing_behaviors()
  {
    // EaseInQuad should be slower than linear at the start
    assert!( EaseInQuad::build().apply( 0.0, 1.0, 0.2 ) < Linear::build().apply( 0.0, 1.0, 0.2 ) );

    // EaseOutQuad should be faster than linear at the start
    assert!( EaseOutQuad::build().apply( 0.0, 1.0, 0.2 ) > Linear::build().apply( 0.0, 1.0, 0.2 ) );
  }

  // test_kind: bug_reproducer(TASK-041)
  /// ## Root Cause
  /// `CubicBezier::new` defaulted `iterations` to `0`, which skips the Newton-Raphson solve loop
  /// in `apply` entirely — `y_get` was evaluated at the raw input fraction instead of the solved
  /// Bezier parameter, producing the wrong easing shape for every named curve.
  /// ## Why Not Caught
  /// `test_cubic_boundaries_and_properties` above only checks `t = 0.0` / `t = 1.0`, where
  /// `apply`'s early-return guards bypass the solve loop regardless of `iterations` — only a
  /// mid-curve point exposes the wrong shape.
  /// ## Fix Applied
  /// Changed the default to `iterations: 8` and had all 24 named curves chain
  /// `.with_iterations( 8 )` explicitly. See `easing/cubic/bezier.rs`.
  /// ## Prevention
  /// A numeric solver's iteration-count parameter should never default to its degenerate "off"
  /// value — boundary-only tests can't distinguish "solved" from "not solved" for a curve whose
  /// endpoints are fixed by construction.
  /// ## Pitfall
  /// Reference values below are solved to near-convergence independently (100 Newton-Raphson
  /// iterations); the buggy `iterations = 0` behavior would instead give the raw input fraction
  /// (0.5) run through `y_get` directly — 0.125 for `EaseInSine`, 0.875 for `EaseOutQuad`.
  #[ test ]
  fn test_cubic_mid_curve_accuracy()
  {
    let eps = 0.001;
    assert_f_eq( EaseInSine::build().apply( 0.0, 1.0, 0.5 ), 0.300_338, eps );
    assert_f_eq( EaseOutQuad::build().apply( 0.0, 1.0, 0.5 ), 0.749_269, eps );
  }

  // test_kind: bug_reproducer(BUG-141)
  /// ## Root Cause
  /// `apply`'s Newton-Raphson `slope` computed the three Bezier terms' derivatives as if
  /// independent (`3(1-t)^2*P1 + 6(1-t)*t*P2 + 3t^2`) instead of the true product-rule
  /// derivative of `x_get` (`3(1-t)^2*(P1-P0) + 6(1-t)*t*(P2-P1) + 3t^2*(P3-P2)`, P0=0, P3=1).
  /// ## Why Not Caught
  /// `test_cubic_mid_curve_accuracy` above only exercises curves/times where the two formulas
  /// happen to nearly agree within its `eps = 0.001` tolerance; `EaseInExpo` at `time = 0.9`
  /// diverges by ~0.04, an order of magnitude past that tolerance.
  /// ## Fix Applied
  /// Corrected `slope` to the standard cubic-Bezier tangent formula. See `easing/cubic/bezier.rs`.
  /// ## Prevention
  /// Added this test, pinning `EaseInExpo::build().apply( 0.0, 1.0, 0.9 )` against a value
  /// independently verified via 200-iteration bisection (a derivative-free root-finding method,
  /// immune to this class of bug).
  /// ## Pitfall
  /// Newton-Raphson with a wrong-but-not-wildly-wrong derivative estimate still converges to the
  /// correct root given enough iterations -- it just converges slower. At the crate's fixed
  /// 8-iteration budget (no convergence check), that shows up as a silently inaccurate result
  /// whose magnitude depends on the specific curve and `time`, not a clean pass/fail signal.
  #[ test ]
  fn test_cubic_newton_raphson_slope_matches_true_derivative()
  {
    let eps = 0.001;
    assert_f_eq( EaseInExpo::build().apply( 0.0, 1.0, 0.9 ), 0.505_609, eps );
  }

  // test_kind: bug_reproducer(TASK-041)
  /// ## Root Cause
  /// `CubicHermite::<Vec<E>>::new` silently `.resize()`d `m1`/`m2` down to the shorter of the two
  /// lengths instead of surfacing the mismatch.
  /// ## Why Not Caught
  /// No existing test constructed a `CubicHermite` with mismatched tangent vector lengths —
  /// there was no coverage of this constructor at all.
  /// ## Fix Applied
  /// Replaced the `.resize()` calls with `assert_eq!` panics naming both lengths. See
  /// `easing/cubic/hermite.rs`.
  /// ## Prevention
  /// `EasingFunction::apply` returns `Self::AnimatableType` directly (no `Result`) for every
  /// implementor, so a malformed-input precondition here can only be surfaced as a loud panic,
  /// not a recoverable error — silent `.resize()` truncation is never the right normalization for
  /// a caller precondition violation.
  /// ## Pitfall
  /// A silently truncated tangent vector produces a plausible-looking but wrong interpolation
  /// result for every subsequent `apply()` call — no signal at the call site that data was lost.
  #[ test ]
  #[ should_panic( expected = "m1 and m2 must have the same length" ) ]
  fn test_cubic_hermite_new_panics_on_mismatched_tangent_lengths()
  {
    let _ = CubicHermite::< Vec< f32 > >::new( vec![ 0.0, 1.0, 2.0 ], vec![ 0.0, 1.0 ] );
  }

  // test_kind: bug_reproducer(TASK-041)
  /// ## Root Cause
  /// `CubicHermite::<Vec<E>>::apply` silently `.resize()`d `start`, `end`, and the tangents down
  /// to the shortest of 3 independent lengths instead of surfacing the mismatch.
  /// ## Why Not Caught
  /// No existing test called `CubicHermite::apply` with a `start`/`end` length differing from the
  /// tangent length established at construction.
  /// ## Fix Applied
  /// Replaced the `.resize()` calls with `assert_eq!` panics naming both lengths. See
  /// `easing/cubic/hermite.rs`.
  /// ## Prevention
  /// Same as the constructor's Prevention — the shared `EasingFunction` trait signature has no
  /// `Result`, so a loud panic on malformed input is the correct fix at every call site here.
  /// ## Pitfall
  /// Same as the constructor's Pitfall — silent truncation produces a plausible-looking but wrong
  /// interpolation result with no signal that components were dropped.
  #[ test ]
  #[ should_panic( expected = "start and end must have the same length" ) ]
  fn test_cubic_hermite_apply_panics_on_mismatched_value_lengths()
  {
    let hermite = CubicHermite::< Vec< f32 > >::new( vec![ 0.0, 1.0 ], vec![ 0.0, 1.0 ] );
    let _ = hermite.apply( vec![ 0.0, 1.0, 2.0 ], vec![ 0.0, 1.0 ], 0.5 );
  }
}
