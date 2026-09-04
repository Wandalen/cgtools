//! Unit pins for [`browser_input::events_apply_to_state`]'s per-pointer button
//! bookkeeping — covers BUG-130: `mouse_buttons`/`active_pointers` were updated
//! as if exactly one button is ever held by one pointer at a time, breaking as
//! soon as two pointers share a button or one pointer holds two buttons.
//! Also covers BUG-212 (`held_buttons` grew without the same cap
//! `active_pointers` already enforced) and BUG-213's mouse half (the
//! `MouseButton::Unknown` fallback variant aliasing two distinct real buttons
//! onto one flat bit).

use browser_input::*;
use browser_input::mouse::MouseButton;
use ndarray_cg::I32x2;

fn ev( event_type : EventType ) -> Event
{
  Event::new( event_type, false, false, false )
}

fn point( x : i32, y : i32 ) -> I32x2
{
  I32x2::from_array( [ x, y ] )
}

fn press( id : i32, button : MouseButton, x : i32, y : i32 ) -> Event
{
  ev( EventType::PointerButton( id, point( x, y ), button, Action::Press ) )
}

fn release( id : i32, button : MouseButton, x : i32, y : i32 ) -> Event
{
  ev( EventType::PointerButton( id, point( x, y ), button, Action::Release ) )
}

fn cancel( id : i32 ) -> Event
{
  ev( EventType::PointerCancel( id ) )
}

// test_kind: bug_reproducer(BUG-130)
/// ## Root Cause
/// `mouse_buttons[button]` was overwritten unconditionally by every
/// `PointerButton` event, keyed only by the button value -- never by
/// `pointer_id`. Per the Pointer Events spec, a touch contact's `button` is
/// always `Main` (`0`), so two simultaneous touches both report `Main`;
/// releasing either one set `mouse_buttons[Main] = false`, even while the
/// other touch was still physically down.
///
/// ## Why Not Caught
/// No existing test exercised two different pointer ids pressing the *same*
/// button at once -- `active_pointers_test.rs`'s multi-pointer cases only used
/// `MouseButton::Main` for one id at a time, and its two-simultaneous-id test
/// never checked `mouse_buttons`, only `active_pointers`.
///
/// ## Fix Applied
/// `mouse_buttons[button]` is now derived from a per-pointer `held_buttons`
/// bitmask map (`src/input.rs`), recomputed as "does any currently-tracked
/// pointer still hold this button" after every press/release/cancel.
///
/// ## Prevention
/// Global input state that is "set" by the latest event instead of "derived
/// from the union of all current sources" silently breaks the instant two
/// sources can overlap -- test the simultaneous case explicitly, not just
/// sequential press/release pairs.
///
/// ## Pitfall
/// The bug is invisible with a single pointer (the overwhelmingly common
/// desktop-mouse case) and only manifests once two pointers are live at once
/// -- exactly the multi-touch scenario this crate's own `active_pointers` API
/// exists to support.
#[ test ]
fn releasing_one_pointer_does_not_clear_a_button_another_pointer_still_holds()
{
  let mut state = State::new();

  events_apply_to_state( &mut state, &[ press( 1, MouseButton::Main, 0, 0 ), press( 2, MouseButton::Main, 10, 10 ) ] );
  assert!( state.mouse_buttons[ MouseButton::Main as usize ], "both pointers hold Main" );

  events_apply_to_state( &mut state, &[ release( 1, MouseButton::Main, 0, 0 ) ] );
  assert!
  (
    state.mouse_buttons[ MouseButton::Main as usize ],
    "pointer 2 still holds Main -- releasing pointer 1 must not clear it \
     (buggy code would have set this false)"
  );

  events_apply_to_state( &mut state, &[ release( 2, MouseButton::Main, 10, 10 ) ] );
  assert!( !state.mouse_buttons[ MouseButton::Main as usize ], "both released -- now correctly false" );
}

// test_kind: bug_reproducer(BUG-130)
/// ## Root Cause
/// `active_pointers.retain( |(id,_)| *id != *pointer_id )` fired on ANY
/// `Release`, regardless of whether other buttons were still held under the
/// same `pointer_id` -- true for a single physical mouse holding two buttons
/// at once (both press/release events share one stable `pointer_id`).
///
/// ## Why Not Caught
/// No existing test pressed two *different* buttons under the *same*
/// `pointer_id` -- `active_pointers_test.rs`'s multi-pointer tests always used
/// distinct ids, never distinct buttons on one id.
///
/// ## Fix Applied
/// `active_pointers` now only evicts a pointer id once its `held_buttons` mask
/// reaches zero (`src/input.rs`), not on the first release of any one button.
///
/// ## Prevention
/// A DOM `pointerId` is stable across all of one device's buttons -- press/
/// release pairing must be tracked per-button, not assumed 1:1 with the
/// pointer's own lifecycle.
///
/// ## Pitfall
/// Looks correct for touch (one press, one release, one button, always) and
/// for a single-button mouse click -- only a multi-button mouse held with more
/// than one button down at once exposes the premature eviction.
#[ test ]
fn releasing_one_button_does_not_evict_a_pointer_still_holding_another()
{
  let mut state = State::new();

  events_apply_to_state( &mut state, &[ press( 1, MouseButton::Main, 5, 5 ), press( 1, MouseButton::Secondary, 5, 5 ) ] );
  assert_eq!( state.active_pointers, [ ( 1, point( 5, 5 ) ) ], "one pointer, two buttons -- one entry" );

  events_apply_to_state( &mut state, &[ release( 1, MouseButton::Main, 5, 5 ) ] );
  assert_eq!
  (
    state.active_pointers,
    [ ( 1, point( 5, 5 ) ) ],
    "Secondary is still held -- releasing Main must not evict pointer 1 \
     (buggy code would have removed this entry)"
  );
  assert!( !state.mouse_buttons[ MouseButton::Main as usize ], "Main itself is correctly now released" );
  assert!( state.mouse_buttons[ MouseButton::Secondary as usize ], "Secondary is untouched by Main's release" );

  events_apply_to_state( &mut state, &[ release( 1, MouseButton::Secondary, 5, 5 ) ] );
  assert!( state.active_pointers.is_empty(), "last button released -- now correctly evicted" );
}

// test_kind: bug_reproducer(BUG-130)
/// ## Root Cause
/// `PointerCancel` only cleared `mouse_buttons` (via `.fill(false)`) when
/// `active_pointers` became fully empty -- if a *different* pointer was still
/// active (holding some other button), the cancelled pointer's own button
/// stayed stuck `true` forever, since the coarse `is_empty()` guard never
/// fired again for it.
///
/// ## Why Not Caught
/// `active_pointers_test.rs::cancel_removes_entry` cancels one of two pointers
/// but never checks `mouse_buttons` at all -- only `active_pointers`.
///
/// ## Fix Applied
/// `PointerCancel` now removes only the cancelled pointer's own entry from
/// `held_buttons` and re-derives just the specific buttons it held from the
/// remaining pointers (`src/input.rs`) -- no more all-or-nothing heuristic.
///
/// ## Prevention
/// "If the aggregate is empty, reset everything" is only sound when the
/// aggregate and the state being reset change in lockstep -- `active_pointers`
/// (per-pointer) and `mouse_buttons` (per-button) diverge as soon as more than
/// one pointer can be active at once.
///
/// ## Pitfall
/// Single-pointer cancellation (the common case) always leaves
/// `active_pointers` empty afterward, so the bug is invisible until a second,
/// unrelated pointer is simultaneously active.
#[ test ]
fn cancel_only_clears_the_cancelled_pointers_own_buttons()
{
  let mut state = State::new();

  events_apply_to_state
  (
    &mut state,
    &[ press( 1, MouseButton::Main, 0, 0 ), press( 2, MouseButton::Secondary, 20, 20 ) ]
  );
  assert!( state.mouse_buttons[ MouseButton::Main as usize ] );
  assert!( state.mouse_buttons[ MouseButton::Secondary as usize ] );

  events_apply_to_state( &mut state, &[ cancel( 1 ) ] );

  assert_eq!( state.active_pointers, [ ( 2, point( 20, 20 ) ) ], "only pointer 1 is evicted" );
  assert!
  (
    !state.mouse_buttons[ MouseButton::Main as usize ],
    "pointer 1's own button must clear on its cancel \
     (buggy code would leave this stuck true: active_pointers is non-empty, \
     so the old is_empty() guard never fires)"
  );
  assert!
  (
    state.mouse_buttons[ MouseButton::Secondary as usize ],
    "pointer 2's unrelated button must be untouched by pointer 1's cancel"
  );
}

// test_kind: bug_reproducer(BUG-212)
/// ## Root Cause
/// `held_buttons` inserted a new pointer id on every `Press` unconditionally,
/// while `active_pointers` already gated new insertions behind
/// `MAX_ACTIVE_POINTERS` (32) -- once more than the cap's worth of distinct
/// pointer ids had pressed a button, `held_buttons` kept growing (and kept
/// contributing to `mouse_buttons`) even for ids `active_pointers` had already
/// started silently ignoring.
///
/// ## Why Not Caught
/// `tests/manual/readme.md`'s "Excessive Pointer Flood" scenario only checked
/// `active_pointers().length` capping -- `held_buttons` is a private field with
/// no existing test observing its effect on `mouse_buttons` under a flood.
///
/// ## Fix Applied
/// `held_buttons` insertion on `Press` is now gated behind the identical
/// "already tracked or under `MAX_ACTIVE_POINTERS`" check `active_pointers`
/// already used (`src/input.rs`) -- a pointer id beyond the cap is now
/// invisible to both collections consistently, not just one of them.
///
/// ## Prevention
/// Two collections meant to track the same conceptual set (currently-active
/// pointer ids) must share the exact same admission check -- a cap added to
/// one sibling collection is not automatically inherited by another.
///
/// ## Pitfall
/// `held_buttons` is a private field, so this can only be observed indirectly
/// through `mouse_buttons` -- with `MAX_ACTIVE_POINTERS` or fewer pointers
/// live, `held_buttons` and `active_pointers` always agreed, hiding the
/// missing cap until a flood actually exceeded it.
#[ test ]
fn held_buttons_respects_the_same_cap_as_active_pointers()
{
  let mut state = State::new();

  // Fill up to the cap (32), then press one more distinct pointer id on top.
  let presses : Vec< Event > = ( 1_i32 ..= 33 )
    .map( | id | press( id, MouseButton::Main, id, id ) )
    .collect();
  events_apply_to_state( &mut state, &presses );

  assert_eq!( state.active_pointers.len(), 32, "active_pointers correctly caps at MAX_ACTIVE_POINTERS" );
  assert!( state.mouse_buttons[ MouseButton::Main as usize ], "Main is held by the 32 tracked pointers" );

  // Release every pointer id that active_pointers actually admitted (1..=32).
  // Pointer 33 was beyond the cap -- active_pointers never tracked it, so no
  // caller iterating active_pointers would ever know it needs releasing.
  let releases : Vec< Event > = ( 1_i32 ..= 32 )
    .map( | id | release( id, MouseButton::Main, id, id ) )
    .collect();
  events_apply_to_state( &mut state, &releases );

  assert!( state.active_pointers.is_empty(), "all admitted pointers released" );
  assert!
  (
    !state.mouse_buttons[ MouseButton::Main as usize ],
    "Main must now read as released -- pointer 33 was never admitted into \
     tracked state in the first place, so it cannot keep Main stuck held \
     (buggy code would have left this true forever, since held_buttons kept \
     an untracked entry for pointer 33 that active_pointers never exposed)"
  );
}

// test_kind: bug_reproducer(BUG-213)
/// ## Root Cause
/// `MouseButton::from_button` maps every DOM button value outside `0..=4` to
/// the single fallback variant `Unknown` -- a flat `held_buttons` bit keyed by
/// that one collapsed discriminant cannot tell "one aliased button held" from
/// "two DIFFERENT aliased buttons held": releasing either one cleared the
/// shared bit, falsely dropping the other's still-held state.
///
/// ## Why Not Caught
/// No existing test pressed `MouseButton::Unknown` twice under one pointer id
/// before releasing once -- every existing multi-button test used two
/// individually-addressable variants (`Main`/`Secondary`), which don't alias.
///
/// ## Fix Applied
/// A per-pointer `unknown_button_counts` map now counts how many aliased
/// presses are outstanding (`src/input.rs`); an `Unknown` release only clears
/// the shared bit once the count reaches zero.
///
/// ## Prevention
/// Any many-to-one fallback variant (a "catch-all" enum case) breaks a flat
/// bit/bool the instant two distinct real inputs alias to it simultaneously --
/// such variants need a hold-COUNT, not a hold-bit.
///
/// ## Pitfall
/// Indistinguishable from the ordinary single-button case as long as only one
/// exotic/unmapped button is ever held at a time -- only two SIMULTANEOUS
/// aliased buttons under the same pointer expose the false-clear.
#[ test ]
fn releasing_one_aliased_unknown_button_does_not_clear_another_still_held()
{
  let mut state = State::new();

  // Two distinct real DOM buttons (e.g. browser button values 5 and 6) both
  // alias to MouseButton::Unknown -- events_apply_to_state cannot and must
  // not need to tell them apart, only count how many are outstanding.
  events_apply_to_state
  (
    &mut state,
    &[ press( 1, MouseButton::Unknown, 0, 0 ), press( 1, MouseButton::Unknown, 0, 0 ) ]
  );
  assert!( state.mouse_buttons[ MouseButton::Unknown as usize ], "both aliased presses register as held" );

  events_apply_to_state( &mut state, &[ release( 1, MouseButton::Unknown, 0, 0 ) ] );
  assert!
  (
    state.mouse_buttons[ MouseButton::Unknown as usize ],
    "one aliased button is still physically held -- releasing the other must \
     not clear the shared Unknown bit \
     (buggy code would have set this false after the first release)"
  );

  events_apply_to_state( &mut state, &[ release( 1, MouseButton::Unknown, 0, 0 ) ] );
  assert!( !state.mouse_buttons[ MouseButton::Unknown as usize ], "both released -- now correctly false" );
}
