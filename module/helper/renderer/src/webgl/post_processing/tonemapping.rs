mod private
{
  use std::marker::PhantomData;
  use minwebgl as gl;
  use crate::webgl::
  { 
    post_processing::{ Pass, VS_TRIANGLE }, 
    program::EmptyShader,
    ProgramInfo 
  };

  pub struct ToneMappingAces;

  pub struct ToneMappingPass< T >
  {
    material : ProgramInfo< EmptyShader >,
    output_texture : Option< gl::web_sys::WebGlTexture >,
    phantom : std::marker::PhantomData< T >
  }

  impl< T > ToneMappingPass< T > 
  {
    fn create_texture( gl : &gl::WebGl2RenderingContext, width : u32, height : u32 ) -> Option< gl::web_sys::WebGlTexture >
    {
      let texture = gl.create_texture();
      gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA8, width as i32, height as i32 );
      gl::texture::d2::filter_linear( &gl );

      texture
    }    

    pub fn get_output_texture( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.output_texture.clone()
    }
  }

  impl< T > Pass for ToneMappingPass< T >
  {
    fn render
    (
      &self,
      gl : &minwebgl::WebGl2RenderingContext,
      input_texture : Option< minwebgl::web_sys::WebGlTexture >
    ) -> Result< Option< minwebgl::web_sys::WebGlTexture >, minwebgl::WebglError > 
    {
      gl.disable( gl::DEPTH_TEST );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      self.material.bind( gl );
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        self.output_texture.as_ref(), 
        0
      );

      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      Ok( self.output_texture.clone() )
    }
  }

  impl ToneMappingPass< ToneMappingAces > 
  {
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32 ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../shaders/tonemapping/aces.frag" );
      let material = gl::ProgramFromSources::new(  VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let material = ProgramInfo::< EmptyShader >::new( material );
      
      let output_texture = Self::create_texture( gl, width, height );

      Ok
      ( 
        Self
        {
          material,
          output_texture,
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