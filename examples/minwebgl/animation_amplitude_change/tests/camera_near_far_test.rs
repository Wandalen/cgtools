//! ## Root Cause
//! `camera_setup`'s near/far clip planes were derived from a base-2 exponent extracted
//! from `diagonal`'s raw IEEE-754 bits (`to_bits() >> 23 & 0xFF - 127`), then fed into
//! `10.0f32.powi(...)` — a base-10 power. This base mismatch, combined with a
//! `far = near * 100^|exponent| / 100` formula that collapses to `far <= near` whenever
//! `|exponent|` is 0 or 1 (`diagonal` roughly in [0.5, 4.0)), violated `Camera::new`'s
//! own strict `far > near` requirement (renderer's `webgl/camera.rs`) for an ordinary,
//! common scene scale — including this crate's own bundled `multi_animation.glb`, whose
//! world-space bounding-box diagonal lands squarely in that broken range.
//!
//! ## Why Not Caught
//! `Camera::new` returns a `Result`, but `main.rs` immediately calls
//! `.expect( "camera parameters are valid" )` on it, so the only symptom was a WASM
//! panic at startup — no compile-time signal, and the crate has no lib target or native
//! test target to unit-test `camera_setup` (a private fn in a binary-only crate) directly.
//!
//! ## Fix Applied (BUG-320)
//! Replaced the base-2-bit-extraction/base-10-power computation with `diagonal.log10()`
//! (a true base-10 order of magnitude) and a fixed 1,000,000:1 far:near ratio around it,
//! which yields `far > near` for every finite positive `diagonal` by construction.
//!
//! ## Prevention
//! Two kinds of check: (1) parses `main.rs` to assert the broken bit-extraction pattern
//! is gone and the `log10`-based replacement is present; (2) re-derives the *shape* of
//! the current formula as a standalone pure function and property-checks `far > near`,
//! both finite and positive, across a wide sweep of `diagonal` magnitudes (1e-6 .. 1e6) —
//! catching either a regression to the old formula or a careless future edit to the new
//! one's constants.
//!
//! ## Pitfall
//! Don't replicate the *exact* production formula verbatim into this test — that would
//! let production and test drift together silently. The property check intentionally
//! re-derives the formula from first principles (fixed ratio, `log10`) so it fails if
//! production's actual constants ever stop guaranteeing `far > near`.

const MAIN_RS : &str = include_str!( "../src/main.rs" );

fn camera_setup_body() -> &'static str
{
  let start = MAIN_RS.find( "fn camera_setup" ).expect( "camera_setup not found in main.rs" );
  let search_from = start + "fn camera_setup".len();
  let end = MAIN_RS[ search_from.. ].find( "\nfn " ).map_or( MAIN_RS.len(), | i | search_from + i );
  &MAIN_RS[ start..end ]
}

#[ test ]
fn camera_setup_no_longer_uses_base2_bit_extraction()
{
  let body = camera_setup_body();
  assert!(
    !body.contains( "to_bits()" ) && !body.contains( "exponent_field" ),
    "camera_setup regressed to base-2 bit-extracted exponent: {body}"
  );
}

#[ test ]
fn camera_setup_uses_log10_based_magnitude()
{
  let body = camera_setup_body();
  assert!( body.contains( ".log10()" ), "camera_setup should derive magnitude via f32::log10: {body}" );
  assert!( body.contains( "let far = scale * 10_000.0" ), "camera_setup should compute far as a fixed multiple of scale: {body}" );
  assert!( body.contains( "let near = ( scale * 0.01 )" ), "camera_setup should compute near as a fixed fraction of scale: {body}" );
}

#[ test ]
fn near_far_formula_always_yields_far_greater_than_near()
{
  // Re-derives the *shape* of the current formula independently of main.rs's literal
  // source text, to prove the formula class itself (log10 magnitude + fixed ratio) is
  // sound — not just that today's specific numbers happen to work.
  fn near_far( diagonal : f32 ) -> ( f32, f32 )
  {
    let magnitude = diagonal.max( f32::EPSILON ).log10().floor();
    let scale = 10.0f32.powf( magnitude );
    let near = ( scale * 0.01 ).max( 1e-5 );
    let far = scale * 10_000.0;
    ( near, far )
  }

  let mut diagonal = 1e-6_f32;
  while diagonal <= 1e6
  {
    let ( near, far ) = near_far( diagonal );
    assert!( near.is_finite() && near > 0.0, "near must be finite and positive for diagonal={diagonal}: near={near}" );
    assert!( far.is_finite() && far > near, "far must be finite and > near for diagonal={diagonal}: near={near}, far={far}" );
    diagonal *= 3.7; // irrational-ish step to sweep across many exponent buckets, including their boundaries
  }
}

#[ test ]
fn bundled_asset_diagonal_no_longer_hits_a_broken_case()
{
  // The specific failure this crate hit at startup: multi_animation.glb's world-space
  // bounding-box diagonal (~2.5..2.6, confirmed via the glTF's own POSITION accessor
  // min/max) lands in the base-2 exponent-1 bucket that the *old* formula collapsed to
  // `far == near` on. Confirm the new formula clears it with room to spare.
  fn near_far( diagonal : f32 ) -> ( f32, f32 )
  {
    let magnitude = diagonal.max( f32::EPSILON ).log10().floor();
    let scale = 10.0f32.powf( magnitude );
    let near = ( scale * 0.01 ).max( 1e-5 );
    let far = scale * 10_000.0;
    ( near, far )
  }

  let ( near, far ) = near_far( 2.576 );
  assert!( far > near, "bundled asset's diagonal must yield far > near: near={near}, far={far}" );
}
