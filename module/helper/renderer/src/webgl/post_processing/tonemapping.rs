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
    exposure_loc : Option< gl::WebGlUniformLocation >,
    exposure : f32,
    phantom : std::marker::PhantomData< T >
  }

  impl< T > ToneMappingPass< T >
  {
    pub fn set_exposure( &mut self, exposure : f32 )
    {
      self.exposure = exposure;
    }
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
      gl.uniform1f( self.exposure_loc.as_ref(), self.exposure );
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
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/tonemapping/aces.frag" );
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let exposure_loc = gl.get_uniform_location( &program, "exposure" );
      let material = EmptyShader::new( gl, &program );

      Ok
      (
        Self
        {
          material,
          exposure_loc,
          exposure : 1.0,
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
