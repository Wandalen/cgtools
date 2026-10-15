//! Unit pins for [`browser_input::events_apply_to_state`]'s keyboard-key
//! bookkeeping — covers BUG-213's keyboard half: `KeyboardKey::from` maps every
//! unrecognized `code` string to the single fallback variant `Unidentified`, so
//! a flat `keyboard_keys[Unidentified]` bool could not tell two different
//! unmapped physical keys apart.

use browser_input::*;
use browser_input::keyboard::KeyboardKey;

fn ev( event_type : EventType ) -> Event
{
  Event::new( event_type, false, false, false )
}

fn press( key : KeyboardKey ) -> Event
{
  ev( EventType::KeyboardKey( key, Action::Press ) )
}

fn release( key : KeyboardKey ) -> Event
{
  ev( EventType::KeyboardKey( key, Action::Release ) )
}

/// Sanity pin (not a bug reproducer): an individually-mapped key, which never
/// aliases with anything else, must keep its simple level-per-press/release
/// behavior unchanged by the BUG-213 fix's `Unidentified`-only branch.
#[ test ]
fn a_normally_mapped_key_is_unaffected_by_the_unidentified_counting_fix()
{
  let mut state = State::new();

  events_apply_to_state( &mut state, &[ press( KeyboardKey::Space ) ] );
  assert!( state.keyboard_keys[ KeyboardKey::Space as usize ] );

  events_apply_to_state( &mut state, &[ release( KeyboardKey::Space ) ] );
  assert!( !state.keyboard_keys[ KeyboardKey::Space as usize ] );
}

// test_kind: bug_reproducer(BUG-213)
/// ## Root Cause
/// `KeyboardKey::from` maps any `code` string it does not specifically
/// recognize to the single fallback variant `Unidentified` -- a flat
/// `keyboard_keys[Unidentified]` bool keyed by that one collapsed
/// discriminant cannot tell "one aliased key held" from "two DIFFERENT
/// aliased keys held": releasing either one cleared the shared slot, falsely
/// dropping the other's still-held state.
///
/// ## Why Not Caught
/// No test existed for `KeyboardKey::Unidentified` at all -- this crate had no
/// dedicated keyboard-key state test file prior to this bug.
///
/// ## Fix Applied
/// `State::unidentified_key_hold_count` now counts how many aliased presses
/// are outstanding (`src/input.rs`); an `Unidentified` release only clears
/// `keyboard_keys[Unidentified]` once the count reaches zero. `saturating_sub`
/// guards a spurious release with no matching press from underflowing.
///
/// ## Prevention
/// Any many-to-one fallback variant (a "catch-all" enum case) breaks a flat
/// bit/bool the instant two distinct real inputs alias to it simultaneously --
/// such variants need a hold-COUNT, not a hold-bit. See the identical
/// `MouseButton::Unknown` fix in `mouse_button_state_test.rs`.
///
/// ## Pitfall
/// Indistinguishable from the ordinary single-key case as long as only one
/// exotic/unmapped key is ever held at a time -- only two SIMULTANEOUS
/// aliased keys expose the false-clear. Also has an OS-key-repeat interaction:
/// see `Input::new`'s `keyboard_callback`, which filters `event.repeat()`
/// before this counting logic ever runs, so this unit test (which constructs
/// `EventType` directly, bypassing that DOM-level filter) does not need to and
/// cannot exercise the repeat case -- see `tests/manual/readme.md`.
#[ test ]
fn releasing_one_aliased_unidentified_key_does_not_clear_another_still_held()
{
  let mut state = State::new();

  events_apply_to_state( &mut state, &[ press( KeyboardKey::Unidentified ), press( KeyboardKey::Unidentified ) ] );
  assert!( state.keyboard_keys[ KeyboardKey::Unidentified as usize ], "both aliased presses register as held" );

  events_apply_to_state( &mut state, &[ release( KeyboardKey::Unidentified ) ] );
  assert!
  (
    state.keyboard_keys[ KeyboardKey::Unidentified as usize ],
    "one aliased key is still physically held -- releasing the other must not \
     clear the shared Unidentified slot \
     (buggy code would have set this false after the first release)"
  );

  events_apply_to_state( &mut state, &[ release( KeyboardKey::Unidentified ) ] );
  assert!( !state.keyboard_keys[ KeyboardKey::Unidentified as usize ], "both released -- now correctly false" );
}

/// Sanity pin (not a bug reproducer): a spurious `Unidentified` release with no
/// matching prior press must not underflow the hold-count or panic.
#[ test ]
fn a_spurious_unidentified_release_with_no_prior_press_does_not_panic()
{
  let mut state = State::new();

  events_apply_to_state( &mut state, &[ release( KeyboardKey::Unidentified ) ] );
  assert!( !state.keyboard_keys[ KeyboardKey::Unidentified as usize ] );
}
