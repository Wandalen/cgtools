use crate::*;
use filters::*;
use framebuffer::Framebuffer;
use minwebgl as gl;
use gl::GL;
use wasm_bindgen::JsCast;
use web_sys::
{
  WebGlProgram,
  WebGlTexture,
};

pub struct Renderer
{
  gl : GL,
  program : Option< WebGlProgram >,
  framebuffer : Framebuffer,
  image_texture : Option< WebGlTexture >,
  original_texture : Option< WebGlTexture >,
  previous_texture : Option< WebGlTexture >,
  previous_canvas_size : Option< ( u32, u32 ) >,
  current_filter_source : String,
}

impl Renderer
{
  const VERTEX_SOURCE : &'static str = include_str!( "shaders/main.vert" );

  pub fn new( gl : &GL, image_texture : Option< WebGlTexture > ) -> Self
  {
    let width = gl.drawing_buffer_width();
    let height = gl.drawing_buffer_height();
    let framebuffer = Framebuffer::new( gl, width, height ).expect( "Can't create framebuffer" );

    Self
    {
      gl : gl.clone(),
      program : None,
      framebuffer,
      image_texture,
      original_texture : None,
      previous_texture : None,
      previous_canvas_size : None,
      current_filter_source : String::new(),
    }
  }

  pub fn set_image_texture( &mut self, image_texture : Option< WebGlTexture > )
  {
    self.image_texture = image_texture;
  }

  pub fn set_original_texture( &mut self, original_texture : Option< WebGlTexture > )
  {
    self.original_texture = original_texture;
  }

  pub fn restore_original_texture( &mut self )
  {
    if let Some( original ) = &self.original_texture
    {
      self.image_texture = Some( original.clone() );
    }
  }

  pub fn save_previous_texture( &mut self )
  {
    self.previous_texture = self.image_texture.clone();
    // Save current canvas dimensions so they can be restored on cancel
    if let Some( canvas ) = self.gl.canvas()
    {
      if let Ok( canvas ) = canvas.dyn_into::< web_sys::HtmlCanvasElement >()
      {
        self.previous_canvas_size = Some( ( canvas.width(), canvas.height() ) );
      }
    }
  }

  pub fn restore_previous_texture( &mut self )
  {
    if let Some( previous ) = &self.previous_texture
    {
      self.image_texture = Some( previous.clone() );
    }
    // Restore canvas dimensions
    if let Some( ( w, h ) ) = self.previous_canvas_size.take()
    {
      if let Some( canvas ) = self.gl.canvas()
      {
        if let Ok( canvas ) = canvas.dyn_into::< web_sys::HtmlCanvasElement >()
        {
          canvas.set_width( w );
          canvas.set_height( h );
        }
      }
    }
  }

  pub fn clear_previous_state( &mut self )
  {
    self.previous_texture = None;
    self.previous_canvas_size = None;
  }

  pub fn update_framebuffer_size( &mut self, width : i32, height : i32 )
  {
    self.framebuffer = Framebuffer::new( &self.gl, width, height ).expect( "Can't create framebuffer" );
  }

  pub fn apply_filter( &mut self, filter : &impl Filter )
  {
    if self.image_texture.is_none()
    {
      return;
    }

    let filter_source = filter.glsl_fragment_source();
    if self.current_filter_source != filter_source
    {
      // Recompile program
      self.program = Some( Self::create_program( &self.gl, &filter_source ) );
      self.current_filter_source = filter_source;
    }

    filter.draw( self );
  }

  fn create_program( gl : &GL, filter_source : &str ) -> WebGlProgram
  {
    gl::ProgramFromSources::new( Self::VERTEX_SOURCE, filter_source )
    .compile_and_link( &gl )
    .expect( "Unable to compile program" )
  }
}

impl FilterRenderer for Renderer
{
  fn gl( &self ) -> &GL
  {
    &self.gl
  }

  fn get_image_texture( &self ) -> Option< &WebGlTexture >
  {
    self.image_texture.as_ref()
  }

  fn get_program( &self ) -> &WebGlProgram
  {
    self.program.as_ref().expect( "No filter was provided" )
  }

  fn draw( &self )
  {
    self.gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }

  fn get_framebuffer( &self ) -> &Framebuffer
  {
    &self.framebuffer
  }
}
