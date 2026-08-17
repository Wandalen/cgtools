//! Unit pins for [`browser_input::events_apply_to_state`]'s handling of
//! [`EventType::FocusLost`] — covers BUG-214: nothing previously reset "currently
//! held" input state when the page lost focus (alt-tab, tab switch), so a
//! key/button held at that moment stayed stuck `true` forever (its matching
//! release is delivered by the OS to whichever window/app now has focus, not
//! this page).
//!
//! This only pins the *state-reset* half of the fix (`events_apply_to_state`'s
//! `FocusLost` arm), which is directly constructible and testable here. The
//! other half -- that a real browser `blur`/`visibilitychange` event actually
//! gets queued as `EventType::FocusLost` -- is DOM listener wiring with no live
//! browser context in `cargo test`; see `tests/manual/readme.md`.

use browser_input::*;
use browser_input::keyboard::KeyboardKey;
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

// test_kind: bug_reproducer(BUG-214)
/// ## Root Cause
/// No event, listener, or code path ever cleared "currently held" state on
/// focus loss -- a key or button held at the moment the OS moved focus away
/// from the page kept its `true`/held entry indefinitely, since the matching
/// release event is delivered to whichever window/app now has focus, never to
/// this page.
///
/// ## Why Not Caught
/// No prior test exercised a focus-loss scenario at all -- every existing
/// test only sent matched press/release (or cancel) pairs, which is exactly
/// the case that never triggers this bug.
///
/// ## Fix Applied
/// A new `EventType::FocusLost` variant, queued by `Input`'s `blur` (window)
/// and `visibilitychange` (document) listeners, resets every "currently held"
/// field in `events_apply_to_state`'s new `FocusLost` arm (`src/input.rs`).
/// Last-known-value/accumulator fields (`pointer_position`, `scroll`) are
/// deliberately left untouched -- they remain meaningful after focus returns.
///
/// ## Prevention
/// Any global "currently held" state fed exclusively by matched press/release
/// event pairs needs an explicit external reset path for the case where the
/// OS delivers the release somewhere else entirely -- focus loss is exactly
/// that case for keyboard and pointer input.
///
/// ## Pitfall
/// Invisible in every normal press/release sequence -- only manifests once
/// focus actually leaves the page mid-hold, which no sequential test can
/// exercise without an explicit `FocusLost` event.
#[ test ]
fn focus_lost_clears_all_currently_held_state()
{
  let mut state = State::new();

  events_apply_to_state
  (
    &mut state,
    &[
      ev( EventType::KeyboardKey( KeyboardKey::Space, Action::Press ) ),
      ev( EventType::PointerButton( 1, point( 5, 5 ), MouseButton::Main, Action::Press ) ),
    ]
  );
  assert!( state.keyboard_keys[ KeyboardKey::Space as usize ], "precondition: key held" );
  assert!( state.mouse_buttons[ MouseButton::Main as usize ], "precondition: button held" );
  assert_eq!( state.active_pointers, [ ( 1, point( 5, 5 ) ) ], "precondition: pointer active" );

  events_apply_to_state( &mut state, &[ ev( EventType::FocusLost ) ] );

  assert!
  (
    !state.keyboard_keys[ KeyboardKey::Space as usize ],
    "key must release on focus loss -- its matching keyup will never arrive"
  );
  assert!
  (
    !state.mouse_buttons[ MouseButton::Main as usize ],
    "button must release on focus loss -- its matching pointerup will never arrive"
  );
  assert!( state.active_pointers.is_empty(), "no pointer contact survives a focus loss" );
}

/// Sanity pin (not a bug reproducer): `FocusLost` must not disturb
/// last-known-value/accumulator fields, which remain meaningful (and are
/// never re-delivered) after focus returns to the page.
#[ test ]
fn focus_lost_does_not_reset_last_known_position_or_accumulated_scroll()
{
  let mut state = State::new();

  let scroll_delta = ndarray_cg::F64x3::new( 0.0, 3.0, 0.0 );
  events_apply_to_state
  (
    &mut state,
    &[
      ev( EventType::PointerMove( 1, point( 42, 24 ) ) ),
      ev( EventType::Wheel( scroll_delta ) ),
    ]
  );
  assert_eq!( state.pointer_position, point( 42, 24 ), "precondition: position recorded" );
  assert_eq!( state.scroll, scroll_delta, "precondition: scroll accumulated" );

  events_apply_to_state( &mut state, &[ ev( EventType::FocusLost ) ] );

  assert_eq!( state.pointer_position, point( 42, 24 ), "last-known position survives a focus loss" );
  assert_eq!( state.scroll, scroll_delta, "accumulated scroll survives a focus loss" );
}

/// Sanity pin (not a bug reproducer): `FocusLost` while nothing is held must
/// not panic (e.g. via an unguarded arithmetic underflow in the internal
/// hold-count bookkeeping it also clears).
#[ test ]
fn focus_lost_with_nothing_held_is_a_harmless_no_op()
{
  let mut state = State::new();
  events_apply_to_state( &mut state, &[ ev( EventType::FocusLost ) ] );
  assert!( state.active_pointers.is_empty() );
  assert!( state.keyboard_keys.iter().all( | held | !held ) );
  assert!( state.mouse_buttons.iter().all( | held | !held ) );
}
