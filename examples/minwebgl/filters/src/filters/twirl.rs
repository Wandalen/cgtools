use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Twirl
{
  #[ serde( rename = "centerX" ) ]
  pub center_x : f32,
  #[ serde( rename = "centerY" ) ]
  pub center_y : f32,
  pub radius : f32,
  pub strength : f32,
}

impl Filter for Twirl
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_aspect;
    uniform vec2 u_twirl_center;
    uniform float u_twirl_radius;
    uniform float u_twirl_strength;

    void main()
    {
      vec2 coord = v_tex_coord - u_twirl_center;
      float distance = length( vec2( coord.x, coord.y / u_aspect ) );
      if ( distance < u_twirl_radius )
      {
        float percent = ( u_twirl_radius - distance ) / u_twirl_radius;
        float angle = u_twirl_strength * percent * percent;
        float sin_angle = sin( angle );
        float cos_angle = cos( angle );
        coord = vec2
        (
          coord.x * cos_angle - coord.y * sin_angle,
          coord.x * sin_angle + coord.y * cos_angle
        );
      }
      coord += u_twirl_center;
      frag_color = texture( u_image, coord );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();

    let aspect_location = gl.get_uniform_location( renderer.get_program(), "u_aspect" );
    let twirl_center_location = gl.get_uniform_location( renderer.get_program(), "u_twirl_center" );
    let twirl_radius_location = gl.get_uniform_location( renderer.get_program(), "u_twirl_radius" );
    let twirl_strength_location = gl.get_uniform_location( renderer.get_program(), "u_twirl_strength" );

    gl.use_program( Some( &renderer.get_program() ) );
    let aspect = gl.drawing_buffer_width() as f32 / gl.drawing_buffer_height() as f32;
    gl::uniform::upload( gl, aspect_location, &aspect ).unwrap();
    gl::uniform::upload( gl, twirl_center_location, [ self.center_x, self.center_y ].as_slice() ).unwrap();
    gl::uniform::upload( gl, twirl_radius_location, &self.radius ).unwrap();
    gl::uniform::upload( gl, twirl_strength_location, &self.strength ).unwrap();

    default_render_pass( renderer );
  }
}
