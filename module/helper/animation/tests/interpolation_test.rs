//! Integration tests related to Tween struct and trait Animatable
#![ expect( clippy::float_cmp, reason = "assertions check deterministic interpolation arithmetic against exact expected values" ) ]

#[ cfg( test ) ]
mod tests
{
  use animation::
  {
    Tween,
    Animatable,
    AnimatablePlayer,
    AnimationState,
    easing::base::{ EasingBuilder, Linear }
  };

  // --- Animatable Trait Tests ---

  #[ test ]
  fn test_f32_interpolation()
  {
    let start = 10.0_f32;
    let end = 20.0_f32;
    assert_eq!( start.interpolate( &end, 0.0 ), 10.0 );
    assert_eq!( start.interpolate( &end, 1.0 ), 20.0 );
    assert_eq!( start.interpolate( &end, 0.5 ), 15.0 );
  }

  #[ test ]
  fn test_i32_interpolation()
  {
    let start = 5_i32;
    let end = 15_i32;
    assert_eq!( start.interpolate( &end, 0.0 ), 5 );
    assert_eq!( start.interpolate( &end, 1.0 ), 15 );
    assert_eq!( start.interpolate( &end, 0.5 ), 10 );
  }

  // --- Tween Core Logic Tests ---

  #[ test ]
  fn test_tween_initial_state()
  {
    let tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() );
    assert_eq!( tween.state(), AnimationState::Pending );
    assert_eq!( tween.progress(), 0.0 );
    assert!( !tween.is_completed() );
  }

  #[ test ]
  fn test_tween_progress_and_completion()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() );

    let val1 = tween.update( 0.5 );
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( val1, 5.0 );
    assert_eq!( tween.progress(), 0.5 );

    let val2 = tween.update( 0.5 );
    assert_eq!( tween.state(), AnimationState::Completed );
    assert_eq!( val2, 10.0 );
    assert_eq!( tween.progress(), 1.0 );
    assert!( tween.is_completed() );
  }

  #[ test ]
  fn test_tween_with_delay_behavior()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
    .with_delay( 0.5 );

    // First update: still in delay
    let val1 = tween.update( 0.2 );
    assert_eq!( val1, 0.0 );
    assert_eq!( tween.state(), AnimationState::Pending );

    // Second update: delay ends, animation starts
    let val2 = tween.update( 0.3 ); // 0.2 + 0.3 = 0.5 total elapsed time
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( val2, 0.0 ); // Since 0 remaining time for animation

    // Third update: animates
    let val3 = tween.update( 0.5 );
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( val3, 5.0 );
  }

  #[ test ]
  fn test_tween_pause_resume()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::build() );
    tween.update( 0.5 ); // Progress to 2.5
    assert_eq!( tween.value_get(), 2.5 );

    tween.pause();
    assert_eq!( tween.state(), AnimationState::Paused );

    let val = tween.update( 1.0 ); // Update while paused, value should not change
    assert_eq!( val, 2.5 );
    assert_eq!( tween.state(), AnimationState::Paused );

    tween.resume();
    assert_eq!( tween.state(), AnimationState::Running );

    let val2 = tween.update( 1.5 ); // Update for remaining duration
    assert_eq!( val2, 10.0 );
    assert!( tween.is_completed() );
  }

  #[ test ]
  fn test_tween_finite_repeat()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_repeat( 2 );

    tween.update( 1.0 ); // First loop finishes
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 1 );

    tween.update( 1.0 ); // Second loop finishes
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 2 );

    tween.update( 1.0 ); // Third loop finishes, which is the final repeat
    assert!( tween.is_completed() );
  }

  #[ test ]
  fn test_tween_infinite_repeat()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
    .with_repeat( -1 );

    tween.update( 1.0 );
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 1 );

    tween.update( 10.0 );
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 11 );
  }

  // test_kind: bug_reproducer(TASK-015)
  /// ## Root Cause
  /// `repeat_handle`'s repeat branches clamped the post-wrap elapsed time with `.min( 0.0 )`
  /// instead of `.max( 0.0 )`. `elapsed - duration * floor( elapsed / duration )` is a
  /// floor-division remainder, mathematically always `>= 0.0`, so `.min( 0.0 )` forced `elapsed`
  /// back to exactly `0.0` on every repeat regardless of the real remainder.
  /// ## Why Not Caught
  /// Every existing repeat test drives `update()` with deltas that are exact multiples of
  /// `duration`, where the remainder is `0.0` either way — the bug is invisible unless elapsed
  /// time crosses a repeat boundary mid-frame with time left over.
  /// ## Fix Applied
  /// Changed both repeat branches' `.min( 0.0 )` to `.max( 0.0 )`. See `interpolation.rs`.
  /// ## Prevention
  /// A clamp guarding against floating-point drift must clamp toward the valid side, not away
  /// from it — review the sign of what `.max`/`.min` actually keeps, not just that a clamp is
  /// present.
  /// ## Pitfall
  /// `.min( 0.0 )` on an always-non-negative value silently discards it in favor of `0.0` every
  /// time — no panic, no warning, just quietly wrong progress after every repeat.
  #[ test ]
  fn test_tween_infinite_repeat_preserves_overflow_elapsed()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_repeat( -1 );

    let val = tween.update( 1.25 ); // crosses the 1.0s repeat boundary with 0.25s left over

    assert_eq!( tween.current_repeat(), 1 );
    assert_eq!( val, 2.5 ); // 0.25 / 1.0 progress into the new loop, scaled to [ 0.0, 10.0 ]
  }

  #[ test ]
  fn test_tween_finite_repeat_preserves_overflow_elapsed()
  {
    // Same `.min( 0.0 )` -> `.max( 0.0 )` fix as test_tween_infinite_repeat_preserves_overflow_elapsed
    // above, applied to the finite-repeat branch instead of the infinite one.
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_repeat( 2 );

    let val = tween.update( 1.25 );

    assert_eq!( tween.current_repeat(), 1 );
    assert_eq!( val, 2.5 );
  }

  // test_kind: bug_reproducer(TASK-015)
  /// ## Root Cause
  /// `[Tween<T>; N]::duration_get` computed `min_start` via `.max()` seeded at `0.0`, returning
  /// the largest per-element delay instead of the smallest; `delay_get` seeded its `.min()`
  /// reduction at `0.0` instead of `f64::MAX`, so it always returned `0.0` whenever every real
  /// delay was positive.
  /// ## Why Not Caught
  /// No existing test constructed a `[Tween<T>; N]` array with differing per-element delays —
  /// the only array-based coverage used tweens sharing the same (often zero) delay, for which
  /// both bugs happen to return the correct answer by coincidence.
  /// ## Fix Applied
  /// Reseeded `duration_get`'s `min_start` at `f64::MAX` to match a min-reduction, and changed
  /// `delay_get` to reduce via `.min()` instead of `.max()`. See `interpolation.rs`.
  /// ## Prevention
  /// Review a min/max-reduction's seed value and comparison direction together — copy-pasting
  /// one half of the pattern without the other produces a reduction that only fails once real
  /// data crosses the wrongly-chosen seed.
  /// ## Pitfall
  /// A min-reduction seeded at a real domain value like `0.0` silently returns that seed whenever
  /// every element is `>= 0.0`, masking the bug for any array containing a zero-delay tween.
  #[ test ]
  fn test_tween_array_duration_and_delay_get()
  {
    let tweens : [ Tween< f32 >; 2 ] =
    [
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() ).with_delay( 2.0 ),
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() ).with_delay( 0.5 ),
    ];

    assert_eq!( tweens.delay_get(), 0.5 );
    assert_eq!( tweens.duration_get(), 2.5 ); // ( 2.0 + 1.0 ) - 0.5
  }

  #[ test ]
  fn test_tween_yoyo_with_repeat()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
    .with_repeat( 1 ).with_yoyo( true );

    // First loop: 0.0 -> 10.0
    let val1 = tween.update( 0.5 );
    assert_eq!( val1, 5.0 );
    tween.update( 0.5 );
    assert_eq!( tween.value_get(), 10.0 );
    assert_eq!( tween.current_repeat(), 1 );

    // Second loop: 10.0 -> 0.0 (yoyo)
    let val2 = tween.update( 0.5 );
    assert_eq!( val2, 5.0 );
    tween.update( 0.5 );
    assert_eq!( tween.value_get(), 0.0 );
    assert!( tween.is_completed() );
  }
}
