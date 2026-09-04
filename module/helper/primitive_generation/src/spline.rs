//! Catmull-Rom spline evaluation over a `[f32; 2]` waypoint path — evaluates
//! a global `[0, 1]` progress value into a world position or tangent
//! direction, e.g. for a patrol path or a camera move. Ported from
//! three.js's `CatmullRomCurve3` (`getPointAt`/`getTangentAt`) usage.
//!
//! Two deliberate simplifications, neither of which affects the path's
//! shape, only the exact speed profile along it:
//! - Uniform (not centripetal) parametrization - three.js's default
//!   `curveType` is `'centripetal'`, which reduces cusping on paths with
//!   very uneven segment lengths. Plain uniform parametrization reads the
//!   same visually as long as segments are a similar length.
//! - Segment-uniform (not arc-length-corrected) `t`-to-position mapping -
//!   three.js's `getPointAt`/`getTangentAt` build an arc-length lookup table
//!   so movement along the path happens at constant *speed* regardless of
//!   segment length; this maps `t` directly onto segment index instead, so
//!   a long segment gets covered slightly faster (in wall-clock terms) than
//!   a short one. Not worth the lookup-table machinery unless a path has
//!   segments of very different lengths.

mod private
{
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
  /// at the start of the path (no wraparound; open path, not a closed loop),
  /// matching three.js's non-closed `CatmullRomCurve3` boundary handling.
  fn point_at( points : &[ [ f32; 2 ] ], i : isize ) -> F32x2
  {
    let last = points.len() as isize - 1;
    F32x2::from( points[ i.clamp( 0, last ) as usize ] )
  }

  /// Evaluates the path's world position at global progress `t` in `[0, 1]`.
  /// `points` must have at least 2 waypoints.
  #[ must_use ]
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
  #[ must_use ]
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
}

crate::mod_interface!
{
  orphan use
  {
    point_at_progress,
    tangent_at_progress,
  };
}
