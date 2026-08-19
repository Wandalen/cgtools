//! Cosmic dust particle field scattered through the scene volume, ported
//! from `examples/threejs/falling_frontier/src/world/starfield.js`. Own
//! tiny unlit point-sprite program (not `hull.rs`'s flat-shaded one - points
//! have no meaningful surface normal).

use minwebgl as gl;
use gl::GL;
use rand::RngExt as _;

const PARTICLE_COUNT : usize = 2000;

/// Matches three.js `Color.setHSL` (standard HSL→RGB, s and l both in
/// `[0,1]`, h wraps mod 1).
fn hsl_to_rgb( h : f32, s : f32, l : f32 ) -> [ f32; 3 ]
{
  if s == 0.0 { return [ l, l, l ]; }

  let q = if l < 0.5 { l * ( 1.0 + s ) } else { l + s - l * s };
  let p = 2.0 * l - q;

  let hue_to_rgb = | p : f32, q : f32, t : f32 |
  {
    let t = ( ( t % 1.0 ) + 1.0 ) % 1.0;
    if t < 1.0 / 6.0 { p + ( q - p ) * 6.0 * t }
    else if t < 0.5 { q }
    else if t < 2.0 / 3.0 { p + ( q - p ) * ( 2.0 / 3.0 - t ) * 6.0 }
    else { p }
  };

  [ hue_to_rgb( p, q, h + 1.0 / 3.0 ), hue_to_rgb( p, q, h ), hue_to_rgb( p, q, h - 1.0 / 3.0 ) ]
}

struct StarfieldUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
}

pub struct Starfield
{
  vao : gl::WebGlVertexArrayObject,
  vertex_count : i32,
  program : gl::WebGlProgram,
  uniforms : StarfieldUniforms,
}

impl Starfield
{
  pub fn new( gl : &GL ) -> Self
  {
    let mut positions = Vec::with_capacity( PARTICLE_COUNT );
    let mut colors = Vec::with_capacity( PARTICLE_COUNT );

    for _ in 0 .. PARTICLE_COUNT
    {
      positions.push
      (
        [
          ( rand::rng().random_range( 0.0 .. 1.0 ) - 0.5 ) * 1200.0,
          ( rand::rng().random_range( 0.0 .. 1.0 ) - 0.5 ) * 600.0,
          ( rand::rng().random_range( 0.0 .. 1.0 ) - 0.5 ) * 1200.0,
        ]
      );

      let color = if rand::rng().random_range( 0.0 .. 1.0 ) > 0.4
      {
        let h = 0.55 + rand::rng().random_range( 0.0 .. 1.0 ) * 0.1;
        let l = 0.6 + rand::rng().random_range( 0.0 .. 1.0 ) * 0.3;
        hsl_to_rgb( h, 0.8, l )
      }
      else
      {
        [ 1.0, 1.0, 1.0 ]
      };
      colors.push( color );
    }

    let vao = gl::vao::create( gl ).unwrap();
    gl.bind_vertex_array( Some( &vao ) );

    let position_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 0, &position_buffer )
    .unwrap();

    let color_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &color_buffer, colors.as_slice(), GL::STATIC_DRAW );
    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 1, &color_buffer )
    .unwrap();

    let vertex_shader = include_str!( "shaders/starfield.vert" );
    let fragment_shader = include_str!( "shaders/starfield.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = StarfieldUniforms { view_proj : gl.get_uniform_location( &program, "u_view_proj" ) };

    Self { vao, vertex_count : PARTICLE_COUNT as i32, program, uniforms }
  }

  pub fn draw( &self, gl : &GL, view_proj : gl::F32x4x4 )
  {
    gl.use_program( Some( &self.program ) );
    gl::uniform::matrix_upload( gl, self.uniforms.view_proj.clone(), view_proj.to_array().as_slice(), true ).unwrap();

    gl.enable( GL::BLEND );
    gl.blend_func( GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA );
    gl.depth_mask( false );

    gl.bind_vertex_array( Some( &self.vao ) );
    gl.draw_arrays( GL::POINTS, 0, self.vertex_count );

    gl.depth_mask( true );
    gl.disable( GL::BLEND );
  }
}
