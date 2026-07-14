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

use renderer::webgl::loaders::ktx2::{ ColorModel, Ktx2Error, Ktx2Image, SupercompressionScheme };

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
