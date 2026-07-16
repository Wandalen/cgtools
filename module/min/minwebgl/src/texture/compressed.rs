//! Compressed-texture format capability detection.
//!
//! A GPU can only sample a compressed texture in a format its hardware understands, and no
//! single format is universally supported: desktop GPUs speak BC ( S3TC / BPTC ), mobile GPUs
//! speak ETC and ASTC. WebGL exposes each family behind its own extension, so the format an
//! application may upload is a **runtime** property of the context, not a compile-time one.
//!
//! This module answers the only two questions a caller actually has:
//!
//! * [ `Support::query` ] -- which compressed families can *this* context actually use?
//! * [ `Support::best` ]  -- given that, which format should I upload?
//!
//! # Relationship to UASTC
//!
//! The [ `Format` ] set is deliberately narrow: it lists exactly the formats a UASTC
//! ( Basis Universal ) block can be transcoded into, plus an uncompressed fallback. **No GPU
//! can sample UASTC itself** -- it is an intermediate encoding, not a GPU format -- so a KTX2
//! loader must pick a real target here and transcode into it.
//!
//! # Color space
//!
//! [ `Format::internal_format` ] takes an explicit [ `ColorSpace` ] rather than assuming one.
//! Whether a texture's bytes are sRGB-encoded is a property of the *asset* and of what the
//! consuming shader does; a low-level binding cannot know it. Picking wrong is not subtle:
//! uploading as sRGB to a shader that already linearises produces a double decode.

use crate::*;

type GL = web_sys::WebGl2RenderingContext;

/// Whether the texel values are sRGB-encoded, and so should be linearised by the sampler.
///
/// Choose `Linear` if the consuming shader linearises the sampled value itself, and `Srgb` if
/// it expects the hardware to have done it. Doing both double-decodes and visibly darkens the
/// image.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum ColorSpace
{
  /// Texel values are used as sampled.
  Linear,
  /// Texel values are sRGB-encoded; the sampler converts them to linear on read.
  Srgb,
}

/// A GPU texture format that a UASTC block can be transcoded into.
///
/// Every compressed variant here is 4x4-block based with a 16-byte block, which is what makes
/// them reachable from UASTC ( itself a 4x4, 128-bit block encoding ).
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum Format
{
  /// ASTC, 4x4 footprint. Requires `WEBGL_compressed_texture_astc`.
  ///
  /// The preferred target: UASTC is a subset of ASTC 4x4, so the transcode is a bit-level
  /// repack and is **lossless** -- the result decodes identically to the UASTC source.
  Astc4x4,
  /// BC7. Requires `EXT_texture_compression_bptc`.
  ///
  /// The desktop target, and *almost* lossless. UASTC -> BC7 is a genuine re-encode rather
  /// than a repack, but both are 128-bit 4x4 block formats with the same partition / endpoint
  /// / weight structure, so it is cheap and costs no memory relative to ASTC.
  Bc7,
  /// ETC2 RGBA ( with EAC alpha ). Requires `WEBGL_compressed_texture_etc`.
  ///
  /// The older-mobile target, and the lowest-quality of the three: transcoding decodes to
  /// pixels and re-encodes, so unlike the two above it is meaningfully lossy.
  Etc2Rgba,
  /// Uncompressed 8-bit RGBA. Always available.
  ///
  /// The fallback when the context can use none of the above. Correct everywhere, but costs
  /// 4x the VRAM of a 16-byte-block format, so it is a last resort rather than a default.
  Rgba8,
}

impl Format
{
  /// The `internalformat` to pass to `compressedTexImage2D` ( or, for [ `Format::Rgba8` ],
  /// to `texImage2D` ).
  ///
  /// Legal only once the owning extension has been enabled -- which is what [ `Support::query` ]
  /// does. Calling this for a format the context does not support yields a constant WebGL will
  /// reject with `INVALID_ENUM`.
  #[ must_use ]
  pub const fn internal_format( self, color_space : ColorSpace ) -> u32
  {
    match ( self, color_space )
    {
      ( Self::Astc4x4,  ColorSpace::Linear ) => COMPRESSED_RGBA_ASTC_4X4,
      ( Self::Astc4x4,  ColorSpace::Srgb   ) => COMPRESSED_SRGB8_ALPHA8_ASTC_4X4,
      ( Self::Bc7,      ColorSpace::Linear ) => COMPRESSED_RGBA_BPTC_UNORM,
      ( Self::Bc7,      ColorSpace::Srgb   ) => COMPRESSED_SRGB_ALPHA_BPTC_UNORM,
      ( Self::Etc2Rgba, ColorSpace::Linear ) => COMPRESSED_RGBA8_ETC2_EAC,
      ( Self::Etc2Rgba, ColorSpace::Srgb   ) => COMPRESSED_SRGB8_ALPHA8_ETC2_EAC,
      ( Self::Rgba8,    ColorSpace::Linear ) => GL::RGBA8,
      ( Self::Rgba8,    ColorSpace::Srgb   ) => GL::SRGB8_ALPHA8,
    }
  }

  /// The WebGL extension that must be enabled before this format may be uploaded, or `None`
  /// for [ `Format::Rgba8` ], which is core.
  #[ must_use ]
  pub const fn extension_name( self ) -> Option< &'static str >
  {
    match self
    {
      Self::Astc4x4  => Some( ASTC_EXTENSION ),
      Self::Bc7      => Some( BPTC_EXTENSION ),
      Self::Etc2Rgba => Some( ETC_EXTENSION ),
      Self::Rgba8    => None,
    }
  }

  /// Bytes occupied by one 4x4 block, or `None` for the uncompressed [ `Format::Rgba8` ],
  /// which is not block-based.
  #[ must_use ]
  pub const fn block_bytes( self ) -> Option< usize >
  {
    match self
    {
      Self::Astc4x4 | Self::Bc7 | Self::Etc2Rgba => Some( 16 ),
      Self::Rgba8 => None,
    }
  }

  /// Whether this is a block-compressed format, and so must be uploaded with
  /// `compressedTexImage2D` rather than `texImage2D`.
  #[ must_use ]
  pub const fn is_compressed( self ) -> bool
  {
    self.block_bytes().is_some()
  }

  /// Bytes a single mip level of `width` x `height` texels occupies in this format.
  ///
  /// For the block formats the dimensions are rounded **up** to whole blocks, so the small mip
  /// levels of a non-multiple-of-4 texture still cost a full block. This is the size the data
  /// passed to `compressedTexImage2D` must have exactly -- WebGL validates it and rejects a
  /// mismatch with `INVALID_VALUE`.
  #[ must_use ]
  pub const fn level_size( self, width : u32, height : u32 ) -> usize
  {
    match self.block_bytes()
    {
      Some( block_bytes ) =>
      {
        let blocks_x = width.div_ceil( 4 ) as usize;
        let blocks_y = height.div_ceil( 4 ) as usize;
        blocks_x * blocks_y * block_bytes
      },
      None => width as usize * height as usize * 4,
    }
  }
}

/// `WEBGL_compressed_texture_astc` -- ASTC, all footprints ( only 4x4 is reachable from UASTC ).
pub const ASTC_EXTENSION : &str = "WEBGL_compressed_texture_astc";
/// `EXT_texture_compression_bptc` -- BC6H / BC7. Note the `EXT_` prefix, not `WEBGL_`.
pub const BPTC_EXTENSION : &str = "EXT_texture_compression_bptc";
/// `WEBGL_compressed_texture_etc` -- ETC2 / EAC. *Not* `..._etc1`, a different, RGB-only
/// extension.
pub const ETC_EXTENSION : &str = "WEBGL_compressed_texture_etc";
/// `WEBGL_compressed_texture_s3tc` -- BC1 / BC2 / BC3. Queried only to detect driver emulation
/// ( see [ `Support::without_emulated` ] ); there is no UASTC transcode path into it.
pub const S3TC_EXTENSION : &str = "WEBGL_compressed_texture_s3tc";

/// Which compressed-texture families a given WebGL2 context can use.
///
/// Build one with [ `Support::query` ] and keep it: the query is a set of `getExtension` calls
/// and the answer cannot change over a context's lifetime, so re-querying per texture is pure
/// waste.
#[ derive( Debug, Clone, Copy, Default, PartialEq, Eq ) ]
pub struct Support
{
  /// ASTC ( [ `ASTC_EXTENSION` ] ).
  pub astc : bool,
  /// BC6H / BC7 ( [ `BPTC_EXTENSION` ] ).
  pub bptc : bool,
  /// ETC2 / EAC ( [ `ETC_EXTENSION` ] ).
  pub etc2 : bool,
  /// BC1 / BC2 / BC3 ( [ `S3TC_EXTENSION` ] ).
  ///
  /// Never selected by [ `Support::best` ] -- UASTC has no transcode path into S3TC. It is
  /// tracked purely because its presence is a signal in the emulation heuristic of
  /// [ `Support::without_emulated` ].
  pub s3tc : bool,
}

impl Support
{
  /// Queries `gl` for each compressed-texture family, then drops the ones the driver is
  /// probably only pretending to support ( [ `Support::without_emulated` ] ).
  ///
  /// This is the constructor to use. For the unfiltered answer -- what the context *claims* --
  /// see [ `Support::advertised` ].
  ///
  /// Querying also **enables** the extensions it finds: in WebGL, `getExtension` is not a
  /// passive question -- an extension's formats only become legal `internalformat` values once
  /// it has been asked for at least once. So this must run before any compressed upload.
  #[ must_use ]
  pub fn query( gl : &GL ) -> Self
  {
    Self::advertised( gl ).without_emulated( is_mesa_like_platform() )
  }

  /// Every compressed-texture family the context advertises, taken at face value.
  ///
  /// Prefer [ `Support::query` ], which additionally discards formats that are advertised but
  /// software-emulated. This is exposed for diagnostics -- reporting what the driver claimed,
  /// as opposed to what we chose to believe.
  #[ must_use ]
  pub fn advertised( gl : &GL ) -> Self
  {
    /// An extension is supported when `getExtension` returns an object rather than `null`. A
    /// thrown exception ( `Err` ) is read as "not supported", which is the safe direction: it
    /// means the browser could not enable it.
    fn enabled( gl : &GL, name : &str ) -> bool
    {
      gl.get_extension( name ).ok().flatten().is_some()
    }

    Self
    {
      astc : enabled( gl, ASTC_EXTENSION ),
      bptc : enabled( gl, BPTC_EXTENSION ),
      etc2 : enabled( gl, ETC_EXTENSION ),
      s3tc : enabled( gl, S3TC_EXTENSION ),
    }
  }

  /// Discards ASTC and ETC2 if they look software-emulated rather than implemented in hardware.
  ///
  /// # Why this exists
  ///
  /// A WebGL extension is supposed to mean "the GPU can sample this". On desktop Linux, Mesa's
  /// AMD and Intel drivers break that contract: they expose ASTC and ETC on hardware with no
  /// support for either, and then **decompress in software, on the main thread**, inside the
  /// driver. Believing the advertisement there does not merely fail to help -- it is far worse
  /// than uploading uncompressed RGBA, and it stalls the frame.
  ///
  /// Because ASTC is our *first* preference, this is not an edge case we can shrug off: on an
  /// affected machine the naive choice would always be the worst one available. Dropping ASTC
  /// and ETC2 makes [ `Support::best` ] fall through to BC7, which those GPUs really do have.
  ///
  /// # How the guess is made
  ///
  /// A Mesa-like platform ( `mesa_like_platform` ) is only *half* the signal, and on its own it
  /// would be too blunt: a Linux machine with genuinely ASTC-capable hardware exists, and
  /// disabling ASTC there would be a self-inflicted quality loss. The other half is that the
  /// context advertises **all four** families at once -- ASTC *and* ETC2 *and* BPTC *and* S3TC.
  /// No real GPU implements the mobile and the desktop families both, so a context claiming
  /// every one of them is not describing hardware; it is a driver that says yes to everything.
  /// Genuine ASTC hardware advertises ASTC and ETC but not BPTC and S3TC, and so is untouched.
  ///
  /// This is the same two-part heuristic three.js applies in `KTX2Loader`. It is kept pure, and
  /// separate from the platform sniffing in [ `is_mesa_like_platform` ], so that it is testable
  /// and so that an application which knows better -- because it recognises the actual GPU, say
  /// -- can supply its own verdict.
  #[ must_use ]
  pub const fn without_emulated( self, mesa_like_platform : bool ) -> Self
  {
    let advertises_every_family = self.astc && self.etc2 && self.bptc && self.s3tc;

    if mesa_like_platform && advertises_every_family
    {
      Self { astc : false, etc2 : false, ..self }
    }
    else
    {
      self
    }
  }

  /// The best format to transcode UASTC into, given what this context can use.
  ///
  /// The order is by transcode fidelity:
  ///
  /// 1. **ASTC 4x4** -- UASTC is a subset of it, so the transcode is lossless.
  /// 2. **BC7** -- a re-encode, but almost lossless, and into an equally compact 4x4 block.
  /// 3. **ETC2** -- a full decode-and-re-encode, and meaningfully lossy.
  /// 4. **RGBA8** -- always correct, but 4x the memory. Chosen only when nothing else exists.
  ///
  /// This matches the ordering Khronos recommends in the KTX Developer Guide, and the one both
  /// three.js and Bevy implement.
  ///
  /// In practice a context offers at most one of the first three -- desktop exposes BPTC,
  /// mobile exposes ASTC -- so the ordering only decides the rare overlap. It is pure logic
  /// over [ `Support` ], deliberately separated from [ `Support::query` ] so it can be tested
  /// without a GPU.
  #[ must_use ]
  pub const fn best( self ) -> Format
  {
    if self.astc { Format::Astc4x4 }
    else if self.bptc { Format::Bc7 }
    else if self.etc2 { Format::Etc2Rgba }
    else { Format::Rgba8 }
  }
}

/// Whether this is a platform whose drivers are known to advertise compressed formats they do
/// not implement in hardware -- see [ `Support::without_emulated` ] for what that costs, and
/// for the second half of the test, which this function does not perform.
///
/// The signal is desktop Linux, where Mesa is the usual driver. Android is excluded: it reports
/// as Linux but has genuine ETC and ASTC hardware. Returns `false` off the browser, where there
/// is no `navigator` to ask.
#[ must_use ]
pub fn is_mesa_like_platform() -> bool
{
  let Some( navigator ) = web_sys::window().map( | w | w.navigator() )
  else
  {
    return false;
  };

  // `platform` is deprecated but is still the only thing that names the *host*, and it is what
  // three.js keys this same heuristic on. A missing value is read as "not affected".
  let platform = navigator.platform().unwrap_or_default();
  let user_agent = navigator.user_agent().unwrap_or_default();

  platform.contains( "Linux" ) && !user_agent.contains( "Android" )
}
