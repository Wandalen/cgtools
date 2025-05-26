
mod private
{
  use minwebgl as gl;
  use crate::webgl::{ post_processing::{ Pass, VS_TRIANGLE }, program::EmptyShader, ProgramInfo };

  pub struct BlendPass
  {
    pub src_factor : u32,
    pub dst_factor : u32,
    pub equation : u32,
    material : ProgramInfo< EmptyShader >,
    pub blend_texture : Option< gl::web_sys::WebGlTexture >
  }

  impl BlendPass 
  {
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

      // Unbind the attachment
      gl.bind_texture( gl::TEXTURE_2D, None );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        None, 
        0
      );

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