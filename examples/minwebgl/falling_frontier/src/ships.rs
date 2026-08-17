//! Ship hulls, procedurally composited from box/cylinder/cone primitives -
//! ported from `examples/threejs/falling_frontier/src/world/ships.js`
//! (mesh shapes) and `fleet.js` (roster + starting position/rotation/patrol
//! path).

use minwebgl as gl;
use gl::math::{ F32x3, F32x4x4, mat3x3h };

use crate::hull::{ HullPart, upload_mesh, AMBIENT_LIT, AMBIENT_GLOW };
use crate::primitives::{ box_mesh, cylinder_mesh };
use crate::spline;

// Matches fleet.js's FLEET_Y - all ships fly at the same altitude as the
// asteroid belt, so the whole scene reads as one flat tactical plane. `pub`
// since `trajectories.rs` (M7) needs it to place ribbon points at the same
// altitude the ships themselves fly at.
pub const SHIP_Y : f32 = 12.0;

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
  name : &'static str,
  commander : &'static str,
  position : [ f32; 3 ],
  rotation_y : f32,
  /// Patrol waypoints (XZ - every point sits at `SHIP_Y`, same as
  /// `position`), ported from `fleet.js`'s `path`. First waypoint always
  /// equals `position`.
  path : &'static [ [ f32; 2 ] ],
  /// Progress-per-frame at `speed_multiplier = 1.0` - matches the JS
  /// reference's own `progress += item.speed * speedMultiplier` running
  /// once per rendered frame (not scaled by `delta_time`), so ship speed
  /// here is implicitly tied to display refresh rate exactly like the JS.
  speed : f32,
  trajectory_color : [ f32; 3 ],
  /// `None` matches `fleet.js`'s `sensorRadius: null` - only some ships
  /// show a sensor ring.
  sensor_radius : Option< f32 >,
}

/// Number of pickable ship ids `main.rs` needs to reserve — kept in sync
/// with `FLEET` automatically rather than as a separate literal.
pub const SHIP_COUNT : usize = FLEET.len();

// trajectoryColor hexes (0x00f0ff / 0x00e5ff) from fleet.js.
const TRAJECTORY_CYAN : [ f32; 3 ] = [ 0.0, 0.9412, 1.0 ];
const TRAJECTORY_CYAN_DEEP : [ f32; 3 ] = [ 0.0, 0.898, 1.0 ];

const FLEET : [ ShipSpec; 4 ] =
[
  ShipSpec
  {
    kind : ShipKind::Frigate, name : "TDF RELIANT", commander : "NOLAN ADAMS",
    position : [ -41.55, SHIP_Y, 57.63 ], rotation_y : -1.56,
    path : &[ [ -41.55, 57.63 ], [ -61.55, 32.63 ], [ -86.55, 2.63 ], [ -71.55, -42.37 ], [ -31.55, -67.37 ] ],
    speed : 0.0008, trajectory_color : TRAJECTORY_CYAN, sensor_radius : Some( 45.0 ),
  },
  ShipSpec
  {
    kind : ShipKind::Cruiser, name : "TDF PINNACLE", commander : "CAPTAIN STERLING",
    position : [ -58.26, SHIP_Y, -145.5 ], rotation_y : -0.85,
    path : &[ [ -58.26, -145.5 ], [ -38.26, -195.5 ], [ -8.26, -245.5 ] ],
    speed : 0.0005, trajectory_color : TRAJECTORY_CYAN_DEEP, sensor_radius : Some( 65.0 ),
  },
  ShipSpec
  {
    kind : ShipKind::Corvette, name : "TDF VALIANT", commander : "LT. CHEN",
    position : [ 25.06, SHIP_Y, -15.2 ], rotation_y : -1.02,
    path : &[ [ 25.06, -15.2 ], [ 5.06, -45.2 ], [ 35.06, -75.2 ] ],
    speed : 0.0012, trajectory_color : TRAJECTORY_CYAN, sensor_radius : None,
  },
  ShipSpec
  {
    kind : ShipKind::Scout, name : "TDF OSPREY", commander : "ENS. REYES",
    position : [ 87.85, SHIP_Y, -34.42 ], rotation_y : -1.08,
    path : &[ [ 87.85, -34.42 ], [ 107.85, -64.42 ], [ 67.85, -84.42 ] ],
    speed : 0.001, trajectory_color : TRAJECTORY_CYAN, sensor_radius : None,
  },
];

/// One ship's mutable state - `position`/`rotation_y` start from `FLEET`
/// but move under the M6 gizmo (`drag_to`/`rotate_to`) or M7's patrol
/// (`advance`). Index-aligned with `FLEET`; `pick_id` is stored (rather
/// than re-derived as `id_base + i`) so `drag_to`/`rotate_to`/`advance` can
/// filter `Ships::parts` without needing `id_base` passed back in.
struct ShipObject
{
  position : [ f32; 2 ],
  rotation_y : f32,
  /// `[0, 1]` progress along `FLEET[i].path` - wraps back to `0` past `1`,
  /// matching the JS reference's own `if (progress > 1) progress = 0`.
  progress : f32,
  pick_id : i32,
}

impl ShipObject
{
  fn transform( &self ) -> F32x4x4
  {
    mat3x3h::translation( F32x3::new( self.position[ 0 ], SHIP_Y, self.position[ 1 ] ) )
    * mat3x3h::rot( 0.0, self.rotation_y, 0.0 )
  }
}

pub struct Ships
{
  parts : Vec< HullPart >,
  objects : Vec< ShipObject >,
}

impl Ships
{
  /// `id_base` is the first pick id this fleet may hand out - ship `i` (and
  /// every `HullPart` making it up) gets `id_base + i` (see `picking.rs`);
  /// the caller reserves `SHIP_COUNT` contiguous ids starting there.
  pub fn new( gl : &gl::GL, id_base : i32 ) -> Self
  {
    let mut parts = Vec::new();
    let mut objects = Vec::with_capacity( FLEET.len() );
    for ( i, spec ) in FLEET.iter().enumerate()
    {
      let pick_id = id_base + i as i32;
      let object = ShipObject { position : [ spec.position[ 0 ], spec.position[ 2 ] ], rotation_y : spec.rotation_y, progress : 0.0, pick_id };
      let ship_transform = object.transform();
      objects.push( object );

      match spec.kind
      {
        ShipKind::Cruiser => build_cruiser( gl, &mut parts, ship_transform, pick_id ),
        ShipKind::Frigate => build_frigate_or_corvette( gl, &mut parts, ship_transform, pick_id, 18.0, 8.0 ),
        ShipKind::Corvette => build_frigate_or_corvette( gl, &mut parts, ship_transform, pick_id, 12.0, 8.0 ),
        ShipKind::Scout => build_frigate_or_corvette( gl, &mut parts, ship_transform, pick_id, 8.0, 5.0 ),
      }
    }

    Self { parts, objects }
  }

  pub fn parts( &self ) -> &[ HullPart ]
  {
    &self.parts
  }

  pub fn position( &self, index : usize ) -> [ f32; 2 ]
  {
    self.objects[ index ].position
  }

  pub fn rotation_y( &self, index : usize ) -> f32
  {
    self.objects[ index ].rotation_y
  }

  /// The `index`-th ship's current world transform - what the gizmo (M6)
  /// draws its handle at.
  pub fn object_transform( &self, index : usize ) -> F32x4x4
  {
    self.objects[ index ].transform()
  }

  /// Moves ship `index` to a new XZ position (Y stays at `SHIP_Y`) - called
  /// by the M6 gizmo's translate drag.
  pub fn drag_to( &mut self, index : usize, position : [ f32; 2 ] )
  {
    self.objects[ index ].position = position;
    self.sync_parts( index );
  }

  /// Sets ship `index`'s heading - called by the M6 gizmo's rotate drag.
  pub fn rotate_to( &mut self, index : usize, rotation_y : f32 )
  {
    self.objects[ index ].rotation_y = rotation_y;
    self.sync_parts( index );
  }

  /// This ship's patrol waypoints (XZ, `SHIP_Y` implicit) - feeds both
  /// `advance` and the M7 trajectory ribbon visual. Static spec data, so
  /// this ignores drag state deliberately - the ribbon shows the *planned*
  /// route, not wherever the gizmo temporarily moved the ship.
  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn path( &self, index : usize ) -> &'static [ [ f32; 2 ] ]
  {
    FLEET[ index ].path
  }

  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn trajectory_color( &self, index : usize ) -> [ f32; 3 ]
  {
    FLEET[ index ].trajectory_color
  }

  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn speed( &self, index : usize ) -> f32
  {
    FLEET[ index ].speed
  }

  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn sensor_radius( &self, index : usize ) -> Option< f32 >
  {
    FLEET[ index ].sensor_radius
  }

  /// M8: the HUD's unit-info card reads name/commander/class off a
  /// selected ship, ported from `fleet.js`'s own `name`/`commander` and
  /// `ships.js`'s `userData.type`.
  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn name( &self, index : usize ) -> &'static str
  {
    FLEET[ index ].name
  }

  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn commander( &self, index : usize ) -> &'static str
  {
    FLEET[ index ].commander
  }

  #[ expect( clippy::unused_self, reason = "reads static FLEET spec data, but kept as a method (not an associated fn) for call-site consistency with position()/rotation_y() and friends, which do need self" ) ]
  pub fn class_label( &self, index : usize ) -> &'static str
  {
    match FLEET[ index ].kind
    {
      ShipKind::Frigate => "FRIGATE",
      ShipKind::Corvette => "CORVETTE",
      ShipKind::Scout => "SCOUT",
      ShipKind::Cruiser => "CRUISER",
    }
  }

  /// Advances ship `index` along its patrol path by `delta_progress`
  /// (already scaled by speed and any playback multiplier - see
  /// `FLEET`'s own `speed` doc), wrapping past `1.0` back to `0.0`
  /// (`main.js`'s `if (progress > 1) progress = 0`), and orients it along
  /// the path's tangent at the new progress (`item.mesh.lookAt(pos +
  /// tangent)` in the JS - our ships' local forward is `-Z`, same
  /// convention `-> rotation_y = tangent.x.atan2(tangent.z)` already uses
  /// for the M6 gizmo's rotate-drag angle, verified visually there).
  pub fn advance( &mut self, index : usize, delta_progress : f32 )
  {
    let mut progress = self.objects[ index ].progress + delta_progress;
    if progress > 1.0 { progress = 0.0; }
    self.objects[ index ].progress = progress;

    let path = FLEET[ index ].path;
    self.objects[ index ].position = spline::point_at_progress( path, progress );
    let tangent = spline::tangent_at_progress( path, progress );
    self.objects[ index ].rotation_y = tangent[ 0 ].atan2( tangent[ 1 ] );

    self.sync_parts( index );
  }

  /// Recomputes `model` for every `HullPart` sharing ship `index`'s
  /// `pick_id` - a ship is several parts (hull, bow, bridge, engines...),
  /// all of which must move together.
  fn sync_parts( &mut self, index : usize )
  {
    let object_transform = self.objects[ index ].transform();
    let pick_id = self.objects[ index ].pick_id;
    for part in self.parts.iter_mut().filter( | p | p.pick_id == pick_id )
    {
      part.set_model( object_transform );
    }
  }
}

#[ expect( clippy::too_many_arguments, reason = "mirrors the JS builder's own parameter surface (transform + mesh + material + now pick id) - splitting it up would just move the same argument count into a struct with no real grouping" ) ]
fn push_part
(
  gl : &gl::GL,
  parts : &mut Vec< HullPart >,
  ship_transform : F32x4x4,
  local_transform : F32x4x4,
  mesh : &( Vec< [ f32; 3 ] >, Vec< u32 > ),
  color : [ f32; 3 ],
  ambient : f32,
  pick_id : i32,
)
{
  let ( vao, index_count ) = upload_mesh( gl, &mesh.0, &mesh.1 );
  let model = ship_transform * local_transform;
  parts.push( HullPart { vao, index_count, local_transform, model, color, ambient, pick_id } );
}

fn build_cruiser( gl : &gl::GL, parts : &mut Vec< HullPart >, ship_transform : F32x4x4, pick_id : i32 )
{
  push_part( gl, parts, ship_transform, F32x4x4::identity(), &box_mesh( 4.0, 3.0, 14.0 ), SHIP_HULL, AMBIENT_LIT, pick_id );

  let bow = mat3x3h::translation( F32x3::new( 0.0, 0.0, -18.0 ) ) * mat3x3h::rot( -std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
  push_part( gl, parts, ship_transform, bow, &cylinder_mesh( 0.0, 4.0, 10.0, 4 ), SHIP_HULL, AMBIENT_LIT, pick_id );

  let bridge = mat3x3h::translation( F32x3::new( 0.0, 4.0, 2.0 ) );
  push_part( gl, parts, ship_transform, bridge, &box_mesh( 2.0, 2.0, 3.0 ), SHIP_DARK, AMBIENT_LIT, pick_id );

  for x in [ -2.5, 0.0, 2.5 ]
  {
    let engine = mat3x3h::translation( F32x3::new( x, -0.5, 15.0 ) ) * mat3x3h::rot( std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
    push_part( gl, parts, ship_transform, engine, &cylinder_mesh( 1.2, 1.5, 4.0, 12 ), SHIP_DARK, AMBIENT_LIT, pick_id );

    let glow = mat3x3h::translation( F32x3::new( x, -0.5, 18.0 ) ) * mat3x3h::rot( -std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
    push_part( gl, parts, ship_transform, glow, &cylinder_mesh( 0.0, 1.1, 5.0, 12 ), ENGINE_GLOW, AMBIENT_GLOW, pick_id );
  }
}

fn build_frigate_or_corvette( gl : &gl::GL, parts : &mut Vec< HullPart >, ship_transform : F32x4x4, pick_id : i32, length : f32, wing_span : f32 )
{
  push_part( gl, parts, ship_transform, F32x4x4::identity(), &box_mesh( 2.0, 1.5, length * 0.5 ), SHIP_HULL, AMBIENT_LIT, pick_id );

  let nose = mat3x3h::translation( F32x3::new( 0.0, -0.5, -length * 0.5 - 2.0 ) );
  push_part( gl, parts, ship_transform, nose, &box_mesh( 1.25, 1.0, 3.0 ), SHIP_DARK, AMBIENT_LIT, pick_id );

  let wings = mat3x3h::translation( F32x3::new( 0.0, 0.0, 2.0 ) );
  push_part( gl, parts, ship_transform, wings, &box_mesh( wing_span * 0.5, 0.4, 3.0 ), SHIP_HULL, AMBIENT_LIT, pick_id );

  let engine = mat3x3h::translation( F32x3::new( 0.0, 0.0, length * 0.5 + 1.0 ) ) * mat3x3h::rot( std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
  push_part( gl, parts, ship_transform, engine, &cylinder_mesh( 1.0, 1.2, 3.0, 12 ), SHIP_DARK, AMBIENT_LIT, pick_id );

  let glow = mat3x3h::translation( F32x3::new( 0.0, 0.0, length * 0.5 + 3.0 ) ) * mat3x3h::rot( -std::f32::consts::FRAC_PI_2, 0.0, 0.0 );
  push_part( gl, parts, ship_transform, glow, &cylinder_mesh( 0.0, 0.9, 4.0, 12 ), ENGINE_GLOW, AMBIENT_GLOW, pick_id );
}
