//! ## Root Cause
//! `src/main.rs`'s top-level doc comment was copy-pasted from `uniforms_ubo`'s main.rs and never
//! updated: it claimed this example "renders a triangle in the middle of the screen" using
//! "Uniform Block Objects (UBOs)", while this crate actually renders MSDF (Multi-Channel Signed
//! Distance Field) text via instanced quads -- confirmed against `readme.md`'s own description
//! ("high-quality text rendering using Multi-Channel Signed Distance Fields (MSDF) in WebGL2").
//!
//! ## Why Not Caught
//! No test file existed for this crate -- it is a `fn main()`-only WebGL demo binary with no
//! lib target, and a doc comment has no compiler link to what the code actually does; nothing
//! short of a reader or a text-content test catches drift.
//!
//! ## Fix Applied
//! Replaced the copy-pasted doc comment with an accurate description matching `readme.md`.
//!
//! ## Prevention
//! This test parses `src/main.rs`'s own text via `include_str!` and asserts the doc comment
//! mentions MSDF/text rendering while no longer claiming the stale "triangle"/"Uniform Block
//! Objects" description.
//!
//! ## Pitfall
//! A crate's own top-level doc comment is disconnected from its actual behavior with zero
//! compiler enforcement -- copy-pasting a sibling crate's file header is an easy, silent mistake.

// BUG-XXX task/bug/XXX_text_msdf_stale_triangle_ubo_doc_comment.md -- reproducer for main.rs's
// doc comment falsely describing a UBO-driven triangle instead of MSDF text rendering.
// test_kind: bug_reproducer(BUG-XXX)
#[ test ]
fn doc_comment_describes_msdf_text_not_stale_triangle_ubo_claim()
{
  let source = include_str!( "../src/main.rs" );
  let header_end = source.find( "use minwebgl" ).expect( "main.rs should import minwebgl" );
  let header = &source[ ..header_end ];

  assert!(
    header.to_lowercase().contains( "msdf" ),
    "main.rs's doc comment must describe MSDF text rendering, matching readme.md"
  );
  assert!(
    !header.contains( "triangle in the middle of the screen" ),
    "main.rs's doc comment must not carry the stale copy-pasted claim from uniforms_ubo"
  );
  assert!(
    !header.contains( "Uniform Block Objects" ),
    "main.rs's doc comment must not carry the stale copy-pasted UBO claim from uniforms_ubo"
  );
}
