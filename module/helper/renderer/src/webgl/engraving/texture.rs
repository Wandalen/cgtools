mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use crate::webgl::
  {
    MagFilterMode,
    MinFilterMode,
    Sampler,
    Texture,
    TextureInfo,
    WrappingMode,
    engraving::EngravingBitmap,
  };

  /// A GPU-resident engraving mask texture that is allocated once and updated in place.
  ///
  /// Wraps the crate's generic [`Texture`]/[`TextureInfo`] so it can be handed directly to
  /// [`crate::webgl::material::PbrMaterial::set_engraving_texture`]. Filtering is `LINEAR`
  /// magnification with trilinear (`LINEAR_MIPMAP_LINEAR`) minification, and wrapping is
  /// `CLAMP_TO_EDGE`, per the engraving texture contract: the shader already bounds-checks
  /// `vEngravingUv` against `[0, 1]` itself (see `main.frag`), so edge clamping only needs to
  /// avoid sampling garbage at the exact boundary, not implement the "outside the strip" logic.
  ///
  /// Mipmaps matter more here than on a typical color texture: the mask's `dFdx`/`dFdy`
  /// derivative drives the relief-normal perturbation in `main.frag`, and without mipmaps a
  /// minified (zoomed-out) mask aliases hard at the black/white glyph edges — each screen pixel
  /// can land on a wildly different raw texel, so the derivative spikes and the bevel highlight
  /// sparkles. Mipmapping pre-filters the mask so both the sampled value and its derivative stay
  /// smooth under minification.
  pub struct EngravingTexture
  {
    info : TextureInfo,
    width : u32,
    height : u32,
    /// Number of mip levels a complete chain from `width` x `height` down to `1` x `1` requires
    /// (`floor(log2(max(width, height))) + 1`). Every [`Self::update`] call must supply exactly
    /// this many bitmaps, in the same order [`Self::mip_dimensions`] yields.
    levels : u32,
  }

  /// `floor(log2(max(width, height))) + 1` — the standard GL rule for how many mip levels a
  /// complete chain from `width` x `height` down to `1` x `1` requires.
  fn mip_level_count( width : u32, height : u32 ) -> u32
  {
    u32::BITS - width.max( height ).leading_zeros()
  }

  impl EngravingTexture
  {
    /// Allocates a full mip chain of RGBA8 storage (initially transparent/black), from `width` x
    /// `height` down to `1` x `1`, bound to UV set `uv_position` (i.e. `vUv_{uv_position}` in the
    /// shader — see [`crate::webgl::engraving::EngravingNodeConfig::uv_channel`]).
    ///
    /// Every level is allocated individually rather than via `generateMipmap`: see
    /// [`crate::webgl::engraving::EngravingCanvasManager::render_text_mip_chain`] for why a
    /// hand-rasterized chain looks better than a box-filtered one for a binary text mask.
    ///
    /// `width`/`height` should come from [`crate::webgl::engraving::EngravingNodeConfig::texture_size`]
    /// and must match every bitmap chain later passed to [`Self::update`] — the whole point of
    /// this type is that this allocation happens exactly once.
    pub fn new( gl : &GL, width : u32, height : u32, uv_position : u32 ) -> Result< Self, gl::WebglError >
    {
      let source = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "engraving texture" ) )?;
      gl.bind_texture( gl::TEXTURE_2D, Some( &source ) );

      let levels = mip_level_count( width, height );
      let ( mut w, mut h ) = ( width, height );
      for level in 0..levels
      {
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
        (
          gl::TEXTURE_2D,
          level as i32,
          gl::RGBA as i32,
          w as i32,
          h as i32,
          0,
          gl::RGBA,
          gl::UNSIGNED_BYTE,
          None,
        )
        .map_err( | _ | gl::WebglError::FailedToAllocateResource( "engraving texture storage" ) )?;
        w = ( w / 2 ).max( 1 );
        h = ( h / 2 ).max( 1 );
      }

      let sampler = Sampler
      {
        mag_filter : Some( MagFilterMode::Linear ),
        min_filter : Some( MinFilterMode::LinearMipMapLinear ),
        wrap_s : Some( WrappingMode::ClampToEdge ),
        wrap_t : Some( WrappingMode::ClampToEdge ),
        ..Default::default()
      };

      let texture = Texture { target : gl::TEXTURE_2D, source : Some( source ), sampler };
      let info = TextureInfo { texture : Rc::new( RefCell::new( texture ) ), uv_position };

      Ok( Self { info, width, height, levels } )
    }

    /// Uploads a freshly rasterized mip chain into the existing GPU allocation via
    /// `texSubImage2D` per level, so re-engraving text (e.g. on every keystroke) never
    /// reallocates or triggers a texture-completeness re-check.
    ///
    /// `bitmaps` must have exactly [`Self::level_count`] entries, ordered the same way
    /// [`crate::webgl::engraving::EngravingCanvasManager::render_text_mip_chain`] returns them
    /// (index `0` is the full `width` x `height` level).
    pub fn update( &self, gl : &GL, bitmaps : &[ EngravingBitmap ] ) -> Result< (), gl::WebglError >
    {
      if bitmaps.len() as u32 != self.levels
      {
        return Err( gl::WebglError::Other( "engraving bitmap chain does not match the allocated mip level count" ) );
      }

      gl.bind_texture( gl::TEXTURE_2D, self.info.texture.borrow().source.as_ref() );

      let ( mut w, mut h ) = ( self.width, self.height );
      for ( level, bitmap ) in bitmaps.iter().enumerate()
      {
        if bitmap.width != w || bitmap.height != h
        {
          return Err( gl::WebglError::Other( "engraving bitmap size does not match its mip level's allocated size" ) );
        }

        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array
        (
          gl::TEXTURE_2D,
          level as i32,
          0,
          0,
          w as i32,
          h as i32,
          gl::RGBA,
          gl::UNSIGNED_BYTE,
          Some( &bitmap.pixels ),
        )
        .map_err( | _ | gl::WebglError::Other( "tex_sub_image_2d failed for engraving texture" ) )?;

        w = ( w / 2 ).max( 1 );
        h = ( h / 2 ).max( 1 );
      }

      Ok( () )
    }

    /// Width in pixels of mip level 0 of the GPU allocation.
    pub fn width( &self ) -> u32 { self.width }

    /// Height in pixels of mip level 0 of the GPU allocation.
    pub fn height( &self ) -> u32 { self.height }

    /// Number of mip levels [`Self::update`] expects a bitmap for, from level 0 (`width` x
    /// `height`) down to the final `1` x `1` level.
    pub fn level_count( &self ) -> u32 { self.levels }

    /// Returns a [`TextureInfo`] handle to this texture, suitable for
    /// `PbrMaterial::set_engraving_texture`. Cloning is cheap (the underlying `Texture` is
    /// `Rc`-shared), and further [`Self::update`] calls remain visible through every clone.
    pub fn texture_info( &self ) -> TextureInfo
    {
      self.info.clone()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    EngravingTexture,
  };
}
