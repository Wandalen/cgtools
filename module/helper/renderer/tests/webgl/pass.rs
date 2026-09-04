//! Regression coverage for BUG-259: `SwapFramebuffer::new`'s doc comment claimed the function
//! initializes "its WebGL framebuffer, renderbuffer, and the primary output texture" -- but the
//! function body has never called `create_renderbuffer` since the renderbuffer was deliberately
//! removed from this struct in commit a54d680b ("Added bloom", 2025-05-28); the doc comment
//! (written earlier, never revisited) kept claiming it anyway.

const PASS_SRC : &str = include_str!( "../../src/webgl/post_processing/pass.rs" );

/// Returns the text strictly between the first occurrence of `start` and the first occurrence of
/// `end` found after it. Panics with a descriptive message if either marker is missing, since a
/// missing marker means `pass.rs` has been restructured in a way this test's anchors no longer
/// match -- silently passing (or failing for the wrong reason) would be worse than a clear panic
/// naming exactly which anchor broke.
fn between< 'h >( haystack : &'h str, start : &str, end : &str ) -> &'h str
{
  let after_start = haystack.find( start )
  .map_or_else
  (
    || panic!( "start marker {start:?} not found in pass.rs -- has the file been restructured?" ),
    | i | &haystack[ i + start.len().. ]
  );
  let end_idx = after_start.find( end )
  .unwrap_or_else( || panic!( "end marker {end:?} not found after {start:?} in pass.rs -- has the file been restructured?" ) );
  &after_start[ ..end_idx ]
}

/// Joins every `///` doc-comment line in `text` into a single space-separated string, stripping
/// the `///` marker itself. Plain `//` comments (e.g. this repo's `Fix( BUG-NNN )` annotations)
/// are deliberately excluded, so a fix's own explanatory comment can freely discuss the defect in
/// prose (including the word this test greps for) without being mistaken for the doc comment it
/// sits above.
fn doc_comment_text( text : &str ) -> String
{
  text.lines()
  .map( str::trim )
  .filter( | line | line.starts_with( "///" ) )
  .map( | line | line.trim_start_matches( '/' ).trim() )
  .collect::< Vec< _ > >()
  .join( " " )
}

/// ## Root Cause
/// `SwapFramebuffer::new`'s doc comment claimed the function initializes "its WebGL framebuffer,
/// renderbuffer, and the primary output texture", and that the framebuffer is "configured with a
/// single color attachment point and a depth/stencil renderbuffer" -- but the function body has
/// never called `create_renderbuffer`, `renderbuffer_storage`, or `framebuffer_renderbuffer`.
/// Commit a54d680b ("Added bloom", 2025-05-28) deliberately removed this struct's renderbuffer
/// field and its creation code once depth testing was no longer needed for post-processing
/// passes, but the doc comment describing it (added in an earlier commit) was never revisited.
/// ## Why Not Caught
/// Rust doc comments are free-form prose with no compiler-enforced contract against the function
/// body beneath them, so a stale claim compiles cleanly forever. Every `Pass` implementation in
/// this crate explicitly disables `DEPTH_TEST` before drawing regardless of what `SwapFramebuffer`
/// provides, so the missing depth/stencil buffer never manifested as an observable rendering
/// defect -- the mismatch was purely a documentation trap for a future caller trusting the stated
/// contract enough to skip disabling depth testing.
/// ## Fix Applied
/// Rewrote `SwapFramebuffer::new`'s doc comment to state only what the function actually does:
/// creates a framebuffer with a single color attachment and the output texture, no renderbuffer,
/// and calls out that any `Pass` rendering into it must not rely on depth testing.
/// ## Prevention
/// This test extracts the real `///` doc comment above `SwapFramebuffer::new` (filtering out
/// plain `//` comments, including this crate's own `Fix( BUG-NNN )` annotations, so a fix's own
/// explanatory prose is never mistaken for the doc contract it sits above) and the function's own
/// body text, then asserts the doc's initialization-enumeration sentence claims a `renderbuffer`
/// if and only if the body actually calls `create_renderbuffer` -- catching a repeat of "doc
/// promises a resource the code never creates" without needing a live GL context.
/// ## Pitfall
/// A doc comment surviving a refactor that deletes the resource it describes is invisible to the
/// compiler and to any test that only exercises runtime behavior -- the doc and the code never
/// disagree about what the *program does*, only about what a reader is told it does. Only a
/// direct source-text comparison between the comment and the code it describes catches that class
/// of drift.
// test_kind: bug_reproducer(BUG-259)
#[ test ]
fn swap_framebuffer_new_doc_comment_renderbuffer_claim_matches_body()
{
  let impl_block = between( PASS_SRC, "impl SwapFramebuffer", "pub fn bind" );
  let new_idx = impl_block.find( "pub fn new" )
  .expect( "`pub fn new` not found inside the `impl SwapFramebuffer` block -- has pass.rs been restructured?" );
  let ( doc_region, body_region ) = impl_block.split_at( new_idx );

  let doc = doc_comment_text( doc_region );
  // Only the first sentence makes an initialization-enumeration claim ("initializing its WebGL
  // framebuffer[, renderbuffer,] and the primary output texture") -- later sentences may
  // legitimately *discuss* the absence of a renderbuffer (e.g. "there is no depth/stencil
  // renderbuffer"), which must not be mistaken for a claim that one exists.
  let first_sentence = doc.split( '.' ).next().unwrap_or( "" );
  let doc_claims_renderbuffer = first_sentence.to_lowercase().contains( "renderbuffer" );
  let body_creates_renderbuffer = body_region.contains( "create_renderbuffer" );

  assert_eq!
  (
    doc_claims_renderbuffer, body_creates_renderbuffer,
    "SwapFramebuffer::new's doc comment {} initializing a renderbuffer ({first_sentence:?}), but the function body {} call create_renderbuffer -- doc and code have drifted apart",
    if doc_claims_renderbuffer { "claims" } else { "does not claim" },
    if body_creates_renderbuffer { "does" } else { "does not" }
  );
}
