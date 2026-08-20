//! Assets validation tests.
//!
//! Coverage matrix:
//! - **Empty baseline** — an all-empty `Assets` struct passes validation with zero errors
//! - **No false positives** — two distinct ids in the same list produce no errors
//! - **Duplicate detection per type** — image, geometry, sprite, gradient, clip mask, path
//!   each produce exactly one error when two entries share the same id
//! - **Cross-type id scoping** — the same id in two different asset types is not a duplicate
//! - **Multiple simultaneous errors** — duplicate ids in two independent lists each
//!   produce their own error, all reported in a single `validate()` call

mod helpers;
use helpers::empty_assets;

use tilemap_renderer::types::*;
use tilemap_renderer::assets::*;

/// Verifies that an `Assets` struct with all empty vecs passes validation
/// with zero errors — the empty state is always valid.
#[ test ]
fn assets_validate_empty()
{
  let assets = empty_assets();
  assert!( assets.validate().is_empty() );
}

/// Verifies that two images with distinct ids produce no validation errors.
/// Ensures the duplicate-detection logic does not produce false positives.
#[ test ]
fn assets_validate_no_duplicates()
{
  let assets = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
      ImageAsset { id : ResourceId::new( 1 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
    ],
    ..empty_assets()
  };
  assert!( assets.validate().is_empty() );
}

/// Verifies that two images sharing the same id produce exactly one
/// validation error whose message names the asset type and the duplicate id.
#[ test ]
fn assets_validate_duplicate_image_ids()
{
  let assets = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
      ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
    ],
    ..empty_assets()
  };
  let errors = assets.validate();
  assert_eq!( errors.len(), 1 );
  let msg = format!( "{}", errors[ 0 ] );
  assert!( msg.contains( "image" ) );
  assert!( msg.contains( '5' ) );
}

/// Verifies that two geometry assets sharing the same id produce exactly
/// one validation error — duplicate detection works for the geometry list.
#[ test ]
fn assets_validate_duplicate_geometry_ids()
{
  let assets = Assets
  {
    geometries : vec![
      GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
      GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
    ],
    ..empty_assets()
  };
  let errors = assets.validate();
  assert_eq!( errors.len(), 1 );
}

/// Verifies that an image and a geometry asset sharing id 0 do not trigger
/// a duplicate error — ids are scoped per asset type, not globally.
#[ test ]
fn assets_validate_cross_type_ids_ok()
{
  let assets = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
    ],
    geometries : vec![
      GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
    ],
    ..empty_assets()
  };
  assert!( assets.validate().is_empty() );
}

/// Verifies that duplicate ids in two independent lists (images and sprites)
/// each produce their own error — all lists are checked independently.
#[ test ]
fn assets_validate_multiple_duplicate_types()
{
  let assets = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off, wrap : WrapMode::Clamp, premultiplied : false },
    ],
    sprites : vec![
      SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 0 ), region : [ 0.0; 4 ] },
      SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 0 ), region : [ 0.0; 4 ] },
    ],
    ..empty_assets()
  };
  let errors = assets.validate();
  assert_eq!( errors.len(), 2 );
}

/// Verifies that duplicate gradient ids produce a validation error.
/// Covers the gradient asset list which is separate from image/geometry.
#[ test ]
fn assets_validate_gradient_duplicates()
{
  let stop = GradientStop { offset : 0.0, color : [ 1.0, 1.0, 1.0, 1.0 ] };
  let assets = Assets
  {
    gradients : vec![
      GradientAsset { id : ResourceId::new( 0 ), kind : GradientKind::Linear { start : [ 0.0, 0.0 ], end : [ 1.0, 1.0 ] }, stops : vec![ stop ] },
      GradientAsset { id : ResourceId::new( 0 ), kind : GradientKind::Linear { start : [ 0.0, 0.0 ], end : [ 1.0, 1.0 ] }, stops : vec![ stop ] },
    ],
    ..empty_assets()
  };
  assert_eq!( assets.validate().len(), 1 );
}

/// Verifies that duplicate clip-mask ids produce a validation error.
/// Covers the clip-mask asset list.
#[ test ]
fn assets_validate_clip_mask_duplicates()
{
  let assets = Assets
  {
    clip_masks : vec![
      ClipMaskAsset { id : ResourceId::new( 0 ), segments : vec![] },
      ClipMaskAsset { id : ResourceId::new( 0 ), segments : vec![] },
    ],
    ..empty_assets()
  };
  assert_eq!( assets.validate().len(), 1 );
}

/// Verifies that duplicate path ids produce a validation error.
/// Covers the path asset list.
#[ test ]
fn assets_validate_path_duplicates()
{
  let assets = Assets
  {
    paths : vec![
      PathAsset { id : ResourceId::new( 3 ), segments : vec![] },
      PathAsset { id : ResourceId::new( 3 ), segments : vec![] },
    ],
    ..empty_assets()
  };
  assert_eq!( assets.validate().len(), 1 );
}

/// `to_rgba8` conversion tests (task 218) -- gated on `adapter-native` only,
/// not also `adapter-webgpu`, which this crate's own convention compiles
/// test code for under wasm32 only (see `webgpu_backend_test.rs`'s
/// file-level gate); `adapter-native` is the gate that lets `cargo test`
/// exercise these on a plain native host. `to_rgba8` itself is shared by
/// both adapters -- cfg-gated `any(adapter-native, adapter-webgpu)` in
/// `src/assets.rs`.
#[ cfg( feature = "adapter-native" ) ]
mod to_rgba8_conversion
{
  use super::*;

  /// T01 -- `Rgba8` input passes through byte-for-byte unchanged: no
  /// conversion needed, since the format already matches the GPU upload
  /// format.
  #[ test ]
  fn to_rgba8_expands_rgba8_passthrough()
  {
    let bytes : [ u8; 16 ] = [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 ];
    assert_eq!( to_rgba8( &bytes, PixelFormat::Rgba8 ), bytes.to_vec() );
  }

  /// T02 -- `Rgb8` input gets an opaque (`255`) alpha byte appended after
  /// each 3-byte pixel, expanding 3 bytes/pixel to 4.
  #[ test ]
  fn to_rgba8_expands_rgb8_pads_opaque_alpha()
  {
    let bytes = [ 10u8, 20, 30 ];
    assert_eq!( to_rgba8( &bytes, PixelFormat::Rgb8 ), vec![ 10, 20, 30, 255 ] );
  }

  /// T03 -- `Gray8` input broadcasts the single gray byte into R, G, and B,
  /// with an opaque alpha appended -- 1 byte/pixel expands to 4.
  #[ test ]
  fn to_rgba8_expands_gray8_broadcasts_to_rgb()
  {
    let bytes = [ 42u8 ];
    assert_eq!( to_rgba8( &bytes, PixelFormat::Gray8 ), vec![ 42, 42, 42, 255 ] );
  }

  /// T04 -- `GrayAlpha8` input broadcasts the gray byte into R, G, and B,
  /// preserving the real (non-255) alpha byte -- 2 bytes/pixel expands to 4.
  #[ test ]
  fn to_rgba8_expands_grayalpha8_preserves_alpha()
  {
    let bytes = [ 42u8, 128 ];
    assert_eq!( to_rgba8( &bytes, PixelFormat::GrayAlpha8 ), vec![ 42, 42, 42, 128 ] );
  }
}
