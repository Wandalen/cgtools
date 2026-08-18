//! ## Root Cause
//! `src/main.rs`'s top-level doc comment claimed "just draw a large point in the middle of the
//! screen", while the code actually renders a rotating wireframe cube (8 vertices, 24 line
//! indices, an animated `angle` uniform driving 2-axis rotation, a perspective projection) --
//! confirmed against `readme.md`'s own description ("size-optimized version of a spinning cube").
//!
//! ## Why Not Caught
//! No test file existed for this crate -- it is a `fn main()`-only WebGL demo binary with no
//! lib target, and a doc comment has no compiler link to what the code actually does.
//!
//! ## Fix Applied
//! Replaced the stale doc comment with an accurate description matching `readme.md`.
//!
//! ## Prevention
//! This test parses `src/main.rs`'s own text via `include_str!` and asserts the doc comment
//! mentions the cube/spinning demo while no longer claiming the stale "large point" description.
//!
//! ## Pitfall
//! A crate's own top-level doc comment is disconnected from its actual behavior with zero
//! compiler enforcement -- this is the second such copy-paste/stale-description drift found in
//! this exact bug-hunt pass (see the sibling `text_msdf` crate).

// BUG-337 task/bug/XXX_spinning_cube_size_opt_stale_large_point_doc_comment.md -- reproducer for
// main.rs's doc comment falsely describing "a large point" instead of a spinning wireframe cube.
// test_kind: bug_reproducer(BUG-337)
#[ test ]
fn doc_comment_describes_spinning_cube_not_stale_large_point_claim()
{
  let source = include_str!( "../src/main.rs" );
  let header_end = source.find( "use minwebgl" ).expect( "main.rs should import minwebgl" );
  let header = &source[ ..header_end ];

  assert!(
    header.to_lowercase().contains( "cube" ),
    "main.rs's doc comment must describe the spinning cube demo, matching readme.md"
  );
  assert!(
    !header.contains( "a large point in the middle of the screen" ),
    "main.rs's doc comment must not carry the stale 'large point' claim"
  );
}
