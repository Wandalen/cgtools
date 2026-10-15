//! Doc-text regression tests for `field_of_view_demo/readme.md`.
//!
//! Pure `include_str!` + substring assertions -- no library target needed
//! (this crate is binary-only), matching this session's established
//! `include_str!` precedent for doc-only defects that resist black-box
//! runtime testing.

// test_kind: bug_reproducer(BUG-302)
/// ## Root Cause
/// `readme.md` described the demo's field-of-view algorithm lineup as
/// "shadowcasting, ray casting, and Bresenham line tracing" -- an
/// enumerated, completeness-implying list. `src/main.rs` actually
/// exercises 4 algorithms via `FOVAlgorithm::{Shadowcasting, RayCasting,
/// Bresenham, FloodFill}` (see its own `"=== Flood Fill Algorithm ==="`
/// output section), so the readme undercounted by one, omitting flood
/// fill entirely.
/// ## Why Not Caught
/// This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero
/// pre-existing test coverage of any kind, so nothing tied the readme's
/// enumerated claim to the actual `FOVAlgorithm` variants demonstrated.
/// ## Fix Applied
/// Added "and flood fill" to the readme's algorithm list so it names all
/// 4 algorithms the demo actually runs.
/// ## Prevention
/// An enumerated/completeness-style doc claim ("compares X, Y, and Z")
/// needs its own doc-text regression test reading the file's actual
/// prose -- a claim like this can silently undercount again if a 5th
/// algorithm is added to the demo without updating the readme in the same
/// change.
/// ## Pitfall
/// A prose list that reads as a complete enumeration is a falsifiable
/// claim, unlike a loose descriptive summary -- treat it as testable doc
/// text, not as informal color commentary exempt from verification.
#[ test ]
fn readme_lists_all_four_fov_algorithms_including_flood_fill()
{
  let readme = include_str!( "../readme.md" );
  assert!
  (
    readme.contains( "flood fill" ),
    "field_of_view_demo/readme.md's algorithm-lineup sentence must list flood fill alongside \
    shadowcasting, ray casting, and Bresenham line tracing -- the demo actually exercises all 4 \
    via FOVAlgorithm::FloodFill (BUG-302)"
  );
  assert!
  (
    readme.contains( "shadowcasting" ) && readme.contains( "ray casting" )
    && readme.contains( "Bresenham line tracing" ),
    "field_of_view_demo/readme.md must still name the other 3 algorithms alongside flood fill \
    (BUG-302)"
  );
}
