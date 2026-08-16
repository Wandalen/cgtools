//! Integration tests related to Sequencer struct
#![ expect( clippy::float_cmp, reason = "assertions check deterministic tween/sequencer arithmetic against exact expected values" ) ]

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
    AnimatablePlayer,
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

    let float_tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() );
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

    let tween1 = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() );
    let tween2 = Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::build() );
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
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
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
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
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
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() )
    );
    sequencer.insert
    (
      "tween2",
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() )
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
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
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
      Tween::new( 0.0_f32, 10.0_f32, 1.0, EaseInSine::build() )
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
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 0.5 )
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
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() ).with_delay( 2.0 ),
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::build() ).with_delay( 1.0 ),
    ];

    let result = Sequence::new( players );
    assert!( matches!( result, Err( SequenceError::Unsorted ) ) );
  }

  // test_kind: bug_reproducer(BUG-138)
  /// ## Root Cause
  /// `Sequence::update` used `binary_search_by`'s `Err( id )` insertion-point directly as the
  /// active player index. `Err( id )` means "the first player whose delay has NOT yet been
  /// reached" -- the player that should actually be active is the one just before it, `id - 1`.
  /// ## Why Not Caught
  /// No existing test called `.update()` on a valid, multi-player `Sequence` -- the only
  /// `Sequence` test (`test_sequence_new_rejects_unsorted_players`) exercises only the
  /// constructor's error path.
  /// ## Fix Applied
  /// Changed `Ok( id ) | Err( id ) => id` to `Ok( id ) => id, Err( id ) => id.saturating_sub( 1 )`.
  /// See `sequencer.rs`.
  /// ## Prevention
  /// Added this test, which fails loudly (wrong player index, wrong value) whenever the very
  /// first frame of a multi-player `Sequence` is processed.
  /// ## Pitfall
  /// `Ok( id )` (an exact delay match) and `Err( id )` (an insertion point) look interchangeable
  /// at a glance but have different index semantics -- only `Err` needs the `- 1` adjustment.
  #[ test ]
  fn test_sequence_update_selects_player_whose_delay_has_already_passed()
  {
    let players = vec!
    [
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ),
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 1.0 ),
    ];
    let mut sequence = Sequence::new( players ).unwrap();

    // elapsed = 0.1, strictly between the two players' delays ( 0.0 and 1.0 ) -- player 0's
    // delay has passed, player 1's has not, so player 0 must be the active one.
    sequence.update( 0.1 );

    assert_eq!
    (
      sequence.current_id_get(), 0,
      "binary_search's Err(id) insertion-point was used directly instead of id-1, selecting the wrong (not-yet-started) player"
    );
    assert_eq!( sequence.current_get().unwrap().value_get(), 1.0 );
  }

  // test_kind: bug_reproducer(BUG-139)
  /// ## Root Cause
  /// `Sequence::update`'s `Ordering::Equal` arm (the same player still active across frames)
  /// reconstructed an absolute "elapsed since this player started" value from
  /// `delay_get() + progress() * duration_get()` and called `current.update( old_elapsed +
  /// delta_time )` -- but `AnimatablePlayer::update` is a pure incremental delta API. Every
  /// steady-state frame after the player left its initial `Pending` state, this re-fed the
  /// player's own already-accumulated progress back into itself on top of the real delta.
  /// ## Why Not Caught
  /// No existing test called `.update()` twice in a row on a `Sequence` while the same player
  /// stayed active -- the only `Sequence` test using `.update()` prior to this session's own
  /// BUG-138 fix never exercised the `Equal` arm past a player's initial `Pending` state.
  /// ## Fix Applied
  /// Replaced the `old_elapsed` reconstruction with a direct `current.update( delta_time )`,
  /// matching `AnimatablePlayer::update`'s incremental contract. See `sequencer.rs`.
  /// ## Prevention
  /// Added this test, which drives the same player across two consecutive frames and checks its
  /// resulting value against the wall-clock-correct expectation.
  /// ## Pitfall
  /// The bug is invisible on a player's very first `update()` call (still `Pending`, so
  /// `progress()` returns `0.0` and `old_elapsed` degenerates to `0.0`) -- only a SECOND
  /// consecutive frame on the same already-`Running` player exposes the over-accumulation.
  #[ test ]
  fn test_sequence_update_continuing_player_receives_only_the_new_delta()
  {
    let players = vec!
    [
      Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::build() ),
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 2.0 ),
    ];
    let mut sequence = Sequence::new( players ).unwrap();

    sequence.update( 0.5 ); // frame 1: player 0 Pending -> Running, elapsed becomes 0.5
    sequence.update( 0.5 ); // frame 2: player 0 still active (Equal arm) -- must add only 0.5

    assert_eq!( sequence.current_id_get(), 0 );
    assert_eq!
    (
      sequence.current_get().unwrap().value_get(), 5.0,
      "continuing player's internal elapsed over-accumulated -- old (reconstructed) elapsed was added to the new delta instead of just the delta"
    );
  }

  // test_kind: bug_reproducer(BUG-147)
  /// ## Root Cause
  /// `Sequencer::insert`'s state-revival guard only checks `self.state == AnimationState::Pending`,
  /// never `AnimationState::Completed`. Once every contained player finishes and `update()` flips
  /// the Sequencer to `Completed`, inserting a brand-new (not-yet-run) player leaves `state` stuck
  /// at `Completed` -- `is_completed()` keeps reporting `true` despite an incomplete player now
  /// being present, and `update()` early-returns on every subsequent call (`if self.state !=
  /// Running { return; }`), so the new player's `update()` is never invoked at all.
  /// ## Why Not Caught
  /// No existing test ever called `insert()` after a `Sequencer` reached `Completed` -- every
  /// prior test either stopped once `is_completed()` became `true`, or only inserted players
  /// while still `Pending`/`Running`.
  /// ## Fix Applied
  /// Widened the revival guard from `self.state == AnimationState::Pending` to also cover
  /// `AnimationState::Completed`. See `sequencer.rs`.
  /// ## Prevention
  /// Added this test, which inserts a second player only after the first has driven the
  /// `Sequencer` to `Completed`, and asserts both `is_completed()` flips back to `false` and the
  /// new player actually advances on the next `update()`.
  /// ## Pitfall
  /// Invisible whenever a `Sequencer` instance is used for exactly one "batch" of players and
  /// discarded once complete -- only a `Sequencer` reused across independent batches (insert more
  /// work after a previous batch finished) exposes the stuck `Completed` state.
  #[ test ]
  fn test_sequencer_insert_after_completion_revives_running_state()
  {
    let mut sequencer = Sequencer::new();
    sequencer.insert( "first", Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ) );

    sequencer.update( 1.0 );
    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.state(), AnimationState::Completed );

    sequencer.insert( "second", Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ) );

    assert!
    (
      !sequencer.is_completed(),
      "is_completed() still true right after inserting a fresh, not-yet-run player"
    );
    assert_eq!( sequencer.state(), AnimationState::Running );

    sequencer.update( 0.5 );
    let value = sequencer.get::< Tween< f32 > >( "second" ).unwrap();
    assert_eq!
    (
      value.value_get(), 5.0,
      "newly-inserted player never advanced -- Sequencer::update() early-returns while state is stuck at Completed"
    );
  }
}
