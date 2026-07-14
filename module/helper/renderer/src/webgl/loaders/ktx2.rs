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
//! Zstandard or no supercompression. Everything else is out of scope, and this module is careful to
//! *name* what it found rather than fail vaguely.

mod private
{
  use minwebgl as gl;
  use std::borrow::Cow;

  // The extern crate and this module share a name. Anchor the path at the crate root so it cannot
  // resolve to this module instead.
  use ::ktx2::{ Reader, TransferFunction };

  // Re-exported, not merely imported: both appear in this module's *public* API -- `ColorModel` in
  // `Payload::Unsupported`, `SupercompressionScheme` in `Info` and `Ktx2Error` -- so without this a
  // caller could receive one of these values and have no way to name its type in order to match on
  // it.
  pub use ::ktx2::{ ColorModel, SupercompressionScheme };

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
    /// Anything else -- a KTX2 file carrying an already-GPU-ready encoding ( BC7, ASTC, ETC2 ), or
    /// plain uncompressed texels.
    ///
    /// The raw color model is kept so the error can say what it actually was.
    ///
    /// # Why these are rejected, when uploading them would be *less* work
    ///
    /// This is the obvious objection, and it deserves an answer: a BC7 KTX2 holds blocks the GPU can
    /// sample directly, so handling it would mean *skipping* the transcode rather than adding
    /// anything. It looks like UASTC minus a step.
    ///
    /// It is not. It is a **device-locked** asset, and that is the one property UASTC exists to
    /// avoid. A UASTC file feeds every GPU -- ASTC on mobile, BC7 on desktop, ETC2 on older mobile,
    /// RGBA when a device has none of them. A BC7 file works only where BPTC does. Worse, there is no
    /// fallback available to us even in principle: `uastc_tools` is an encoder *into* BC7 / ASTC /
    /// ETC2, not a decoder *out of* them -- every entry point it exposes takes UASTC input. Handed a
    /// BC7 file on a device without BPTC, this renderer could not produce a single pixel from it
    /// without a software BC7 decoder it does not have and would have to take a new dependency for.
    /// The step saved on desktop is paid for with a decode path on mobile.
    ///
    /// And under `KHR_texture_basisu` such a file is out of spec regardless, so accepting one here
    /// would mean quietly blessing a non-conformant asset through the very code path meant to
    /// implement that extension.
    ///
    /// Supporting GPU-ready KTX2 is a reasonable thing to want -- but it belongs in a *sibling*
    /// loader keyed on the header's `vkFormat` ( which such files set, and which Basis files leave
    /// as `VK_FORMAT_UNDEFINED` ), with its own capability check and its own hard failure when the
    /// device lacks the format. It is not a widening of this one.
    Unsupported( Option< ColorModel > ),
  }

  /// How each mip level's bytes are wrapped, narrowed to the schemes this loader can undo.
  ///
  /// This is the *positive* result of [ `Ktx2Image::check_supported` ], and exists so that the
  /// decode step cannot be reached without having passed that check: there is no way to obtain a
  /// `Wrapping` for an ETC1S or ZLIB file, so no way to start decoding one. `Info::supercompression`
  /// keeps the file's raw claim; this is what we agreed to do about it.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum Wrapping
  {
    /// Level data is raw UASTC blocks, ready to transcode.
    None,
    /// Level data is Zstandard-compressed UASTC blocks, and must be inflated first.
    ///
    /// This is what `gltf-transform uastc` emits by default, so it is the common case, not the
    /// exotic one.
    Zstandard,
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

    /// The texture is ETC1S / BasisLZ encoded, which this renderer cannot decode.
    ///
    /// This is the error that most needs to be *loud*. ETC1S is fully legal under
    /// `KHR_texture_basisu` -- a file another viewer displays without complaint -- so a user who hits
    /// it has done nothing wrong and has no reason to suspect their asset. It is also the one case
    /// where a half-hearted implementation could plausibly produce *something* on screen and call it
    /// a day. Failing with an actionable message beats rendering garbage.
    Etc1s,

    /// The texture is neither UASTC nor ETC1S -- see [ `Payload::Unsupported` ] for why a
    /// GPU-ready KTX2 is out of scope rather than merely unimplemented.
    UnsupportedPayload( Option< ColorModel > ),

    /// The level data is wrapped in a supercompression scheme this loader cannot undo.
    ///
    /// `KHR_texture_basisu` allows only Zstandard or none for a UASTC payload, so in practice this
    /// means a file that is out of spec ( ZLIB, or a vendor scheme ).
    UnsupportedSupercompression( SupercompressionScheme ),

    /// Zstandard decompression of a mip level failed -- the level data is corrupt or truncated.
    Supercompression( String ),

    /// A UASTC block failed to decode. The level data is corrupt.
    Transcode( String ),

    /// A mip level is not the size its dimensions imply.
    ///
    /// Caught here rather than at upload because the symptom otherwise is a bare `INVALID_VALUE`
    /// from `compressedTexImage2D` with no indication of which level or by how much.
    LevelSizeMismatch
    {
      /// Bytes the level's dimensions imply it must contain.
      expected : usize,
      /// Bytes it actually contains.
      actual : usize,
    },

    /// The GPU rejected an upload.
    Upload( String ),
  }

  impl core::fmt::Display for Ktx2Error
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      match self
      {
        Self::Malformed( detail ) => write!( f, "Not a valid KTX2 file : {detail}" ),
        Self::UnsupportedShape( detail ) => write!( f, "Unsupported KTX2 texture shape : {detail}" ),

        Self::Etc1s => write!
        (
          f,
          "This KTX2 texture is ETC1S / BasisLZ encoded. It is a valid KHR_texture_basisu asset, \
           but this renderer decodes UASTC only. Re-encode the texture as UASTC -- for example \
           `gltf-transform uastc in.glb out.glb` -- and load it again."
        ),

        Self::UnsupportedPayload( color_model ) => write!
        (
          f,
          "This KTX2 texture is encoded as {color_model:?}, which KHR_texture_basisu does not permit \
           ( it allows only UASTC or ETC1S ). Re-encode it as UASTC -- for example \
           `gltf-transform uastc in.glb out.glb`."
        ),

        Self::UnsupportedSupercompression( scheme ) => write!
        (
          f,
          "This KTX2 texture uses {scheme:?} supercompression, which this renderer cannot undo. \
           KHR_texture_basisu allows only Zstandard, or none, for a UASTC payload. Re-encode it -- \
           for example `gltf-transform uastc in.glb out.glb`."
        ),

        Self::Supercompression( detail ) =>
          write!( f, "Failed to decompress a Zstandard-supercompressed KTX2 mip level : {detail}" ),

        Self::Transcode( detail ) => write!( f, "Failed to decode a UASTC block : {detail}" ),

        Self::LevelSizeMismatch { expected, actual } => write!
        (
          f,
          "KTX2 mip level is {actual} bytes, but its dimensions imply {expected}. The file is \
           corrupt or truncated."
        ),

        Self::Upload( detail ) => write!( f, "The GPU rejected a KTX2 texture upload : {detail}" ),

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

  // Hand-written because `ktx2::Reader` is not `Debug`, and dumping the raw file bytes would be
  // useless anyway. The `Info` is the entire interesting content.
  impl core::fmt::Debug for Ktx2Image< '_ >
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "Ktx2Image" ).field( "info", &self.info ).finish_non_exhaustive()
    }
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

    /// Checks that this file is one the renderer can actually draw, and reports how its levels are
    /// wrapped.
    ///
    /// [ `Ktx2Image::parse` ] deliberately succeeds for files it cannot draw, reporting their
    /// encoding through [ `Info::payload` ]. This is where that classification becomes a verdict. It
    /// must be called before any attempt to decode level data -- the whole point is that an
    /// undrawable file fails *here*, with a message naming the problem and the fix, rather than
    /// downstream as a nonsense block decode or, worse, as a texture full of garbage.
    ///
    /// # Errors
    ///
    /// Fails for an ETC1S payload, for any other encoding, and for a supercompression scheme this
    /// loader cannot undo.
    pub fn check_supported( &self ) -> Result< Wrapping, Ktx2Error >
    {
      // BasisLZ is tested first, and independently of the DFD, because it is the stronger statement
      // about how the level bytes are *actually* coded: it is not a generic compression wrapper but
      // an inseparable part of ETC1S transcoding, and it never wraps anything else. So a file whose
      // DFD claims UASTC while its header says BasisLZ is self-contradictory -- and its bytes are
      // unreadable to us on either reading, which makes the ETC1S message the useful one to give.
      if self.info.supercompression == Some( SupercompressionScheme::BasisLZ )
      {
        return Err( Ktx2Error::Etc1s );
      }

      match self.info.payload
      {
        Payload::Etc1s => return Err( Ktx2Error::Etc1s ),
        Payload::Unsupported( color_model ) => return Err( Ktx2Error::UnsupportedPayload( color_model ) ),
        Payload::Uastc => {},
      }

      match self.info.supercompression
      {
        None => Ok( Wrapping::None ),
        Some( SupercompressionScheme::Zstandard ) => Ok( Wrapping::Zstandard ),
        Some( other ) => Err( Ktx2Error::UnsupportedSupercompression( other ) ),
      }
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

  /// Bytes in one UASTC block. UASTC is a 4x4, 128-bit encoding, always.
  const UASTC_BLOCK_BYTES : usize = 16;

  /// Undoes Zstandard supercompression.
  fn inflate( data : &[ u8 ] ) -> Result< Vec< u8 >, Ktx2Error >
  {
    use std::io::Read as _;

    let mut decoder = ruzstd::decoding::StreamingDecoder::new( data )
    .map_err( | e | Ktx2Error::Supercompression( format!( "{e:?}" ) ) )?;

    let mut out = Vec::new();
    decoder.read_to_end( &mut out )
    .map_err( | e | Ktx2Error::Supercompression( format!( "{e:?}" ) ) )?;

    Ok( out )
  }

  /// Decodes one mip level into bytes ready to hand to the GPU in `format`.
  ///
  /// This is the whole decode pipeline, and it is deliberately **pure**: bytes in, bytes out, no
  /// WebGL context anywhere. That is what lets it be tested natively, without a browser -- and the
  /// transcode is the part most worth testing, since a wrong block here is a wrong pixel forever.
  ///
  /// The two compressions are undone in order:
  ///
  /// 1. **Zstandard**, if [ `Wrapping::Zstandard` ] -- generic and lossless, yielding UASTC blocks.
  /// 2. **UASTC**, always -- transcoded into `format`, because no GPU can sample UASTC itself.
  ///
  /// For the compressed targets this is a 1:1 block map: block *n* of the output is block *n* of the
  /// input, transcoded. Both UASTC and all three targets store blocks in row-major order with a
  /// 16-byte block, so no reordering is needed and the output length is the input length.
  ///
  /// # Errors
  ///
  /// Fails if Zstandard decompression fails, if the level is not the size its dimensions imply, or
  /// if a UASTC block is malformed.
  pub fn decode_level
  (
    data : &[ u8 ],
    wrapping : Wrapping,
    format : gl::texture::compressed::Format,
    width : u32,
    height : u32,
  ) -> Result< Vec< u8 >, Ktx2Error >
  {
    use gl::texture::compressed::Format;

    // Borrowed when there is nothing to undo -- an uncompressed level needs no copy.
    let uastc : Cow< '_, [ u8 ] > = match wrapping
    {
      Wrapping::None => Cow::Borrowed( data ),
      Wrapping::Zstandard => Cow::Owned( inflate( data )? ),
    };

    let blocks_x = ( width.div_ceil( 4 ) ) as usize;
    let blocks_y = ( height.div_ceil( 4 ) ) as usize;

    // The level's dimensions, not the file's own length field, are the authority: this is exactly
    // the arithmetic `compressedTexImage2D` will apply, so checking it here turns a bare
    // `INVALID_VALUE` at upload into an error that says which level and by how much.
    let expected = blocks_x * blocks_y * UASTC_BLOCK_BYTES;
    if uastc.len() != expected
    {
      return Err( Ktx2Error::LevelSizeMismatch { expected, actual : uastc.len() } );
    }

    // All three compressed targets share one signature -- 4x4 in, 16 bytes out -- so the format is
    // resolved once, here, rather than re-matched for every one of the ~65k blocks in a 1024x1024
    // level.
    let transcode : fn( [ u8; UASTC_BLOCK_BYTES ] ) -> core::result::Result< [ u8; 16 ], String > =
    match format
    {
      Format::Astc4x4 => uastc_tools::transcode_uastc_block_to_astc,
      Format::Bc7 => uastc_tools::transcode_uastc_block_to_bc7,
      Format::Etc2Rgba => uastc_tools::transcode_uastc_block_to_etc2,
      // The uncompressed fallback is not a block format, so it takes the other path entirely: its
      // texels must be scattered out of block order into image order.
      Format::Rgba8 => return unpack_to_rgba( &uastc, blocks_x, width, height ),
    };

    let mut out = Vec::with_capacity( expected );
    for block in uastc.chunks_exact( UASTC_BLOCK_BYTES )
    {
      // `chunks_exact` guarantees the length, so this cannot fail.
      let block : [ u8; UASTC_BLOCK_BYTES ] = block.try_into().unwrap_or( [ 0; UASTC_BLOCK_BYTES ] );
      let transcoded = transcode( block ).map_err( Ktx2Error::Transcode )?;
      out.extend_from_slice( &transcoded );
    }

    Ok( out )
  }

  /// Decodes UASTC blocks to a plain RGBA8 image, for devices that support no compressed format.
  ///
  /// Unlike a block-to-block transcode this has to **de-block**: UASTC stores texels grouped by 4x4
  /// tile, an image stores them by row. Edge tiles of a texture whose size is not a multiple of 4
  /// are only partly covered, and their surplus texels are dropped rather than written past the end
  /// of a row -- which is what makes the bounds checks below load-bearing rather than defensive.
  fn unpack_to_rgba
  (
    uastc : &[ u8 ],
    blocks_x : usize,
    width : u32,
    height : u32,
  ) -> Result< Vec< u8 >, Ktx2Error >
  {
    let width = width as usize;
    let height = height as usize;
    let mut image = vec![ 0_u8; width * height * 4 ];

    for ( index, block ) in uastc.chunks_exact( UASTC_BLOCK_BYTES ).enumerate()
    {
      let block : [ u8; UASTC_BLOCK_BYTES ] = block.try_into().unwrap_or( [ 0; UASTC_BLOCK_BYTES ] );
      let texels = uastc_tools::unpack_uastc_block_to_rgba( block ).map_err( Ktx2Error::Transcode )?;

      let origin_x = ( index % blocks_x ) * 4;
      let origin_y = ( index / blocks_x ) * 4;

      for row in 0..4
      {
        let y = origin_y + row;
        if y >= height
        {
          break;
        }

        for column in 0..4
        {
          let x = origin_x + column;
          if x >= width
          {
            break;
          }

          // `uastc_tools` packs each texel as little-endian `[ r, g, b, a ]`, so the bytes come back
          // in exactly the order WebGL wants for `RGBA` / `UNSIGNED_BYTE`. This is worth stating
          // because it is not universal -- `texture2ddecoder`, for one, packs BGRA, and swapping the
          // two produces an image that looks plausible until you notice red and blue are exchanged.
          let texel = texels[ row * 4 + column ].to_le_bytes();
          let offset = ( y * width + x ) * 4;
          image[ offset..offset + 4 ].copy_from_slice( &texel );
        }
      }
    }

    Ok( image )
  }

  /// Decodes a KTX2 file and uploads it as a complete `TEXTURE_2D`, mip chain and all.
  ///
  /// This is the entry point the glTF loader calls. `format` should come from
  /// [ `gl::texture::compressed::Support::best` ] -- what the device can actually sample.
  ///
  /// # Color space
  ///
  /// `color_space` is a parameter rather than a decision made here, but note what this renderer
  /// passes: **`Linear`**, even for a texture whose KTX2 header declares itself sRGB
  /// ( [ `Info::srgb` ] ). That is not an oversight. The fragment shader applies `SrgbToLinear` to
  /// base-color, specular and emissive samples itself, so asking the sampler to linearise as well
  /// would decode twice and visibly darken the image.
  ///
  /// # Mip levels
  ///
  /// The file's **own** mip chain is uploaded, level by level. `generate_mipmap` is never called,
  /// for two independent reasons, either of which alone would settle it:
  ///
  /// * It would not work. `generateMipmap` on a compressed internal format is an
  ///   `INVALID_OPERATION` -- the GPU cannot downsample block-compressed data, because doing so
  ///   would mean decoding, filtering and re-encoding it.
  /// * It would be worse if it did. The levels in the file were downsampled from the *source*
  ///   image and only then compressed. A mip chain derived from already-compressed level 0 would
  ///   compound its error at every step.
  ///
  /// `TEXTURE_MAX_LEVEL` is set to the last level the file actually provides. This is what makes the
  /// texture **complete**: without it, a sampler using a mipmap filter would expect a full chain
  /// down to 1x1, not find one, and silently sample black.
  ///
  /// # Cost
  ///
  /// The transcode is synchronous and on the calling thread -- roughly 40 ms natively for a
  /// three-texture model, and some multiple of that in wasm ( T1 ). Acceptable at load time; if it
  /// ever becomes a problem the fix is a worker, not a different decoder.
  ///
  /// # Errors
  ///
  /// Fails if the file is malformed, if its payload is not UASTC ( see
  /// [ `Ktx2Image::check_supported` ] ), or if a level fails to decode or upload.
  pub fn load_into_texture
  (
    gl : &gl::WebGl2RenderingContext,
    bytes : &[ u8 ],
    format : gl::texture::compressed::Format,
    color_space : gl::texture::compressed::ColorSpace,
  ) -> Result< gl::web_sys::WebGlTexture, Ktx2Error >
  {
    use gl::GL;

    let image = Ktx2Image::parse( bytes )?;
    let wrapping = image.check_supported()?;
    let level_count = image.info().level_count;

    let texture = gl.create_texture()
    .ok_or_else( || Ktx2Error::Upload( "failed to create a texture object".to_string() ) )?;

    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );

    // Both of these make `compressedTexImage2D` fail with `INVALID_OPERATION` if they are left set,
    // and either could have been left set by an earlier upload on this context. Neither is
    // meaningful for us anyway: block-compressed data *cannot* be flipped or premultiplied without
    // decoding it first. ( glTF wants no flip regardless -- its UV origin is the top-left, which is
    // exactly what an unflipped upload gives. )
    gl.pixel_storei( GL::UNPACK_FLIP_Y_WEBGL, 0 );
    gl.pixel_storei( GL::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0 );

    // Drain any error left over from earlier, unrelated calls, so that the check after the uploads
    // can only be reporting on us.
    while gl.get_error() != GL::NO_ERROR {}

    for ( level, data ) in image.levels().enumerate()
    {
      let level = level as u32;
      let ( width, height ) = image.level_size( level );

      let decoded = decode_level( data.data, wrapping, format, width, height )?;
      upload_level( gl, format, color_space, level, width, height, &decoded )?;
    }

    // `compressedTexImage2D` has no return value and web-sys does not mark it `catch`, so a rejected
    // upload is **silent**: the texture is simply left undefined and samples black, with nothing to
    // distinguish that from a legitimately black asset. The most likely causes are all real
    // possibilities here -- an `internalformat` whose extension was never enabled ( `INVALID_ENUM` ),
    // or a byte count that disagrees with the dimensions ( `INVALID_VALUE` ) -- so the error flag is
    // read once, after the whole chain. One synchronous round-trip per texture at load time is a
    // small price for not shipping a silent failure mode.
    let error = gl.get_error();
    if error != GL::NO_ERROR
    {
      gl.delete_texture( Some( &texture ) );
      return Err( Ktx2Error::Upload( format!
      (
        "WebGL error 0x{error:04X} while uploading a {format:?} texture. The device most likely does \
         not support that format -- was `Support::query` used to choose it?"
      ) ) );
    }

    // The file's chain is authoritative, and it may stop short of 1x1. Telling the sampler where it
    // ends is what makes the texture mipmap-complete; leaving `MAX_LEVEL` at its default of 1000
    // would have a mipmap-filtered sampler look for levels that were never uploaded and render
    // black.
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_BASE_LEVEL, 0 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAX_LEVEL, ( level_count - 1 ) as i32 );

    // Sensible defaults, which the glTF sampler is free to overwrite afterwards. A file with a
    // single level gets a non-mipmap filter: legal either way once `MAX_LEVEL` is 0, but there is no
    // point asking for trilinear filtering across a chain of one.
    let min_filter = if level_count > 1 { GL::LINEAR_MIPMAP_LINEAR } else { GL::LINEAR };
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, min_filter as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );

    Ok( texture )
  }

  /// Uploads one decoded mip level into the currently-bound `TEXTURE_2D`.
  ///
  /// `bytes` must have come from [ `decode_level` ] with the same `format`, `width` and `height`.
  ///
  /// # Errors
  ///
  /// Fails if `bytes` is not the length `format` and the dimensions imply, or if the GPU rejects the
  /// upload.
  pub fn upload_level
  (
    gl : &gl::WebGl2RenderingContext,
    format : gl::texture::compressed::Format,
    color_space : gl::texture::compressed::ColorSpace,
    level : u32,
    width : u32,
    height : u32,
    bytes : &[ u8 ],
  ) -> Result< (), Ktx2Error >
  {
    use gl::GL;

    // WebGL validates this itself and answers with a bare `INVALID_VALUE` -- which, for a
    // compressed upload, is reported asynchronously and names neither the level nor the size. Check
    // it here so the error can.
    let expected = format.level_size( width, height );
    if bytes.len() != expected
    {
      return Err( Ktx2Error::LevelSizeMismatch { expected, actual : bytes.len() } );
    }

    let internal_format = format.internal_format( color_space );

    if format.is_compressed()
    {
      gl.compressed_tex_image_2d_with_u8_array
      (
        GL::TEXTURE_2D,
        level as i32,
        internal_format,
        width as i32,
        height as i32,
        0,
        bytes,
      );
    }
    else
    {
      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
      (
        GL::TEXTURE_2D,
        level as i32,
        internal_format as i32,
        width as i32,
        height as i32,
        0,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        Some( bytes ),
      )
      .map_err( | e | Ktx2Error::Upload( format!( "{e:?}" ) ) )?;
    }

    Ok( () )
  }
}

crate::mod_interface!
{
  orphan use
  {
    ColorModel,
    Info,
    Ktx2Error,
    Ktx2Image,
    Payload,
    SupercompressionScheme,
    Wrapping,
    decode_level,
    load_into_texture,
    upload_level,
  };
}
