mod private
{
  use mingl::Former;
  use minwebgl::{ self as gl, JsCast };
  use std::{ cell::Cell, rc::Rc };
  use web_sys::wasm_bindgen::prelude::Closure;
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
    /// The target of the texture (e.g., `TEXTURE_2D`, `TEXTURE_CUBE_MAP`).  Defaults to `TEXTURE_2D`,
    /// for the `Former` builder as well as `Default`.
    #[ former( default = gl::TEXTURE_2D ) ]
    pub target : u32,
    /// The actual WebGL texture object.  Wrapped in an `Option` as it may not always be initialized.
    pub source : Option< gl::web_sys::WebGlTexture >,
    /// The sampler associated with the texture, which defines how the texture is sampled.
    pub sampler : Sampler,
    /// Shared owner of the GPU texture this `Texture` was created around by
    /// [`Texture::owning`]; `None` for a view. Ownership can only be established by
    /// that constructor: [`TextureOwner`] has private fields and no public
    /// constructor, and the builder gets no setter for this field.
    #[ scalar( setter = false ) ]
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

  /// What the image handler of one [`Texture::load_from_path`] call has done so far.
  ///
  /// Only observable through `Texture::load_from_path_for_test` (feature
  /// `test_internals`), so a test can wait for the handler instead of a fixed delay.
  #[ non_exhaustive ]
  #[ derive( Clone, Copy, Debug, PartialEq, Eq ) ]
  pub enum ImageLoad
  {
    /// Neither `load` nor `error` has fired yet.
    Pending,
    /// The image arrived and was uploaded into the texture.
    Uploaded,
    /// The image arrived after every clone of the texture had been dropped, so
    /// nothing was uploaded.
    Skipped,
    /// The image failed to load; the texture has no image data.
    Failed,
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
    pub fn owning
    (
      gl : &gl::GL,
      target : u32,
      source : gl::web_sys::WebGlTexture,
      sampler : Sampler,
    ) -> Self
    {
      let owner = Rc::new( TextureOwner { gl : gl.clone(), texture : source.clone() } );
      Self { target, source : Some( source ), sampler, owner : Some( owner ) }
    }

    /// Whether this `Texture` shares ownership of a GPU texture (built by
    /// [`Texture::owning`]) rather than being a view.
    ///
    /// It refers to the texture the value was built around, which `source` no
    /// longer names if `source` has been reassigned since; the owned texture is
    /// still the one deleted.
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
    /// this `Texture` alone, so the result is [owning](Texture::owning): keep it, or a clone,
    /// alive while its `source` is in use.
    ///
    /// The image uploads when it arrives. If every clone of the returned `Texture` has
    /// been dropped by then, the GPU texture is already deleted and the upload is skipped.
    /// If the image fails to load, the failure is logged to the console and the texture
    /// stays without image data.
    ///
    /// # Panics
    ///
    /// Panics if the browser has no `window`/`document`, or if the `<img>` element or the
    /// WebGL texture can't be created.
    #[ must_use ]
    pub fn load_from_path( gl : &gl::WebGl2RenderingContext, image_path : &str, flip : bool ) -> Self
    {
      Self::load_from_path_tracked( gl, image_path, flip ).0
    }

    /// [`Texture::load_from_path`], also returning what its image handler has done so far.
    fn load_from_path_tracked
    (
      gl : &gl::WebGl2RenderingContext,
      image_path : &str,
      flip : bool,
    ) -> ( Self, Rc< Cell< ImageLoad > > )
    {
      let state = Rc::new( Cell::new( ImageLoad::Pending ) );
      let source = gl.create_texture().expect( "Failed to create a texture" );

      let sampler = Sampler::former()
      .min_filter( MinFilterMode::Linear )
      .mag_filter( MagFilterMode::Linear )
      .wrap_s( WrappingMode::Repeat )
      .wrap_t( WrappingMode::Repeat )
      .end();

      let texture = Self::owning( gl, gl::TEXTURE_2D, source, sampler );
      let owner = Rc::downgrade( texture.owner.as_ref().expect( "Texture::owning sets an owner" ) );

      let document = web_sys::window().expect( "Can't get window" ).document().expect( "Can't get document" );
      let img = document.create_element( "img" )
      .expect( "Can't create img" )
      .dyn_into::< web_sys::HtmlImageElement >()
      .expect( "Can't convert to HtmlImageElement" );

      // Not minwebgl's `image_upload_from_path`: its forgotten `onload` closure holds the
      // raw handle, so once this owning `Texture` dropped it would still bind the deleted
      // texture. WebGL rejects that with INVALID_OPERATION and keeps the previous binding,
      // so the image and filter settings would land in whatever texture is bound. A
      // pending handler can't be cancelled; it reaches the texture through a `Weak`
      // owner instead (a strong `Rc` there would keep the texture alive forever).
      //
      // One `FnOnce` handler serves as both `onload` and `onerror`. The image fires
      // exactly one of them, and wasm-bindgen frees a `once_into_js` closure, with the
      // image and the `gl` clone it holds, after its call, instead of keeping a
      // forgotten closure alive for the page's lifetime.
      let settle = Closure::once_into_js
      (
        {
          let gl = gl.clone();
          let img = img.clone();
          let image_path = image_path.to_owned();
          let state = state.clone();
          move | event : web_sys::Event |
          {
            img.set_onload( None );
            img.set_onerror( None );
            if event.type_() == "error"
            {
              gl::browser::error!( "Failed to load texture image '{image_path}'" );
              state.set( ImageLoad::Failed );
              return;
            }
            // Every clone dropped: the texture is deleted, so there is nothing to upload into.
            let Some( owner ) = owner.upgrade()
            else
            {
              state.set( ImageLoad::Skipped );
              return;
            };
            if flip
            {
              gl::texture::d2::upload( &gl, Some( &owner.texture ), &img );
            }
            else
            {
              gl::texture::d2::upload_no_flip( &gl, Some( &owner.texture ), &img );
            }
            gl::texture::d2::filter_linear( &gl );
            state.set( ImageLoad::Uploaded );
          }
        }
      );
      img.set_onload( Some( settle.unchecked_ref() ) );
      img.set_onerror( Some( settle.unchecked_ref() ) );
      img.set_src( image_path );

      ( texture, state )
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

  /// The load signal of `Texture::load_from_path` -- reachable from `tests/` under
  /// `test_internals`. Its handler runs whenever the browser delivers the image, so
  /// a test waits on this state rather than on a fixed delay that a slow load can
  /// outlast.
  #[ cfg( feature = "test_internals" ) ]
  impl Texture
  {
    /// [`Texture::load_from_path`], plus the state its image handler sets once it
    /// has run (see [`ImageLoad`]).
    ///
    /// # Panics
    ///
    /// Same as [`Texture::load_from_path`].
    #[ doc( hidden ) ]
    #[ must_use ]
    pub fn load_from_path_for_test
    (
      gl : &gl::WebGl2RenderingContext,
      image_path : &str,
      flip : bool,
    ) -> ( Self, Rc< Cell< ImageLoad > > )
    {
      Self::load_from_path_tracked( gl, image_path, flip )
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

  #[ cfg( feature = "test_internals" ) ]
  orphan use ImageLoad;
}
