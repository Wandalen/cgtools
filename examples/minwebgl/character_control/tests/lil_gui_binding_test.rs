//! Regression tests verifying every `wasm_bindgen` FFI binding in `lil_gui.rs` names a JS
//! function that actually exists in this crate's own `gui.js`.
//!
//! `character_control` is a binary-only example crate (no `[lib]` target), so this test reads
//! both files' own real source text via `include_str!` rather than exercising the wasm-only
//! bindings.

use std::collections::HashSet;

const LIL_GUI_RS : &str = include_str!( "../src/lil_gui.rs" );
const GUI_JS : &str = include_str!( "../gui.js" );

/// Extracts every `js_name = "<name>"` value bound in `lil_gui.rs`'s `wasm_bindgen` extern block.
fn bound_js_names( src : &str ) -> Vec< String >
{
  src
  .split( "js_name = \"" )
  .skip( 1 )
  .filter_map( | s | s.split( '"' ).next() )
  .map( str::to_string )
  .collect()
}

/// Extracts every `export function <name>(` declared in `gui.js`.
fn exported_js_functions( src : &str ) -> HashSet< String >
{
  src
  .split( "export function " )
  .skip( 1 )
  .filter_map( | s | s.split( '(' ).next() )
  .map( | s | s.trim().to_string() )
  .collect()
}

/// ## Root Cause
/// `lil_gui.rs`'s `name_set( gui : &JsValue, value : &str )` binding was wired to
/// `js_name = "getTitle"`. This crate's own `gui.js` does not export any function named
/// `getTitle` at all — it exports a matching `set_name( gui, name )` setter, consistent with the
/// Rust function's own name and signature. `lil_gui.rs` was evidently copy-pasted from a sibling
/// crate whose own `gui.js` does define `getTitle`, without re-checking the binding against this
/// crate's own JS file. Independent of, and not overlapping with, this crate's separately-filed
/// BUG-312 (visible-mesh yaw halving at a `Quat::from_angle_y` call site).
///
/// ## Why Not Caught
/// `name_set` is never actually called anywhere in this crate — the binding is dead code, so its
/// runtime-error-on-call defect has no way to surface during normal use of the demo.
///
/// ## Fix Applied
/// Changed the binding's `js_name` from `"getTitle"` to `"set_name"`, matching this crate's own
/// `gui.js` export and the setter semantics the Rust function's name/signature already promise.
///
/// ## Prevention
/// This test extracts every `js_name = "..."` bound in `lil_gui.rs` and asserts each one is
/// actually exported by this crate's own `gui.js` — catches any binding pointing at a JS function
/// that doesn't exist in this crate's own bundle, not just this one instance.
///
/// ## Pitfall
/// A `wasm_bindgen` extern binding compiles successfully regardless of whether the named JS export
/// actually exists — the mismatch is invisible until the specific binding is called at runtime in
/// a browser, and copying a bindings file between crates with differently-shaped `gui.js` files is
/// exactly the situation that introduces this silently.
#[ test ]
fn bug_reproducer_bug_339_lil_gui_bindings_match_real_gui_js_exports()
{
  let exported = exported_js_functions( GUI_JS );
  assert!( exported.contains( "set_name" ), "sanity: gui.js should still export set_name" );

  let bound = bound_js_names( LIL_GUI_RS );
  assert!( !bound.is_empty(), "sanity: lil_gui.rs should contain js_name bindings" );

  for name in &bound
  {
    assert!
    (
      exported.contains( name ),
      "lil_gui.rs binds js_name = \"{name}\" but gui.js does not export a function named \
      \"{name}\" — calling this binding would be a wasm_bindgen runtime error (BUG-339)"
    );
  }
}
