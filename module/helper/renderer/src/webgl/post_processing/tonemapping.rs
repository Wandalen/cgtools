mod private
{
  use std::marker::PhantomData;
  use minwebgl as gl;
  use gl::web_sys::WebGlProgram;
  use rustc_hash::FxHashMap;
  use crate::webgl::
  {
    ShaderProgram, ProgramInfo, post_processing::{ Pass, VS_TRIANGLE }
  };
  use crate::webgl::impl_locations;

  // The tone mapping program: the HDR source on unit 0, the optional transparent-coverage
  // texture on unit 1.
  impl_locations!
  (
    ToneMappingShader,
    "sourceTexture",
    "transparentCoverage",
    "useTransparentCoverage"
  );

  /// Represents the ACES (Academy Color Encoding System) tone mapping algorithm.
  pub struct ToneMappingAces;

  /// A generic post-processing pass for applying tone mapping to a texture.
  pub struct ToneMappingPass< T >
  {
    /// The WebGL program used for the tone mapping operation.
    material : ToneMappingShader,
    /// The renderer's transparent-coverage texture; `None` keeps the MSAA-edge
    /// reconstruction off.
    transparent_coverage : Option< gl::web_sys::WebGlTexture >,
    phantom : std::marker::PhantomData< T >
  }

  impl< T > ToneMappingPass< T >
  {
    /// Turns on the MSAA-edge fix by telling the pass which pixels hold transparent
    /// coverage : pass `Renderer::transparent_coverage_texture()` here once after the
    /// renderer is created and again after every `Renderer::resize`, which replaces the
    /// texture. The handle stays valid between those; the renderer keeps its contents
    /// current every frame.
    ///
    /// With the texture, the pass tone maps a partially covered pixel with the background
    /// it lets through, estimated from neighbouring background pixels, which removes the
    /// light fringe on bright MSAA silhouettes. The fix is opt-in because it needs the
    /// texture : transparent ( WBOIT ) surfaces over the background are partially covered
    /// too, and only their outline has background neighbours, so without knowing which
    /// pixels hold glass the pass would tone map that outline differently from the rest of
    /// the surface and draw a 1-px ring. With `None`, the default, every partially covered
    /// pixel takes the plain blend: no ring, but bright silhouettes keep their fringe.
    ///
    /// The pass does not own the texture and binds whatever it was last given.
    pub fn transparent_coverage_texture_set( &mut self, texture : Option< gl::web_sys::WebGlTexture > )
    {
      self.transparent_coverage = texture;
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
      // Screen-space pass: the fullscreen triangle is back-facing, so it must not
      // inherit the scene's CULL_FACE state or it gets culled and nothing draws.
      gl.disable( gl::CULL_FACE );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      self.material.bind( gl );
      let locations = self.material.locations();
      gl.uniform1i
      (
        locations.get( "useTransparentCoverage" ).and_then( Clone::clone ).as_ref(),
        i32::from( self.transparent_coverage.is_some() )
      );

      // Unit 1 always gets a texture, the source itself when there is no coverage
      // texture: the shader doesn't read it then, but a sampler on an empty unit still
      // draws a "no texture bound" warning every frame.
      gl.active_texture( gl::TEXTURE1 );
      gl.bind_texture( gl::TEXTURE_2D, self.transparent_coverage.as_ref().or( input_texture.as_ref() ) );
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

      gl::clean::texture_2d_active( gl, 1 );
      gl.active_texture( gl::TEXTURE0 );
      gl::clean::texture_2d( gl );
      gl::clean::framebuffer_texture_2d( gl );

      Ok( output_texture.clone() )
    }
  }

  impl ToneMappingPass< ToneMappingAces >
  {
    /// Creates an ACES tone mapping pass.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if the tone-mapping shader fails to compile or link.
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/tonemapping/aces.frag" );
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = ToneMappingShader::new( gl, &program );

      material.bind( gl );
      let locations = material.locations();
      gl.uniform1i( locations.get( "sourceTexture" ).and_then( Clone::clone ).as_ref(), 0 );
      gl.uniform1i( locations.get( "transparentCoverage" ).and_then( Clone::clone ).as_ref(), 1 );

      Ok
      (
        Self
        {
          material,
          transparent_coverage : None,
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
