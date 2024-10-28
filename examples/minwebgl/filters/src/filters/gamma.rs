use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Gamma
{
  pub gamma : f32
}

impl Filter for Gamma
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_inv_gamma;

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );
      frag_color = vec4( pow( pixel.rgb, vec3( u_inv_gamma ) ), pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let inv_gamma_location = gl.get_uniform_location( renderer.get_program(), "u_inv_gamma" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, inv_gamma_location, &( 1.0 / self.gamma ) ).unwrap();
    default_render_pass( renderer );
  }
}
