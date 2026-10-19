
mod private
{
  use minwebgl as gl;
  use crate::webgl::{ ShaderProgram, post_processing::{Pass, VS_TRIANGLE}, program::EmptyShader };

  /// A post-processing pass responsible for converting a linear color space texture
  /// to the sRGB color space.
  pub struct ToSrgbPass
  {
    /// The WebGL program used for the sRGB conversion.
    material : EmptyShader,
    /// A boolean flag indicating whether the output of this pass should be
    /// rendered directly to the screen's default framebuffer or
    /// to an offscreen `output_texture`.
    render_to_screen : bool,
    /// Whether the output keeps the input's alpha (see `alpha_forward_set`).
    alpha_forward : bool,
  }

  impl ToSrgbPass
  {
    /// Sets whether the pass should render its output directly to the screen.
    pub fn render_to_screen_set( &mut self, render_to_screen : bool )
    {
      self.render_to_screen = render_to_screen;
    }

    /// Whether the output keeps the input's alpha instead of being opaque.
    ///
    /// Off by default: the output is opaque, as a page expects of a canvas that
    /// paints its own background. On, background pixels keep the alpha the scene
    /// cleared them with (0 for a transparent clear), so the canvas can be
    /// alpha-composited over other content — a photo or video behind it.
    ///
    /// The output is premultiplied alpha, for a canvas created with
    /// `premultipliedAlpha: true` (the WebGL default): the pass treats its input as
    /// premultiplied, sRGB-encodes the straight colour and multiplies alpha back in,
    /// so `rgb <= alpha` holds on every pixel. The input is premultiplied only when
    /// the scene is cleared with colour 0: with any other clear colour, the MSAA edge
    /// pixels carry that colour blended in, and the background itself comes out
    /// fully transparent.
    ///
    /// The alpha only arrives here if every pass between the scene and this one
    /// passes it through. `ToneMappingPass< ToneMappingAces >` and
    /// `ColorGradingPass` do; a custom pass that writes alpha 1 makes the whole
    /// canvas opaque again. `ColorGradingPass` grades the premultiplied colour, so
    /// at partially covered edge pixels its result is approximate.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if the shader variant fails to compile or link.
    pub fn alpha_forward_set( &mut self, gl : &gl::WebGl2RenderingContext, alpha_forward : bool ) -> Result< (), gl::WebglError >
    {
      if alpha_forward != self.alpha_forward
      {
        self.material = Self::program( gl, alpha_forward )?;
        self.alpha_forward = alpha_forward;
      }
      Ok( () )
    }

    /// Whether the output keeps the input's alpha (see `alpha_forward_set`).
    #[ must_use ]
    pub fn alpha_forward( &self ) -> bool
    {
      self.alpha_forward
    }

    fn program( gl : &gl::WebGl2RenderingContext, alpha_forward : bool ) -> Result< EmptyShader, gl::WebglError >
    {
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, &fragment_source( alpha_forward ) ).compile_and_link( gl )?;
      Ok( EmptyShader::new( gl, &program ) )
    }

    /// Creates a new `ToSrgbPass` instance with opaque output.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if the shader fails to compile or link.
    pub fn new( gl : &gl::WebGl2RenderingContext, render_to_screen : bool ) -> Result< Self, gl::WebglError >
    {
      Ok
      (
        Self
        {
          material : Self::program( gl, false )?,
          render_to_screen,
          alpha_forward : false,
        }
      )
    }
  }

  /// The fragment shader source [`ToSrgbPass`] compiles: `to_srgb.frag`, with
  /// `#define FORWARD_ALPHA` inserted after its `#version` line when `alpha_forward` is set
  /// ( see [`ToSrgbPass::alpha_forward_set`] ). Reachable from `tests/` under `test_internals`.
  #[ must_use ]
  pub fn fragment_source( alpha_forward : bool ) -> String
  {
    let fs_shader = include_str!( "../shaders/post_processing/to_srgb.frag" );
    if alpha_forward
    {
      // Defines must follow the `#version` line.
      fs_shader.replacen( "precision highp float;", "#define FORWARD_ALPHA\nprecision highp float;", 1 )
    }
    else
    {
      fs_shader.to_string()
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
      // Screen-space pass: the fullscreen triangle is back-facing, so it must not
      // inherit the scene's CULL_FACE state or it gets culled and nothing draws.
      gl.disable( gl::CULL_FACE );
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

/// The internals `tests/` reaches, exposed only under `test_internals`.
///
/// Not part of the surface: with the feature off, the default and what every dependent gets,
/// this module does not exist.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use super::private::fragment_source;
}
