//! Regression coverage for `src/main.rs`'s top-of-file doc comment.
//!
//! `main.rs` is a `wasm32`-only binary in practice, but the doc comment defect this test
//! guards is plain text -- `include_str!` reads it as a string with zero dependency on the
//! crate's own (wasm32-gated) items, so this test compiles and runs on any host target.

// test_kind: bug_reproducer(BUG-306-B)
/// ## Root Cause
/// `src/main.rs`'s module doc comment read "Just draw a large point in the middle of the
/// screen." -- copy-pasted from an unrelated example and never updated to describe this
/// crate's actual behavior: a deferred renderer that draws a grid of models into a G-buffer
/// (albedo, normal, position) then composites it in a lighting pass against a set of point
/// lights, each drawn with a small visualization mesh (see `src/light.rs`, `src/model.rs`,
/// `shaders/gbuffer.wgsl`, `shaders/light.wgsl`).
/// ## Why Not Caught
/// This crate has no automated test coverage at all prior to this fix -- nothing cross-checked
/// the doc comment's factual claim against the crate's own render passes.
/// ## Fix Applied
/// Corrected the doc comment to describe the actual deferred G-buffer rendering pipeline, and
/// added this test as a standing regression guard against the specific wrong text reappearing.
/// ## Prevention
/// A demo crate's own top-of-file doc comment is a factual claim about what the crate does,
/// exactly like any other doc comment -- it must be cross-checked against the crate's actual
/// source (here, its render passes) rather than trusted at face value.
/// ## Pitfall
/// Copy-pasted boilerplate doc comments across sibling example crates drift silently once one
/// crate's actual behavior diverges from the text -- each sibling needs its own cross-check,
/// not just the first one written.
#[ test ]
fn main_doc_comment_describes_deferred_rendering_not_a_point()
{
  let main_rs = include_str!( "../src/main.rs" );
  assert!(
    !main_rs.contains( "Just draw a large point in the middle of the screen" ),
    "src/main.rs's module doc comment must not reintroduce the stale copy-pasted \
    point-drawing description (BUG-306-B)"
  );
  assert!(
    main_rs.contains( "Deferred rendering" ) && main_rs.contains( "G-buffer" ),
    "src/main.rs's module doc comment must describe this crate's actual deferred G-buffer \
    rendering behavior (BUG-306-B)"
  );
}
