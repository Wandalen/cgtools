use super::*;

pub struct GrayScale;

impl Filter for GrayScale
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
      float gray = 0.114 * pixel.b + 0.587 * pixel.g + 0.299 * pixel.r;
      frag_color = vec4( gray, gray, gray, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    default_render_pass( renderer );
  }
}
