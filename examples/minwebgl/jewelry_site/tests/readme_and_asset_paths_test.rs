//! Regression tests for `jewelry_site`: readme accuracy and jewelry-preview asset-path coverage.
//!
//! `jewelry_site` is a static HTML/CSS/JS site (`src/main.rs` is an inert wasm-bindgen
//! placeholder — see its own doc comment); the real client logic lives in `index.js`. These tests
//! read `readme.md` via `include_str!` and cross-check the preview-image template against the
//! real files under `assets/jewelry/` via `CARGO_MANIFEST_DIR`.

use std::path::Path;

const README : &str = include_str!( "../readme.md" );

const METALS : [ &str; 3 ] = [ "copper", "gold", "silver" ];
const GEMS : [ &str; 3 ] = [ "red", "white", "green" ];
const RINGS : [ u32; 3 ] = [ 1, 2, 3 ];

/// Strips `<!-- ... -->` HTML comments so a "readme must not claim X" check only inspects text a
/// reader actually sees rendered, not fix-comment prose that legitimately discusses X.
fn strip_html_comments( text : &str ) -> String
{
  let mut result = String::new();
  let mut rest = text;
  while let Some( start ) = rest.find( "<!--" )
  {
    result.push_str( &rest[ ..start ] );
    rest = match rest[ start.. ].find( "-->" )
    {
      Some( end ) => &rest[ start + end + 3.. ],
      None => "",
    };
  }
  result.push_str( rest );
  result
}

/// ## Root Cause
/// `readme.md` claimed a "product-ready 3D jewelry configurator" and listed `WebGL` as a
/// keyword. The site has no `<canvas>`, no WebGL context, and loads no 3D library anywhere —
/// `src/main.rs` is an inert wasm-bindgen placeholder (see its own doc comment) and `index.js`
/// only swaps a single `<img src>` between pre-rendered 2D PNGs.
///
/// ## Why Not Caught
/// The demo visibly "works" as a configurator either way — a reader has no reason to doubt the
/// WebGL/3D claim without grepping the scripts for canvas/WebGL/3D-library usage, and this crate
/// sits in a directory (`examples/minwebgl/`) full of genuinely WebGL-based demos.
///
/// ## Fix Applied
/// Removed the "3D"/"WebGL" claims from the summary paragraph and the keywords line.
///
/// ## Prevention
/// This test strips HTML comments (so the fix comment itself, which legitimately discusses the
/// removed terms, doesn't trip the check) and asserts neither "3D" nor "WebGL" appears in the
/// rendered readme text.
///
/// ## Pitfall
/// A naive "readme must not contain X" text search fails against its own fix comment if that
/// comment quotes X verbatim — strip non-rendered content (HTML comments) before checking what a
/// reader actually sees, not the raw file bytes.
#[ test ]
fn bug_reproducer_bug_xxx_readme_does_not_claim_webgl_or_3d()
{
  let visible = strip_html_comments( README );
  assert!( !visible.contains( "WebGL" ), "readme should not claim WebGL usage (BUG-XXX)" );
  assert!( !visible.contains( "3D" ), "readme should not claim 3D rendering (BUG-XXX)" );
  assert!
  (
    visible.contains( "2D" ),
    "sanity: readme should still describe the real 2D image-swap mechanism"
  );
}

/// ## Root Cause
/// N/A — this test found no defect; it verifies `index.js`'s
/// `./assets/jewelry/${metal}_${gem}_${ring}.png` template (the BUG-109-pattern asset-path risk
/// this crate was flagged for) actually resolves for every `(metal, gem, ring)` combination the
/// configurator's own selector UI can produce.
///
/// ## Why Not Caught
/// N/A — preventative coverage, not a regression test for a found defect.
///
/// ## Fix Applied
/// N/A — no fix; added to close the BUG-109-pattern gap this crate was flagged for.
///
/// ## Prevention
/// Enumerates every `(metal, gem, ring)` triple `index.js`'s `bindSelector` value-getters can
/// produce and asserts the corresponding `assets/jewelry/{metal}_{gem}_{ring}.png` file exists on
/// disk — catches a renamed/missing asset or a metal/gem/ring value drifting out of sync with the
/// real filenames before it becomes a broken image at runtime.
///
/// ## Pitfall
/// A template-built path (`${a}_${b}_${c}.png`) compiles and "looks right" regardless of whether
/// the file it resolves to actually exists — only checking against the real filesystem catches
/// drift between the UI's value vocabulary and the asset directory's real contents.
#[ test ]
fn bug_reproducer_bug_xxx_jewelry_preview_asset_paths_all_exist()
{
  let assets_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "assets" ).join( "jewelry" );
  let mut checked = 0;
  for metal in METALS
  {
    for gem in GEMS
    {
      for ring in RINGS
      {
        let path = assets_dir.join( format!( "{metal}_{gem}_{ring}.png" ) );
        assert!( path.is_file(), "missing jewelry preview asset: {} (BUG-XXX)", path.display() );
        checked += 1;
      }
    }
  }
  assert_eq!( checked, 27, "sanity: 3 metals x 3 gems x 3 rings should be 27 combinations" );
}
