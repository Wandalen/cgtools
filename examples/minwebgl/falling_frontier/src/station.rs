//! Space station: procedural core/ring/spokes/docking-modules/beacon,
//! ported from
//! `examples/threejs/falling_frontier/src/world/spaceStation.js`.
//! Static placement only (M4 scope) - the JS `spinStations` idle rotation
//! is cosmetic polish, not tracked here.

use minwebgl as gl;
use gl::math::{ F32x3, F32x4x4, mat3x3h };

use crate::hull::{ HullPart, upload_mesh, AMBIENT_LIT, AMBIENT_GLOW };
use crate::primitives::{ box_mesh, cylinder_mesh, torus_mesh, icosphere };

// STATION_SPEC.position from spaceStation.js - negative Y sits it below the
// tactical grid plane (y = 0), same as the JS reference.
const STATION_POSITION : [ f32; 3 ] = [ -89.6, -15.0, 17.32 ];

// COLORS.shipDark / shipHull / engineGlow, matching ships.rs.
const DARK : [ f32; 3 ] = [ 0.3373, 0.4392, 0.4902 ];
const METALLIC : [ f32; 3 ] = [ 0.8, 0.8471, 0.8863 ];
const GLOW : [ f32; 3 ] = [ 0.0, 0.9412, 1.0 ];

pub struct Station
{
  parts : Vec< HullPart >,
}

impl Station
{
  /// `pick_id` is shared by every part below - the station is one
  /// selectable object, not one per module (see `picking.rs`).
  pub fn new( gl : &gl::GL, pick_id : i32 ) -> Self
  {
    let station_transform = mat3x3h::translation( F32x3::from( STATION_POSITION ) );
    let mut parts = Vec::new();

    // Central axis core cylinder.
    push_part( gl, &mut parts, station_transform, F32x4x4::identity(), &cylinder_mesh( 4.0, 4.0, 30.0, 12 ), DARK, AMBIENT_LIT, pick_id );

    // Rotating ring structure - three.js's TorusGeometry lies flat in XY by
    // default (donut-hole axis Z); rotating X by 90° stands it up around Y,
    // same as `ring.rotation.x = Math.PI / 2` in the JS.
    let ring = mat3x3h::rot( std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
    push_part( gl, &mut parts, station_transform, ring, &torus_mesh( 18.0, 1.8, 8, 24 ), METALLIC, AMBIENT_LIT, pick_id );

    // Connecting spoke arms.
    for i in 0 .. 4
    {
      let angle = std::f32::consts::FRAC_PI_4 * i as f32;
      let spoke = mat3x3h::rot( 0.0, angle, 0.0 );
      push_part( gl, &mut parts, station_transform, spoke, &box_mesh( 16.0, 0.5, 0.75 ), DARK, AMBIENT_LIT, pick_id );
    }

    // Docking modules ringed around the core.
    for i in 0 .. 6
    {
      let angle = std::f32::consts::FRAC_PI_3 * i as f32;
      let pos = mat3x3h::translation( F32x3::new( angle.cos() * 18.0, 0.0, angle.sin() * 18.0 ) );
      push_part( gl, &mut parts, station_transform, pos, &cylinder_mesh( 2.0, 2.5, 6.0, 8 ), METALLIC, AMBIENT_LIT, pick_id );
    }

    // Beacon glow light.
    let beacon = mat3x3h::translation( F32x3::new( 0.0, 16.0, 0.0 ) ) * mat3x3h::scale( F32x3::splat( 0.8 ) );
    let ( beacon_positions, beacon_indices ) = icosphere();
    let ( vao, index_count ) = upload_mesh( gl, &beacon_positions, &beacon_indices );
    parts.push( HullPart { vao, index_count, model : station_transform * beacon, color : GLOW, ambient : AMBIENT_GLOW, pick_id } );

    Self { parts }
  }

  pub fn parts( &self ) -> &[ HullPart ]
  {
    &self.parts
  }
}

#[ expect( clippy::too_many_arguments, reason = "mirrors the JS builder's own parameter surface (transform + mesh + material + now pick id) - splitting it up would just move the same argument count into a struct with no real grouping" ) ]
fn push_part
(
  gl : &gl::GL,
  parts : &mut Vec< HullPart >,
  station_transform : F32x4x4,
  local_transform : F32x4x4,
  mesh : &( Vec< [ f32; 3 ] >, Vec< u32 > ),
  color : [ f32; 3 ],
  ambient : f32,
  pick_id : i32,
)
{
  let ( vao, index_count ) = upload_mesh( gl, &mesh.0, &mesh.1 );
  let model = station_transform * local_transform;
  parts.push( HullPart { vao, index_count, model, color, ambient, pick_id } );
}
