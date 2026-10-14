//! The GPU side of an engraving mask: a texture whose full mip chain is allocated once and then
//! updated in place, and [`mip_dimensions`], the mip-chain sizes it shares with the canvas.

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
  /// [`crate::webgl::material::PbrMaterial::engraving_texture_set`]. Filtering is `LINEAR`
  /// magnification with trilinear (`LINEAR_MIPMAP_LINEAR`) minification, and wrapping is
  /// `CLAMP_TO_EDGE`, per the engraving texture contract: the shader already bounds-checks
  /// `vEngravingUv` against `[0, 1]` itself (see `main.frag`), so edge clamping only needs to
  /// avoid sampling garbage at the exact boundary, not implement the "outside the strip" logic.
  ///
  /// Mipmaps matter more here than on a typical color texture: `main.frag` builds the relief
  /// normal from central differences of `textureLod` samples, at the LOD that matches the
  /// fragment's UV footprint. Without mips a minified (zoomed-out) mask has only the full-size
  /// level to sample, where neighbouring samples land on unrelated texels at the hard
  /// black/white glyph edges, so the gradient spikes and the bevel highlight sparkles. The mip
  /// chain gives that LOD a pre-filtered mask whose edges stay smooth at every distance.
  pub struct EngravingTexture
  {
    info : TextureInfo,
    width : u32,
    height : u32,
    /// Number of mip levels a complete chain from `width` x `height` down to `1` x `1` requires
    /// (`floor(log2(max(width, height))) + 1`). Every [`Self::update`] call must supply exactly
    /// this many bitmaps, in the order [`mip_dimensions`] yields.
    levels : u32,
  }

  /// The size of every level of a complete mip chain from `width` x `height` down to `1` x `1`,
  /// level 0 first: each level halves both dimensions, rounding down and never below `1`, the
  /// standard GL rule. It yields `floor(log2(max(width, height))) + 1` sizes.
  ///
  /// [`EngravingTexture`] allocates and uploads by it and
  /// [`crate::webgl::engraving::EngravingCanvasManager::render_text_mip_chain`] rasterizes by
  /// it, so the bitmap chain always matches the texture's levels. A `0` dimension is treated as
  /// `1`.
  pub fn mip_dimensions( width : u32, height : u32 ) -> impl Iterator< Item = ( u32, u32 ) >
  {
    std::iter::successors
    (
      Some( ( width.max( 1 ), height.max( 1 ) ) ),
      | &( w, h ) | ( w > 1 || h > 1 ).then( || ( ( w / 2 ).max( 1 ), ( h / 2 ).max( 1 ) ) ),
    )
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
    ///
    /// # Errors
    ///
    /// Returns [`gl::WebglError::Other`] if `width` or `height` is `0` or larger than
    /// [`Self::max_size`]: WebGL reports an oversized `texImage2D` only through `getError`, so
    /// without this check the texture would silently stay incomplete and sample black. Returns
    /// [`gl::WebglError::FailedToAllocateResource`] if the texture or the storage of one of its
    /// mip levels can't be allocated.
    pub fn new( gl : &GL, width : u32, height : u32, uv_position : u32 ) -> Result< Self, gl::WebglError >
    {
      if width == 0 || height == 0
      {
        return Err( gl::WebglError::Other( "engraving texture must be at least 1x1" ) );
      }
      let max = Self::max_size( gl );
      if width > max || height > max
      {
        return Err( gl::WebglError::Other( "engraving texture is larger than the context's MAX_TEXTURE_SIZE" ) );
      }

      let source = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "engraving texture" ) )?;
      gl.bind_texture( gl::TEXTURE_2D, Some( &source ) );

      let mut levels = 0;
      for ( level, ( w, h ) ) in mip_dimensions( width, height ).enumerate()
      {
        levels += 1;
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
      }

      let sampler = Sampler
      {
        mag_filter : Some( MagFilterMode::Linear ),
        min_filter : Some( MinFilterMode::LinearMipMapLinear ),
        wrap_s : Some( WrappingMode::ClampToEdge ),
        wrap_t : Some( WrappingMode::ClampToEdge ),
        ..Default::default()
      };

      // This texture is a fresh GPU allocation made above, solely owned by this
      // `EngravingTexture`, so it is built with `Texture::owning`: dropping the last clone of
      // it frees the GPU texture (see `TextureOwner`).
      let texture = Texture::owning( gl, gl::TEXTURE_2D, source, sampler );
      let info = TextureInfo { texture : Rc::new( RefCell::new( texture ) ), uv_position };

      Ok( Self { info, width, height, levels } )
    }

    /// Uploads a freshly rasterized mip chain into the existing GPU allocation via
    /// `texSubImage2D` per level, so re-engraving text never reallocates or triggers a
    /// texture-completeness re-check.
    ///
    /// `bitmaps` must have exactly [`Self::level_count`] entries, ordered the same way
    /// [`crate::webgl::engraving::EngravingCanvasManager::render_text_mip_chain`] returns them
    /// (index `0` is the full `width` x `height` level).
    ///
    /// # Errors
    ///
    /// Returns [`gl::WebglError::Other`] if `bitmaps` doesn't have exactly one bitmap per mip
    /// level, if a bitmap's size differs from its level's, or if a `texSubImage2D` upload fails.
    pub fn update( &self, gl : &GL, bitmaps : &[ EngravingBitmap ] ) -> Result< (), gl::WebglError >
    {
      if bitmaps.len() as u32 != self.levels
      {
        return Err( gl::WebglError::Other( "engraving bitmap chain does not match the allocated mip level count" ) );
      }

      gl.bind_texture( gl::TEXTURE_2D, self.info.texture.borrow().source.as_ref() );

      for ( level, ( bitmap, ( w, h ) ) ) in bitmaps.iter().zip( mip_dimensions( self.width, self.height ) ).enumerate()
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
      }

      Ok( () )
    }

    /// The largest width or height `gl` accepts for a 2D texture (`MAX_TEXTURE_SIZE`). WebGL2
    /// guarantees at least 2048; many mobile GPUs report 4096. Returns `u32::MAX` if the query
    /// fails (a lost context), leaving the allocation itself to fail.
    #[ must_use ]
    pub fn max_size( gl : &GL ) -> u32
    {
      gl.get_parameter( gl::MAX_TEXTURE_SIZE ).ok()
      .and_then( | v | v.as_f64() )
      .map_or( u32::MAX, | v | v as u32 )
    }

    /// Width in pixels of mip level 0 of the GPU allocation.
    #[ must_use ]
    pub fn width( &self ) -> u32 { self.width }

    /// Height in pixels of mip level 0 of the GPU allocation.
    #[ must_use ]
    pub fn height( &self ) -> u32 { self.height }

    /// Number of mip levels [`Self::update`] expects a bitmap for, from level 0 (`width` x
    /// `height`) down to the final `1` x `1` level.
    #[ must_use ]
    pub fn level_count( &self ) -> u32 { self.levels }

    /// Returns a [`TextureInfo`] handle to this texture, suitable for
    /// `PbrMaterial::engraving_texture_set`. Cloning is cheap (the underlying `Texture` is
    /// `Rc`-shared), and further [`Self::update`] calls remain visible through every clone.
    #[ must_use ]
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
    mip_dimensions,
  };
}
