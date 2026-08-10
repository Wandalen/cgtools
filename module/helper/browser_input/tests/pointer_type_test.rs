//! Unit pins for [`browser_input::PointerType::from_dom_str`] — the DOM
//! `pointerType` string → variant mapping ( mouse / touch / pen, with every
//! unrecognised or empty string collapsing to `Unknown` ) — and the `Unknown`
//! default. End-to-end wiring through DOM callbacks needs a
//! `wasm-bindgen-test` environment and is deliberately out of scope here.
//!
//! Relocated from `src/input.rs` by task 076 ( bodies verbatim ).

use browser_input::PointerType;

#[ test ]
fn from_dom_str_mouse()
{
  assert_eq!( PointerType::from_dom_str( "mouse" ), PointerType::Mouse );
}

#[ test ]
fn from_dom_str_touch()
{
  assert_eq!( PointerType::from_dom_str( "touch" ), PointerType::Touch );
}

#[ test ]
fn from_dom_str_pen()
{
  assert_eq!( PointerType::from_dom_str( "pen" ), PointerType::Pen );
}

#[ test ]
fn from_dom_str_empty_string_is_unknown()
{
  assert_eq!( PointerType::from_dom_str( "" ), PointerType::Unknown );
}

#[ test ]
fn from_dom_str_unrecognised_is_unknown()
{
  assert_eq!( PointerType::from_dom_str( "stylus" ), PointerType::Unknown );
}

#[ test ]
fn default_is_unknown()
{
  assert_eq!( PointerType::default(), PointerType::Unknown );
}
