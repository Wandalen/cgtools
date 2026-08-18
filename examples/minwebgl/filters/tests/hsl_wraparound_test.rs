//! Regression tests verifying `filters::hsl_adjustment`'s hue-shift wraparound keeps the hue
//! argument passed into `hsl2rgb` within its assumed `[0, 1)` domain.
//!
//! `filters` is a binary-only example crate (no `[lib]` target) whose fragment shaders are Rust
//! string literals rather than separate `.frag` files, so this test reads `hsl_adjustment.rs`'s
//! own real source text via `include_str!` (to anchor against regression of the actual GLSL fix),
//! plus a hand-ported pure-Rust mirror of the fixed hue-wraparound formula (GLSL has no native
//! test harness) to verify the formula's numeric domain property directly.

const HSL_ADJUSTMENT_RS : &str = include_str!( "../src/filters/hsl_adjustment.rs" );

/// Mirrors `hsl_adjustment.rs`'s fixed hue-shift line: `hsl.x = mod( hsl.x + u_hsl.x, 1.0 );`.
/// GLSL's `mod( x, 1.0 )` is `x - floor(x)`, matched here by `f32::rem_euclid`.
fn apply_hue_shift( original_hue : f32, shift : f32 ) -> f32
{
  ( original_hue + shift ).rem_euclid( 1.0 )
}

/// ## Root Cause
/// `main()` applied the user's hue-shift slider (`u_hsl.x`, range ±1.0 per
/// `ui_setup/filter_setup_advanced.rs`) via a bare `hsl.x += u_hsl.x;`, with no wraparound back
/// into `hsl2rgb`'s assumed `[0, 1)` hue domain. `hsl2rgb`'s own `hue2rgb` helper only corrects a
/// single unit of overflow/underflow (`if (t<0) t+=1; if (t>1) t-=1;`), which is exactly enough
/// for a hue already in `[0, 1)` phase-shifted by `hue2rgb`'s own internal `±1/3` (a combined range
/// of at most `[-1/3, 4/3)`) — but NOT enough once `main()` first adds an unwrapped external shift
/// of up to `±1.0` on top, pushing the phase-shifted `r`/`b` channel inputs as far out as
/// `[-4/3, 7/3)`, a wider-than-1-unit excursion the single-step correction cannot fully undo.
///
/// ## Why Not Caught
/// The filter visibly shifts hue correctly across the vast majority of the slider's range and for
/// most source-image hues — the under-wrap only manifests at the combination of an extreme slider
/// position (near ±1.0) AND a source pixel hue near the corresponding domain edge, a narrow
/// interactive corner nothing in a visual skim is likely to land on.
///
/// ## Fix Applied
/// Changed `hsl.x += u_hsl.x;` to `hsl.x = mod( hsl.x + u_hsl.x, 1.0 );`, restoring the `[0, 1)`
/// invariant `hue2rgb`'s single-step correction already assumes before any phase offset is added.
///
/// ## Prevention
/// This test asserts the real GLSL source now performs the `mod(...)` wrap (catching a regression
/// back to the bare `+=`), and numerically checks a pure-Rust mirror of the fixed formula keeps
/// its output in `[0, 1)` across the full reachable `(hue, shift)` input space.
///
/// ## Pitfall
/// A helper function correct for its ASSUMED input domain (here, `hue2rgb` assuming a hue already
/// in `[0, 1)`) silently becomes wrong the moment a caller feeds it a value from a wider domain —
/// the helper itself never changed and has no way to signal the violated assumption.
#[ test ]
fn bug_reproducer_bug_xxx_hue_shift_wraps_into_zero_one_domain()
{
  assert!
  (
    HSL_ADJUSTMENT_RS.contains( "mod( hsl.x + u_hsl.x, 1.0 )" ),
    "hsl_adjustment.rs should wrap the hue shift back into [0, 1) via mod(...) before hsl2rgb \
    sees it — a bare `hsl.x += u_hsl.x;` under-wraps at slider extremes (BUG-XXX)"
  );

  // Sweep the full reachable (original_hue, shift) space: original_hue from rgb2hsl is [0, 1),
  // and the UI's hue slider is [-1.0, 1.0].
  let mut checked = 0;
  for hue_steps in 0..100
  {
    let original_hue = hue_steps as f32 / 100.0;
    for shift_steps in -100..=100
    {
      let shift = shift_steps as f32 / 100.0;
      let wrapped = apply_hue_shift( original_hue, shift );
      assert!
      (
        ( 0.0..1.0 ).contains( &wrapped ),
        "apply_hue_shift( {original_hue}, {shift} ) = {wrapped}, outside [0, 1) — hue2rgb's \
        single-step ±1 correction assumes its input is already in this range (BUG-XXX)"
      );
      checked += 1;
    }
  }
  assert!( checked > 0, "sanity: the sweep should have actually run" );
}
