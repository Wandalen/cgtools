use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Posterize
{
  pub levels : i32
}

impl Filter for Posterize
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform int u_posterize_levels;

    void main()
    {
      vec4 color = texture( u_image, v_tex_coord );
      vec3 posterized_color = ( floor( color.rgb * float( u_posterize_levels ) ) + 0.5 ) / float( u_posterize_levels );
      frag_color = vec4( posterized_color, color.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let levels_location = gl.get_uniform_location( renderer.get_program(), "u_posterize_levels" );
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, levels_location, &self.levels ).unwrap();
    default_render_pass( renderer );
  }
}
