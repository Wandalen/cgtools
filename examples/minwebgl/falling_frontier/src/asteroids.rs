//! Asteroid belt: procedural low-poly "rock" geometry + rendering, pulled
//! forward from M4 just far enough to give M3's view-zone ribbon something
//! to wrap around. Positions/radii ported from `ASTEROID_SPECS` in
//! `examples/threejs/falling_frontier/src/world/asteroidBelt.js`.
//!
//! Simplification vs. the JS reference: JS deforms a `DodecahedronGeometry`
//! (12 faces, three.js `detail=1` subdivision); this jitters
//! `primitives::icosphere()` (20 faces) instead, since the exact polytope is
//! purely cosmetic (`deformToRock`'s own comment: "Purely cosmetic noise")
//! and doesn't affect the boundary polyline / glow math, which only ever
//! sees each asteroid as a padded circle (`blockRadius`).

use minwebgl as gl;
use gl::math::{ F32x3, mat3x3h };
use rand::RngExt as _;

use crate::boundary::Blocker;
use crate::hull::{ HullPart, upload_mesh, AMBIENT_LIT };
use primitive_generation::icosphere;

// Asteroids sit at the same altitude as the ships in the JS scene, so the
// belt reads as one flat tactical plane - kept here even though ships
// themselves aren't ported yet (M4), so the boundary math and grid share the
// same XZ plane asteroids actually occupy.
const ASTEROID_Y : f32 = 12.0;

// deformToRock() bulges vertices up to ~17.5% past the nominal radius, so
// the blocking radius (used by the view-zone ribbon) is padded past the max
// bulge to keep the ribbon clear of the jagged mesh.
const BLOCK_PADDING : f32 = 1.3;

// Ambient asteroid body color, matches `COLORS.asteroid` (0x3d4a54) in
// `examples/threejs/falling_frontier/src/config/colors.js`.
const ASTEROID_COLOR : [ f32; 3 ] = [ 0.2392, 0.2902, 0.3294 ];

struct AsteroidSpec
{
  radius : f32,
  position : [ f32; 3 ],
}

/// Number of pickable asteroid ids `main.rs` needs to reserve — kept in sync
/// with `ASTEROID_SPECS` automatically rather than as a separate literal.
pub const ASTEROID_COUNT : usize = ASTEROID_SPECS.len();

const ASTEROID_SPECS : [ AsteroidSpec; 8 ] =
[
  AsteroidSpec { radius : 9.0, position : [ -44.76, ASTEROID_Y, -73.56 ] },
  AsteroidSpec { radius : 13.0, position : [ 43.93, ASTEROID_Y, 110.36 ] },
  AsteroidSpec { radius : 6.0, position : [ -28.67, ASTEROID_Y, -137.09 ] },
  AsteroidSpec { radius : 15.0, position : [ -150.55, ASTEROID_Y, 63.03 ] },
  AsteroidSpec { radius : 8.0, position : [ 47.08, ASTEROID_Y, -60.33 ] },
  AsteroidSpec { radius : 11.0, position : [ 158.46, ASTEROID_Y, -173.57 ] },
  AsteroidSpec { radius : 7.0, position : [ 104.19, ASTEROID_Y, -9.05 ] },
  AsteroidSpec { radius : 12.0, position : [ 153.53, ASTEROID_Y, 93.93 ] },
];

/// One asteroid's mutable state - `position`/`extra_rotation_y` start from
/// `ASTEROID_SPECS` but move under the M6 gizmo (`drag_to`/`rotate_to`);
/// `block_radius` is fixed (derived from the spec's `radius`, never
/// resized). Kept 1:1 index-aligned with `Asteroids::parts` - asteroid `i`
/// always owns `parts[i]`, no separate id lookup needed.
struct AsteroidObject
{
  position : [ f32; 2 ],
  /// Extra Y rotation applied on top of the fixed per-asteroid jitter
  /// baked into each part's `local_transform` - starts at 0, only the
  /// gizmo's rotate mode (M6) ever changes it.
  extra_rotation_y : f32,
  block_radius : f32,
}

impl AsteroidObject
{
  fn transform( &self ) -> gl::F32x4x4
  {
    mat3x3h::translation( F32x3::new( self.position[ 0 ], ASTEROID_Y, self.position[ 1 ] ) )
    * mat3x3h::rot( 0.0, self.extra_rotation_y, 0.0 )
  }
}

pub struct Asteroids
{
  parts : Vec< HullPart >,
  objects : Vec< AsteroidObject >,
}

impl Asteroids
{
  /// `id_base` is the first pick id this belt may hand out - asteroid `i`
  /// gets `id_base + i` (see `picking.rs`); the caller reserves
  /// `ASTEROID_COUNT` contiguous ids starting there.
  pub fn new( gl : &gl::GL, id_base : i32 ) -> Self
  {
    let ( base_vertices, faces ) = icosphere();

    let mut parts = Vec::with_capacity( ASTEROID_SPECS.len() );
    let mut objects = Vec::with_capacity( ASTEROID_SPECS.len() );

    for ( i, spec ) in ASTEROID_SPECS.iter().enumerate()
    {
      // Per-vertex radial jitter, same magnitude as the JS deformToRock()
      // (factor in [0.825, 1.175]) - regenerated per asteroid instead of
      // sharing one deformed mesh, so all 8 rocks don't look identical.
      let positions : Vec< [ f32; 3 ] > = base_vertices
      .iter()
      .map( | v |
      {
        let factor = 1.0 + ( rand::rng().random_range( 0.0 .. 1.0 ) - 0.5 ) * 0.35;
        [ v[ 0 ] * factor, v[ 1 ] * factor, v[ 2 ] * factor ]
      } )
      .collect();

      let ( vao, index_count ) = upload_mesh( gl, &positions, &faces );

      // Fixed per-rock jitter (shape only) - never touched again, unlike
      // `extra_rotation_y` above which the gizmo does mutate.
      let rx = rand::rng().random_range( 0.0 .. std::f32::consts::PI );
      let ry = rand::rng().random_range( 0.0 .. std::f32::consts::PI );
      let rz = rand::rng().random_range( 0.0 .. std::f32::consts::PI );
      let local_transform = mat3x3h::rot( rx, ry, rz ) * mat3x3h::scale( F32x3::splat( spec.radius ) );

      let object = AsteroidObject
      {
        position : [ spec.position[ 0 ], spec.position[ 2 ] ],
        extra_rotation_y : 0.0,
        block_radius : spec.radius * BLOCK_PADDING,
      };
      let model = object.transform() * local_transform;

      parts.push( HullPart { vao, index_count, local_transform, model, color : ASTEROID_COLOR, ambient : AMBIENT_LIT, pick_id : id_base + i as i32 } );
      objects.push( object );
    }

    Self { parts, objects }
  }

  pub fn parts( &self ) -> &[ HullPart ]
  {
    &self.parts
  }

  /// The `index`-th asteroid's current world transform (translate + gizmo
  /// rotation, no shape jitter) - what the gizmo (M6) draws its handle at.
  pub fn object_transform( &self, index : usize ) -> gl::F32x4x4
  {
    self.objects[ index ].transform()
  }

  pub fn position( &self, index : usize ) -> [ f32; 2 ]
  {
    self.objects[ index ].position
  }

  pub fn rotation_y( &self, index : usize ) -> f32
  {
    self.objects[ index ].extra_rotation_y
  }

  /// Moves asteroid `index` to a new XZ position (Y stays at `ASTEROID_Y`) -
  /// called by the M6 gizmo's translate drag.
  pub fn drag_to( &mut self, index : usize, position : [ f32; 2 ] )
  {
    self.objects[ index ].position = position;
    self.parts[ index ].set_model( self.objects[ index ].transform() );
  }

  /// Sets asteroid `index`'s extra Y rotation - called by the M6 gizmo's
  /// rotate drag.
  pub fn rotate_to( &mut self, index : usize, rotation_y : f32 )
  {
    self.objects[ index ].extra_rotation_y = rotation_y;
    self.parts[ index ].set_model( self.objects[ index ].transform() );
  }

  /// Every asteroid as a boundary-blocking circle, padded past its max
  /// visual bulge - feeds `boundary::build_boundary_polyline`.
  pub fn blockers( &self ) -> Vec< Blocker >
  {
    self.objects.iter()
    .map( | a | Blocker { x : a.position[ 0 ], z : a.position[ 1 ], radius : a.block_radius } )
    .collect()
  }

  /// Asteroids within `view_radius` of `focus`, for the shader's proximity
  /// glow - pre-filtered on the CPU like the JS `updateAsteroidGlow` does,
  /// so nothing outside view range ever reaches the shader.
  pub fn glow_candidates( &self, focus : [ f32; 2 ], view_radius : f32 ) -> Vec< ( [ f32; 2 ], f32 ) >
  {
    self.objects.iter()
    .filter_map( | a |
    {
      let dx = a.position[ 0 ] - focus[ 0 ];
      let dz = a.position[ 1 ] - focus[ 1 ];
      let dist_to_surface = dx.hypot( dz ) - a.block_radius;
      ( dist_to_surface < view_radius ).then_some( ( a.position, a.block_radius ) )
    } )
    .collect()
  }
}
