use super::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

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

    void main()
    {
      // Simple transpose: swap x and y coordinates
      vec2 tex_coord = vec2( 1.0 - v_tex_coord.y, v_tex_coord.x );
      frag_color = texture( u_image, tex_coord );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let gl = renderer.gl();

    // Swap canvas dimensions for transpose
    if let Some( canvas ) = gl.canvas()
    {
      if let Ok( canvas ) = canvas.dyn_into::< HtmlCanvasElement >()
      {
        let width = canvas.width();
        let height = canvas.height();
        canvas.set_width( height );
        canvas.set_height( width );
      }
    }

    gl.use_program( Some( renderer.get_program() ) );

    // Clear with transparent color before drawing transposed image
    gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
    gl.clear( GL::COLOR_BUFFER_BIT );

    default_render_pass( renderer );
  }
}
