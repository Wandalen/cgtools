use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Rescale
{
  pub scale : f32
}

impl Filter for Rescale
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_scale;

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );
      frag_color = vec4( pixel.rgb * u_scale, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let scale_location = gl.get_uniform_location( renderer.get_program(), "u_scale" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, scale_location, &self.scale ).unwrap();
    default_render_pass( renderer );
  }
}
