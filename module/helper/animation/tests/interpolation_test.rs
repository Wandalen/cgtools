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

  #[ test ]
  fn test_vec_interpolation()
  {
    let start = vec![ 0.0_f32, 10.0_f32 ];
    let end = vec![ 10.0_f32, 20.0_f32 ];
    assert_eq!( start.interpolate( &end, 0.0 ), start );
    assert_eq!( start.interpolate( &end, 1.0 ), end );
    assert_eq!( start.interpolate( &end, 0.5 ), vec![ 5.0_f32, 15.0_f32 ] );
  }

  // test_kind: bug_reproducer(BUG-148)
  /// ## Root Cause
  /// `Vec<E>::interpolate` used `self.iter().zip( other.iter() )`, which silently truncates to
  /// the shorter of the two Vecs whenever their lengths differ, instead of surfacing the
  /// mismatch -- the same defect shape `CubicHermite::new`/`apply` already guard against via
  /// `assert_eq!` (see `test_cubic_hermite_new_panics_on_mismatched_tangent_lengths` in
  /// `easing_test.rs`), which this sibling `Animatable` impl had never been brought into line
  /// with.
  /// ## Why Not Caught
  /// No existing test exercised `Vec<E>::interpolate` at all -- not even the equal-length happy
  /// path, let alone a length mismatch.
  /// ## Fix Applied
  /// Added an `assert_eq!` on `self.len() == other.len()` before the `.zip()`, matching
  /// `CubicHermite`'s established convention. See `interpolation.rs`.
  /// ## Prevention
  /// This test constructs two `Vec<f32>` of differing length and asserts `interpolate` panics
  /// naming both lengths, rather than silently returning a shorter-than-expected result.
  /// ## Pitfall
  /// A silently truncated result is a plausible-looking `Vec` of the wrong length -- nothing
  /// about the return value itself signals that trailing elements from the longer side were
  /// dropped; only a length assertion against a known-correct expectation catches it.
  #[ test ]
  #[ should_panic( expected = "self and other must have the same length" ) ]
  fn test_vec_interpolate_panics_on_mismatched_lengths()
  {
    let start = vec![ 0.0_f32, 1.0_f32, 2.0_f32 ];
    let end = vec![ 10.0_f32, 20.0_f32 ];
    let _ = start.interpolate( &end, 0.5 );
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

  // test_kind: bug_reproducer(BUG-140)
  /// ## Root Cause
  /// `Tween::progress()` computed `( elapsed - delay ) / duration`, but `update` only ever adds
  /// to `elapsed` AFTER the delay countdown has been fully consumed -- `elapsed` is already
  /// delay-exclusive by construction (mirrors `value_get`'s own `elapsed / duration`, which
  /// performs no such subtraction). Subtracting `delay` a second time undercounted progress.
  /// ## Why Not Caught
  /// `progress()` was only ever exercised with zero-delay tweens (where the subtraction is a
  /// no-op) in `test_tween_initial_state`/`test_tween_progress_and_completion`;
  /// `test_tween_with_delay_behavior` uses a nonzero delay but never calls `.progress()`.
  /// ## Fix Applied
  /// Changed `( self.elapsed - self.delay ) / self.duration` to `self.elapsed / self.duration`,
  /// matching `value_get`'s `normalized_time` formula exactly. See `interpolation.rs`.
  /// ## Prevention
  /// Added this test, which drives a delayed tween all the way to `Completed` and checks
  /// `progress()` reports `1.0` there, not undercounted by the delay.
  /// ## Pitfall
  /// Invisible for zero-delay tweens (the subtraction is a no-op) and even for a
  /// still-`Running`, not-yet-`Completed` delayed tween checked only via `value_get()` (which
  /// never had the bug) -- only checking `progress()` itself, on a delayed tween, exposes it.
  #[ test ]
  fn test_tween_progress_with_delay_reaches_full_completion()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 0.5 );

    tween.update( 0.5 ); // consumes the delay entirely
    tween.update( 1.0 ); // full duration elapsed -> Completed

    assert_eq!( tween.state(), AnimationState::Completed );
    assert_eq!( tween.value_get(), 10.0 );
    assert_eq!
    (
      tween.progress(), 1.0,
      "a fully-completed delayed tween must report progress 1.0, not undercounted by a second delay subtraction"
    );
  }

  // test_kind: bug_reproducer(BUG-142)
  /// ## Root Cause
  /// `with_duration` clamped its argument with `.max( 0.0 )` instead of the `.max( 0.001 )` floor
  /// `Tween::new` itself uses ("Minimum duration to avoid division by zero") -- `with_duration(
  /// 0.0 )` reintroduced exactly the `elapsed / duration` == `0.0 / 0.0` == NaN case `new`'s own
  /// clamp exists to prevent.
  /// ## Why Not Caught
  /// `with_duration` had zero existing test coverage of any kind, zero-duration or otherwise.
  /// ## Fix Applied
  /// Changed `with_duration`'s clamp from `.max( 0.0 )` to `.max( 0.001 )`, matching `new`'s own
  /// floor exactly. See `interpolation.rs`.
  /// ## Prevention
  /// Added this test, which drives a `with_duration( 0.0 )` tween through one `update()` and
  /// asserts the returned value is finite, not NaN.
  /// ## Pitfall
  /// A builder method re-deriving a sibling constructor's documented invariant must copy the
  /// sibling's actual clamp value, not just its direction (`.max`) -- `0.0` still satisfies
  /// "non-negative" while silently reintroducing the exact division-by-zero the invariant exists
  /// to prevent.
  #[ test ]
  fn test_tween_with_duration_zero_does_not_produce_nan()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_duration( 0.0 );

    let value = tween.update( 0.1 );

    assert!( !value.is_nan(), "Tween::update produced NaN after with_duration( 0.0 )" );
    assert_eq!( tween.state(), AnimationState::Completed );
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

  // test_kind: bug_reproducer(BUG-352)
  /// ## Root Cause
  /// `Tween::pause` gated the `Paused` transition on `state == Running` only, so calling it
  /// while still mid-delay ( `state == Pending`, `with_delay(...)`'s countdown not yet finished )
  /// was a silent no-op -- `update`'s own match never early-returns for `Pending`, so a later
  /// `update` kept ticking the delay ( and, once it expired, the animation itself ) forward
  /// exactly as if `pause()` had never been called.
  /// ## Why Not Caught
  /// The only existing pause/resume test ( `test_tween_pause_resume` ) pauses only after the
  /// Tween is already `Running` -- no test ever called `.pause()` while a delayed Tween was
  /// still `Pending`.
  /// ## Fix Applied
  /// Widened `pause`'s gate to `matches!( self.state, Running | Pending )`, and changed `resume`
  /// to restore `Pending` ( not unconditionally `Running` ) whenever `self.remain` still holds
  /// leftover delay, mirroring `update`'s own `Pending` arm. See `interpolation.rs`.
  /// ## Prevention
  /// Added this test, which pauses a delayed Tween mid-countdown, drives a large `update()`
  /// while paused and asserts it stays frozen, then resumes and confirms the remaining delay --
  /// not the full duration -- is what is left to consume before the animation itself starts.
  /// ## Pitfall
  /// Invisible for any zero-delay Tween ( `Pending` is a one-tick pass-through there, see
  /// `update`'s `else { self.state = Running }` branch ) and for any caller that only ever
  /// pauses after the first `update()` call already returned `Running` -- only a caller pausing
  /// during an active `.with_delay(...)` countdown exposes it.
  #[ test ]
  fn test_tween_pause_during_pending_delay_freezes_and_resume_preserves_remaining_delay()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 5.0 );

    tween.update( 1.0 ); // still Pending, 4.0s of delay left
    assert_eq!( tween.state(), AnimationState::Pending );

    tween.pause();
    assert_eq!( tween.state(), AnimationState::Paused, "pause() was a no-op while still Pending" );

    let value = tween.update( 10.0 ); // large update while paused -- must not advance anything
    assert_eq!( tween.state(), AnimationState::Paused );
    assert_eq!( value, 0.0_f32, "a paused Tween advanced past its start value" );
    assert!( !tween.is_completed() );

    tween.resume();
    assert_eq!
    (
      tween.state(), AnimationState::Pending,
      "resume() jumped straight to Running, skipping the remaining delay"
    );

    // 2.0s of the remaining 4.0s delay -- still short of it, must stay Pending.
    let value = tween.update( 2.0 );
    assert_eq!( tween.state(), AnimationState::Pending );
    assert_eq!( value, 0.0_f32 );

    // The final 2.0s exhausts the remaining delay exactly; the animation itself has not yet
    // accumulated any elapsed time in this same call.
    let value = tween.update( 2.0 );
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( value, 0.0_f32, "animation value advanced before the resumed delay fully elapsed" );

    // Sanity: the animation now genuinely progresses on subsequent updates.
    let value = tween.update( 0.5 );
    assert_eq!( value, 5.0_f32 );
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

  // test_kind: bug_reproducer(BUG-232)
  /// ## Root Cause
  /// `repeat_handle`'s finite-repeat branch added the whole of `elapsed_repeats` (every repeat
  /// boundary crossed by this single `update()` call) to `current_repeat` unconditionally, with
  /// no check against `repeat_count`. A single large `delta_time` that crosses more repeat
  /// boundaries than remain in the budget let `current_repeat` overshoot past `repeat_count`
  /// while `state` stayed `Running` -- the crossing that should have completed the Tween instead
  /// let it silently run extra, unrequested loops.
  /// ## Why Not Caught
  /// Every existing repeat test drives `update()` one boundary crossing at a time (`delta_time`
  /// close to `duration`); none ever passed a single `delta_time` large enough to cross more
  /// repeat boundaries than the configured `repeat_count` allows in one call.
  /// ## Fix Applied
  /// The finite-repeat branch now compares `elapsed_repeats` against the remaining budget
  /// (`repeat_count - current_repeat`); if this call's crossings exceed it, `current_repeat` is
  /// capped at `repeat_count` and the Tween completes immediately (`elapsed = duration`),
  /// discarding the extra crossings -- matching what processing them one at a time would have
  /// produced. See `interpolation.rs`.
  /// ## Prevention
  /// Added this test, which drives a `repeat_count( 2 )` Tween past its entire budget in one
  /// oversized `update()` call and asserts it completes with `current_repeat()` capped at
  /// exactly `2`, not overshot to the raw crossed-boundary count.
  /// ## Pitfall
  /// Batching multiple repeat-boundary crossings into one `update()` call must reproduce the
  /// same outcome as processing them individually -- a budget check applied only per-increment
  /// (never against the batch total) lets a single large `delta_time` silently exceed a bound
  /// that per-frame deltas would have respected.
  #[ test ]
  fn test_tween_finite_repeat_large_delta_completes_without_overshooting_repeat_count()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_repeat( 2 );

    // Crosses 3 repeat boundaries in one call -- only 2 repeats are allowed.
    tween.update( 3.5 );

    assert!( tween.is_completed(), "large delta_time should complete the Tween once its repeat budget is exhausted" );
    assert_eq!( tween.current_repeat(), 2, "current_repeat overshot repeat_count instead of being capped" );
    assert_eq!( tween.time(), 1.0, "elapsed should snap to duration on completion, not retain leftover overshoot" );
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

  // test_kind: bug_reproducer(BUG-143)
  /// ## Root Cause
  /// `[Tween<T>; N]::progress()` reconstructed "elapsed since the group's own start" from
  /// `self[ 0 ].time() - self.delay_get()` -- omitting `self[ 0 ]`'s own delay entirely, so the
  /// result is wrong from the very first tick whenever element 0's delay differs from the
  /// group's earliest delay (`delay_get()`), regardless of whether anything has completed yet.
  /// ## Why Not Caught
  /// No existing test called `.progress()` on a `[Tween<T>; N]` array at all.
  /// ## Fix Applied
  /// Changed the numerator to reconstruct elapsed time from the element that determines the
  /// group's own end (`max` by `delay + duration`, matching `duration_get()`'s own aggregation),
  /// including that element's `delay`. See `interpolation.rs`.
  /// ## Prevention
  /// Added this test: two tweens with different delays and durations, checked mid-animation
  /// (before either completes) against the true global-elapsed-time answer.
  /// ## Pitfall
  /// The wrong formula doesn't panic or return an out-of-range value -- it silently returns a
  /// plausible-looking but wrong fraction, and the error is present immediately, not only once
  /// some array element individually completes.
  #[ test ]
  fn test_tween_array_progress_uses_last_to_finish_element()
  {
    let mut tweens : [ Tween< f32 >; 2 ] =
    [
      Tween::new( 0.0_f32, 1.0_f32, 2.0, Linear::build() ).with_delay( 2.0 ), // ends at t = 4.0
      Tween::new( 0.0_f32, 1.0_f32, 6.0, Linear::build() ),                   // ends at t = 6.0
    ];

    tweens.update( 3.0 ); // global t = 3.0 out of the group's 6.0 span -- neither tween completed yet

    assert_eq!( tweens.progress(), 0.5 );
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
