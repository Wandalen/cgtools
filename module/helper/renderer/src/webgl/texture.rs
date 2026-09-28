mod private
{
  use mingl::Former;
  use minwebgl::{ self as gl };
  use std::rc::Rc;
  use crate::webgl::{ Sampler, MinFilterMode, MagFilterMode, WrappingMode };


  /// Represents a texture in WebGL.
  ///
  /// This struct encapsulates the necessary data and functionality for working with WebGL textures.
  /// It includes the texture's target, the actual WebGL texture object, and a sampler for controlling
  /// how the texture is sampled.
  ///
  /// GPU ownership: by default a `Texture` is a **view** — `source` is a GPU texture
  /// created and released elsewhere (a framebuffer attachment re-wrapped for sampling, one glTF
  /// image referenced by several glTF textures, ...), and dropping the view deletes nothing.
  /// A texture built with [`Texture::owning`] also holds a shared [`TextureOwner`]: clones of
  /// it share that owner, and the GPU texture is deleted once the last of them drops, so an
  /// owning `Texture` can be cloned freely without a double delete.
  #[ non_exhaustive ]
  #[ derive( Former, Clone, Debug ) ]
  pub struct Texture
  {
    /// The target of the texture (e.g., `TEXTURE_2D`, `TEXTURE_CUBE_MAP`).  Defaults to `TEXTURE_2D`.
    pub target : u32,
    /// The actual WebGL texture object.  Wrapped in an `Option` as it may not always be initialized.
    pub source : Option< gl::web_sys::WebGlTexture >,
    /// The sampler associated with the texture, which defines how the texture is sampled.
    pub sampler : Sampler,
    /// Shared owner of the GPU texture this `Texture` was created around by
    /// [`Texture::owning`]; `None` for a view. Private so ownership can only be
    /// established by that constructor.
    owner : Option< Rc< TextureOwner > >,
  }

  /// Sole owner of one GPU texture: deletes it when dropped.
  ///
  /// Only created by [`Texture::owning`] and only reachable through the `Rc` every
  /// clone of that `Texture` shares, so the deletion happens exactly once, after the
  /// last clone is gone. It keeps its own handle, so reassigning `Texture::source`
  /// afterwards does not redirect what gets deleted.
  #[ derive( Debug ) ]
  pub struct TextureOwner
  {
    gl : gl::GL,
    texture : gl::web_sys::WebGlTexture,
  }

  impl Drop for TextureOwner
  {
    fn drop( &mut self )
    {
      self.gl.delete_texture( Some( &self.texture ) );
    }
  }

  impl Texture
  {
    /// Creates a new `Texture` with default values.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// A `Texture` that owns `source`: the GPU texture is deleted once this
    /// `Texture` and every clone of it have been dropped (see [`TextureOwner`]).
    ///
    /// Use it for a GPU texture created for this `Texture` alone; wrap textures
    /// managed elsewhere with the `Former` builder, which makes a non-owning view.
    #[ must_use ]
    pub fn owning( gl : &gl::GL, target : u32, source : gl::web_sys::WebGlTexture, sampler : Sampler ) -> Self
    {
      let owner = Rc::new( TextureOwner { gl : gl.clone(), texture : source.clone() } );
      Self { target, source : Some( source ), sampler, owner : Some( owner ) }
    }

    /// Whether this `Texture` shares ownership of its GPU texture (built by
    /// [`Texture::owning`]) rather than being a view.
    #[ must_use ]
    pub fn is_owning( &self ) -> bool
    {
      self.owner.is_some()
    }

    /// Loads a 2D texture from `image_path`, sampled with linear filtering and repeat wrapping
    /// on both axes -- the sampler configuration duplicated, with no variation, by every
    /// example that loaded a texture from a path before this helper existed. `flip` controls
    /// whether the image is flipped vertically on upload ( WebGL's texture origin is
    /// bottom-left; most image formats decode top-left first ). The GPU texture is created for
    /// this `Texture` alone, so the result is [owning](Texture::owning).
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

      Self::owning( gl, gl::TEXTURE_2D, source, sampler )
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
        sampler : Sampler::default(),
        owner : None,
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Texture,
    TextureOwner
  };
}
