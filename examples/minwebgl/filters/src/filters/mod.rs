pub mod original;
pub mod blur;
pub mod binarize;
pub mod brightness_contrast;
pub mod channels;
pub mod color_transform;
pub mod desaturate;
pub mod dithering;
pub mod edge;
pub mod emboss;
pub mod enrich;
pub mod flip;
pub mod gamma;
pub mod gray_scale;
pub mod hsl_adjustment;
pub mod invert;
pub mod mosaic;
pub mod oil;
pub mod posterize;
pub mod rescale;
pub mod resize;
pub mod sepia;
pub mod sharpen;
pub mod solarize;
pub mod transpose;
pub mod twirl;

use crate::*;
use framebuffer::Framebuffer;
use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlProgram,
  WebGlTexture,
};

// This trait is meant to be used internally by the renderer.
pub trait Filter
{
  fn glsl_fragment_source( &self ) -> String;
  fn draw( &self, renderer : &impl FilterRenderer );
}

// This trait is meant to be used internally by the filters.
pub trait FilterRenderer
{
  fn gl( &self ) -> &GL;
  fn get_image_texture( &self ) -> Option< &WebGlTexture >;
  fn get_program( &self ) -> &WebGlProgram;
  fn get_framebuffer( &self ) -> &Framebuffer;
  fn draw( &self );
}

fn default_render_pass( renderer : &impl FilterRenderer )
{
  let gl = renderer.gl();
  gl.active_texture( GL::TEXTURE0 );
  gl.bind_texture( GL::TEXTURE_2D, renderer.get_image_texture() );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  gl.use_program( Some( renderer.get_program() ) );
  gl.viewport( 0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height() );

  renderer.draw();
}
