#![ allow( clippy::min_ident_chars ) ]

//! Assets tests

mod helpers;
use helpers::empty_assets;

use tilemap_renderer::types::*;
use tilemap_renderer::assets::*;

#[ test ]
fn assets_validate_empty()
{
  let a = empty_assets();
  assert!( a.validate().is_empty() );
}

#[ test ]
fn assets_validate_no_duplicates()
{
  let a = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
      ImageAsset { id : ResourceId::new( 1 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
    ],
    ..empty_assets()
  };
  assert!( a.validate().is_empty() );
}

#[ test ]
fn assets_validate_duplicate_image_ids()
{
  let a = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
      ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
    ],
    ..empty_assets()
  };
  let errors = a.validate();
  assert_eq!( errors.len(), 1 );
  let msg = format!( "{}", errors[ 0 ] );
  assert!( msg.contains( "image" ) );
  assert!( msg.contains( '5' ) );
}

#[ test ]
fn assets_validate_duplicate_geometry_ids()
{
  let a = Assets
  {
    geometries : vec![
      GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
      GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
    ],
    ..empty_assets()
  };
  let errors = a.validate();
  assert_eq!( errors.len(), 1 );
}

#[ test ]
fn assets_validate_cross_type_ids_ok()
{
  let a = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
    ],
    geometries : vec![
      GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
    ],
    ..empty_assets()
  };
  assert!( a.validate().is_empty() );
}

#[ test ]
fn assets_validate_multiple_duplicate_types()
{
  let a = Assets
  {
    images : vec![
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
      ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : SamplerFilter::Linear },
    ],
    sprites : vec![
      SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 0 ), region : [ 0.0; 4 ] },
      SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 0 ), region : [ 0.0; 4 ] },
    ],
    ..empty_assets()
  };
  let errors = a.validate();
  assert_eq!( errors.len(), 2 );
}

#[ test ]
fn assets_validate_gradient_duplicates()
{
  let stop = GradientStop { offset : 0.0, color : [ 1.0, 1.0, 1.0, 1.0 ] };
  let a = Assets
  {
    gradients : vec![
      GradientAsset { id : ResourceId::new( 0 ), kind : GradientKind::Linear { start : [ 0.0, 0.0 ], end : [ 1.0, 1.0 ] }, stops : vec![ stop ] },
      GradientAsset { id : ResourceId::new( 0 ), kind : GradientKind::Linear { start : [ 0.0, 0.0 ], end : [ 1.0, 1.0 ] }, stops : vec![ stop ] },
    ],
    ..empty_assets()
  };
  assert_eq!( a.validate().len(), 1 );
}

#[ test ]
fn assets_validate_clip_mask_duplicates()
{
  let a = Assets
  {
    clip_masks : vec![
      ClipMaskAsset { id : ResourceId::new( 0 ), segments : vec![] },
      ClipMaskAsset { id : ResourceId::new( 0 ), segments : vec![] },
    ],
    ..empty_assets()
  };
  assert_eq!( a.validate().len(), 1 );
}

#[ test ]
fn assets_validate_path_duplicates()
{
  let a = Assets
  {
    paths : vec![
      PathAsset { id : ResourceId::new( 3 ), segments : vec![] },
      PathAsset { id : ResourceId::new( 3 ), segments : vec![] },
    ],
    ..empty_assets()
  };
  assert_eq!( a.validate().len(), 1 );
}
