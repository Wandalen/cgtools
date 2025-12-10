mod private
{
  use minwebgl as gl;
  use crate::webgl::
  {
    ProgramInfo, ShaderProgram, post_processing::{ Pass, VS_TRIANGLE }, program::ColorGradingShader
  };

  /// Parameters for color grading adjustments.
  ///
  /// All parameters are designed to use **0.0 as neutral** and range from **-1.0 to 1.0**
  /// for consistent, linear behavior. The shader remaps these to appropriate internal ranges
  /// to prevent over-sensitivity.
  #[ derive( Debug, Clone ) ]
  pub struct ColorGradingParams
  {
    /// White balance temperature adjustment (-1.0 to 1.0, 0.0 is neutral)
    /// - Positive: warmer (orange tones)
    /// - Negative: cooler (blue tones)
    pub temperature : f32,

    /// White balance tint adjustment (-1.0 to 1.0, 0.0 is neutral)
    /// - Positive: magenta tint
    /// - Negative: green tint
    pub tint : f32,

    /// Overall exposure/brightness adjustment (-1.0 to 1.0, 0.0 is neutral)
    /// - Simple, predictable brightness control
    /// - Negative: darker overall (2^-1 = 0.5x brightness)
    /// - Positive: brighter overall (2^1 = 2x brightness)
    /// - Uses exponential scaling for natural feel
    pub exposure : f32,

    /// Shadow recovery and control (-1.0 to 1.0, 0.0 is neutral)
    /// - Only affects dark areas of the image
    /// - Negative: crush/darken shadows (dramatic look)
    /// - Positive: lift/brighten shadows (recover detail)
    /// - Uses smooth masking to avoid affecting highlights
    pub shadows : f32,

    /// Highlight recovery and control (-1.0 to 1.0, 0.0 is neutral)
    /// - Only affects bright areas of the image
    /// - Negative: blow out highlights (dreamy look)
    /// - Positive: compress/recover highlights (preserve detail)
    /// - Uses smooth masking to avoid affecting shadows
    pub highlights : f32,

    /// Cinematic tone curve (-1.0 to 1.0, 0.0 is neutral)
    /// - Creates a filmic look with lifted shadows and rolled-off highlights
    /// - Positive: cinematic look with depth (smoothstep-based S-curve)
    /// - Negative: flattened/matte look (reduced dynamic range)
    /// - At 0: passes through unchanged (true neutral)
    pub contrast : f32,

    /// Vibrance adjustment (-1.0 to 1.0, 0.0 is neutral)
    /// - Smart saturation that affects less-saturated colors more
    /// - Preserves skin tones better than regular saturation
    pub vibrance : f32,

    /// Saturation adjustment (-1.0 to 1.0, 0.0 is neutral)
    /// - Positive: more saturated colors
    /// - Negative: desaturated (moves toward grayscale)
    pub saturation : f32,
  }

  impl Default for ColorGradingParams
  {
    /// Returns neutral color grading parameters (0.0 for all values).
    ///
    /// This produces no color correction - the image passes through unchanged.
    /// Start with these defaults and adjust to taste.
    fn default() -> Self
    {
      Self
      {
        temperature : 0.0,  // Neutral white balance
        tint : 0.0,         // Neutral tint
        exposure : 0.0,     // Neutral brightness
        shadows : 0.0,      // Neutral shadows
        highlights : 0.0,   // Neutral highlights
        contrast : 0.0,     // Neutral contrast
        vibrance : 0.0,     // Neutral vibrance
        saturation : 0.0,   // Neutral saturation
      }
    }
  }

  /// A post-processing pass for color grading operations.
  ///
  /// Applies various color correction adjustments including white balance,
  /// lift-gamma-gain, contrast, vibrance, and saturation.
  pub struct ColorGradingPass
  {
    /// The WebGL program used for color grading.
    material : ProgramInfo,
    /// Color grading parameters
    pub params : ColorGradingParams,
  }

  impl ColorGradingPass
  {
    /// Creates a new `ColorGradingPass` with default parameters.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/post_processing/color_grading.frag" );
      let material = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = ProgramInfo::new( gl, &material, ColorGradingShader.dyn_clone() );

      Ok
      (
        Self
        {
          material,
          params : ColorGradingParams::default(),
        }
      )
    }

    /// Sets the color grading parameters.
    pub fn set_params( &mut self, params : ColorGradingParams )
    {
      self.params = params;
    }

    /// Gets a reference to the current color grading parameters.
    pub fn get_params( &self ) -> &ColorGradingParams
    {
      &self.params
    }

    /// Gets a mutable reference to the current color grading parameters.
    pub fn get_params_mut( &mut self ) -> &mut ColorGradingParams
    {
      &mut self.params
    }
  }

  impl Pass for ColorGradingPass
  {
    fn renders_to_input( &self ) -> bool
    {
      false
    }

    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< gl::web_sys::WebGlTexture >,
      output_texture : Option< gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >
    {
      // Disable depth testing and blending for post-processing
      gl.disable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      // Bind the color grading shader program
      self.material.bind( gl );
      let locations = self.material.get_locations();

      // Upload all uniforms
      gl::uniform::upload( gl, locations.get( "temperature" ).unwrap().clone(), &self.params.temperature )?;
      gl::uniform::upload( gl, locations.get( "tint" ).unwrap().clone(), &self.params.tint )?;
      gl::uniform::upload( gl, locations.get( "exposure" ).unwrap().clone(), &self.params.exposure )?;
      gl::uniform::upload( gl, locations.get( "shadows" ).unwrap().clone(), &self.params.shadows )?;
      gl::uniform::upload( gl, locations.get( "highlights" ).unwrap().clone(), &self.params.highlights )?;
      gl::uniform::upload( gl, locations.get( "contrast" ).unwrap().clone(), &self.params.contrast )?;
      gl::uniform::upload( gl, locations.get( "vibrance" ).unwrap().clone(), &self.params.vibrance )?;
      gl::uniform::upload( gl, locations.get( "saturation" ).unwrap().clone(), &self.params.saturation )?;

      // Bind input texture
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );

      // Set output framebuffer attachment
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        output_texture.as_ref(),
        0
      );

      // Clear and render
      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // Unbind the texture and framebuffer attachment to restore default state
      gl::clean::texture_2d( gl );
      gl::clean::framebuffer_texture_2d( gl );

      Ok( output_texture )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ColorGradingPass,
    ColorGradingParams
  };
}
