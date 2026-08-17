//! Shared "hard surface" rendering: one flat-shaded-via-derivatives program
//! (`shaders/hull.{vert,frag}`) reused by every opaque compositional mesh in
//! the scene - asteroids, ship hulls, station modules - each just a list of
//! `HullPart`s (own VAO + model matrix + color + ambient factor) drawn
//! through the one shared `HullProgram`. Keeps M4's ship/station code from
//! re-deriving the same program-setup/uniform-upload boilerplate M3's
//! `asteroids.rs` already worked out.

use minwebgl as gl;
use gl::GL;

/// Not normalized on the CPU side - `hull.frag` normalizes it itself.
pub const LIGHT_DIR : [ f32; 3 ] = [ 0.4, 1.0, 0.3 ];

/// A normally-lit part (asteroid rock, ship/station hull plating).
pub const AMBIENT_LIT : f32 = 0.35;
/// A fully self-lit part (engine glow, beacon light) - stands in for
/// three.js's separate unlit `MeshBasicMaterial` glow parts.
pub const AMBIENT_GLOW : f32 = 1.0;

/// One drawable piece: its own geometry (VAO), world transform, color and
/// ambient factor. Ships/stations are composed from several of these.
pub struct HullPart
{
  pub vao : gl::WebGlVertexArrayObject,
  pub index_count : i32,
  pub model : gl::F32x4x4,
  pub color : [ f32; 3 ],
  pub ambient : f32,
}

/// Uploads `positions`/`indices` as a new VAO and returns it alongside the
/// index count `draw_elements` needs.
pub fn upload_mesh( gl : &GL, positions : &[ [ f32; 3 ] ], indices : &[ u32 ] ) -> ( gl::WebGlVertexArrayObject, i32 )
{
  let vao = gl::vao::create( gl ).unwrap();
  gl.bind_vertex_array( Some( &vao ) );

  let position_buffer = gl::buffer::create( gl ).unwrap();
  gl::buffer::upload( gl, &position_buffer, positions, GL::STATIC_DRAW );
  gl::BufferDescriptor::new::< [ f32; 3 ] >()
  .stride( 0 )
  .offset( 0 )
  .attribute_pointer( gl, 0, &position_buffer )
  .unwrap();

  let index_buffer = gl::buffer::create( gl ).unwrap();
  gl::index::upload( gl, &index_buffer, indices, GL::STATIC_DRAW );

  ( vao, indices.len() as i32 )
}

struct HullUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
  model : Option< gl::WebGlUniformLocation >,
  color : Option< gl::WebGlUniformLocation >,
  light_dir : Option< gl::WebGlUniformLocation >,
  ambient : Option< gl::WebGlUniformLocation >,
}

pub struct HullProgram
{
  program : gl::WebGlProgram,
  uniforms : HullUniforms,
}

impl HullProgram
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "shaders/hull.vert" );
    let fragment_shader = include_str!( "shaders/hull.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = HullUniforms
    {
      view_proj : gl.get_uniform_location( &program, "u_view_proj" ),
      model : gl.get_uniform_location( &program, "u_model" ),
      color : gl.get_uniform_location( &program, "u_color" ),
      light_dir : gl.get_uniform_location( &program, "u_light_dir" ),
      ambient : gl.get_uniform_location( &program, "u_ambient" ),
    };

    Self { program, uniforms }
  }

  /// Activates the program and uploads the frame's view_proj - call once
  /// per frame before any `draw_part` calls.
  pub fn begin_frame( &self, gl : &GL, view_proj : gl::F32x4x4 )
  {
    gl.use_program( Some( &self.program ) );
    gl::uniform::matrix_upload( gl, self.uniforms.view_proj.clone(), view_proj.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, self.uniforms.light_dir.clone(), LIGHT_DIR.as_slice() ).unwrap();
  }

  pub fn draw_part( &self, gl : &GL, part : &HullPart )
  {
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.model.clone(), part.model.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, u.color.clone(), part.color.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.ambient.clone(), &part.ambient ).unwrap();

    gl.bind_vertex_array( Some( &part.vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count, GL::UNSIGNED_INT, 0 );
  }
}
