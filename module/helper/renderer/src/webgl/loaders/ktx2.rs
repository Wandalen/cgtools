//! KTX2 container reading, for the `KHR_texture_basisu` glTF extension.
//!
//! This layer is the *container* half of the KTX2 story: it opens the file, reads the header and
//! the Data Format Descriptor, checks that the payload is something this renderer can actually
//! draw, and hands back the still-encoded mip levels. It deliberately decodes nothing. The
//! decompression and transcoding half follows on top of it.
//!
//! # What a KTX2 file actually contains here
//!
//! A `KHR_texture_basisu` texture is not one encoding but two stacked ones, and separating them is
//! the whole reason this module exists:
//!
//! 1. **Supercompression** -- a generic, lossless byte-level pass over each mip level. For UASTC
//!    this is Zstandard, or nothing at all. Undoing it yields UASTC blocks.
//! 2. **UASTC** -- a texture-specific, 4x4-block encoding. **No GPU can sample it.** It is an
//!    intermediate, chosen because it transcodes cheaply into whatever block format the device
//!    *does* support ( see `minwebgl::texture::compressed` ).
//!
//! The spec permits exactly two payloads: ETC1S with BasisLZ supercompression, or UASTC with
//! Zstandard or no supercompression. Everything else -- including a plain BC7 KTX2, which is a
//! perfectly valid KTX2 file but not a valid `KHR_texture_basisu` one -- is out of scope, and this
//! module is careful to *name* what it found rather than fail vaguely.

mod private
{
  // The extern crate and this module share a name. Anchor the path at the crate root so it cannot
  // resolve to this module instead.
  use ::ktx2::{ ColorModel, Reader, SupercompressionScheme, TransferFunction };

  /// The texture encoding a KTX2 file carries, as reported by its Data Format Descriptor.
  ///
  /// This is a *classification*, not a verdict: reading a file and deciding whether to draw it are
  /// separate concerns, and only the caller knows which of these it is equipped to handle.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum Payload
  {
    /// UASTC -- a 4x4, 128-bit block encoding, and the one this renderer supports.
    ///
    /// Not a GPU format: it must be transcoded to BC7 / ASTC / ETC2 before upload.
    Uastc,
    /// ETC1S -- the other encoding `KHR_texture_basisu` permits, and one this renderer cannot
    /// decode.
    ///
    /// Called out as its own variant rather than lumped in with `Unsupported` precisely because it
    /// is *spec-legal*: a file that is entirely valid and that another viewer would display. That
    /// deserves an error message saying "re-encode as UASTC", not "unsupported color model 163".
    Etc1s,
    /// Anything else -- a KTX2 file carrying some other encoding entirely ( BC7, ASTC, plain RGBA ).
    ///
    /// Valid KTX2, but not valid `KHR_texture_basisu`. The raw color model is kept so the error can
    /// say what it actually was.
    Unsupported( Option< ColorModel > ),
  }

  /// Errors that can arise while reading a KTX2 container.
  ///
  /// `Display` is implemented by hand rather than derived. The `error::typed::Error` derive used
  /// elsewhere in the workspace expands to `thiserror` paths, and `renderer` does not depend on
  /// `thiserror` -- adding a dependency to phrase three messages is a poor trade.
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum Ktx2Error
  {
    /// The bytes are not a well-formed KTX2 file at all.
    Malformed( String ),

    /// The file parses, but describes a texture shape this loader does not handle.
    ///
    /// `KHR_texture_basisu` textures are plain 2D images; a cubemap, array or 3D texture would need
    /// a different upload path entirely, so it is rejected here rather than silently uploading only
    /// its first slice.
    UnsupportedShape( String ),

    /// The file has no Data Format Descriptor, so its encoding cannot be identified.
    ///
    /// The DFD is mandatory in KTX2. Its absence means the file is either truncated or was written
    /// by something badly broken, and guessing the payload from the header alone is not possible.
    MissingDataFormatDescriptor,
  }

  impl core::fmt::Display for Ktx2Error
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      match self
      {
        Self::Malformed( detail ) => write!( f, "Not a valid KTX2 file : {detail}" ),
        Self::UnsupportedShape( detail ) => write!( f, "Unsupported KTX2 texture shape : {detail}" ),
        Self::MissingDataFormatDescriptor =>
          write!( f, "KTX2 file has no basic Data Format Descriptor, so its encoding is unknown" ),
      }
    }
  }

  impl std::error::Error for Ktx2Error {}

  /// What a KTX2 file says about itself: everything needed to decide whether it can be drawn, and
  /// how, without having touched a single texel.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct Info
  {
    /// Width of mip level 0, in texels. Always non-zero.
    pub width : u32,
    /// Height of mip level 0, in texels. Always non-zero -- a 1D texture is rejected at parse.
    pub height : u32,
    /// Number of mip levels actually stored in the file, always at least 1.
    ///
    /// KTX2 encodes "no stored mip chain" as a level count of `0`; that is normalised to `1` here,
    /// so this is a count of levels that really exist and can be uploaded.
    pub level_count : u32,
    /// The texture encoding of the level data.
    pub payload : Payload,
    /// The supercompression wrapped around each level, if any.
    ///
    /// For a UASTC payload this is `Zstandard` or `None`. `BasisLZ` is not supercompression in the
    /// same sense -- it is inseparable from ETC1S transcoding -- and only ever appears alongside
    /// [ `Payload::Etc1s` ].
    pub supercompression : Option< SupercompressionScheme >,
    /// Whether the file declares its texels to be sRGB-encoded.
    ///
    /// Reported, not acted upon. What to do about it is the *renderer's* decision and depends on
    /// what the fragment shader does -- and this renderer linearises in the shader, so it will
    /// deliberately upload these as linear anyway. Getting that backwards double-decodes and
    /// darkens the image, which is exactly why the file's own claim is surfaced rather than
    /// quietly applied here.
    pub srgb : bool,
  }

  /// A parsed KTX2 container: its [ `Info` ], plus access to the still-encoded mip levels.
  ///
  /// Borrows the input bytes rather than copying them. Level data is returned exactly as stored --
  /// supercompressed and transcoded-from-nothing -- because undoing either of those is a separate
  /// step with its own cost.
  pub struct Ktx2Image< 'data >
  {
    reader : Reader< &'data [ u8 ] >,
    info : Info,
  }

  impl< 'data > Ktx2Image< 'data >
  {
    /// Reads the header and Data Format Descriptor of a KTX2 file.
    ///
    /// Succeeds for any well-formed 2D KTX2 file, *including* ones this renderer cannot draw --
    /// their encoding is reported through [ `Info::payload` ] rather than raised as an error. That
    /// split is deliberate: a caller that only wants to inspect a file should not have to catch an
    /// error to find out what is in it.
    ///
    /// # Errors
    ///
    /// Fails if the bytes are not a valid KTX2 file, if it describes a shape other than a single 2D
    /// image, or if it carries no Data Format Descriptor.
    pub fn parse( bytes : &'data [ u8 ] ) -> Result< Self, Ktx2Error >
    {
      let reader = Reader::new( bytes )
      .map_err( | e | Ktx2Error::Malformed( format!( "{e:?}" ) ) )?;

      let header = reader.header();

      // A `KHR_texture_basisu` image is a plain 2D texture. Reject every other shape rather than
      // upload a slice of it and render something subtly wrong.
      if header.pixel_depth > 0
      {
        return Err( Ktx2Error::UnsupportedShape
        (
          format!( "3D texture ( depth {} ); only 2D images are supported", header.pixel_depth )
        ) );
      }
      if header.face_count > 1
      {
        return Err( Ktx2Error::UnsupportedShape
        (
          format!( "cubemap ( {} faces ); only 2D images are supported", header.face_count )
        ) );
      }
      if header.layer_count > 1
      {
        return Err( Ktx2Error::UnsupportedShape
        (
          format!( "texture array ( {} layers ); only 2D images are supported", header.layer_count )
        ) );
      }
      if header.pixel_height == 0
      {
        return Err( Ktx2Error::UnsupportedShape
        (
          "1D texture ( height 0 ); only 2D images are supported".to_string()
        ) );
      }

      // The DFD is the only place the payload encoding is recorded: for a Basis Universal file the
      // header's `format` is deliberately VK_FORMAT_UNDEFINED, because the real format is not
      // decided until transcode time.
      let dfd = reader.basic_dfd().ok_or( Ktx2Error::MissingDataFormatDescriptor )?;

      let payload = match dfd.color_model
      {
        Some( ColorModel::UASTC ) => Payload::Uastc,
        Some( ColorModel::ETC1S ) => Payload::Etc1s,
        other => Payload::Unsupported( other ),
      };

      let info = Info
      {
        width : header.pixel_width,
        height : header.pixel_height,
        // `0` means "no mip chain stored", which still leaves level 0 in the file.
        level_count : header.level_count.max( 1 ),
        payload,
        supercompression : header.supercompression_scheme,
        srgb : dfd.transfer_function == Some( TransferFunction::SRGB ),
      };

      Ok( Self { reader, info } )
    }

    /// What the file says about itself.
    #[ must_use ]
    pub const fn info( &self ) -> &Info
    {
      &self.info
    }

    /// The mip levels, largest first, **exactly as stored** -- still supercompressed, still UASTC.
    ///
    /// Decoding them is the caller's job, and needs [ `Info::supercompression` ] to know what, if
    /// anything, is wrapped around each one.
    pub fn levels( &self ) -> impl ExactSizeIterator< Item = ::ktx2::Level< '_ > > + '_
    {
      self.reader.levels()
    }

    /// Dimensions of mip level `level`, halving each axis per level and clamping at 1.
    ///
    /// KTX2 stores the size of level 0 only; every other level's size is implied. Getting this
    /// wrong is not a subtle bug -- `compressedTexImage2D` validates the byte count against the
    /// dimensions and rejects a mismatch outright.
    #[ must_use ]
    pub fn level_size( &self, level : u32 ) -> ( u32, u32 )
    {
      let width = ( self.info.width >> level ).max( 1 );
      let height = ( self.info.height >> level ).max( 1 );
      ( width, height )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Info,
    Ktx2Error,
    Ktx2Image,
    Payload,
  };
}
