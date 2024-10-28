use super::*;

pub struct Solarize;

impl Filter for Solarize
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;

    void main()
    {
      vec4 pixel = texture( u_image, v_tex_coord );
      vec3 color = vec3
      (
        pixel.r > 0.5 ? ( pixel.r - 0.5 ) * 2.0 : ( 0.5 - pixel.r ) * 2.0,
        pixel.g > 0.5 ? ( pixel.g - 0.5 ) * 2.0 : ( 0.5 - pixel.g ) * 2.0,
        pixel.b > 0.5 ? ( pixel.b - 0.5 ) * 2.0 : ( 0.5 - pixel.b ) * 2.0
      );
      frag_color = vec4( color, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    default_render_pass( renderer );
  }
}
