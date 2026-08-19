//! ## Root Cause
//! `src/main.rs`'s top-level doc comment claimed "just draw a large point in the middle of the
//! screen", while the code actually implements an interactive OBJ model viewer: loads a
//! multi-material, multi-texture model (`lost-empire`) with mouse-driven orbit-camera controls
//! (`CameraOrbitControls`), splitting meshes into opaque and transparent render passes.
//!
//! ## Why Not Caught
//! No test file existed for this crate -- it is a `fn main()`-only WebGL demo binary with no
//! lib target, and a doc comment has no compiler link to what the code actually does. The 4
//! parallel forks that bug-hunted `examples/minwebgl`'s 44 crates (task #184) each had this
//! crate in their assigned list but did not check its doc comment against its own source.
//!
//! ## Fix Applied
//! Replaced the stale doc comment with an accurate description of the interactive OBJ viewer.
//!
//! ## Prevention
//! This test parses `src/main.rs`'s own text via `include_str!` and asserts the doc comment
//! mentions the OBJ viewer while no longer claiming the stale "large point" description.
//!
//! ## Pitfall
//! A crate's own top-level doc comment is disconnected from its actual behavior with zero
//! compiler enforcement -- this exact stale sentence was independently found and fixed in
//! `attributes_vao`/BUG-318, `attributes_instanced`/BUG-319, and `spinning_cube_size_opt`/BUG-337,
//! but a repo-wide grep after those fixes turned up this crate (plus `make_cube_map`,
//! `obj_load`) still carrying the identical unfixed sentence, despite all being explicitly
//! in-scope for the same bug-hunt pass -- per-crate fork review missed what a single grep across
//! all 44 crates would have caught immediately.

// BUG-340 task/bug/340_make_cube_map_obj_load_obj_viewer_stale_large_point_doc_comment.md --
// reproducer for main.rs's doc comment falsely describing "a large point" instead of the actual
// interactive OBJ-viewer demo.
// test_kind: bug_reproducer(BUG-340)
#[ test ]
fn doc_comment_describes_obj_viewer_demo_not_stale_large_point_claim()
{
  let source = include_str!( "../src/main.rs" );
  let header_end = source.find( "use std" ).expect( "main.rs should import std" );
  let header = &source[ ..header_end ];

  assert!(
    header.to_lowercase().contains( "viewer" ),
    "main.rs's doc comment must describe the interactive OBJ viewer demo"
  );
  assert!(
    !header.contains( "a large point in the middle of the screen" ),
    "main.rs's doc comment must not carry the stale 'large point' claim"
  );
}
