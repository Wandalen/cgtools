//! Regression tests verifying `filter`'s readme claims match its actual shader implementation.
//!
//! `filter` is a binary-only example crate (no `[lib]` target), so this test reads the crate's
//! own real source text via `include_str!` rather than exercising library code.

const README : &str = include_str!( "../readme.md" );
const MAIN_FRAG : &str = include_str!( "../src/shaders/main.frag" );

/// ## Root Cause
/// `readme.md` claimed this demo "shows how to apply various shader-based filters to images or
/// rendered scenes, including blur, sharpen, edge detection, and color grading." `main.frag`
/// implements exactly one hardcoded 3x3 convolution kernel (`EMBOSS_KERNEL`, an emboss effect)
/// applied inside a radius around the cursor — there is no blur pass, no sharpen pass, no edge
/// detection pass, and no color-grading pass anywhere in this crate.
///
/// ## Why Not Caught
/// The demo genuinely does apply a real, working convolution-kernel filter, and visibly reacts to
/// mouse movement — so a skim of the demo's actual behavior looks consistent with the readme's
/// general "post-processing filter" framing, and nothing ever cross-checks the four specific named
/// filter types against what the single shader actually implements.
///
/// ## Fix Applied
/// Reworded `readme.md` to describe the actual implemented technique (a convolution-kernel emboss
/// filter revealed within a cursor-centered radius) instead of the four unimplemented filter types.
///
/// ## Prevention
/// This test greps the readme for each of the four specific unimplemented filter-type claims and
/// fails if any reappear, while sanity-asserting the real technique (`EMBOSS_KERNEL`) is still
/// actually present — catches either a false claim reappearing or the real kernel silently
/// regressing out from under an unchanged readme.
///
/// ## Pitfall
/// A demo whose own purpose IS "show a post-processing filter" is exactly the place a list of
/// aspirational-but-unimplemented filter types is most likely to go unnoticed — the demo still
/// visibly "does its job" (one working filter reacting to the mouse) even when most of the
/// specifically named filter types were never real.
#[ test ]
fn bug_reproducer_bug_323_readme_does_not_claim_unimplemented_filter_types()
{
  assert!
  (
    MAIN_FRAG.contains( "EMBOSS_KERNEL" ),
    "sanity: the emboss convolution kernel should still be the implemented filter"
  );

  let readme_lower = README.to_lowercase();
  for claimed in [ "blur", "sharpen", "edge detection", "color grading" ]
  {
    assert!
    (
      !readme_lower.contains( claimed ),
      "readme claims \"{claimed}\" but main.frag only implements a single hardcoded emboss \
      convolution kernel — no blur/sharpen/edge-detection/color-grading pass exists (BUG-323)"
    );
  }
}
