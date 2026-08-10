//! Test for the `field_of_view` module's direct `VisibilityMap` construction API —
//! `VisibilityMap::new` + `set_visibility` — which the feature-gated integration
//! suite (`tests/integration/field_of_view_tests.rs`) never exercises directly
//! (it only observes maps produced by `calculate_fov`).
//!
//! Relocated from `src/field_of_view.rs` by task 072. The module's five other
//! public-surface inline tests were consolidated onto their near-verbatim twins in
//! the integration suite; one builder-state test remains inline as a documented
//! exception (private fields, no public accessor).

#![allow(clippy::float_cmp)] // Tests assert exact stored/configured values; no arithmetic precedes the comparisons.

#![ cfg( feature = "enabled" ) ]


use tiles_tools::field_of_view::{ VisibilityMap, VisibilityState };
use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, EightConnected };

#[ test ]
fn test_visibility_map_basic()
{
  let viewer = SquareCoord::< EightConnected >::new( 0, 0 );
  let mut visibility_map = VisibilityMap::new( viewer, 10 );

  let target = SquareCoord::< EightConnected >::new( 3, 3 );
  visibility_map.set_visibility( &target, VisibilityState::new( true, 5, 0.7 ) );

  assert!( visibility_map.is_visible( &target ) );
  assert_eq!( visibility_map.distance_to( &target ), Some( 5 ) );
  assert_eq!( visibility_map.light_level_at( &target ), 0.7 );
}
