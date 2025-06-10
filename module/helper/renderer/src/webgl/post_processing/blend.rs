
mod private
{
  use minwebgl as gl;
  use crate::webgl::{ post_processing::{ Pass, VS_TRIANGLE }, program::EmptyShader, ProgramInfo };

  /// A post-processing pass designed to blend a source texture (`blend_texture`)
  /// onto a destination texture (`output_texture`) using specified blending parameters.
  pub struct BlendPass
  {
    /// The source blending factor
    pub src_factor : u32,
    /// The destination blending factor
    pub dst_factor : u32,
    // The blending equation, specifying how source and destination components are combined.
    pub equation : u32,
    /// The WebGL program used for the blending operation.
    material : ProgramInfo< EmptyShader >,
    /// The texture that will be blended onto the `output_texture`. This is the source
    pub blend_texture : Option< gl::web_sys::WebGlTexture >
  }

  impl BlendPass 
  {
    /// Set the blending texture of the pass
    pub fn set_blend_texture( &mut self, texture : Option< gl::web_sys::WebGlTexture > )
    {
      self.blend_texture = texture;
    }

    /// Creates a new `BlendPass` instance with default blending parameters.
    ///
    /// By default, it sets up alpha blending (`gl::SRC_ALPHA`, `gl::ONE_MINUS_SRC_ALPHA`)
    /// with an additive equation (`gl::FUNC_ADD`). The `blend_texture` is initially `None`.
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Result< Self, gl::WebglError >
    {
      let src_factor = gl::SRC_ALPHA;
      let dst_factor = gl::ONE_MINUS_SRC_ALPHA;
      let equation = gl::FUNC_ADD;
      let blend_texture = None;

      let fs_shader = include_str!( "../shaders/copy.frag" );
      let material = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = ProgramInfo::< EmptyShader >::new( material );
      
      Ok
      (
        Self
        {
          src_factor,
          dst_factor,
          equation,
          material,
          blend_texture
        }
      )
    }    
  }

  impl Pass for BlendPass 
  {
    fn renders_to_input( &self ) -> bool 
    {
      true
    }

    /// Belnds the `self.blend_texture` with the `output_texture`, setting the `output_texture` as destination
    fn render
    (
        &self,
        gl : &minwebgl::WebGl2RenderingContext,
        _input_texture : Option< minwebgl::web_sys::WebGlTexture >,
        output_texture : Option< minwebgl::web_sys::WebGlTexture >
    ) -> Result< Option< minwebgl::web_sys::WebGlTexture >, minwebgl::WebglError > 
    {
      gl.disable( gl::DEPTH_TEST );
      gl.enable( gl::BLEND );
      gl.blend_equation( self.equation );
      gl.blend_func( self.src_factor, self.dst_factor );

      // Bind the copy shader.
      self.material.bind( gl );
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, self.blend_texture.as_ref() );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        output_texture.as_ref(), 
        0
      );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // --- Cleanup ---
      // Unbind the texture and framebuffer attachment to restore default state.
      gl::clean::texture_2d( gl );
      gl::clean::framebuffer_texture_2d( gl );

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
    BlendPass
  };
}