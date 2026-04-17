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
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
      ImageAsset { id : ResourceId::new( 1 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
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
      ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
      ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
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
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
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
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear, mipmap : MipmapMode::Off },
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
