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
  ///
  /// `source` is frequently a **non-owning view** onto a GPU texture created and managed elsewhere
  /// (e.g. a `SwapFramebuffer`/`CanvasRenderer` output re-wrapped as a `Texture` to sample it, or
  /// multiple glTF textures referencing one shared source image). `owned` + `gl` exist so `Drop`
  /// can tell the two cases apart: only a `Texture` explicitly marked `owned = true` (with `gl` set)
  /// deletes `source` on drop. Both default to "not owning" — the safe default, since aliasing was
  /// already relied upon before `Drop` existed and must not start deleting resources still in use
  /// elsewhere. Callers that construct a `Texture` around a GPU texture they alone are responsible
  /// for must opt in explicitly (`.owned( true ).gl( gl.clone() )` via the `Former` builder, or by
  /// setting the fields directly).
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
    /// Whether this `Texture` is the sole owner of `source` and must delete it on drop.
    /// See the struct docs — defaults to `false` (non-owning / aliasing).
    pub owned : bool,
    /// WebGL context used by `Drop` to delete `source` when `owned` is `true`. `None` means
    /// this `Texture` cannot delete its GPU resources even if later marked `owned` — the two
    /// should always be set together.
    pub gl : Option< gl::GL >,
  }

  impl Texture
  {
    /// Creates a new `Texture` with default values.
    #[ must_use ]
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
        sampler : Default::default(),
        owned : false,
        gl : None,
      }
    }
  }

  /// Deletes `source` only when this `Texture` was explicitly marked as its owner — see the
  /// struct docs. A non-owning `Texture` (the default) is a pure view and must not delete
  /// anything, since the same GPU texture is very likely still bound to a `SwapFramebuffer`,
  /// another glTF texture entry sharing one source image, or similar.
  impl Drop for Texture
  {
    fn drop( &mut self )
    {
      if self.owned
      {
        if let Some( gl ) = &self.gl
        {
          gl.delete_texture( self.source.as_ref() );
        }
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
