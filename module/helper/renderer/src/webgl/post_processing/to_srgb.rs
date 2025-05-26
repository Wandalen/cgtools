
mod private
{
  use minwebgl as gl;
  use crate::webgl::{ post_processing::{Pass, VS_TRIANGLE}, program::EmptyShader, ProgramInfo };

  pub struct ToSrgbPass
  {
    material : ProgramInfo< EmptyShader >,
    render_to_screen : bool
  }

  impl ToSrgbPass 
  {
    pub fn set_render_to_screen( &mut self, render_to_screen : bool )
    {
      self.render_to_screen = render_to_screen;
    }

    pub fn new( gl : &gl::WebGl2RenderingContext, render_to_screen : bool ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/post_processing/to_srgb.frag" );
      let material = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = ProgramInfo::< EmptyShader >::new( material );

      Ok
      (
        Self
        {
          material,
          render_to_screen
        }
      )
    }    
  }

  impl Pass for ToSrgbPass
  {
    fn renders_to_input( &self ) -> bool 
    {
      false
    }
    
    fn render
    (
      &self,
      gl : &minwebgl::WebGl2RenderingContext,
      input_texture : Option< minwebgl::web_sys::WebGlTexture >,
      output_texture : Option< minwebgl::web_sys::WebGlTexture >
    ) -> Result< Option< minwebgl::web_sys::WebGlTexture >, minwebgl::WebglError > 
    {
      gl.disable( gl::DEPTH_TEST );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      self.material.bind( gl );
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );
      
      if self.render_to_screen
      {
        gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      }
      else 
      {
        gl.framebuffer_texture_2d
        (
          gl::FRAMEBUFFER, 
          gl::COLOR_ATTACHMENT0, 
          gl::TEXTURE_2D, 
          output_texture.as_ref(), 
          0
        );    
      }

      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      gl.bind_texture( gl::TEXTURE_2D, None );
      if !self.render_to_screen
      {
        gl.framebuffer_texture_2d
        (
          gl::FRAMEBUFFER, 
          gl::COLOR_ATTACHMENT0, 
          gl::TEXTURE_2D, 
          None, 
          0
        );
      }

      Ok
      (
        output_texture
      )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ToSrgbPass
  };
}