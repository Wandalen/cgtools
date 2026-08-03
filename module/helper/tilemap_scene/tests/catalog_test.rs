//! Integration tests for [`tilemap_scene::Catalog`] and its builder.

#![ allow( clippy::min_ident_chars ) ]
#![ allow
(
  clippy::default_trait_access,
  clippy::too_many_lines,
) ]

extern crate alloc;
use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;

use tilemap_scene::
{
  Anchor,
  Asset,
  AssetKind,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PipelineLayer,
  RenderPipeline,
  RenderSpec,
  Scene,
  SortMode,
  SpriteRef,
  SpriteSource,
  TilingStrategy,
};
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

// Same two-object fixture (`grass`, `knight` with idle / walk) as
// `scene_state_test.rs`. Inlined here to keep tests independent.

fn make_layer( asset : &str, frame : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Static
    (
      SpriteRef { asset : asset.into(), frame : frame.into() }
    ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn build_spec() -> Arc< RenderSpec >
{
  let mut grass_states = HashMap::default();
  grass_states.insert( "default".into(), vec![ make_layer( "terrain", "0" ) ] );

  let mut knight_states = HashMap::default();
  knight_states.insert( "idle".into(), vec![ make_layer( "terrain", "0" ) ] );
  knight_states.insert( "walk".into(), vec![ make_layer( "terrain", "1" ) ] );

  let spec = RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec!
    [
      Asset
      {
        id : "terrain".into(),
        path : "terrain.png".into(),
        kind : AssetKind::Atlas
        {
          tile_size : ( 72, 64 ),
          columns : 2,
          origin : ( 0, 0 ),
          gap : ( 0, 0 ),
          frames : HashMap::default(),
          frame_rects : HashMap::default(),
          image_size : None,
        },
        filter : SamplerFilter::Linear,
        mipmap : MipmapMode::Off,
        wrap : WrapMode::Clamp,
      },
    ],
    tints : Vec::new(),
    animations : Vec::new(),
    effects : Vec::new(),
    objects : vec!
    [
      Object
      {
        id : "grass".into(),
        anchor : Anchor::Hex,
        global_layer : "terrain".into(),
        priority : None,
        sort_y_source : Default::default(),
        pivot : ( 0.5, 0.5 ),
        default_state : "default".into(),
        states : grass_states,
      },
      Object
      {
        id : "knight".into(),
        anchor : Anchor::Hex,
        global_layer : "terrain".into(),
        priority : None,
        sort_y_source : Default::default(),
        pivot : ( 0.5, 0.5 ),
        default_state : "idle".into(),
        states : knight_states,
      },
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  };
  Arc::new( spec )
}

#[ test ]
fn catalog_resolves_required_object_and_state_handles()
{
  let scene = Scene::new( build_spec() );
  let cat = scene.catalog()
    .require_object( "grass" )
    .require_state( "knight", "idle" )
    .require_state( "knight", "walk" )
    .build()
    .expect( "all ids declared in spec" );

  // Object handles round-trip with Scene::object.
  assert_eq!( cat.object( "grass" ), scene.object( "grass" ).unwrap() );
  let knight = scene.object( "knight" ).unwrap();
  assert_eq!( cat.object( "knight" ), knight, "require_state implies object" );

  // State handles round-trip with Scene::state.
  assert_eq!( cat.state( "knight", "idle" ), scene.state( knight, "idle" ).unwrap() );
  assert_eq!( cat.state( "knight", "walk" ), scene.state( knight, "walk" ).unwrap() );
}

#[ test ]
fn catalog_build_reports_every_missing_object_together()
{
  let scene = Scene::new( build_spec() );
  let err = scene.catalog()
    .require_object( "grass" )       // declared
    .require_object( "wizard" )      // missing
    .require_object( "dragon" )      // missing
    .build()
    .expect_err( "two ids are missing" );

  assert_eq!( err.missing_objects.len(), 2 );
  assert!( err.missing_objects.iter().any( | id | id == "wizard" ) );
  assert!( err.missing_objects.iter().any( | id | id == "dragon" ) );
  assert!( err.missing_states.is_empty() );
}

#[ test ]
fn catalog_build_reports_missing_state_on_declared_object()
{
  let scene = Scene::new( build_spec() );
  let err = scene.catalog()
    .require_state( "knight", "idle" )    // declared
    .require_state( "knight", "attack" )  // missing state
    .build()
    .expect_err( "one state missing" );

  assert!( err.missing_objects.is_empty(), "knight is declared" );
  assert_eq!( err.missing_states.len(), 1 );
  assert_eq!
  (
    err.missing_states[ 0 ],
    ( "knight".to_owned(), "attack".to_owned() ),
  );
}

#[ test ]
fn catalog_build_does_not_double_report_state_when_object_missing()
{
  // Requesting a state on a missing object surfaces the object miss
  // once and skips the state miss — partial repair: the user fixes
  // the object id, re-runs, then sees any state misses.
  let scene = Scene::new( build_spec() );
  let err = scene.catalog()
    .require_state( "wizard", "fireball" )
    .build()
    .expect_err( "object missing" );

  assert_eq!( err.missing_objects.len(), 1 );
  assert_eq!( err.missing_objects[ 0 ], "wizard" );
  assert!
  (
    err.missing_states.is_empty(),
    "state miss should be suppressed while its object is unknown: {:?}",
    err.missing_states,
  );
}

#[ test ]
fn catalog_try_lookups_return_none_for_unrequired_ids()
{
  let scene = Scene::new( build_spec() );
  let cat = scene.catalog()
    .require_object( "grass" )
    .build()
    .unwrap();
  assert!( cat.try_object( "knight" ).is_none(), "knight was not requested" );
  assert!( cat.try_state( "knight", "idle" ).is_none() );
  // The requested one resolves.
  assert!( cat.try_object( "grass" ).is_some() );
}

#[ test ]
#[ should_panic( expected = "was not required at build time" ) ]
fn catalog_object_panics_for_unrequired_id()
{
  let scene = Scene::new( build_spec() );
  let cat = scene.catalog().build().unwrap();
  // No objects required at build time — every lookup panics.
  let _ = cat.object( "grass" );
}
