use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Binarize
{
  pub threshold : f32
}

impl Filter for Binarize
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_threshold;

    void main()
    {
      vec4 color = texture( u_image, v_tex_coord );
      color.rgb = vec3( ( color.x + color.y + color.z ) / 3.0 >= u_threshold ? 1.0 : 0.0 );
      frag_color = color;
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let threshold_location = gl.get_uniform_location( renderer.get_program(), "u_threshold" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, threshold_location, &self.threshold ).unwrap();
    default_render_pass( renderer );
  }
}
