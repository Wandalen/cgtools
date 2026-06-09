mod private
{
  use std::marker::PhantomData;
  use minwebgl as gl;
  use crate::webgl::
  {
    ShaderProgram, post_processing::{ Pass, VS_TRIANGLE }, program::EmptyShader
  };

  /// Represents the ACES (Academy Color Encoding System) tone mapping algorithm.
  pub struct ToneMappingAces;

  /// A generic post-processing pass for applying tone mapping to a texture.
  pub struct ToneMappingPass< T >
  {
    /// The WebGL program used for the tone mapping operation.
    material : EmptyShader,
    phantom : std::marker::PhantomData< T >
  }

  impl< T > Pass for ToneMappingPass< T >
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
      gl.disable( gl::BLEND );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      self.material.bind( gl );
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        output_texture.as_ref(),
        0
      );

      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      gl::clean::texture_2d( gl );
      gl::clean::framebuffer_texture_2d( gl );

      Ok( output_texture.clone() )
    }
  }

  impl ToneMappingPass< ToneMappingAces >
  {
    /// Creates an ACES tone mapping pass.
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/tonemapping/aces.frag" );
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = EmptyShader::new( gl, &program );

      Ok
      (
        Self
        {
          material,
          phantom : PhantomData
        }
      )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ToneMappingAces,
    ToneMappingPass
  };
}
