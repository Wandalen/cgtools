mod private
{
  use mingl::Former;
  use minwebgl::{ self as gl };
  use crate::webgl::Sampler;

  #[ derive( Former ) ]
  pub struct Texture
  {
    pub target : u32,
    pub source : Option< gl::web_sys::WebGlTexture >,
    pub sampler : Sampler
  }

  impl Texture
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn upload( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.bind( gl );
      self.sampler.upload( gl, self.target );
    }

    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_texture( self.target, self.source.as_ref() );
    }
  }

  impl Default for Texture 
  {
    fn default() -> Self 
    {
      let target = gl::TEXTURE_2D;
      
      Self
      {
        target,
        source : None,
        sampler : Default::default()
      }
    }    
  }
}

crate::mod_interface!
{
  orphan use
  {
    Texture
  };
}