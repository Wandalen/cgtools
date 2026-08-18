//! ## Root Cause
//! The crate's module doc comment was a stale copy-paste leftover from an earlier,
//! simpler sketch — it described drawing "a large point in the middle of the screen",
//! but this crate actually instanced-draws 6 triangles with 5 per-instance Y offsets via
//! `draw_arrays_instanced`. The crate's own readme.md already correctly described it as
//! an instanced-rendering demo, contradicting the stale doc comment.
//!
//! ## Why Not Caught
//! The crate has no lib target or native test target at all (`main.rs`-only wasm binary);
//! rustdoc renders whatever text is present without validating it against the demo's
//! actual behavior, so a stale sentence compiles and renders cleanly forever.
//!
//! ## Fix Applied (BUG-ZZZ)
//! Replaced the stale `//!` doc comment in `examples/minwebgl/attributes_instanced/
//! src/main.rs` with an accurate description of the instanced-triangle demo.
//!
//! ## Prevention
//! This structural test parses `main.rs` (no lib target to unit-test directly) and
//! asserts the stale sentence is gone and the doc comment mentions the demo's actual
//! mechanism (`draw_arrays_instanced`).
//!
//! ## Pitfall
//! `attributes_vao`'s sibling `main.rs` carried the exact same stale sentence — check
//! other early-stage `attributes_*` demo crates for the same leftover before assuming
//! this was a one-off.

const MAIN_RS : &str = include_str!( "../src/main.rs" );

#[ test ]
fn doc_comment_no_longer_describes_stale_single_point_demo()
{
  assert!(
    !MAIN_RS.contains( "large point in the middle of the screen" ),
    "stale doc comment text regressed: {MAIN_RS}"
  );
}

#[ test ]
fn doc_comment_mentions_instanced_drawing()
{
  let doc_block_end = MAIN_RS.find( "\n\nuse minwebgl" ).unwrap_or( MAIN_RS.len() );
  let doc_block = &MAIN_RS[ ..doc_block_end ];
  assert!(
    doc_block.contains( "draw_arrays_instanced" ),
    "module doc comment should describe the instanced-drawing mechanism: {doc_block}"
  );
}
