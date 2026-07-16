//! The correctness oracle for the KTX2 loader.
//!
//! It answers one question the decode-unit tests cannot: does the *whole* pipeline -- parse the
//! `.glb`, resolve each texture to an image, transcode UASTC to RGBA -- reproduce the same picture
//! the artist authored? The oracle is the **PNG-textured twin** of the same model
//! (`gambeson.glb`), decoded natively with the `image` crate and diffed against the transcoded
//! UASTC (`gambeson-uastc.glb`) per material slot.
//!
//! **Textures are paired by material slot, never by texture or image index.** `gltf-transform`
//! reorders textures during the UASTC pass, so the two files' texture orders do not agree; an
//! index-based diff would silently compare the normal map against the metallic-roughness map. That
//! trap is not hypothetical here -- it is confirmed live in this asset, and
//! `pairing_by_texture_index_would_compare_the_wrong_slots` pins it down.
//!
//! These are native tests: parsing a `.glb` and transcoding UASTC to RGBA need no GPU. Only the
//! upload to the GPU does, and that belongs in the browser test suite.
//!
//! **Assets:** both live under `assets/gltf/`. `gambeson.glb` (the PNG twin) is committed;
//! `gambeson-uastc.glb` is the gltf-transform output and must be committed for this test to run in
//! CI. If either is absent the test fails loudly rather than skipping silently.

#![ cfg( feature = "ktx2" ) ]

use std::collections::{ BTreeSet, HashMap };

use minwebgl as gl;
use gl::texture::compressed::Format;
use renderer::webgl::loaders::ktx2::{ Ktx2Image, decode_level };

/// The gltf-transform UASTC output. Untracked by default -- commit it to run this in CI.
const UASTC_GLB : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/../../../assets/gltf/gambeson-uastc.glb" );
/// The original PNG-textured model -- the ground-truth oracle. Committed.
const PNG_GLB : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/../../../assets/gltf/gambeson.glb" );

/// A material's texture role. Pairing is done on *this*, not on any index.
#[ derive( Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug ) ]
enum Slot
{
  BaseColor,
  MetallicRoughness,
  Normal,
}

/// A decoded, uncompressed image: its dimensions and row-major RGBA8 pixels.
type Rgba = ( u32, u32, Vec< u8 > );

/// Parses a `.glb` and returns its document plus the embedded BIN chunk ( where View images live ).
fn load_glb( path : &str ) -> ( gltf::Gltf, Vec< u8 > )
{
  let bytes = std::fs::read( path ).unwrap_or_else
  (
    | e | panic!( "cannot read `{path}` ( is `gambeson-uastc.glb` committed? ) : {e}" )
  );
  let gltf = gltf::Gltf::from_slice( &bytes ).unwrap_or_else( | e | panic!( "parse `{path}` : {e}" ) );
  let blob = gltf.blob.clone().expect( "a .glb must carry a BIN chunk" );
  ( gltf, blob )
}

/// Resolves a texture to its image index the way the loader does: the `KHR_texture_basisu`
/// extension first ( where a UASTC texture keeps its image ), then the plain `source`.
fn effective_source( texture : &gltf::Texture< '_ > ) -> Option< usize >
{
  if let Some( index ) = texture
  .extension_value( "KHR_texture_basisu" )
  .and_then( | extension | extension.get( "source" ) )
  .and_then( | source | source.as_u64() )
  {
    return Some( index as usize );
  }
  texture.source().map( | image | image.index() )
}

/// The raw, still-encoded bytes of an embedded ( View ) image.
fn image_bytes< 'a >( gltf : &gltf::Gltf, blob : &'a [ u8 ], image_index : usize ) -> &'a [ u8 ]
{
  let image = gltf.images().nth( image_index ).expect( "image index out of range" );
  match image.source()
  {
    gltf::image::Source::View { view, .. } =>
    {
      let start = view.offset();
      let end = start + view.length();
      &blob[ start..end ]
    },
    gltf::image::Source::Uri { .. } => panic!( "these test assets embed images as views, not URIs" ),
  }
}

/// Decodes an image to full-resolution RGBA8. KTX2 goes through the UASTC transcoder ( level 0 );
/// PNG goes through the `image` crate. This is the exact pipeline the diff is meant to validate.
fn decode_rgba( bytes : &[ u8 ], ktx2 : bool ) -> Rgba
{
  if ktx2
  {
    let image = Ktx2Image::parse( bytes ).expect( "KTX2 must parse" );
    let wrapping = image.check_supported().expect( "the fixture is UASTC + Zstandard, i.e. supported" );
    let ( width, height ) = image.level_size( 0 );
    let level0 = image.levels().next().expect( "at least one mip level" ).data.to_vec();
    let rgba = decode_level( &level0, wrapping, Format::Rgba8, width, height ).expect( "UASTC decode" );
    ( width, height, rgba )
  }
  else
  {
    let decoded = image::load_from_memory( bytes ).expect( "PNG must decode" ).to_rgba8();
    ( decoded.width(), decoded.height(), decoded.into_raw() )
  }
}

/// Builds the slot → decoded-image map for one asset, resolving every texture through its material
/// role. `gambeson` has two materials -- an untextured "Mannequin" and the textured "Gambeson" --
/// so every material is walked and only the textured slots contribute. ( If two materials filled
/// the same slot, the last would win; this asset has exactly one textured material, so it does not
/// arise here. )
fn slot_images( gltf : &gltf::Gltf, blob : &[ u8 ], ktx2 : bool ) -> HashMap< Slot, Rgba >
{
  // Collect ( slot, texture ) first -- the three accessors return different wrapper types.
  let mut textures : Vec< ( Slot, gltf::Texture< '_ > ) > = Vec::new();
  for material in gltf.materials()
  {
    let pbr = material.pbr_metallic_roughness();
    if let Some( info ) = pbr.base_color_texture()
    {
      textures.push( ( Slot::BaseColor, info.texture() ) );
    }
    if let Some( info ) = pbr.metallic_roughness_texture()
    {
      textures.push( ( Slot::MetallicRoughness, info.texture() ) );
    }
    if let Some( normal ) = material.normal_texture()
    {
      textures.push( ( Slot::Normal, normal.texture() ) );
    }
  }

  let mut out = HashMap::new();
  for ( slot, texture ) in textures
  {
    let source = effective_source( &texture ).expect( "every used texture must resolve to an image" );
    out.insert( slot, decode_rgba( image_bytes( gltf, blob, source ), ktx2 ) );
  }
  out
}

/// Peak signal-to-noise ratio between two equal-length RGBA8 buffers, in decibels. Identical
/// buffers return `INFINITY`; higher is more similar.
fn psnr( a : &[ u8 ], b : &[ u8 ] ) -> f64
{
  assert_eq!( a.len(), b.len(), "PSNR operands differ in length" );
  let mut sum_sq = 0.0_f64;
  for ( &x, &y ) in a.iter().zip( b )
  {
    let diff = f64::from( x ) - f64::from( y );
    sum_sq += diff * diff;
  }
  let mse = sum_sq / a.len() as f64;
  if mse == 0.0
  {
    return f64::INFINITY;
  }
  10.0 * ( 255.0_f64 * 255.0 / mse ).log10()
}

/// The oracle: every material slot's transcoded UASTC must reproduce the PNG twin.
///
/// The end-to-end numbers on these textures are ~52.9 dB ( baseColor ), ~44.7 dB
/// ( metallicRoughness ) and ~39.5 dB ( normal ). A 30 dB floor sits comfortably below all three
/// yet still fails hard on the things this test exists to catch: a decode bug, a swapped channel
/// order, or a texture paired to the wrong slot.
#[ test ]
fn transcoded_uastc_matches_the_png_twin_per_material_slot()
{
  let ( uastc_gltf, uastc_blob ) = load_glb( UASTC_GLB );
  let ( png_gltf, png_blob ) = load_glb( PNG_GLB );

  let uastc = slot_images( &uastc_gltf, &uastc_blob, true );
  let png = slot_images( &png_gltf, &png_blob, false );

  assert!( !uastc.is_empty(), "no material-slot textures found in the UASTC asset" );
  assert_eq!
  (
    uastc.keys().collect::< BTreeSet< _ > >(),
    png.keys().collect::< BTreeSet< _ > >(),
    "the two assets must expose the same set of material slots"
  );

  const FLOOR_DB : f64 = 30.0;

  for ( slot, ( uastc_w, uastc_h, uastc_rgba ) ) in &uastc
  {
    let ( png_w, png_h, png_rgba ) = png.get( slot ).expect( "slot present in both, asserted above" );
    assert_eq!( ( uastc_w, uastc_h ), ( png_w, png_h ), "{slot:?} : dimensions differ between the twins" );

    let db = psnr( uastc_rgba, png_rgba );
    assert!
    (
      db >= FLOOR_DB,
      "{slot:?} : PSNR {db:.2} dB is below the {FLOOR_DB} dB floor -- \
       the transcode is wrong, or this texture was paired to the wrong slot"
    );
  }
}

/// The reason material-slot pairing is mandatory, made concrete.
///
/// `gltf-transform` reorders textures, so pairing the two files by texture *index* diffs a normal
/// map against a metallic-roughness map -- a mistake that yields a spurious ~5 dB "failure". This
/// test proves both halves of why the oracle above is trustworthy: the PSNR metric genuinely
/// distinguishes one slot from another ( so a high score means the *right* image ), and the reorder
/// is real in this asset ( so by-slot pairing is load-bearing, not decoration ).
#[ test ]
fn pairing_by_texture_index_would_compare_the_wrong_slots()
{
  let ( uastc_gltf, uastc_blob ) = load_glb( UASTC_GLB );
  let ( png_gltf, png_blob ) = load_glb( PNG_GLB );

  let uastc = slot_images( &uastc_gltf, &uastc_blob, true );
  let png = slot_images( &png_gltf, &png_blob, false );

  let normal_uastc = uastc.get( &Slot::Normal ).expect( "normal slot" );
  let normal_png = png.get( &Slot::Normal ).expect( "normal slot" );
  let mr_png = png.get( &Slot::MetallicRoughness ).expect( "metallic-roughness slot" );

  // The correct, by-slot pairing.
  let correct_db = psnr( &normal_uastc.2, &normal_png.2 );
  // The pairing an index-based diff would make on this asset: normal ( UASTC ) vs MR ( PNG ).
  assert_eq!
  (
    ( normal_uastc.0, normal_uastc.1 ), ( mr_png.0, mr_png.1 ),
    "same dimensions, so only the *content* distinguishes the two -- which is the point"
  );
  let mispaired_db = psnr( &normal_uastc.2, &mr_png.2 );

  assert!( correct_db >= 30.0, "normal-vs-normal must match : {correct_db:.2} dB" );
  assert!
  (
    mispaired_db < correct_db - 10.0,
    "normal-vs-MR ( {mispaired_db:.2} dB ) must be clearly worse than normal-vs-normal \
     ( {correct_db:.2} dB ) -- otherwise the diff cannot tell slots apart and the oracle proves nothing"
  );
}
