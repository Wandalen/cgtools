use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Sharpen
{
  pub factor : f32,
}

impl Filter for Sharpen
{
    fn glsl_fragment_source( &self ) -> String
    {
      "#version 300 es
      precision mediump float;

      in vec2 v_tex_coord;
      out vec4 frag_color;

      uniform sampler2D u_image;
      uniform vec2 u_texel_size;
      uniform float u_sharpen_factor;

      vec3 apply_3x3_kernel( float kernel[ 9 ] )
      {
        const vec2 OFFSETS[] = vec2[]
        (
          vec2( -1.0,  1.0 ), vec2( 0.0,  1.0 ), vec2( 1.0,  1.0 ),
          vec2( -1.0,  0.0 ), vec2( 0.0,  0.0 ), vec2( 1.0,  0.0 ),
          vec2( -1.0, -1.0 ), vec2( 0.0, -1.0 ), vec2( 1.0, -1.0 )
        );

        vec4 sum = vec4( 0.0 );
        for ( int i = 0; i < 9; i++ )
        {
          vec2 offset = v_tex_coord + OFFSETS[ i ] * u_texel_size;
          vec4 pixel = texture( u_image, offset );
          sum += pixel * kernel[ i ];
        }

        return sum.rgb;
      }

      void main()
      {
        float sharpen_kernel[] = float[]
        (
          -u_sharpen_factor / 16.0, -u_sharpen_factor / 8.0,        -u_sharpen_factor / 16.0,
          -u_sharpen_factor /  8.0,  u_sharpen_factor * 0.75 + 1.0, -u_sharpen_factor /  8.0,
          -u_sharpen_factor / 16.0, -u_sharpen_factor / 8.0,        -u_sharpen_factor / 16.0
        );
        frag_color = vec4( apply_3x3_kernel( sharpen_kernel ), 1.0 );
      }
      ".to_string()
    }

    fn draw( &self, renderer : &impl FilterRenderer )
    {
      let gl = renderer.gl();
      let texel_size = [ 1.0 / gl.drawing_buffer_width() as f32, 1.0 / gl.drawing_buffer_height() as f32 ];
      let texel_size_location = gl.get_uniform_location( renderer.get_program(), "u_texel_size" );
      let sharpen_location = gl.get_uniform_location( renderer.get_program(), "u_sharpen_factor" );
      gl.use_program( Some( &renderer.get_program() ) );
      gl::uniform::upload( gl, texel_size_location, texel_size.as_slice() ).unwrap();
      gl::uniform::upload( gl, sharpen_location, &self.factor ).unwrap();
      default_render_pass( renderer );
    }
}
