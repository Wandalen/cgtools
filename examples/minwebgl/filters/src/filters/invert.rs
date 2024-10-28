use super::*;

pub struct Invert;

impl Filter for Invert
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
      frag_color = vec4( ( vec4( 1.0 ) - pixel ).rgb, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    default_render_pass( renderer );
  }
}
