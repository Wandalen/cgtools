
mod private
{
  use minwebgl as gl;
  use crate::webgl::{ post_processing::{ Pass, VS_TRIANGLE }, program::EmptyShader, ProgramInfo };

  /// A post-processing pass that converts shadow texture (R8 format) to a colored base color texture.
  ///
  /// The pass reads shadow values from the red channel (where 1.0 = maximum shadow, 0.0 = no shadow)
  /// and applies the formula: (1 - shadow_value) * color
  pub struct ShadowToColorPass
  {
    /// The WebGL program used for the shadow-to-color conversion.
    material : ProgramInfo< EmptyShader >,
    /// The color to multiply with the inverted shadow value.
    color : [ f32; 3 ],
    /// Location of the color uniform in the shader.
    color_location : Option< gl::web_sys::WebGlUniformLocation >
  }

  impl ShadowToColorPass
  {
    /// Creates a new `ShadowToColorPass` instance.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    /// * `color` - The base color to apply (RGB values, typically in range [0.0, 1.0])
    pub fn new( gl : &gl::WebGl2RenderingContext, color : [ f32; 3 ] ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/post_processing/shadow_to_color.frag" );
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let color_location = gl.get_uniform_location( &program, "color" );
      let material = ProgramInfo::< EmptyShader >::new( gl, program );

      Ok
      (
        Self
        {
          material,
          color,
          color_location
        }
      )
    }

    /// Sets the color to be used in the conversion.
    ///
    /// # Arguments
    ///
    /// * `color` - The new color (RGB values)
    pub fn set_color( &mut self, color : [ f32; 3 ] )
    {
      self.color = color;
    }
  }

  impl Pass for ShadowToColorPass
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
      gl.disable( gl::CULL_FACE );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      self.material.bind( gl );

      gl::uniform::upload( gl, self.color_location.clone(), &self.color )?;

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

      Ok( output_texture )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ShadowToColorPass
  };
}
