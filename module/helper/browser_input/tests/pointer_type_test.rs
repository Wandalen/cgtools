//! Unit pins for [`browser_input::PointerType`]'s `From< &str >` conversion — the DOM
//! `pointerType` string → variant mapping ( mouse / touch / pen, with every
//! unrecognised or empty string collapsing to `Unknown` ) — and the `Unknown`
//! default. End-to-end wiring through DOM callbacks needs a
//! `wasm-bindgen-test` environment and is deliberately out of scope here.
//!
//! Relocated from `src/input.rs` by task 076 ( bodies verbatim ).

use browser_input::PointerType;

#[ test ]
fn from_mouse()
{
  assert_eq!( PointerType::from( "mouse" ), PointerType::Mouse );
}

#[ test ]
fn from_touch()
{
  assert_eq!( PointerType::from( "touch" ), PointerType::Touch );
}

#[ test ]
fn from_pen()
{
  assert_eq!( PointerType::from( "pen" ), PointerType::Pen );
}

#[ test ]
fn from_empty_string_is_unknown()
{
  assert_eq!( PointerType::from( "" ), PointerType::Unknown );
}

#[ test ]
fn from_unrecognised_is_unknown()
{
  assert_eq!( PointerType::from( "stylus" ), PointerType::Unknown );
}

#[ test ]
fn default_is_unknown()
{
  assert_eq!( PointerType::default(), PointerType::Unknown );
}
