use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct ColorTransform
{
  pub red_multiplier : f32,
  pub green_multiplier : f32,
  pub blue_multiplier : f32,
  pub red_offset : f32,
  pub green_offset : f32,
  pub blue_offset : f32,
}

impl Filter for ColorTransform
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform vec3 u_rgb_multipliers;
    uniform vec3 u_rgb_offsets;

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );
      pixel.rgb *= u_rgb_multipliers;
      pixel.rgb += u_rgb_offsets;
      frag_color = pixel;
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let rgb_multipliers_location = gl.get_uniform_location( renderer.get_program(), "u_rgb_multipliers" );
    let rgb_offsets_location = gl.get_uniform_location( renderer.get_program(), "u_rgb_offsets" );

    gl.use_program( Some( &renderer.get_program() ) );

    let multipliers = [ self.red_multiplier, self.green_multiplier, self.blue_multiplier ];
    gl::uniform::upload( gl, rgb_multipliers_location, multipliers.as_slice() ).unwrap();

    let offsets = [ self.red_offset, self.green_offset, self.blue_offset ];
    gl::uniform::upload( gl, rgb_offsets_location, offsets.as_slice() ).unwrap();

    default_render_pass( renderer );
  }
}

impl Default for ColorTransform
{
  fn default() -> Self
  {
    Self
    {
      red_multiplier: 1.0,
      green_multiplier: 1.0,
      blue_multiplier: 1.0,
      red_offset: 0.0,
      green_offset: 0.0,
      blue_offset: 0.0,
    }
  }
}
