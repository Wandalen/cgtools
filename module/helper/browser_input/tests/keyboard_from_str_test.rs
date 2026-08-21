//! Covers `KeyboardKey::from_str`'s string-parsing behavior -- kept separate from
//! `keyboard_key_state_test.rs`, which covers press/release *state* bookkeeping, a distinct
//! responsibility. Sibling to `pointer_type_test.rs` (DOM-string parsing for a different type).

use std::str::FromStr;
use browser_input::keyboard::KeyboardKey;

/// UX/DX fix: `KeyboardKey::from_str` previously matched case-sensitively, while
/// `MouseButton::from_str` (this crate's sibling `FromStr` impl, `src/mouse.rs`) already
/// lowercases via `s.to_lowercase().as_str()`. Made uniform by lowercasing `KeyboardKey`'s side
/// to match -- this pins the new behavior so it can't silently regress back to case-sensitive.
#[ test ]
fn from_str_is_case_insensitive()
{
  assert_eq!( KeyboardKey::from_str( "AltLeft" ), Ok( KeyboardKey::AltLeft ) );
  assert_eq!( KeyboardKey::from_str( "altleft" ), Ok( KeyboardKey::AltLeft ) );
  assert_eq!( KeyboardKey::from_str( "ALTLEFT" ), Ok( KeyboardKey::AltLeft ) );
  assert_eq!( KeyboardKey::from_str( "AltLEFT" ), Ok( KeyboardKey::AltLeft ) );

  assert_eq!( KeyboardKey::from_str( "KeyA" ), Ok( KeyboardKey::KeyA ) );
  assert_eq!( KeyboardKey::from_str( "keya" ), Ok( KeyboardKey::KeyA ) );
  assert_eq!( KeyboardKey::from_str( "KEYA" ), Ok( KeyboardKey::KeyA ) );
}

/// The real caller (`browser_input::input`, via `KeyboardKey::from( event.code().as_str() )`)
/// always passes the browser-native, canonically-cased `KeyboardEvent.code` string -- this pins
/// that the exact-case path (the only path any real caller exercises today) is unaffected by
/// the case-insensitivity widening.
#[ test ]
fn from_str_still_accepts_canonical_case()
{
  assert_eq!( KeyboardKey::from_str( "ArrowUp" ), Ok( KeyboardKey::ArrowUp ) );
  assert_eq!( KeyboardKey::from_str( "Digit0" ), Ok( KeyboardKey::Digit0 ) );
  assert_eq!( KeyboardKey::from_str( "NumpadEnter" ), Ok( KeyboardKey::NumpadEnter ) );
}

/// An unrecognized string, regardless of case, falls back to `Unidentified` (never `Err` --
/// `KeyboardKey::from_str`'s `Err` type is `()` but the match's final arm is a catch-all `Ok`).
#[ test ]
fn from_str_falls_back_to_unidentified_for_unknown_input()
{
  assert_eq!( KeyboardKey::from_str( "NotARealKey" ), Ok( KeyboardKey::Unidentified ) );
  assert_eq!( KeyboardKey::from_str( "notarealkey" ), Ok( KeyboardKey::Unidentified ) );
}
