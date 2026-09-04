//! Cosmic dust particle field scattered through the scene volume, ported
//! from `examples/threejs/falling_frontier/src/world/starfield.js`. Own
//! tiny unlit point-sprite program (not `hull.rs`'s flat-shaded one - points
//! have no meaningful surface normal).

use minwebgl as gl;
use gl::GL;
use rand::RngExt as _;

const PARTICLE_COUNT : usize = 2000;

// Random per-star point size / opacity ranges - a uniform size+alpha for
// every star (the original ported-from-JS behavior) reads as a flat,
// samey dust haze instead of individual twinkling points.
const MIN_SIZE : f32 = 1.0;
const MAX_SIZE : f32 = 4.0;
const MIN_ALPHA : f32 = 0.12;
const MAX_ALPHA : f32 = 0.6;

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
    // The explicit `[ f32; 3 ]` here is load-bearing, not decoration:
    // nothing else in this function pins the element type.
    // `gl::buffer::upload` is generic over the slice's own type, and
    // `BufferDescriptor::new::<[ f32; 3 ]>()` below is a *separate* type
    // parameter that only describes the GPU-side attribute layout - it
    // does not constrain what type this `Vec` actually holds. Without this
    // annotation the ambiguous float literals below defaulted to `f64`, so
    // the buffer silently held 24-byte-per-vertex data while the attribute
    // pointer was configured for a 12-byte `f32` stride - every vertex read
    // walked through the wrong byte offsets, reinterpreting fragments of
    // unrelated doubles as garbage positions (symptom: stars looked
    // clustered near the origin instead of spread through the box).
    let mut positions : Vec< [ f32; 3 ] > = Vec::with_capacity( PARTICLE_COUNT );
    let mut colors : Vec< [ f32; 3 ] > = Vec::with_capacity( PARTICLE_COUNT );
    let mut sizes : Vec< f32 > = Vec::with_capacity( PARTICLE_COUNT );
    let mut alphas : Vec< f32 > = Vec::with_capacity( PARTICLE_COUNT );

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

      sizes.push( rand::rng().random_range( MIN_SIZE .. MAX_SIZE ) );
      alphas.push( rand::rng().random_range( MIN_ALPHA .. MAX_ALPHA ) );
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

    let size_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &size_buffer, sizes.as_slice(), GL::STATIC_DRAW );
    gl::BufferDescriptor::new::< f32 >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 2, &size_buffer )
    .unwrap();

    let alpha_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &alpha_buffer, alphas.as_slice(), GL::STATIC_DRAW );
    gl::BufferDescriptor::new::< f32 >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 3, &alpha_buffer )
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

    // Additive, not the usual alpha-compositing blend_func - against the
    // black backdrop this had originally, alpha compositing read fine, but
    // once `background.rs` replaced that with a lit sky, these pale white/
    // cyan points (`starfield.frag`'s flat 0.75 alpha) sit close enough in
    // hue/lightness to the sky that most individual stars alpha-composite
    // into near-invisibility - only the spots where several overlap stayed
    // visible, reading as false "clustering" rather than an even field.
    // Additive always brightens the pixel behind it regardless of the
    // backdrop color, so it reads correctly against any background.
    gl.enable( GL::BLEND );
    gl.blend_func( GL::SRC_ALPHA, GL::ONE );
    gl.depth_mask( false );

    gl.bind_vertex_array( Some( &self.vao ) );
    gl.draw_arrays( GL::POINTS, 0, self.vertex_count );

    gl.depth_mask( true );
    gl.disable( GL::BLEND );
  }
}
