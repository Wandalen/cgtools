
mod private
{
  use minwebgl as gl;
  use crate::webgl::{ ProgramInfo, ShaderProgram, post_processing::{Pass, VS_TRIANGLE}, program::EmptyShader };

  /// A post-processing pass responsible for converting a linear color space texture
  /// to the sRGB color space.
  pub struct ToSrgbPass
  {
    /// The WebGL program used for the sRGB conversion.
    material : ProgramInfo,
    /// A boolean flag indicating whether the output of this pass should be
    /// rendered directly to the screen's default framebuffer or
    /// to an offscreen `output_texture`.
    render_to_screen : bool
  }

  impl ToSrgbPass
  {
    /// Sets whether the pass should render its output directly to the screen.
    pub fn set_render_to_screen( &mut self, render_to_screen : bool )
    {
      self.render_to_screen = render_to_screen;
    }

    /// Creates a new `ToSrgbPass` instance.
    pub fn new( gl : &gl::WebGl2RenderingContext, render_to_screen : bool ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/post_processing/to_srgb.frag" );
      let material = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = ProgramInfo::new( gl, &material, EmptyShader.dyn_clone() );

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
      // Disable depth testing
      gl.disable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      // Bind the sRGB conversion shader program.
      self.material.bind( gl );
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );

      // Determine the rendering target: screen or offscreen texture.
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

      // Clear the color buffer of the currently bound framebuffer.
      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // --- Cleanup ---
      // Unbind the texture and framebuffer attachment to restore default state.
      gl::clean::texture_2d( gl );
      if !self.render_to_screen
      {
        gl::clean::framebuffer_texture_2d( gl );
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
