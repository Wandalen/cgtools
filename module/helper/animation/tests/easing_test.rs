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
    CubicBezier,
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

  // test_kind: bug_reproducer(BUG-233)
  /// ## Root Cause
  /// `Step::new` stored its `steps` argument as given, with no floor. `apply`'s
  /// `( time * self.steps ).ceil() / self.steps` then divided by that stored value directly, so
  /// `Step::new( 0.0 )` produced a `0.0` divisor -- `f64` division never panics, so `0.0 / 0.0`
  /// silently evaluated to `NaN` instead of erroring.
  /// ## Why Not Caught
  /// The only existing `Step` test, `test_step_function`, exercises `Step::new( 5.0 )` only --
  /// no test ever constructed a `Step` with `0.0` (or a negative value).
  /// ## Fix Applied
  /// `Step::new` now floors its argument with `steps.max( 0.001 )`, mirroring `Tween::new`'s own
  /// `duration.max( 0.001 )` guard against the identical division-by-zero shape. See
  /// `easing/base.rs`.
  /// ## Prevention
  /// Added this test, which constructs `Step::new( 0.0 )` and asserts `apply` returns a finite,
  /// non-`NaN` value rather than propagating `NaN` into the interpolated result.
  /// ## Pitfall
  /// Rust's `f64` division never panics on a zero divisor -- it silently returns `NaN` or
  /// `±inf` -- so any `f64` constructor parameter that later becomes a division's divisor needs
  /// an explicit floor; there is no language-level safety net that would surface the mistake.
  #[ test ]
  fn test_step_function_zero_steps_does_not_produce_nan()
  {
    let step_func = Step::new( 0.0 );
    let value = step_func.apply( 0.0_f64, 1.0_f64, 0.5 );
    assert!( !value.is_nan(), "Step::new( 0.0 ) produced NaN instead of a finite value" );
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

  // test_kind: bug_reproducer(BUG-502)
  /// ## Root Cause
  /// `CubicBezier::new` defaults `iterations` to `8` specifically because `0` skips the
  /// Newton-Raphson solve loop in `apply` entirely (Fix(TASK-041), tested above by
  /// `test_cubic_mid_curve_accuracy`) -- but `iterations_set`/`with_iterations` still wrote
  /// `iterations` unchecked, so an explicit `.with_iterations( 0 )` (or `.iterations_set( 0 )`)
  /// call reintroduced the exact TASK-041 defect via caller action, bypassing the constructor's
  /// safe default entirely.
  /// ## Why Not Caught
  /// TASK-041's own test only exercises the *default* (`iterations: 8` from `new`, via the
  /// named-curve builders' `.with_iterations( 8 )` chains) -- nothing called either setter with
  /// `0` to check whether the same degenerate case the constructor default was chosen to avoid
  /// was still reachable through the public setter API.
  /// ## Fix Applied
  /// Changed both `iterations_set` and `with_iterations` to `self.iterations = iterations.max( 1
  /// );`, flooring at `1` instead of writing the caller's value unchecked. See
  /// `easing/cubic/bezier.rs`. Verified against this file's call sites: all 24
  /// `impl_easing_function!` invocations pass `8`, so the floor changes no existing behavior.
  /// ## Prevention
  /// Fixing a bad default only closes the constructor path -- any public setter writing the
  /// same field re-opens the identical defect unless it enforces the same constraint the
  /// default was protecting. A field with a "degenerate value" (like `iterations: 0` disabling
  /// the solve loop) needs that constraint enforced at every write site, not just the one the
  /// original bug happened to be filed against.
  /// ## Pitfall
  /// `with_iterations( 0 )` compiles and runs with no error or warning -- the defect is silent
  /// exactly like the original TASK-041 default was, just reachable through a different call
  /// path (explicit caller action instead of an unset default).
  #[ test ]
  fn test_cubic_iterations_floored_at_one_via_with_iterations()
  {
    // EaseInSine's own curve ( [ 0.12, 0.0, 0.39, 0.0 ] ), whose y-tangents are both 0.0, makes
    // y_get( t ) == t^3 exactly -- so the pre-fix, unfloored `iterations = 0` behavior (the solve
    // loop's `for _ in 0..0` body never runs, leaving `bezier_t` at the raw input `time`) gives
    // the exact, independently-known value `y_get( 0.5 ) == 0.5_f64.powi( 3 ) == 0.125`, matching
    // the TASK-041 doc comment's own worked example above.
    let curve = CubicBezier::< f32 >::new( [ 0.12, 0.0, 0.39, 0.0 ] ).with_iterations( 0 );
    let result = curve.apply( 0.0, 1.0, 0.5 );

    assert!
    (
      ( result - 0.125 ).abs() > 0.01,
      "with_iterations( 0 ) should be floored to 1 (running one Newton-Raphson step), not \
       silently accepted -- got {result}, which matches the unfloored iterations = 0 raw \
       pass-through value of 0.125 ( == y_get( 0.5 ) == 0.5^3 ) almost exactly"
    );
  }

  #[ test ]
  fn test_cubic_iterations_floored_at_one_via_iterations_set()
  {
    let mut curve = CubicBezier::< f32 >::new( [ 0.12, 0.0, 0.39, 0.0 ] );
    curve.iterations_set( 0 );
    let result = curve.apply( 0.0, 1.0, 0.5 );

    assert!
    (
      ( result - 0.125 ).abs() > 0.01,
      "iterations_set( 0 ) should be floored to 1, not silently accepted -- got {result}, \
       which matches the unfloored raw pass-through value of 0.125 almost exactly"
    );
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
