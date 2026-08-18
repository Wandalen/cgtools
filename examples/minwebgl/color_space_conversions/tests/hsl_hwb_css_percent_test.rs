//! ## Root Cause
//! `Srgb::convert::< Hsl >`/`< Hwb >` (the `color` crate) return saturation/lightness and
//! whiteness/blackness as numeric `[0,100]`, not `[0,1]`. The CSS `hsl()`/`hwb()` function
//! grammar types those two trailing components as `<percentage>`, requiring a `%` suffix —
//! unlike `lab()`/`lch()`/`oklab()`/`oklch()`, which accept a bare `<number>` for their
//! non-hue components. Formatting S/L or W/B without `%` produces a syntactically invalid
//! CSS color value.
//!
//! ## Why Not Caught
//! No test exercised the literal CSS strings this crate builds; the crate has no lib target
//! or native test target at all (`main.rs`-only wasm binary), and the 14-arm match's other
//! 12 arms (rgb-triplet and bare-number arms) are all independently valid, so nothing here
//! previously flagged the two percentage-typed arms as different.
//!
//! ## Fix Applied (BUG-XXX)
//! Added `%` to the saturation/lightness placeholders in the `"hsl"` arm and to the
//! whiteness/blackness placeholders in the `"hwb"` arm of `examples/minwebgl/
//! color_space_conversions/src/main.rs`'s color-space match. Left every other arm
//! (`lab`, `lch`, `oklab`, `oklch`, and the `rgb(...)`-triplet arms) untouched.
//!
//! ## Prevention
//! This structural test parses the actual `main.rs` source text (the crate has no
//! lib target to unit-test the match arm directly) and asserts the `hsl`/`hwb` arms'
//! format strings carry `%` on their percentage-typed components, while asserting the
//! `lab`/`lch` arms do NOT gain a spurious `%` (CSS types their components as plain
//! `<number> | <percentage>`, and this crate's own convention leaves them bare).
//!
//! ## Pitfall
//! Do not "fix" this by appending `%` to every arm uniformly — `lab`/`lch`/`oklab`/`oklch`
//! are correct as bare numbers per CSS Color 4; only `hsl`/`hwb`'s S/L and W/B components
//! are percentage-typed.

const MAIN_RS : &str = include_str!( "../src/main.rs" );

fn arm_body( match_arm_name : &str ) -> &str
{
  let needle = format!( "\"{match_arm_name}\" =>" );
  let start = MAIN_RS.find( &needle )
  .unwrap_or_else( || panic!( "match arm \"{match_arm_name}\" not found in main.rs" ) );
  let after = &MAIN_RS[ start.. ];
  let end = after[ needle.len().. ].find( "\",\n" )
  .map_or( MAIN_RS.len(), | i | start + needle.len() + i );
  &MAIN_RS[ start..end ]
}

#[ test ]
fn hsl_arm_formats_saturation_and_lightness_as_css_percentages()
{
  let body = arm_body( "hsl" );
  assert!(
    body.contains( "{saturation:.2}%" ),
    "hsl() arm must format saturation with a % suffix (CSS <percentage> type): {body}"
  );
  assert!(
    body.contains( "{lightness:.2}%" ),
    "hsl() arm must format lightness with a % suffix (CSS <percentage> type): {body}"
  );
}

#[ test ]
fn hwb_arm_formats_whiteness_and_blackness_as_css_percentages()
{
  let body = arm_body( "hwb" );
  assert!(
    body.contains( "{whiteness:.2}%" ),
    "hwb() arm must format whiteness with a % suffix (CSS <percentage> type): {body}"
  );
  assert!(
    body.contains( "{blackness:.2}%" ),
    "hwb() arm must format blackness with a % suffix (CSS <percentage> type): {body}"
  );
}

#[ test ]
fn lab_and_lch_arms_stay_bare_numbers_not_percentages()
{
  // CSS lab()/lch() type their non-hue components as `<number> | <percentage>` — this
  // crate's convention leaves them bare. Guards against a blanket "add % everywhere" fix.
  let lab = arm_body( "lab" );
  assert!(
    !lab.contains( "{lightness:.2}%" ) && !lab.contains( "{a_axis:.2}%" ) && !lab.contains( "{b_axis:.2}%" ),
    "lab() arm should NOT gain a % suffix — CSS accepts bare <number> here: {lab}"
  );

  let lch = arm_body( "lch" );
  assert!(
    !lch.contains( "{lightness:.2}%" ) && !lch.contains( "{chroma:.2}%" ) && !lch.contains( "{hue:.2}%" ),
    "lch() arm should NOT gain a % suffix — CSS accepts bare <number> here: {lch}"
  );
}
