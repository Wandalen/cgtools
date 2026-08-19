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
