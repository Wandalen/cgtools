//! Regression test verifying every `wasm_bindgen` FFI binding in `lil_gui.rs` names a JS
//! function that actually exists in this crate's own `gui.js`. Prevents a BUG-339-style
//! binding/export mismatch (a `js_name` copied from a sibling crate's differently-shaped
//! `gui.js`, compiling fine but failing at runtime the first time it's actually called).
//!
//! `simple_pbr` is a binary-only example crate (no `[lib]` target), so this test reads both
//! files' own real source text via `include_str!` rather than exercising the wasm-only bindings.

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

#[ test ]
fn test_lil_gui_bindings_match_gui_js_exports()
{
  let exported = exported_js_functions( GUI_JS );
  assert!( !exported.is_empty(), "sanity: gui.js should export at least one function" );

  let bound = bound_js_names( LIL_GUI_RS );
  assert!( !bound.is_empty(), "sanity: lil_gui.rs should contain js_name bindings" );

  for name in &bound
  {
    assert!
    (
      exported.contains( name ),
      "lil_gui.rs binds js_name = \"{name}\" but gui.js does not export a function named \
      \"{name}\" — calling this binding would be a wasm_bindgen runtime error (BUG-339-style)"
    );
  }
}
