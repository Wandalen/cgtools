//! Integration tests related to Sequencer struct

#![ allow( clippy::float_cmp ) ]

#[ cfg( test ) ]
mod tests
{
  use animation::
  {
    Tween,
    Sequencer,
    Sequence,
    SequenceError,
    AnimationState,
    easing::
    {
      base::EasingBuilder,
      Linear,
      cubic::bezier::EaseInSine
    }
  };

  fn assert_f_eq( first : f64, second : f64, eps : f64 )
  {
    assert!( second - eps < first && first < second + eps );
  }

  #[ test ]
  fn test_sequencer_basic_flow()
  {
    let mut sequencer = Sequencer::new();

    assert_eq!( sequencer.state(), AnimationState::Pending );
    assert_eq!( sequencer.animation_count(), 0 );

    let float_tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );
    sequencer.insert( "test", float_tween );

    assert_eq!( sequencer.state(), AnimationState::Running );
    assert_eq!( sequencer.animation_count(), 1 );
    assert!( !sequencer.is_completed() );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.time(), 0.5 );
    assert_eq!( sequencer.state(), AnimationState::Running );

    let value = sequencer.get::< Tween< f32 > >( "test" ).unwrap();
    assert_eq!( value.value_get(), 5.0 );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.time(), 1.0 );

    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.state(), AnimationState::Completed );
  }

  #[ test ]
  fn test_sequencer_multiple_tweens()
  {
    let mut sequencer = Sequencer::new();

    let tween1 = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );
    let tween2 = Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::new() );
    sequencer.insert( "short_tween", tween1 );
    sequencer.insert( "long_tween", tween2 );

    sequencer.update( 1.5 );

    assert!( !sequencer.is_completed() );
    assert_eq!( sequencer.state(), AnimationState::Running );
    assert_eq!( sequencer.time(), 1.5 );

    sequencer.update( 0.5 );

    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.time(), 2.0 );
    assert_eq!( sequencer.state(), AnimationState::Completed );
  }

  #[ test ]
  fn test_sequencer_pause_resume()
  {
    let mut sequencer = Sequencer::new();
    sequencer.insert
    (
      "test",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.get::< Tween< f32 > >( "test" ).unwrap().value_get(), 5.0 );

    sequencer.pause();
    assert_eq!( sequencer.state(), AnimationState::Paused );

    sequencer.update( 0.5 );
    let value = sequencer.get::< Tween< f32 > >( "test" ).unwrap();
    assert_eq!( value.value_get(), 5.0 );

    sequencer.resume();
    assert_eq!( sequencer.state(), AnimationState::Running );

    sequencer.update( 0.5 );
    assert!( sequencer.is_completed() );
    let value = sequencer.get::< Tween< f32 > >( "test" ).unwrap();
    assert_eq!( value.value_get(), 10.0 );
  }

  #[ test ]
  fn test_sequencer_reset()
  {
    let mut sequencer = Sequencer::new();
    sequencer.insert
    (
      "test",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.time(), 0.5 );
    assert_eq!( sequencer.get::< Tween< f32 > >( "test" ).unwrap().value_get(), 5.0 );

    sequencer.reset();

    assert_eq!( sequencer.time(), 0.0 );
    assert_eq!( sequencer.state(), AnimationState::Running );
    assert_eq!( sequencer.get::< Tween< f32 > >( "test" ).unwrap().value_get(), 0.0 );

    sequencer.update( 1.0 );
    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.get::< Tween< f32 > >( "test" ).unwrap().value_get(), 10.0 );
  }

  #[ test ]
  fn test_sequencer_remove()
  {
    let mut sequencer = Sequencer::new();

    sequencer.insert
    (
      "tween1",
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() )
    );
    sequencer.insert
    (
      "tween2",
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() )
    );
    assert_eq!( sequencer.animation_count(), 2 );

    assert!( sequencer.remove( "tween1" ) );
    assert_eq!( sequencer.animation_count(), 1 );

    assert!( sequencer.get::< Tween< f32 > >( "tween1" ).is_none() );
    assert!( sequencer.get::< Tween< f32 > >( "tween2" ).is_some() );

    assert!( !sequencer.remove( "tween1" ) );
  }

  #[ test ]
  fn test_sequencer_get_wrong_type()
  {
    let mut sequencer = Sequencer::new();

    sequencer.insert
    (
      "float_tween",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    );

    assert!( sequencer.get::< Tween< i32 > >( "float_tween" ).is_none() );

    assert!( sequencer.get::< Tween< f32 > >( "float_tween" ).is_some() );
  }

  #[ test ]
  fn test_sequencer_ease_in()
  {
    let mut sequencer = Sequencer::new();

    sequencer.insert
    (
      "ease_in_tween",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, EaseInSine::new() )
    );

    sequencer.update( 0.5 );

    let value = sequencer.get::< Tween< f32 > >( "ease_in_tween" ).unwrap();
    // Fix(TASK-041): this literal was 1.25 — the value CubicBezier produced at the old buggy
    // default of iterations=0, where the Newton-Raphson solve loop never ran and `apply`
    // evaluated `y_get` at the raw input fraction ( 0.5 ) instead of the solved Bezier parameter.
    // At the fixed default of iterations=8, EaseInSine( 0.5 ) converges to ~0.300338, not 0.5 —
    // see easing/cubic/bezier.rs.
    assert_f_eq( f64::from( value.value_get() ), 3.00338, 0.001 );

    sequencer.update( 0.5 );
    assert!( sequencer.is_completed() );
    let value = sequencer.get::< Tween< f32 > >( "ease_in_tween" ).unwrap();
    assert_eq!( value.value_get(), 10.0 );
  }

  // test_kind: bug_reproducer(TASK-015)
  /// ## Root Cause
  /// `Sequencer::delay_get` seeded its reduction at `f64::MAX` (correct for a min-reduction) but
  /// reduced via `.max( min_delay )` instead of `.min( min_delay )`, so no real delay could ever
  /// displace the seed — it always returned `f64::MAX`.
  /// ## Why Not Caught
  /// No existing test inserted a delayed player and then checked `Sequencer::delay_get()` or
  /// `progress()` directly — coverage only checked `time()` and per-player `value_get()`.
  /// ## Fix Applied
  /// Changed the reduction from `.max( min_delay )` to `.min( min_delay )`. See `sequencer.rs`.
  /// ## Prevention
  /// A min-reduction's seed and its comparison direction must be reviewed together — a correct
  /// seed with a mismatched comparison direction is easy to miss on a glance.
  /// ## Pitfall
  /// `f64::MAX` feeding into `progress()`'s `( time - delay_get() ) / duration_get()` produces a
  /// huge negative number that clamps silently to `0.0` — no panic, no NaN, just a
  /// plausible-looking wrong answer.
  #[ test ]
  fn test_sequencer_delay_get_and_progress_with_delayed_tween()
  {
    let mut sequencer = Sequencer::new();
    sequencer.insert
    (
      "tween",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() ).with_delay( 0.5 )
    );

    assert_eq!( sequencer.delay_get(), 0.5 );

    sequencer.update( 1.0 );
    assert_f_eq( sequencer.progress(), 0.5, 0.0001 );
  }

  // test_kind: bug_reproducer(TASK-015)
  /// ## Root Cause
  /// `Sequence::new`'s validation loop declared `last_delay` immutable and never reassigned it,
  /// so every iteration compared against the initial `0.0` instead of the previous player's
  /// delay — the `Unsorted` check could only fire for a negative delay, which `delay_get()`
  /// never produces.
  /// ## Why Not Caught
  /// No existing test constructed genuinely out-of-order players to exercise the `Unsorted`
  /// branch — coverage only used already-sorted input.
  /// ## Fix Applied
  /// Made `last_delay` mutable and added `last_delay = player.delay_get();` at the end of the
  /// loop body. See `sequencer.rs`.
  /// ## Prevention
  /// A "compare against previous" loop must visibly update its "previous" binding every
  /// iteration — an immutable binding used this way is a signal the update was forgotten, not
  /// that none was needed.
  /// ## Pitfall
  /// The check reads as correct at a glance (right comparison operator, right error variant) —
  /// only the absence of the reassignment reveals it can never trigger on realistic input.
  #[ test ]
  fn test_sequence_new_rejects_unsorted_players()
  {
    let players = vec!
    [
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() ).with_delay( 2.0 ),
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() ).with_delay( 1.0 ),
    ];

    let result = Sequence::new( players );
    assert!( matches!( result, Err( SequenceError::Unsorted ) ) );
  }
}
