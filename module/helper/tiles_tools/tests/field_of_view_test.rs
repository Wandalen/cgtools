//! Test for the `field_of_view` module's direct `VisibilityMap` construction API —
//! `VisibilityMap::new` + `visibility_set` — which the feature-gated integration
//! suite (`tests/integration/field_of_view_tests.rs`) never exercises directly
//! (it only observes maps produced by `fov_calculate`).
//!
//! Relocated from `src/field_of_view.rs` by task 072. The module's five other
//! public-surface inline tests were consolidated onto their near-verbatim twins in
//! the integration suite; the formerly-inline builder-state test moved here once
//! the `algorithm()`/`includes_viewer()` getters made that state publicly
//! observable, per the all-tests-in-tests/ convention.

#![ cfg( feature = "enabled" ) ]

use tiles_tools::field_of_view::{ FieldOfView, FOVAlgorithm, VisibilityMap, VisibilityState };
use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, EightConnected, FourConnected };
use tiles_tools::coordinates::hexagonal::{ Coordinate as HexCoord, Axial, Pointy };
use std::collections::HashSet;

#[ expect( clippy::float_cmp, reason = "the light level read back is the exact literal just stored; no arithmetic in between" ) ]
#[ test ]
fn test_visibility_map_basic()
{
  let mut visibility_map = VisibilityMap::< SquareCoord< EightConnected > >::new();

  let target = SquareCoord::< EightConnected >::new( 3, 3 );
  visibility_map.visibility_set( &target, VisibilityState::new( true, 5, 0.7 ) );

  assert!( visibility_map.is_visible( &target ) );
  assert_eq!( visibility_map.distance_to( &target ), Some( 5 ) );
  assert_eq!( visibility_map.light_level_at( &target ), 0.7 );
}

/// Pins `FieldOfView`'s builder defaults and overrides through the `algorithm()`/
/// `includes_viewer()` getters: `new()` defaults to `Shadowcasting` with the viewer
/// included; `with_algorithm` + `viewer_include( false )` store what they were given.
/// ( `fov_calculate` output cannot distinguish which algorithm actually ran, so the
/// gated integration suite's behavioral coverage cannot replace this. )
#[ test ]
fn test_fov_calculator_creation()
{
  let fov = FieldOfView::new();
  assert_eq!( fov.algorithm(), FOVAlgorithm::Shadowcasting );
  assert!( fov.includes_viewer() );

  let ray_fov = FieldOfView::with_algorithm( FOVAlgorithm::RayCasting )
  .viewer_include( false );
  assert_eq!( ray_fov.algorithm(), FOVAlgorithm::RayCasting );
  assert!( !ray_fov.includes_viewer() );
}

// test_kind: bug_reproducer(BUG-477)
/// ## Root Cause
/// `octant_shadows_cast`'s per-ring direction filter --
/// `(i + total - octant) % total < 3 || (i + total - octant) % total >
/// total - 3` -- does not select a single wedge of directions around
/// `octant`. For a 6-direction (hex) coordinate system it admits every
/// direction *except* the one exactly opposite `octant` (5 of 6); for an
/// 8-direction (square, 8-connected) system it excludes only the 3
/// directions centered on the opposite side (5 of 8). `shadowcasting_fov_calculate`'s
/// doc comment claimed this "processes octants systematically", implying a
/// clean non-overlapping per-direction partition -- the real, much broader
/// filter makes every one of the `total_directions` calls redundantly
/// recompute nearly the entire reachable area.
/// ## Why Not Caught
/// No existing test compared `Shadowcasting`'s visible-position *set*
/// against another algorithm's -- `tests/integration/field_of_view_tests.rs`
/// only checks aggregate counts/booleans, which can't distinguish "correct
/// but redundant" from "correct and efficient".
/// ## Fix Applied
/// This is a **documentation fix, not a behavior fix** -- see judgment call
/// below. `shadowcasting_fov_calculate` and `octant_shadows_cast`'s doc
/// comments (and an inline comment at the filter itself) now describe the
/// actual overlapping-band behavior and its performance cost, instead of
/// claiming a systematic non-overlapping partition.
/// ## Prevention
/// This test is a **characterization test**, not a classic fail-before/
/// pass-after reproducer: the filter's runtime behavior is intentionally
/// left unchanged (see Pitfall), so there is no observable delta for a
/// test to pin across the fix. Instead, this test provides the concrete
/// verification the underlying finding required before choosing doc-only
/// over a behavior change: it asserts `Shadowcasting` and `FloodFill`
/// produce byte-for-byte identical visible-position sets, including
/// correct occlusion behind a wall, across hex/square4/square8 coordinate
/// systems -- proving the filter is redundant-but-not-incorrect, not merely
/// asserting that from re-reading the formula. It also guards against a
/// future "fix" narrowing the filter to a literal single direction
/// (`i == octant` only): hand-tracing that alternative shows it degenerates
/// into a single straight ray per call (fixed absolute direction vectors
/// compose additively ring-over-ring), which would *under*-cover the true
/// wedge and regress real cells to invisible -- a strictly worse bug than
/// today's redundancy. Any future attempt to narrow this filter for
/// performance must keep this test green.
/// ## Pitfall
/// A filter that looks obviously wrong from its formula alone (5 of 6, or 5
/// of 8, directions admitted per "single-direction" call) is not
/// automatically an incorrectness bug -- when every write into a shared
/// result is idempotent (same computation regardless of which caller
/// performs it, as `visibility_map.visibility_set` is here), redundant
/// over-inclusion only costs performance, never correctness. Conversely, a
/// narrower filter that looks "obviously more correct" can silently
/// under-cover instead -- verify empirically against a reference
/// implementation before changing filter math, never from formula
/// inspection alone.
#[ test ]
fn test_shadowcasting_matches_flood_fill_open_field_all_coordinate_systems()
{
  let hex_viewer = HexCoord::< Axial, Pointy >::new( 0, 0 );
  let hex_shadow : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::Shadowcasting )
    .fov_calculate( &hex_viewer, 4, | _ | false ).visible_positions().collect();
  let hex_flood : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::FloodFill )
    .fov_calculate( &hex_viewer, 4, | _ | false ).visible_positions().collect();
  assert_eq!( hex_shadow, hex_flood, "hex open field: shadowcasting and flood-fill must see identical cells" );

  let sq4_viewer = SquareCoord::< FourConnected >::new( 0, 0 );
  let sq4_shadow : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::Shadowcasting )
    .fov_calculate( &sq4_viewer, 4, | _ | false ).visible_positions().collect();
  let sq4_flood : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::FloodFill )
    .fov_calculate( &sq4_viewer, 4, | _ | false ).visible_positions().collect();
  assert_eq!( sq4_shadow, sq4_flood, "square-4 open field: shadowcasting and flood-fill must see identical cells" );

  let sq8_viewer = SquareCoord::< EightConnected >::new( 0, 0 );
  let sq8_shadow : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::Shadowcasting )
    .fov_calculate( &sq8_viewer, 4, | _ | false ).visible_positions().collect();
  let sq8_flood : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::FloodFill )
    .fov_calculate( &sq8_viewer, 4, | _ | false ).visible_positions().collect();
  assert_eq!( sq8_shadow, sq8_flood, "square-8 open field: shadowcasting and flood-fill must see identical cells" );
}

#[ test ]
fn test_shadowcasting_matches_flood_fill_with_obstacle_hex()
{
  let viewer = HexCoord::< Axial, Pointy >::new( 0, 0 );
  // A 3-cell wall at distance 2, positioned to occlude cells strictly behind it.
  let wall : HashSet< HexCoord< Axial, Pointy > > = [
    HexCoord::< Axial, Pointy >::new( 2, 0 ),
    HexCoord::< Axial, Pointy >::new( 2, -1 ),
    HexCoord::< Axial, Pointy >::new( 1, 1 ),
  ].into_iter().collect();
  let blocks = | c : &HexCoord< Axial, Pointy > | wall.contains( c );

  let shadow : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::Shadowcasting )
    .fov_calculate( &viewer, 5, blocks ).visible_positions().collect();
  let flood : HashSet<_> = FieldOfView::with_algorithm( FOVAlgorithm::FloodFill )
    .fov_calculate( &viewer, 5, blocks ).visible_positions().collect();

  assert_eq!( shadow, flood, "hex with obstacle: shadowcasting and flood-fill must see identical cells, including matching occlusion" );

  // Sanity: the wall does occlude something behind it in both algorithms --
  // otherwise this scenario would not actually exercise occlusion at all.
  let behind_wall = HexCoord::< Axial, Pointy >::new( 4, 0 );
  let shadow_map = FieldOfView::with_algorithm( FOVAlgorithm::Shadowcasting ).fov_calculate( &viewer, 5, blocks );
  let flood_map = FieldOfView::with_algorithm( FOVAlgorithm::FloodFill ).fov_calculate( &viewer, 5, blocks );
  assert_eq!(
    shadow_map.is_visible( &behind_wall ), flood_map.is_visible( &behind_wall ),
    "occlusion behind the wall must agree between algorithms"
  );
}
