mod private
{
  use mingl::Former;
  use minwebgl::{ self as gl };
  use crate::webgl::Sampler;


  /// Represents a texture in WebGL.
  ///
  /// This struct encapsulates the necessary data and functionality for working with WebGL textures.
  /// It includes the texture's target, the actual WebGL texture object, and a sampler for controlling
  /// how the texture is sampled.
  #[ derive( Former, Clone, Debug ) ]
  pub struct Texture
  {
    /// The target of the texture (e.g., `TEXTURE_2D`, `TEXTURE_CUBE_MAP`).  Defaults to `TEXTURE_2D`.
    pub target : u32,
    /// The actual WebGL texture object.  Wrapped in an `Option` as it may not always be initialized.
    pub source : Option< gl::web_sys::WebGlTexture >,
    /// The sampler associated with the texture, which defines how the texture is sampled.
    pub sampler : Sampler
  }

  impl Texture
  {
    /// Creates a new `Texture` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// This function binds the texture to the given WebGL context and then uploads the sampler
    /// parameters.
    pub fn upload( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.bind( gl );
      self.sampler.upload( gl, self.target );
    }

    /// Binds the texture to the WebGL context.
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