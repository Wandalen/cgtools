use super::*;

pub struct Transpose;

impl Filter for Transpose
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;

    uniform sampler2D u_image;
    uniform float u_aspect;

    void main()
    {
      vec2 tex_coord = vec2( ( 1.0 - v_tex_coord.y ) / u_aspect, 1.0 - v_tex_coord.x * u_aspect );
      if ( tex_coord.x > 1.0 || tex_coord.y > 1.0 || tex_coord.x < 0.0 || tex_coord.y < 0.0 )
      {
        discard;
      }
      frag_color = texture( u_image, tex_coord );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();
    let aspect_location = gl.get_uniform_location( renderer.get_program(), "u_aspect" );
    let aspect = gl.drawing_buffer_width() as f32 / gl.drawing_buffer_height() as f32;
    gl.use_program( Some( &renderer.get_program() ) );
    gl::uniform::upload( gl, aspect_location, &aspect ).unwrap();

    default_render_pass( renderer );
  }
}
