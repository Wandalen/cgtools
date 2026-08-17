//! Ship hulls, procedurally composited from box/cylinder/cone primitives -
//! ported from `examples/threejs/falling_frontier/src/world/ships.js`
//! (mesh shapes) and `fleet.js` (roster + starting position/rotation).
//!
//! M4 scope is static placement only - `fleet.js`'s patrol paths and
//! trajectory ribbons are M7 (fleet motion + trajectories), so every ship
//! here just sits at its starting `position`/`rotation_y` and doesn't move.

use minwebgl as gl;
use gl::math::{ F32x3, F32x4x4, mat3x3h };

use crate::hull::{ HullPart, upload_mesh, AMBIENT_LIT, AMBIENT_GLOW };
use crate::primitives::{ box_mesh, cylinder_mesh };

// Matches fleet.js's FLEET_Y - all ships fly at the same altitude as the
// asteroid belt, so the whole scene reads as one flat tactical plane.
const SHIP_Y : f32 = 12.0;

// COLORS.shipHull / shipDark / engineGlow from
// examples/threejs/falling_frontier/src/config/colors.js.
const SHIP_HULL : [ f32; 3 ] = [ 0.8, 0.8471, 0.8863 ];
const SHIP_DARK : [ f32; 3 ] = [ 0.3373, 0.4392, 0.4902 ];
const ENGINE_GLOW : [ f32; 3 ] = [ 0.0, 0.9412, 1.0 ];

#[ derive( Clone, Copy ) ]
enum ShipKind
{
  Frigate,
  Corvette,
  Scout,
  Cruiser,
}

struct ShipSpec
{
  kind : ShipKind,
  position : [ f32; 3 ],
  rotation_y : f32,
}

const FLEET : [ ShipSpec; 4 ] =
[
  ShipSpec { kind : ShipKind::Frigate, position : [ -41.55, SHIP_Y, 57.63 ], rotation_y : -1.56 },
  ShipSpec { kind : ShipKind::Cruiser, position : [ -58.26, SHIP_Y, -145.5 ], rotation_y : -0.85 },
  ShipSpec { kind : ShipKind::Corvette, position : [ 25.06, SHIP_Y, -15.2 ], rotation_y : -1.02 },
  ShipSpec { kind : ShipKind::Scout, position : [ 87.85, SHIP_Y, -34.42 ], rotation_y : -1.08 },
];

pub struct Ships
{
  parts : Vec< HullPart >,
}

impl Ships
{
  pub fn new( gl : &gl::GL ) -> Self
  {
    let mut parts = Vec::new();
    for spec in &FLEET
    {
      let ship_transform = mat3x3h::translation( F32x3::from( spec.position ) )
      * mat3x3h::rot( 0.0, spec.rotation_y, 0.0 );

      match spec.kind
      {
        ShipKind::Cruiser => build_cruiser( gl, &mut parts, ship_transform ),
        ShipKind::Frigate => build_frigate_or_corvette( gl, &mut parts, ship_transform, 18.0, 8.0 ),
        ShipKind::Corvette => build_frigate_or_corvette( gl, &mut parts, ship_transform, 12.0, 8.0 ),
        ShipKind::Scout => build_frigate_or_corvette( gl, &mut parts, ship_transform, 8.0, 5.0 ),
      }
    }

    Self { parts }
  }

  pub fn parts( &self ) -> &[ HullPart ]
  {
    &self.parts
  }
}

fn push_part
(
  gl : &gl::GL,
  parts : &mut Vec< HullPart >,
  ship_transform : F32x4x4,
  local_transform : F32x4x4,
  mesh : &( Vec< [ f32; 3 ] >, Vec< u32 > ),
  color : [ f32; 3 ],
  ambient : f32,
)
{
  let ( vao, index_count ) = upload_mesh( gl, &mesh.0, &mesh.1 );
  let model = ship_transform * local_transform;
  parts.push( HullPart { vao, index_count, model, color, ambient } );
}

fn build_cruiser( gl : &gl::GL, parts : &mut Vec< HullPart >, ship_transform : F32x4x4 )
{
  push_part( gl, parts, ship_transform, F32x4x4::identity(), &box_mesh( 4.0, 3.0, 14.0 ), SHIP_HULL, AMBIENT_LIT );

  let bow = mat3x3h::translation( F32x3::new( 0.0, 0.0, -18.0 ) ) * mat3x3h::rot( -std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
  push_part( gl, parts, ship_transform, bow, &cylinder_mesh( 0.0, 4.0, 10.0, 4 ), SHIP_HULL, AMBIENT_LIT );

  let bridge = mat3x3h::translation( F32x3::new( 0.0, 4.0, 2.0 ) );
  push_part( gl, parts, ship_transform, bridge, &box_mesh( 2.0, 2.0, 3.0 ), SHIP_DARK, AMBIENT_LIT );

  for x in [ -2.5, 0.0, 2.5 ]
  {
    let engine = mat3x3h::translation( F32x3::new( x, -0.5, 15.0 ) ) * mat3x3h::rot( std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
    push_part( gl, parts, ship_transform, engine, &cylinder_mesh( 1.2, 1.5, 4.0, 12 ), SHIP_DARK, AMBIENT_LIT );

    let glow = mat3x3h::translation( F32x3::new( x, -0.5, 18.0 ) ) * mat3x3h::rot( -std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
    push_part( gl, parts, ship_transform, glow, &cylinder_mesh( 0.0, 1.1, 5.0, 12 ), ENGINE_GLOW, AMBIENT_GLOW );
  }
}

fn build_frigate_or_corvette( gl : &gl::GL, parts : &mut Vec< HullPart >, ship_transform : F32x4x4, length : f32, wing_span : f32 )
{
  push_part( gl, parts, ship_transform, F32x4x4::identity(), &box_mesh( 2.0, 1.5, length * 0.5 ), SHIP_HULL, AMBIENT_LIT );

  let nose = mat3x3h::translation( F32x3::new( 0.0, -0.5, -length * 0.5 - 2.0 ) );
  push_part( gl, parts, ship_transform, nose, &box_mesh( 1.25, 1.0, 3.0 ), SHIP_DARK, AMBIENT_LIT );

  let wings = mat3x3h::translation( F32x3::new( 0.0, 0.0, 2.0 ) );
  push_part( gl, parts, ship_transform, wings, &box_mesh( wing_span * 0.5, 0.4, 3.0 ), SHIP_HULL, AMBIENT_LIT );

  let engine = mat3x3h::translation( F32x3::new( 0.0, 0.0, length * 0.5 + 1.0 ) ) * mat3x3h::rot( std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
  push_part( gl, parts, ship_transform, engine, &cylinder_mesh( 1.0, 1.2, 3.0, 12 ), SHIP_DARK, AMBIENT_LIT );

  let glow = mat3x3h::translation( F32x3::new( 0.0, 0.0, length * 0.5 + 3.0 ) ) * mat3x3h::rot( -std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
  push_part( gl, parts, ship_transform, glow, &cylinder_mesh( 0.0, 0.9, 4.0, 12 ), ENGINE_GLOW, AMBIENT_GLOW );
}
