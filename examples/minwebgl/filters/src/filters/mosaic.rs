use super::*;
use serde::{ Serialize, Deserialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Mosaic
{
  pub scale : u32,
}

impl Filter for Mosaic
{
  fn glsl_fragment_source( &self ) -> String
  {
    "#version 300 es
    precision mediump float;

    in vec2 v_tex_coord;
    out vec4 frag_color;
    uniform float u_mosaic_scale;

    uniform sampler2D u_image;

    void main()
    {
      frag_color = texture( u_image, v_tex_coord * u_mosaic_scale );
    }
    ".to_string()
  }

  fn draw( &self, renderer : &impl FilterRenderer )
  {
    let scale = self.scale as i32;
    let gl = renderer.gl();

    gl.active_texture( GL::TEXTURE0 );

    let scale_location = gl.get_uniform_location( renderer.get_program(), "u_mosaic_scale" );
    gl.use_program( Some( renderer.get_program() ) );

    gl::uniform::upload( gl, scale_location.clone(), &1.0 ).unwrap();
    gl.bind_texture( GL::TEXTURE_2D, renderer.get_image_texture() );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_NEAREST as i32 );
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( renderer.get_framebuffer().framebuffer() ) );
    gl.viewport( 0, 0, gl.drawing_buffer_width() / scale, gl.drawing_buffer_height() / scale );

    renderer.draw();

    gl::uniform::upload( gl, scale_location, &( 1.0 / scale as f32 ) ).unwrap();
    gl.bind_texture( GL::TEXTURE_2D, Some( renderer.get_framebuffer().color_attachment() ) );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.viewport( 0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height() );

    renderer.draw();
  }
}
