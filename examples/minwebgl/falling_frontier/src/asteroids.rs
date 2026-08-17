//! Asteroid belt: procedural low-poly "rock" geometry + rendering, pulled
//! forward from M4 just far enough to give M3's view-zone ribbon something
//! to wrap around. Positions/radii ported from `ASTEROID_SPECS` in
//! `examples/threejs/falling_frontier/src/world/asteroidBelt.js`.
//!
//! Simplification vs. the JS reference: JS deforms a `DodecahedronGeometry`
//! (12 faces, three.js `detail=1` subdivision); this uses a plain
//! icosahedron (20 triangular faces) with the same per-vertex radial jitter
//! instead, since the exact polytope is purely cosmetic (`deformToRock`'s
//! own comment: "Purely cosmetic noise") and doesn't affect the boundary
//! polyline / glow math, which only ever sees each asteroid as a padded
//! circle (`blockRadius`). Flat shading comes from screen-space derivatives
//! in `asteroid.frag` rather than three.js's `flatShading: true`, so no
//! per-face vertex duplication is needed either - plain indexed draw.

use minwebgl as gl;
use gl::GL;
use gl::math::{ F32x3, mat3x3h };
use rand::RngExt as _;

use crate::boundary::Blocker;

// Asteroids sit at the same altitude as the ships in the JS scene, so the
// belt reads as one flat tactical plane - kept here even though ships
// themselves aren't ported yet (M4), so the boundary math and grid share the
// same XZ plane asteroids actually occupy.
const ASTEROID_Y : f32 = 12.0;

// deformToRock() bulges vertices up to ~17.5% past the nominal radius, so
// the blocking radius (used by the view-zone ribbon) is padded past the max
// bulge to keep the ribbon clear of the jagged mesh.
const BLOCK_PADDING : f32 = 1.3;

struct AsteroidSpec
{
  radius : f32,
  position : [ f32; 3 ],
}

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

// Standard unit icosahedron (12 vertices, 20 triangular faces), the base
// shape jittered per-vertex to fake an irregular rock.
fn icosahedron() -> ( [ [ f32; 3 ]; 12 ], [ [ u32; 3 ]; 20 ] )
{
  let phi = ( 1.0 + 5.0f32.sqrt() ) * 0.5;
  let raw : [ [ f32; 3 ]; 12 ] =
  [
    [ -1.0, phi, 0.0 ], [ 1.0, phi, 0.0 ], [ -1.0, -phi, 0.0 ], [ 1.0, -phi, 0.0 ],
    [ 0.0, -1.0, phi ], [ 0.0, 1.0, phi ], [ 0.0, -1.0, -phi ], [ 0.0, 1.0, -phi ],
    [ phi, 0.0, -1.0 ], [ phi, 0.0, 1.0 ], [ -phi, 0.0, -1.0 ], [ -phi, 0.0, 1.0 ],
  ];
  let vertices = raw.map( | v | F32x3::from( v ).normalize().to_array() );

  let faces : [ [ u32; 3 ]; 20 ] =
  [
    [ 0, 11, 5 ], [ 0, 5, 1 ], [ 0, 1, 7 ], [ 0, 7, 10 ], [ 0, 10, 11 ],
    [ 1, 5, 9 ], [ 5, 11, 4 ], [ 11, 10, 2 ], [ 10, 7, 6 ], [ 7, 1, 8 ],
    [ 3, 9, 4 ], [ 3, 4, 2 ], [ 3, 2, 6 ], [ 3, 6, 8 ], [ 3, 8, 9 ],
    [ 4, 9, 5 ], [ 2, 4, 11 ], [ 6, 2, 10 ], [ 8, 6, 7 ], [ 9, 8, 1 ],
  ];

  ( vertices, faces )
}

struct AsteroidUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
  model : Option< gl::WebGlUniformLocation >,
  color : Option< gl::WebGlUniformLocation >,
  light_dir : Option< gl::WebGlUniformLocation >,
}

struct AsteroidInstance
{
  model : gl::F32x4x4,
  position : [ f32; 2 ],
  block_radius : f32,
  vao : gl::WebGlVertexArrayObject,
}

// Ambient asteroid body color, matches `COLORS.asteroid` (0x3d4a54) in
// `examples/threejs/falling_frontier/src/config/colors.js`.
const ASTEROID_COLOR : [ f32; 3 ] = [ 0.2392, 0.2902, 0.3294 ];
// Not normalized on the CPU side - `asteroid.frag` normalizes it itself.
const LIGHT_DIR : [ f32; 3 ] = [ 0.4, 1.0, 0.3 ];

pub struct Asteroids
{
  index_count : i32,
  program : gl::WebGlProgram,
  uniforms : AsteroidUniforms,
  instances : Vec< AsteroidInstance >,
}

impl Asteroids
{
  pub fn new( gl : &GL ) -> Self
  {
    let ( base_vertices, faces ) = icosahedron();

    // Per-vertex radial jitter, same magnitude as the JS deformToRock()
    // (factor in [0.825, 1.175]) - regenerated per asteroid instead of
    // sharing one deformed mesh, so all 8 rocks don't look identical. One
    // VAO/VBO pair per asteroid is fine at this count.
    let mut instances = Vec::with_capacity( ASTEROID_SPECS.len() );

    for spec in &ASTEROID_SPECS
    {
      let positions : Vec< [ f32; 3 ] > = base_vertices
      .iter()
      .map( | v |
      {
        let factor = 1.0 + ( rand::rng().random_range( 0.0 .. 1.0 ) - 0.5 ) * 0.35;
        [ v[ 0 ] * factor, v[ 1 ] * factor, v[ 2 ] * factor ]
      } )
      .collect();

      let instance_vao = gl::vao::create( gl ).unwrap();
      gl.bind_vertex_array( Some( &instance_vao ) );

      let position_buffer = gl::buffer::create( gl ).unwrap();
      gl::buffer::upload( gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
      gl::BufferDescriptor::new::< [ f32; 3 ] >()
      .stride( 0 )
      .offset( 0 )
      .attribute_pointer( gl, 0, &position_buffer )
      .unwrap();

      let index_buffer = gl::buffer::create( gl ).unwrap();
      gl::index::upload( gl, &index_buffer, faces.as_flattened(), GL::STATIC_DRAW );

      let rx = rand::rng().random_range( 0.0 .. std::f32::consts::PI );
      let ry = rand::rng().random_range( 0.0 .. std::f32::consts::PI );
      let rz = rand::rng().random_range( 0.0 .. std::f32::consts::PI );
      let model = mat3x3h::translation( F32x3::from( spec.position ) )
      * mat3x3h::rot( rx, ry, rz )
      * mat3x3h::scale( F32x3::splat( spec.radius ) );

      instances.push( AsteroidInstance
      {
        model,
        position : [ spec.position[ 0 ], spec.position[ 2 ] ],
        block_radius : spec.radius * BLOCK_PADDING,
        vao : instance_vao,
      } );
    }

    let vertex_shader = include_str!( "shaders/asteroid.vert" );
    let fragment_shader = include_str!( "shaders/asteroid.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = AsteroidUniforms
    {
      view_proj : gl.get_uniform_location( &program, "u_view_proj" ),
      model : gl.get_uniform_location( &program, "u_model" ),
      color : gl.get_uniform_location( &program, "u_color" ),
      light_dir : gl.get_uniform_location( &program, "u_light_dir" ),
    };

    Self { index_count : ( faces.len() * 3 ) as i32, program, uniforms, instances }
  }

  pub fn draw( &self, gl : &GL, view_proj : gl::F32x4x4 )
  {
    gl.use_program( Some( &self.program ) );
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.view_proj.clone(), view_proj.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, u.color.clone(), ASTEROID_COLOR.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.light_dir.clone(), LIGHT_DIR.as_slice() ).unwrap();

    for instance in &self.instances
    {
      gl::uniform::matrix_upload( gl, u.model.clone(), instance.model.to_array().as_slice(), true ).unwrap();
      gl.bind_vertex_array( Some( &instance.vao ) );
      gl.draw_elements_with_i32( GL::TRIANGLES, self.index_count, GL::UNSIGNED_INT, 0 );
    }
  }

  /// Every asteroid as a boundary-blocking circle, padded past its max
  /// visual bulge - feeds `boundary::build_boundary_polyline`.
  pub fn blockers( &self ) -> Vec< Blocker >
  {
    self.instances.iter()
    .map( | a | Blocker { x : a.position[ 0 ], z : a.position[ 1 ], radius : a.block_radius } )
    .collect()
  }

  /// Asteroids within `view_radius` of `focus`, for the shader's proximity
  /// glow - pre-filtered on the CPU like the JS `updateAsteroidGlow` does,
  /// so nothing outside view range ever reaches the shader.
  pub fn glow_candidates( &self, focus : [ f32; 2 ], view_radius : f32 ) -> Vec< ( [ f32; 2 ], f32 ) >
  {
    self.instances.iter()
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
