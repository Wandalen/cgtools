use super::*;
use serde::{ Serialize, Deserialize };

#[ repr( i32 ) ]
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum FlipDirection
{
  FlipX = 1,
  FlipY = 2,
  FlipXY = 3,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Flip
{
  pub flip : FlipDirection
}

impl Filter for Flip
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform int u_flip;

    void main()
    {
      vec2 tc = v_tex_coord;
      if ( bool( u_flip & 1 ) )
      {
        tc.x = 1.0 - tc.x;
      }
      if ( bool( u_flip & 2 ) )
      {
        tc.y = 1.0 - tc.y;
      }
      frag_color = texture( u_image, tc );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let flip_location = gl.get_uniform_location( renderer.get_program(), "u_flip" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, flip_location, &( self.flip as i32 ) ).unwrap();
    default_render_pass( renderer );
  }
}
