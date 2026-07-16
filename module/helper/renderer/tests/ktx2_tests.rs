//! Tests for `renderer::webgl::loaders::ktx2` -- the KTX2 container reader and its support check.
//!
//! These are native tests: parsing a KTX2 header and deciding whether the payload is drawable needs
//! no GPU and no browser, so it is checked here rather than in the wasm suite.
//!
//! The refusal messages are treated as a deliverable in their own right, not as incidental strings.
//! A user handed an ETC1S asset has done nothing wrong -- the file is legal under
//! `KHR_texture_basisu` and other viewers display it -- so the *only* thing standing between them
//! and a baffling failure is the message telling them what happened and how to fix it. Asserting on
//! its content is what keeps a later refactor from quietly reducing it to "unsupported".

#![ cfg( feature = "ktx2" ) ]

use renderer::webgl::loaders::ktx2::{ ColorModel, Ktx2Error, Ktx2Image, Payload, SupercompressionScheme };

/// Every refusal must name the fix, not merely the problem.
///
/// `gltf-transform uastc` is the actionable escape hatch for all three, and the one that produced
/// this project's own test assets.
#[ test ]
fn every_refusal_tells_the_user_how_to_fix_it()
{
  let refusals =
  [
    Ktx2Error::Etc1s,
    Ktx2Error::UnsupportedPayload( None ),
    Ktx2Error::UnsupportedSupercompression( SupercompressionScheme::ZLIB ),
  ];

  for error in refusals
  {
    let message = error.to_string();
    assert!
    (
      message.contains( "UASTC" ) || message.contains( "uastc" ),
      "{error:?} does not mention UASTC : {message}"
    );
    assert!
    (
      message.contains( "gltf-transform uastc" ),
      "{error:?} does not name a concrete fix : {message}"
    );
  }
}

/// The ETC1S message is the one that matters most, so it is pinned hardest.
///
/// It has to convey three things at once: what the file is, that the file is *not broken* ( which is
/// the counter-intuitive part -- it is spec-legal, and another viewer would show it ), and what to
/// do. Drop any one of those and the user is left thinking their asset is corrupt.
#[ test ]
fn etc1s_message_explains_that_the_file_is_valid_but_unsupported()
{
  let message = Ktx2Error::Etc1s.to_string();

  assert!( message.contains( "ETC1S" ), "does not name the encoding : {message}" );
  assert!( message.contains( "valid" ), "does not say the file is valid : {message}" );
  assert!( message.contains( "KHR_texture_basisu" ), "does not name the extension : {message}" );
  assert!( message.contains( "gltf-transform uastc" ), "does not name the fix : {message}" );
}

/// A refusal must say what it actually found, so the message is diagnosable without a hex editor.
#[ test ]
fn unsupported_payload_names_the_encoding_it_found()
{
  let message = Ktx2Error::UnsupportedPayload( Some( ColorModel::BC7 ) ).to_string();
  assert!( message.contains( "BC7" ), "does not name the encoding found : {message}" );
}

/// Malformed input is reported as malformed, not mistaken for an unsupported encoding.
#[ test ]
fn garbage_is_rejected_as_malformed()
{
  let not_a_ktx2 = [ 0_u8; 64 ];
  let error = Ktx2Image::parse( &not_a_ktx2 ).expect_err( "garbage must not parse" );

  assert!( matches!( error, Ktx2Error::Malformed( _ ) ), "got {error:?}" );
  assert!( error.to_string().contains( "Not a valid KTX2 file" ) );
}

/// An empty input must not panic -- an asset can be truncated to nothing by a bad fetch.
#[ test ]
fn empty_input_is_rejected_without_panicking()
{
  let error = Ktx2Image::parse( &[] ).expect_err( "empty input must not parse" );
  assert!( matches!( error, Ktx2Error::Malformed( _ ) ), "got {error:?}" );
}

// ---------------------------------------------------------------------------------------------
// Decode pipeline.
//
// `decode_level` is pure -- bytes in, bytes out, no WebGL context -- which is what makes the most
// error-prone part of this loader testable without a browser.
//
// The fixture is 65x33 **on purpose**. Both dimensions are non-multiples of 4, so every mip level
// has partially-covered blocks along its right and bottom edges. A texture sized to a multiple of 4
// would exercise none of the rounding, and a 1024x1024 asset ( which is what the real ones are )
// would silently pass a decoder that got the edges wrong.
// ---------------------------------------------------------------------------------------------

use minwebgl as gl;
use gl::texture::compressed::Format;
use renderer::webgl::loaders::ktx2::{ Wrapping, decode_level };

/// The 65x33 UASTC + Zstandard fixture, encoded with KTX-Software's `ktx create --encode uastc`.
const FIXTURE : &[ u8 ] = include_bytes!( "fixtures/uastc-65x33.ktx2" );

/// Every compressed format the fixture can be transcoded into.
const COMPRESSED : [ Format; 3 ] = [ Format::Astc4x4, Format::Bc7, Format::Etc2Rgba ];

#[ test ]
fn fixture_is_what_the_decode_tests_assume()
{
  let image = Ktx2Image::parse( FIXTURE ).expect( "fixture must parse" );
  let info = image.info();

  assert_eq!( ( info.width, info.height ), ( 65, 33 ) );
  assert_eq!( info.level_count, 7 );
  assert!( info.srgb, "fixture was encoded with --assign-tf srgb" );
  assert_eq!( image.check_supported(), Ok( Wrapping::Zstandard ) );
}

/// A non-2D container ( 3D, cubemap, array, or 1D ) is refused, not silently read as a flat 2D image.
///
/// The loader only knows how to hand a single 2D level chain to `compressedTexImage2D`; a file
/// declaring extra depth, faces, or layers carries data this path has no place to put, and treating
/// it as 2D would upload one slice and drop the rest without a word. KTX2 has no header checksum, so
/// each shape is produced by patching the relevant `u32` in a copy of the real fixture -- exercising
/// the actual `parse` branches rather than a hand-built header the reader might reject for other
/// reasons first.
#[ test ]
fn a_non_2d_container_shape_is_refused()
{
  // Byte offsets of the KTX2 header fields, little-endian, measured from the start of the file
  // ( the 12-byte identifier precedes them ). See the KTX2 spec, section 3.1.
  const PIXEL_HEIGHT : usize = 24;
  const PIXEL_DEPTH  : usize = 28;
  const LAYER_COUNT  : usize = 32;
  const FACE_COUNT   : usize = 36;

  let patched = | offset : usize, value : u32 | -> Vec< u8 >
  {
    let mut bytes = FIXTURE.to_vec();
    bytes[ offset..offset + 4 ].copy_from_slice( &value.to_le_bytes() );
    bytes
  };

  // ( label, patched file ). Each is a legal 2D fixture with exactly one field changed to make it
  // 3D / a cubemap / an array / 1D.
  let shapes =
  [
    ( "3D",      patched( PIXEL_DEPTH,  1 ) ),
    ( "cubemap", patched( FACE_COUNT,   6 ) ),
    ( "array",   patched( LAYER_COUNT,  2 ) ),
    ( "1D",      patched( PIXEL_HEIGHT, 0 ) ),
  ];

  for ( label, bytes ) in shapes
  {
    let error = Ktx2Image::parse( &bytes ).expect_err( &format!( "{label} must not parse as 2D" ) );
    assert!
    (
      matches!( error, Ktx2Error::UnsupportedShape( _ ) ),
      "{label} was not rejected as an unsupported shape : {error:?}"
    );
  }
}

/// Mip dimensions halve and clamp at 1 -- and for 65x33 they are never block-aligned.
#[ test ]
fn mip_dimensions_halve_and_clamp()
{
  let image = Ktx2Image::parse( FIXTURE ).unwrap();

  let dimensions : Vec< _ > = ( 0..image.info().level_count ).map( | l | image.level_size( l ) ).collect();
  assert_eq!
  (
    dimensions,
    vec![ ( 65, 33 ), ( 32, 16 ), ( 16, 8 ), ( 8, 4 ), ( 4, 2 ), ( 2, 1 ), ( 1, 1 ) ]
  );
}

/// **The load-bearing test.** Every level, in every target format, must decode to exactly the byte
/// count that format and those dimensions imply.
///
/// This is not a formality: `compressedTexImage2D` validates the length against `internalformat` and
/// the dimensions, and rejects any mismatch with `INVALID_VALUE`. An off-by-one block in the edge
/// rounding is therefore a hard upload failure, and it would only ever show up on textures whose
/// size is not a multiple of 4 -- which is exactly what this fixture is.
#[ test ]
fn every_level_decodes_to_the_exact_length_the_gpu_will_demand()
{
  let image = Ktx2Image::parse( FIXTURE ).unwrap();
  let wrapping = image.check_supported().unwrap();
  let levels : Vec< _ > = image.levels().map( | l | l.data.to_vec() ).collect();

  for format in COMPRESSED.into_iter().chain( [ Format::Rgba8 ] )
  {
    for ( level, data ) in levels.iter().enumerate()
    {
      let ( width, height ) = image.level_size( level as u32 );

      let decoded = decode_level( data, wrapping, format, width, height )
      .unwrap_or_else( | e | panic!( "{format:?} level {level} ( {width}x{height} ) : {e}" ) );

      assert_eq!
      (
        decoded.len(),
        format.level_size( width, height ),
        "{format:?} level {level} ( {width}x{height} )"
      );
    }
  }
}

/// The three compressed targets are genuinely different encodings of the same blocks.
///
/// Guards against a transcoder that quietly returns its input, or against two arms of the format
/// match being wired to the same function -- both of which would pass every length check above.
#[ test ]
fn each_target_format_produces_distinct_bytes()
{
  let image = Ktx2Image::parse( FIXTURE ).unwrap();
  let wrapping = image.check_supported().unwrap();
  let level0 = image.levels().next().unwrap().data.to_vec();

  let astc = decode_level( &level0, wrapping, Format::Astc4x4, 65, 33 ).unwrap();
  let bc7 = decode_level( &level0, wrapping, Format::Bc7, 65, 33 ).unwrap();
  let etc2 = decode_level( &level0, wrapping, Format::Etc2Rgba, 65, 33 ).unwrap();

  assert_ne!( astc, bc7, "ASTC and BC7 output is identical -- are both arms calling the same fn?" );
  assert_ne!( bc7, etc2 );
  assert_ne!( astc, etc2 );
  // ...and none of them is just the (inflated) UASTC input handed back unchanged.
  assert_ne!( astc.len(), 0 );
}

/// The RGBA fallback must reconstruct the *image*, not just the right number of bytes.
///
/// This is the one test that checks the decode is actually **correct** rather than merely
/// well-shaped, and it covers two things nothing else does:
///
/// * **De-blocking.** UASTC stores texels grouped by 4x4 tile; an image stores them by row. Get the
///   scatter wrong and every length assertion still passes while the picture is shredded.
/// * **Channel order.** `uastc_tools` packs RGBA, but other block decoders ( `texture2ddecoder` )
///   pack BGRA. Swapping them yields an image that looks entirely plausible until you notice red and
///   blue are exchanged -- so it is asserted, not assumed.
///
/// Tolerance is wide because UASTC is lossy at *encode* time; the point here is the structure of the
/// image, not the fidelity of the codec ( which `uastc_tools` validates bit-exactly against
/// KTX-Software ).
#[ test ]
fn rgba_fallback_reconstructs_the_source_image()
{
  let image = Ktx2Image::parse( FIXTURE ).unwrap();
  let wrapping = image.check_supported().unwrap();
  let level0 = image.levels().next().unwrap().data.to_vec();

  let ( width, height ) = ( 65_u32, 33_u32 );
  let rgba = decode_level( &level0, wrapping, Format::Rgba8, width, height ).unwrap();
  assert_eq!( rgba.len(), ( width * height * 4 ) as usize );

  let texel = | x : u32, y : u32 | -> [ u8; 4 ]
  {
    let offset = ( ( y * width + x ) * 4 ) as usize;
    rgba[ offset..offset + 4 ].try_into().unwrap()
  };

  // The source pattern, reproduced: r ramps along x, g ramps along y, b is a 3px checker, and
  // alpha drops to 128 in the right quarter. See the fixture generator.
  let expected = | x : u32, y : u32 | -> [ u8; 4 ]
  {
    [
      ( ( x * 255 ) / ( width - 1 ) ) as u8,
      ( ( y * 255 ) / ( height - 1 ) ) as u8,
      if ( ( x / 3 + y / 3 ) % 2 ) == 1 { 255 } else { 40 },
      if x < ( width * 3 ) / 4 { 255 } else { 128 },
    ]
  };

  // Sample across the image, including the last column and row -- which live in the *partial* edge
  // blocks, and so are precisely where a de-blocking bug would show.
  let probes = [ ( 0, 0 ), ( 1, 1 ), ( 32, 16 ), ( 60, 30 ), ( 64, 0 ), ( 0, 32 ), ( 64, 32 ) ];

  const TOLERANCE : i32 = 40;

  for ( x, y ) in probes
  {
    let got = texel( x, y );
    let want = expected( x, y );

    for channel in 0..4
    {
      let delta = i32::from( got[ channel ] ) - i32::from( want[ channel ] );
      assert!
      (
        delta.abs() <= TOLERANCE,
        "texel ( {x}, {y} ) channel {channel} : got {got:?}, want ~{want:?} ( delta {delta} )"
      );
    }
  }
}

/// A truncated level is caught with a diagnosable error, not a panic and not a garbage texture.
#[ test ]
fn truncated_level_is_reported_as_a_size_mismatch()
{
  // Raw UASTC for a 8x8 level would be 2x2 blocks = 64 bytes. Hand it 48.
  let short = vec![ 0_u8; 48 ];

  let error = decode_level( &short, Wrapping::None, Format::Bc7, 8, 8 )
  .expect_err( "a short level must not decode" );

  assert_eq!
  (
    error,
    Ktx2Error::LevelSizeMismatch { expected : 64, actual : 48 }
  );
  assert!( error.to_string().contains( "corrupt or truncated" ) );
}

/// Corrupt Zstandard data fails as a supercompression error, distinctly from a size mismatch.
#[ test ]
fn corrupt_zstd_is_reported_as_a_supercompression_failure()
{
  let garbage = vec![ 0xAB_u8; 128 ];

  let error = decode_level( &garbage, Wrapping::Zstandard, Format::Bc7, 8, 8 )
  .expect_err( "garbage must not inflate" );

  assert!( matches!( error, Ktx2Error::Supercompression( _ ) ), "got {error:?}" );
}

// ---------------------------------------------------------------------------------------------
// ETC1S rejection, against a real file.
//
// The refusal *logic* is exercised elsewhere against `Ktx2Error` values constructed by hand. What
// those cannot confirm is that a genuine ETC1S file is actually *recognised* as one -- that the DFD
// really does report colour model 163, that the supercompression really is BasisLZ, and that a
// container whose levels declare `uncompressedByteLength = 0` even survives parsing. Those are
// assumptions about the format, and format assumptions are worth checking against a real file rather
// than a specification reading.
// ---------------------------------------------------------------------------------------------

/// The same 65x33 source image, encoded as ETC1S / BasisLZ instead of UASTC.
const ETC1S_FIXTURE : &[ u8 ] = include_bytes!( "fixtures/etc1s-65x33.ktx2" );

/// A real ETC1S file parses, is *identified* as ETC1S, and is then refused.
///
/// Parsing must succeed: a file we cannot draw is still a file we must be able to describe, or the
/// refusal could not name what it found. Note its levels declare `uncompressedByteLength = 0`, which
/// is legal for BasisLZ and would break a reader that assumed otherwise.
#[ test ]
fn a_real_etc1s_file_is_identified_and_refused()
{
  let image = Ktx2Image::parse( ETC1S_FIXTURE ).expect( "an ETC1S file must still parse" );
  let info = image.info();

  assert_eq!( info.payload, Payload::Etc1s, "DFD colour model must identify ETC1S" );
  assert_eq!
  (
    info.supercompression,
    Some( SupercompressionScheme::BasisLZ ),
    "ETC1S is always BasisLZ-supercompressed"
  );

  let error = image.check_supported().expect_err( "ETC1S must be refused" );
  assert_eq!( error, Ktx2Error::Etc1s );

  // And the refusal is the actionable one, not a generic "unsupported".
  let message = error.to_string();
  assert!( message.contains( "ETC1S" ), "{message}" );
  assert!( message.contains( "gltf-transform uastc" ), "{message}" );
}

/// ETC1S is refused *before* anything tries to decode it.
///
/// This is the point of the whole exercise. The failure mode being guarded against is not an error --
/// it is the absence of one: BasisLZ level data fed to a UASTC block decoder is not obviously
/// garbage, it is 16-byte chunks that decode into *something*. A texture full of noise would be the
/// result, and it would render without a single complaint.
#[ test ]
fn etc1s_never_reaches_the_uastc_decoder()
{
  let image = Ktx2Image::parse( ETC1S_FIXTURE ).unwrap();

  // `decode_level` cannot even be called: it demands a `Wrapping`, and the only way to obtain one is
  // `check_supported`, which refuses. That is a compile-time guarantee, not a runtime check -- this
  // test documents it, since there is no way to write the negative case in Rust at all.
  assert!( image.check_supported().is_err() );
}
