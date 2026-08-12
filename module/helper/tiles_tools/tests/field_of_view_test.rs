//! Test for the `field_of_view` module's direct `VisibilityMap` construction API —
//! `VisibilityMap::new` + `set_visibility` — which the feature-gated integration
//! suite (`tests/integration/field_of_view_tests.rs`) never exercises directly
//! (it only observes maps produced by `calculate_fov`).
//!
//! Relocated from `src/field_of_view.rs` by task 072. The module's five other
//! public-surface inline tests were consolidated onto their near-verbatim twins in
//! the integration suite; the formerly-inline builder-state test moved here once
//! the `algorithm()`/`includes_viewer()` getters made that state publicly
//! observable, per the all-tests-in-tests/ convention.

#![ cfg( feature = "enabled" ) ]

use tiles_tools::field_of_view::{ FieldOfView, FOVAlgorithm, VisibilityMap, VisibilityState };
use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, EightConnected };

#[ expect( clippy::float_cmp, reason = "the light level read back is the exact literal just stored; no arithmetic in between" ) ]
#[ test ]
fn test_visibility_map_basic()
{
  let mut visibility_map = VisibilityMap::< SquareCoord< EightConnected > >::new();

  let target = SquareCoord::< EightConnected >::new( 3, 3 );
  visibility_map.set_visibility( &target, VisibilityState::new( true, 5, 0.7 ) );

  assert!( visibility_map.is_visible( &target ) );
  assert_eq!( visibility_map.distance_to( &target ), Some( 5 ) );
  assert_eq!( visibility_map.light_level_at( &target ), 0.7 );
}

/// Pins `FieldOfView`'s builder defaults and overrides through the `algorithm()`/
/// `includes_viewer()` getters: `new()` defaults to `Shadowcasting` with the viewer
/// included; `with_algorithm` + `include_viewer( false )` store what they were given.
/// ( `calculate_fov` output cannot distinguish which algorithm actually ran, so the
/// gated integration suite's behavioral coverage cannot replace this. )
#[ test ]
fn test_fov_calculator_creation()
{
  let fov = FieldOfView::new();
  assert_eq!( fov.algorithm(), FOVAlgorithm::Shadowcasting );
  assert!( fov.includes_viewer() );

  let ray_fov = FieldOfView::with_algorithm( FOVAlgorithm::RayCasting )
  .include_viewer( false );
  assert_eq!( ray_fov.algorithm(), FOVAlgorithm::RayCasting );
  assert!( !ray_fov.includes_viewer() );
}
