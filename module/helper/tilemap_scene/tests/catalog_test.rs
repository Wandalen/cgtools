//! Integration tests for [`tilemap_scene::Catalog`] and its builder.

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
  SortYSource,
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

fn layer_make( asset : &str, frame : &str ) -> ObjectLayer
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

fn spec_build() -> Arc< RenderSpec >
{
  let mut grass_states = HashMap::default();
  grass_states.insert( "default".into(), vec![ layer_make( "terrain", "0" ) ] );

  let mut knight_states = HashMap::default();
  knight_states.insert( "idle".into(), vec![ layer_make( "terrain", "0" ) ] );
  knight_states.insert( "walk".into(), vec![ layer_make( "terrain", "1" ) ] );

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
        premultiplied : false,
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
        sort_y_source : SortYSource::default(),
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
        sort_y_source : SortYSource::default(),
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
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false, opaque : false },
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
  let scene = Scene::new( spec_build() );
  let cat = scene.catalog()
    .object_require( "grass" )
    .state_require( "knight", "idle" )
    .state_require( "knight", "walk" )
    .build()
    .expect( "all ids declared in spec" );

  // Object handles round-trip with Scene::object.
  assert_eq!( cat.object( "grass" ), scene.object( "grass" ).unwrap() );
  let knight = scene.object( "knight" ).unwrap();
  assert_eq!( cat.object( "knight" ), knight, "state_require implies object" );

  // State handles round-trip with Scene::state.
  assert_eq!( cat.state( "knight", "idle" ), scene.state( knight, "idle" ).unwrap() );
  assert_eq!( cat.state( "knight", "walk" ), scene.state( knight, "walk" ).unwrap() );
}

#[ test ]
fn catalog_build_reports_every_missing_object_together()
{
  let scene = Scene::new( spec_build() );
  let err = scene.catalog()
    .object_require( "grass" )       // declared
    .object_require( "wizard" )      // missing
    .object_require( "dragon" )      // missing
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
  let scene = Scene::new( spec_build() );
  let err = scene.catalog()
    .state_require( "knight", "idle" )    // declared
    .state_require( "knight", "attack" )  // missing state
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
  let scene = Scene::new( spec_build() );
  let err = scene.catalog()
    .state_require( "wizard", "fireball" )
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
  let scene = Scene::new( spec_build() );
  let cat = scene.catalog()
    .object_require( "grass" )
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
  let scene = Scene::new( spec_build() );
  let cat = scene.catalog().build().unwrap();
  // No objects required at build time — every lookup panics.
  let _ = cat.object( "grass" );
}

/// ## Root Cause
/// `CatalogBuilder::build()`'s objects loop dedupes missing ids via a
/// `seen_missing_objects` set, so a repeated `object_require( "x" )` for a
/// missing `"x"` reports the miss exactly once. The sibling states loop had
/// no equivalent guard: it pushed onto `missing_states` unconditionally for
/// every `(obj, state)` pair, so calling `state_require` twice with the
/// identical, missing pair produced two identical entries in
/// `err.missing_states` and inflated `missing_states.len()` beyond the
/// actual number of distinct misses.
/// ## Why Not Caught
/// Every existing `catalog_test.rs` case calls `state_require` with
/// distinct `(obj, state)` pairs — none repeats the identical pair twice,
/// so the missing dedup guard was never exercised.
/// ## Fix Applied
/// Added a `seen_missing_states` set (and a `states.contains_key` guard for
/// the success path) to the states loop in `src/catalog.rs`, mirroring the
/// `seen_missing_objects` / `objects.contains_key` pattern the objects loop
/// already uses.
/// ## Prevention
/// This test pins that requiring the same missing `(obj, state)` pair twice
/// still yields exactly one `missing_states` entry.
/// ## Pitfall
/// Parallel accumulation loops that share a "report every unique miss
/// exactly once" contract need the same dedup guard applied to both —
/// copying one loop's dedup logic but not its sibling's leaves a defect
/// that only the repeated-input path exposes.
#[ test ]
fn catalog_build_does_not_double_report_duplicate_missing_state_require()
{
  let scene = Scene::new( spec_build() );
  let err = scene.catalog()
    .state_require( "knight", "attack" )  // missing state
    .state_require( "knight", "attack" )  // same pair again, deliberately
    .build()
    .expect_err( "state is missing" );

  assert!( err.missing_objects.is_empty(), "knight is declared" );
  assert_eq!
  (
    err.missing_states.len(), 1,
    "duplicate identical state_require calls must not double-report: {:?}",
    err.missing_states,
  );
  assert_eq!
  (
    err.missing_states[ 0 ],
    ( "knight".to_owned(), "attack".to_owned() ),
  );
}
