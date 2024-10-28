use super::*;

pub struct Desaturate;

impl Filter for Desaturate
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
      float max = max( pixel.r, max( pixel.g, pixel.b ) );
      float min = min( pixel.r, min( pixel.g, pixel.b ) );
      float avg = ( min + max ) / 2.0;

      frag_color = vec4( avg, avg, avg, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    default_render_pass( renderer );
  }
}
