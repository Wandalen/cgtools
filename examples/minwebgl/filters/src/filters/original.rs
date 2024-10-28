use super::*;

pub struct Original;

impl Filter for Original
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
      frag_color = texture( u_image, v_tex_coord );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    default_render_pass( renderer );
  }
}
