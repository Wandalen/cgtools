//! Catmull-Rom spline evaluation for fleet patrol paths, ported from
//! `examples/threejs/falling_frontier/src/world/fleet.js`'s use of three.js's
//! `CatmullRomCurve3` (`getPointAt`/`getTangentAt`).
//!
//! Two deliberate simplifications vs. the JS reference, neither of which
//! affects the path's shape, only the exact speed profile along it:
//! - Uniform (not centripetal) parametrization - three.js's default
//!   `curveType` is `'centripetal'`, which reduces cusping on paths with
//!   very uneven segment lengths. This scene's paths are short (3-5 hand-
//!   placed waypoints, no near-degenerate segments), so the plain uniform
//!   form reads the same visually.
//! - Segment-uniform (not arc-length-corrected) `t`-to-position mapping -
//!   three.js's `getPointAt`/`getTangentAt` build an arc-length lookup table
//!   so a ship moves at constant *speed* regardless of segment length; this
//!   maps `t` directly onto segment index instead, so a ship covers a long
//!   segment slightly faster (in wall-clock terms) than a short one. Not
//!   worth the lookup-table machinery for 3-5-waypoint paths where every
//!   segment is a similar length already.

use minwebgl as gl;
use gl::math::F32x2;

/// Evaluates the closed-form Catmull-Rom point at local parameter `t` in
/// `[0, 1]` between `p1` and `p2`, given the point before `p1` and after
/// `p2`.
fn catmull_rom_segment( p0 : F32x2, p1 : F32x2, p2 : F32x2, p3 : F32x2, t : f32 ) -> F32x2
{
  let t2 = t * t;
  let t3 = t2 * t;
  ( p1 * 2.0
    + ( p2 - p0 ) * t
    + ( p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3 ) * t2
    + ( p1 * 3.0 - p0 - p2 * 3.0 + p3 ) * t3
  ) * 0.5
}

/// Derivative of `catmull_rom_segment` with respect to `t` - not
/// normalized, callers that need a direction should normalize the result.
fn catmull_rom_segment_tangent( p0 : F32x2, p1 : F32x2, p2 : F32x2, p3 : F32x2, t : f32 ) -> F32x2
{
  ( ( p2 - p0 )
    + ( p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3 ) * ( 2.0 * t )
    + ( p1 * 3.0 - p0 - p2 * 3.0 + p3 ) * ( 3.0 * t * t )
  ) * 0.5
}

/// Splits a global `[0, 1]` progress value into a segment index and the
/// local `[0, 1]` parameter within that segment, given `segment_count`
/// segments (`points.len() - 1`).
fn segment_and_local_t( progress : f32, segment_count : usize ) -> ( usize, f32 )
{
  let scaled = progress.clamp( 0.0, 1.0 ) * segment_count as f32;
  let segment = ( scaled.floor() as usize ).min( segment_count - 1 );
  ( segment, scaled - segment as f32 )
}

/// The point before `points[i]` for spline purposes - clamped to `points[0]`
/// at the start of the path (no wraparound; these paths are open, not
/// closed loops), matching three.js's non-closed `CatmullRomCurve3`
/// boundary handling.
fn point_at( points : &[ [ f32; 2 ] ], i : isize ) -> F32x2
{
  let last = points.len() as isize - 1;
  F32x2::from( points[ i.clamp( 0, last ) as usize ] )
}

/// Evaluates the path's world position at global progress `t` in `[0, 1]`.
/// `points` must have at least 2 waypoints.
pub fn point_at_progress( points : &[ [ f32; 2 ] ], t : f32 ) -> [ f32; 2 ]
{
  let ( segment, local_t ) = segment_and_local_t( t, points.len() - 1 );
  let i = segment as isize;
  catmull_rom_segment
  (
    point_at( points, i - 1 ), point_at( points, i ), point_at( points, i + 1 ), point_at( points, i + 2 ),
    local_t
  ).to_array()
}

/// Evaluates the path's (non-normalized) tangent direction at global
/// progress `t` in `[0, 1]`.
pub fn tangent_at_progress( points : &[ [ f32; 2 ] ], t : f32 ) -> [ f32; 2 ]
{
  let ( segment, local_t ) = segment_and_local_t( t, points.len() - 1 );
  let i = segment as isize;
  catmull_rom_segment_tangent
  (
    point_at( points, i - 1 ), point_at( points, i ), point_at( points, i + 1 ), point_at( points, i + 2 ),
    local_t
  ).to_array()
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

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
