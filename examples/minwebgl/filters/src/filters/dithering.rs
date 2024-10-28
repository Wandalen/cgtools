use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Dithering
{
  pub levels : i32
}

impl Filter for Dithering
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform int u_dithering_levels;

    void main()
    {
      const float BAYER_MATRIX[ 16 ] = float[]
      (
         0.0 / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0,
        12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0,
         3.0 / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0,
        15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0
      );

      vec4 color = texture( u_image, v_tex_coord );

      ivec2 pixel_pos = ivec2( mod( gl_FragCoord.xy, 4.0 ) );
      float threshold = BAYER_MATRIX[ pixel_pos.y * 4 + pixel_pos.x ];

      vec3 quantized_color = vec3( 0.0 );
      for( int i = 0; i < 3; i++ )
      {
        float scaled = color[ i ] * float( u_dithering_levels );
        float dithered = floor( scaled + threshold );
        quantized_color[ i ] = clamp( dithered / float( u_dithering_levels ), 0.0, 1.0 );
      }

      frag_color = vec4( quantized_color, color.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let levels_location = gl.get_uniform_location( renderer.get_program(), "u_dithering_levels" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, levels_location, &self.levels ).unwrap();
    default_render_pass( renderer );
  }
}
