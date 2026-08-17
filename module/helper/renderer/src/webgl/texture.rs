mod private
{
  use mingl::Former;
  use minwebgl::{ self as gl };
  use crate::webgl::{ Sampler, MinFilterMode, MagFilterMode, WrappingMode };


  /// Represents a texture in WebGL.
  ///
  /// This struct encapsulates the necessary data and functionality for working with WebGL textures.
  /// It includes the texture's target, the actual WebGL texture object, and a sampler for controlling
  /// how the texture is sampled.
  #[ non_exhaustive ]
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
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Loads a 2D texture from `image_path`, sampled with linear filtering and repeat wrapping
    /// on both axes -- the sampler configuration duplicated, with no variation, by every
    /// example that loaded a texture from a path before this helper existed. `flip` controls
    /// whether the image is flipped vertically on upload ( WebGL's texture origin is
    /// bottom-left; most image formats decode top-left first ).
    #[ must_use ]
    pub fn load_from_path( gl : &gl::WebGl2RenderingContext, image_path : &str, flip : bool ) -> Self
    {
      let source = gl::texture::d2::image_upload_from_path( gl, image_path, flip );

      let sampler = Sampler::former()
      .min_filter( MinFilterMode::Linear )
      .mag_filter( MagFilterMode::Linear )
      .wrap_s( WrappingMode::Repeat )
      .wrap_t( WrappingMode::Repeat )
      .end();

      Self::former()
      .target( gl::TEXTURE_2D )
      .source( source )
      .sampler( sampler )
      .end()
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
        sampler : Sampler::default()
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
