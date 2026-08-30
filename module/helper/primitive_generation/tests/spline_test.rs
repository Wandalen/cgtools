//! Integration tests for `primitive_generation::spline`.
//!
//! Catmull-Rom evaluation over a waypoint path, exercised through
//! `point_at_progress` and `tangent_at_progress` — the two items `spline.rs`
//! re-exports through `mod_interface!`, and the whole of what this module
//! offers a caller. The private segment helpers underneath
//! (`catmull_rom_segment`, `segment_and_local_t`, `point_at`) are reached
//! through them rather than directly, so nothing here needs the crate's
//! interior and `rulebook.md § Test placement` puts the file here.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::{ point_at_progress, tangent_at_progress };

  // A straight-line path exercises every segment of the pipeline (boundary
  // clamping, segment selection) while keeping the expected answer trivial
  // to state independently: any point/tangent on a straight line is
  // computable by hand, unlike a genuinely curved path.
  const STRAIGHT_LINE : [ [ f32; 2 ]; 4 ] = [ [ 0.0, 0.0 ], [ 10.0, 0.0 ], [ 20.0, 0.0 ], [ 30.0, 0.0 ] ];

  fn assert_close( a : [ f32; 2 ], b : [ f32; 2 ], eps : f32 )
  {
    assert!( ( a[ 0 ] - b[ 0 ] ).abs() < eps && ( a[ 1 ] - b[ 1 ] ).abs() < eps, "{a:?} != {b:?}" );
  }

  #[ test ]
  fn progress_zero_is_first_waypoint()
  {
    assert_close( point_at_progress( &STRAIGHT_LINE, 0.0 ), STRAIGHT_LINE[ 0 ], 1e-4 );
  }

  #[ test ]
  fn progress_one_is_last_waypoint()
  {
    assert_close( point_at_progress( &STRAIGHT_LINE, 1.0 ), STRAIGHT_LINE[ 3 ], 1e-4 );
  }

  #[ test ]
  fn progress_out_of_range_clamps()
  {
    assert_close( point_at_progress( &STRAIGHT_LINE, -1.0 ), STRAIGHT_LINE[ 0 ], 1e-4 );
    assert_close( point_at_progress( &STRAIGHT_LINE, 2.0 ), STRAIGHT_LINE[ 3 ], 1e-4 );
  }

  #[ test ]
  fn straight_line_point_is_linear_interpolation()
  {
    // On a straight, evenly-spaced line, Catmull-Rom degenerates to plain
    // linear interpolation - progress 0.5 (global) sits at the path's exact
    // midpoint regardless of which segment it falls in.
    let p = point_at_progress( &STRAIGHT_LINE, 0.5 );
    assert_close( p, [ 15.0, 0.0 ], 1e-3 );
  }

  #[ test ]
  fn straight_line_tangent_points_along_the_line()
  {
    let t = tangent_at_progress( &STRAIGHT_LINE, 0.5 );
    // Tangent must point in +X (the line's direction), not -X or off-axis.
    assert!( t[ 0 ] > 0.0, "tangent {t:?} does not point along +X" );
    assert!( t[ 1 ].abs() < 1e-3, "tangent {t:?} has unexpected Y component" );
  }

  #[ test ]
  fn two_waypoint_path_does_not_panic()
  {
    // The minimum valid path length (one segment) - exercises point_at's
    // clamping at both ends with no interior points to fall back on.
    let path = [ [ 0.0, 0.0 ], [ 5.0, 5.0 ] ];
    assert_close( point_at_progress( &path, 0.0 ), path[ 0 ], 1e-4 );
    assert_close( point_at_progress( &path, 1.0 ), path[ 1 ], 1e-4 );
  }
}
