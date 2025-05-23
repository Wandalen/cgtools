
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
    fn render
    (
      &self,
      gl : &minwebgl::WebGl2RenderingContext,
      input_texture : Option< minwebgl::web_sys::WebGlTexture >
    ) -> Result< Option< minwebgl::web_sys::WebGlTexture >, minwebgl::WebglError > 
    {
      self.material.bind( gl );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );
      
      if self.render_to_screen
      {
        gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      }

      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      Ok
      (
        input_texture
      )
    }
  }
}

crate::mod_interface!
{
  
}