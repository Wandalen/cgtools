use super::*;
use serde::{ Serialize, Deserialize };

#[ repr( i32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum Channel
{
  Red = 0,
  Green = 1,
  Blue = 2,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Channels
{
  pub channel : Channel
}

impl Filter for Channels
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform int u_channel;

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );

      if ( u_channel == 0 )
      {
        pixel.rgb = vec3( pixel.r );
      }
      else if ( u_channel == 1 )
      {
        pixel.rgb = vec3( pixel.g );
      }
      else if ( u_channel == 2 )
      {
        pixel.rgb = vec3( pixel.b );
      }

      frag_color = pixel;
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let channel_location = gl.get_uniform_location( renderer.get_program(), "u_channel" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, channel_location, &( self.channel as i32 ) ).unwrap();
    default_render_pass( renderer );
  }
}
