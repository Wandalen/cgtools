use super::*;

pub struct Sepia;

impl Filter for Sepia
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
      float r = pixel.r * 0.393 + pixel.g * 0.769 + pixel.b * 0.189;
      float g = pixel.r * 0.349 + pixel.g * 0.686 + pixel.b * 0.168;
      float b = pixel.r * 0.272 + pixel.g * 0.534 + pixel.b * 0.131;
      frag_color = vec4( r, g, b, pixel.a );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    default_render_pass( renderer );
  }
}
