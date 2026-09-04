//! CPU-side view-zone boundary polyline builder, ported from
//! `buildBoundaryPolyline`/`sampleBoundaryRadius`/`normalizeAngle` in
//! `examples/threejs/falling_frontier/src/world/tacticalGrid.js`.
//!
//! Builds a closed polyline (in world XZ) around a focus point: a faceted
//! circle of `BASE_CIRCLE_SEGMENTS` segments, replaced locally by extra
//! angle samples wrapping the near side of any blocking asteroid, so the
//! ribbon shader (`grid.frag`) can wrap tight around obstacles instead of
//! rendering a perfect circle. See `PORT_PLAN.md`'s "Resume here" section
//! for the full design rationale.

pub const BASE_CIRCLE_SEGMENTS : usize = 32;
pub const ARC_SUBDIVISIONS : usize = 4;
pub const TANGENT_EPS : f32 = 0.0025;
pub const MAX_BOUNDARY_BLOCKERS : usize = 12;
pub const MAX_BOUNDARY_PTS : usize = 128;

/// Per-blocker angle samples pushed in `build_boundary_polyline`: the 4 fixed
/// tangent-adjacent samples (`a_start - eps`, `a_start`, `a_end`,
/// `a_end + eps`) plus `ARC_SUBDIVISIONS - 1` interior arc samples.
const ANGLES_PER_BLOCKER : usize = 4 + ( ARC_SUBDIVISIONS - 1 );

// `build_boundary_polyline` truncates the sorted angle list via
// `&angles[ .. limit ]` rather than downsampling it, so if candidate count
// ever exceeded MAX_BOUNDARY_PTS, the boundary would silently collapse to a
// narrow arc (the lowest-angle candidates only) instead of wrapping the full
// circle. This assertion keeps that scenario unreachable at compile time
// instead of relying on the current constants happening to stay small.
const _ : () = assert!(
  BASE_CIRCLE_SEGMENTS + MAX_BOUNDARY_BLOCKERS * ANGLES_PER_BLOCKER <= MAX_BOUNDARY_PTS,
  "boundary.rs: BASE_CIRCLE_SEGMENTS + MAX_BOUNDARY_BLOCKERS * ANGLES_PER_BLOCKER exceeds \
  MAX_BOUNDARY_PTS -- build_boundary_polyline would silently truncate to a narrow arc"
);

/// A view-blocking obstacle, already padded (see `asteroids::BLOCK_PADDING`)
/// so the boundary wraps outside the visible rock, not through it.
pub struct Blocker
{
  pub x : f32,
  pub z : f32,
  pub radius : f32,
}

struct ActiveBlocker
{
  x : f32,
  z : f32,
  r : f32,
  a_start : f32,
  a_end : f32,
}

fn normalize_angle( a : f32 ) -> f32
{
  let two_pi = std::f32::consts::TAU;
  ( ( a % two_pi ) + two_pi ) % two_pi
}

/// Finds the distance along direction (dx, dz) from the focus point at which
/// the ray first hits a blocker, taking the nearest hit across all active
/// blockers (so overlapping blockers correctly shadow each other), capped at
/// `view_radius`.
fn sample_boundary_radius( dx : f32, dz : f32, focus_x : f32, focus_z : f32, view_radius : f32, active : &[ ActiveBlocker ] ) -> f32
{
  let mut r = view_radius;
  for b in active
  {
    let ocx = b.x - focus_x;
    let ocz = b.z - focus_z;
    let t_center = ocx * dx + ocz * dz;
    if t_center <= 0.0 { continue; }
    let ray_x = ocx - dx * t_center;
    let ray_z = ocz - dz * t_center;
    let dist_to_ray = ray_x.hypot( ray_z );
    if dist_to_ray >= b.r { continue; }
    let half_chord = ( b.r * b.r - dist_to_ray * dist_to_ray ).max( 0.0 ).sqrt();
    let t_hit = t_center - half_chord;
    if t_hit > 0.0 && t_hit < r { r = t_hit; }
  }
  r
}

/// Builds the closed boundary polyline into `out`, returning the point count
/// actually written (always `<= MAX_BOUNDARY_PTS` and `<= out.len()`).
pub fn build_boundary_polyline( focus_x : f32, focus_z : f32, view_radius : f32, blockers : &[ Blocker ], out : &mut [ [ f32; 2 ]; MAX_BOUNDARY_PTS ] ) -> usize
{
  let mut active : Vec< ActiveBlocker > = Vec::new();
  for b in blockers
  {
    if active.len() >= MAX_BOUNDARY_BLOCKERS { break; }
    let ocx = b.x - focus_x;
    let ocz = b.z - focus_z;
    let d = ocx.hypot( ocz );
    // Focus point is inside the asteroid - degenerate, ignore.
    if d <= b.radius { continue; }
    let tangent_len = ( d * d - b.radius * b.radius ).max( 0.0 ).sqrt();
    // Doesn't reach into the view radius - no wrap needed.
    if tangent_len >= view_radius { continue; }
    let center_angle = ocz.atan2( ocx );
    let half_angle = ( b.radius / d ).min( 1.0 ).asin();
    active.push( ActiveBlocker { x : b.x, z : b.z, r : b.radius, a_start : center_angle - half_angle, a_end : center_angle + half_angle } );
  }

  let mut angles : Vec< f32 > = ( 0 .. BASE_CIRCLE_SEGMENTS )
  .map( | i | ( i as f32 / BASE_CIRCLE_SEGMENTS as f32 ) * std::f32::consts::TAU )
  .collect();

  for w in &active
  {
    angles.push( w.a_start - TANGENT_EPS );
    angles.push( w.a_start );
    angles.push( w.a_end );
    angles.push( w.a_end + TANGENT_EPS );
    for k in 1 .. ARC_SUBDIVISIONS
    {
      angles.push( w.a_start + ( w.a_end - w.a_start ) * ( k as f32 / ARC_SUBDIVISIONS as f32 ) );
    }
  }

  for a in &mut angles { *a = normalize_angle( *a ); }
  angles.sort_by( | a, b | a.partial_cmp( b ).unwrap() );

  let mut count = 0;
  let mut last_angle : Option< f32 > = None;
  let limit = angles.len().min( MAX_BOUNDARY_PTS ).min( out.len() );
  for &angle in &angles[ .. limit ]
  {
    if let Some( last ) = last_angle
      && ( angle - last ).abs() < 1e-6 { continue; }
    last_angle = Some( angle );

    let dx = angle.cos();
    let dz = angle.sin();
    let r = sample_boundary_radius( dx, dz, focus_x, focus_z, view_radius, &active );
    out[ count ] = [ focus_x + dx * r, focus_z + dz * r ];
    count += 1;
  }
  count
}

// `build_boundary_polyline` and its helpers are pure CPU-side geometry, no
// GL involved -- unlike most of this crate, genuinely testable natively.
#[ cfg( test ) ]
mod tests
{
  use super::*;

  fn angle_of( p : [ f32; 2 ], focus_x : f32, focus_z : f32 ) -> f32
  {
    normalize_angle( ( p[ 1 ] - focus_z ).atan2( p[ 0 ] - focus_x ) )
  }

  fn dist_of( p : [ f32; 2 ], focus_x : f32, focus_z : f32 ) -> f32
  {
    ( p[ 0 ] - focus_x ).hypot( p[ 1 ] - focus_z )
  }

  fn angular_distance( a : f32, b : f32 ) -> f32
  {
    let tau = std::f32::consts::TAU;
    let d = ( a - b ).abs() % tau;
    d.min( tau - d )
  }

  #[ test ]
  fn normalize_angle_wraps_into_0_tau()
  {
    let tau = std::f32::consts::TAU;
    assert!( normalize_angle( 0.0 ).abs() < 1e-6 );
    assert!( normalize_angle( tau ).abs() < 1e-5, "a full turn wraps back to 0" );
    assert!( ( normalize_angle( -0.1 ) - ( tau - 0.1 ) ).abs() < 1e-5, "negative angles wrap into the top of the range" );
    assert!( ( normalize_angle( tau + 0.3 ) - 0.3 ).abs() < 1e-5, "angles past a full turn wrap back down" );
  }

  #[ test ]
  fn no_blockers_yields_a_plain_faceted_circle()
  {
    let mut out = [ [ 0.0_f32; 2 ]; MAX_BOUNDARY_PTS ];
    let count = build_boundary_polyline( 0.0, 0.0, 10.0, &[], &mut out );

    assert_eq!( count, BASE_CIRCLE_SEGMENTS, "no blockers means exactly the base circle's own segment count" );
    for &p in &out[ .. count ]
    {
      assert!( ( dist_of( p, 0.0, 0.0 ) - 10.0 ).abs() < 1e-4, "every point of a plain circle sits at view_radius" );
    }
  }

  #[ test ]
  fn blocker_containing_the_focus_point_is_ignored()
  {
    let mut out = [ [ 0.0_f32; 2 ]; MAX_BOUNDARY_PTS ];
    // Focus (0,0) is inside this blocker's radius -- degenerate, per the
    // `d <= b.radius` guard.
    let blockers = [ Blocker { x : 0.5, z : 0.0, radius : 2.0 } ];
    let count = build_boundary_polyline( 0.0, 0.0, 10.0, &blockers, &mut out );

    assert_eq!( count, BASE_CIRCLE_SEGMENTS, "a blocker enclosing the focus point must be skipped entirely" );
    for &p in &out[ .. count ]
    {
      assert!( ( dist_of( p, 0.0, 0.0 ) - 10.0 ).abs() < 1e-4 );
    }
  }

  #[ test ]
  fn blocker_whose_tangent_stays_outside_view_radius_is_ignored()
  {
    let mut out = [ [ 0.0_f32; 2 ]; MAX_BOUNDARY_PTS ];
    // d = 20, radius = 1 -> tangent_len = sqrt(399) ~= 19.97, still >= view_radius(5).
    let blockers = [ Blocker { x : 20.0, z : 0.0, radius : 1.0 } ];
    let count = build_boundary_polyline( 0.0, 0.0, 5.0, &blockers, &mut out );

    assert_eq!( count, BASE_CIRCLE_SEGMENTS, "a blocker whose tangent never reaches into view_radius needs no wrap" );
    for &p in &out[ .. count ]
    {
      assert!( ( dist_of( p, 0.0, 0.0 ) - 5.0 ).abs() < 1e-4 );
    }
  }

  #[ test ]
  fn active_blocker_pulls_the_boundary_in_to_its_near_surface()
  {
    let mut out = [ [ 0.0_f32; 2 ]; MAX_BOUNDARY_PTS ];
    // d = 5, radius = 1 -> nearest approach along the direct line is d - radius = 4.0,
    // strictly closer than the tangent-line hit distance sqrt(d^2 - radius^2) ~= 4.899,
    // so 4.0 is the true minimum over every possible direction, not just the sampled ones.
    let blockers = [ Blocker { x : 5.0, z : 0.0, radius : 1.0 } ];
    let count = build_boundary_polyline( 0.0, 0.0, 10.0, &blockers, &mut out );

    let min_dist = out[ .. count ].iter().map( | &p | dist_of( p, 0.0, 0.0 ) ).fold( f32::INFINITY, f32::min );
    let max_dist = out[ .. count ].iter().map( | &p | dist_of( p, 0.0, 0.0 ) ).fold( 0.0_f32, f32::max );

    assert!( ( min_dist - 4.0 ).abs() < 1e-3, "the boundary must wrap in to the blocker's near surface (d - radius); got {min_dist}" );
    assert!( ( max_dist - 10.0 ).abs() < 1e-4, "away from the blocker's narrow wedge the boundary stays at view_radius; got {max_dist}" );
  }

  #[ test ]
  fn blockers_beyond_the_cap_are_dropped()
  {
    let mut out = [ [ 0.0_f32; 2 ]; MAX_BOUNDARY_PTS ];
    let tau = std::f32::consts::TAU;
    let n = MAX_BOUNDARY_BLOCKERS + 1;
    let blockers : Vec< Blocker > = ( 0 .. n )
    .map( | i |
    {
      let a = i as f32 * tau / n as f32;
      Blocker { x : 5.0 * a.cos(), z : 5.0 * a.sin(), radius : 1.0 }
    } )
    .collect();
    let count = build_boundary_polyline( 0.0, 0.0, 10.0, &blockers, &mut out );

    // Blocker index MAX_BOUNDARY_BLOCKERS (the 13th, 0-indexed) is dropped
    // by the `active.len() >= MAX_BOUNDARY_BLOCKERS` cap -- its own angular
    // region must therefore show no wrap at all, unlike every other
    // blocker's identically-shaped wedge.
    let excluded_angle = normalize_angle( MAX_BOUNDARY_BLOCKERS as f32 * tau / n as f32 );
    let nearest = out[ .. count ].iter()
    .map( | &p | ( angular_distance( angle_of( p, 0.0, 0.0 ), excluded_angle ), dist_of( p, 0.0, 0.0 ) ) )
    .min_by( | a, b | a.0.partial_cmp( &b.0 ).unwrap() )
    .expect( "at least one output point must exist" );

    assert!( nearest.0 < 0.15, "expected a sampled point near the excluded blocker's own angle; nearest was {} rad away", nearest.0 );
    assert!
    (
      ( nearest.1 - 10.0 ).abs() < 1e-3,
      "the 13th blocker must be dropped by the cap, leaving this region at view_radius; got {}", nearest.1
    );
  }
}
