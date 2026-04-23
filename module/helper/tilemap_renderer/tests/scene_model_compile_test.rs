//! Integration tests for the Slice-1 compile layer
//! (`scene_model::compile::compile_assets` + `compile_frame`).

#![ cfg( feature = "scene-model" ) ]
#![ allow( clippy::min_ident_chars ) ]
// Test-only idioms: exact array comparisons and ref-in-closure patterns are
// intentional; `Default::default()` reads fine at fixture build sites.
#![ allow
(
  clippy::float_cmp,
  clippy::default_trait_access,
  clippy::redundant_closure_for_method_calls,
  clippy::needless_borrow,
) ]

use std::collections::HashMap;

use tilemap_renderer::commands::RenderCommand;
use tilemap_renderer::scene_model::
{
  Anchor,
  Animation,
  AnimationMode,
  AnimationRef,
  AnimationTiming,
  Asset,
  AssetKind,
  AssetResolver,
  Bounds,
  Camera,
  CompileError,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PathResolver,
  PhaseOffset,
  PipelineLayer,
  RenderPipeline,
  RenderSpec,
  Scene,
  SortMode,
  SpriteRef,
  SpriteSource,
  Tile,
  TilingStrategy,
  Variant,
  VariantSelection,
  compile_assets,
  compile_frame,
};
use tilemap_renderer::assets::ImageSource;
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

// ────────────────────────────────────────────────────────────────────────────
// Fixture builders.
// ────────────────────────────────────────────────────────────────────────────

fn grass_object() -> Object
{
  let mut anims = HashMap::new();
  anims.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : Some( "base".into() ),
        sprite_source : SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ),
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  Object
  {
    id : "grass".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : Some( 10 ),
    sort_y_source : Default::default(),
    default_animation : "default".into(),
    animations : anims,
  }
}

fn minimal_spec() -> RenderSpec
{
  RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec!
    [
      Asset
      {
        id : "terrain".into(),
        path : "terrain.png".into(),
        kind : AssetKind::Atlas { tile_size : ( 72, 64 ), columns : 2 },
        filter : SamplerFilter::Linear,
        mipmap : MipmapMode::Off,
        wrap : WrapMode::Clamp,
      },
    ],
    tints : Vec::new(),
    animations : Vec::new(),
    effects : Vec::new(),
    objects : vec![ grass_object() ],
    pipeline : RenderPipeline
    {
      hex : HexConfig
      {
        tiling : TilingStrategy::HexFlatTop,
        cell_size : ( 72, 64 ),
      },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None },
      ],
      global_tint : None,
      viewport_size : None,
    },
  }
}

fn minimal_scene_3x3() -> Scene
{
  let mut scene = Scene::new( Bounds { min : ( 0, 0 ), max : ( 2, 2 ) } );
  for r in 0..3
  {
    for q in 0..3
    {
      scene.tiles.push( Tile { pos : ( q, r ), objects : vec![ "grass".into() ] } );
    }
  }
  scene
}

// ────────────────────────────────────────────────────────────────────────────
// compile_assets tests.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn compile_assets_allocates_one_image_and_one_sprite()
{
  let spec = minimal_spec();
  let compiled = compile_assets( &spec, &PathResolver ).expect( "compile" );

  assert_eq!( compiled.assets.images.len(), 1, "one atlas = one image" );
  assert_eq!( compiled.assets.sprites.len(), 1, "one sprite ref = one sprite" );

  let sprite = &compiled.assets.sprites[ 0 ];
  assert_eq!( sprite.region, [ 0.0, 0.0, 72.0, 64.0 ], "frame 0 occupies top-left tile" );
}

#[ test ]
fn compile_assets_atlas_region_indexing()
{
  // Grass that references frame "3" — 2 columns, so (3 % 2, 3 / 2) = (1, 1).
  let mut spec = minimal_spec();
  let mut anims = HashMap::new();
  anims.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::Static( SpriteRef( "terrain".into(), "3".into() ) ),
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  spec.objects[ 0 ].animations = anims;

  let compiled = compile_assets( &spec, &PathResolver ).expect( "compile" );
  let sprite = &compiled.assets.sprites[ 0 ];
  assert_eq!( sprite.region, [ 72.0, 64.0, 72.0, 64.0 ], "frame 3 at (col 1, row 1)" );
}

#[ test ]
fn compile_assets_rejects_single_kind()
{
  let mut spec = minimal_spec();
  spec.assets[ 0 ].kind = AssetKind::Single;
  // Make the layer reference *any* frame name — doesn't matter which, because
  // region resolution fails before lookup.
  let err = compile_assets( &spec, &PathResolver ).unwrap_err();
  assert!
  (
    matches!( err, CompileError::UnsupportedAssetKind { ref kind, .. } if *kind == "Single" ),
    "expected UnsupportedAssetKind/Single, got {err:?}",
  );
}

#[ test ]
fn compile_assets_rejects_bad_frame_name()
{
  let mut spec = minimal_spec();
  spec.objects[ 0 ].animations.get_mut( "default" ).unwrap()[ 0 ].sprite_source =
    SpriteSource::Static( SpriteRef( "terrain".into(), "oops".into() ) );
  let err = compile_assets( &spec, &PathResolver ).unwrap_err();
  assert!
  (
    matches!( err, CompileError::InvalidFrameName { ref frame, .. } if frame == "oops" ),
    "expected InvalidFrameName, got {err:?}",
  );
}

// Custom resolver proves the trait is extensible — here every path becomes a
// 1×1 black pixel Bitmap.
struct BlackPixelResolver;
impl AssetResolver for BlackPixelResolver
{
  fn resolve
  (
    &self,
    _asset_id : &str,
    _path : &str,
    _kind : &AssetKind,
  ) -> Result< ImageSource, CompileError >
  {
    Ok( ImageSource::Bitmap
    {
      bytes : vec![ 0, 0, 0, 255 ],
      width : 1,
      height : 1,
      format : tilemap_renderer::assets::PixelFormat::Rgba8,
    })
  }
}

#[ test ]
fn compile_assets_custom_resolver()
{
  let spec = minimal_spec();
  let compiled = compile_assets( &spec, &BlackPixelResolver ).expect( "compile" );
  assert!( matches!( compiled.assets.images[ 0 ].source, ImageSource::Bitmap { .. } ) );
}

// ────────────────────────────────────────────────────────────────────────────
// compile_frame tests.
// ────────────────────────────────────────────────────────────────────────────

fn compile( spec : &RenderSpec, scene : &Scene, camera : &Camera ) -> Vec< RenderCommand >
{
  let compiled = compile_assets( spec, &PathResolver ).expect( "assets" );
  compile_frame( spec, scene, &compiled, camera, 0.0 ).expect( "frame" )
}

#[ test ]
fn first_command_is_clear()
{
  let cmds = compile( &minimal_spec(), &minimal_scene_3x3(), &Camera::default() );
  assert!( matches!( cmds[ 0 ], RenderCommand::Clear( _ ) ), "first command must be Clear" );
}

#[ test ]
fn one_sprite_per_tile()
{
  let cmds = compile( &minimal_spec(), &minimal_scene_3x3(), &Camera::default() );
  let sprite_count = cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count();
  assert_eq!( sprite_count, 9, "3x3 grid = 9 sprites" );
}

fn sprite_x( cmd : &RenderCommand ) -> f32
{
  if let RenderCommand::Sprite( s ) = cmd { s.transform.position[ 0 ] } else { panic!( "not Sprite" ) }
}

fn sprite_y( cmd : &RenderCommand ) -> f32
{
  if let RenderCommand::Sprite( s ) = cmd { s.transform.position[ 1 ] } else { panic!( "not Sprite" ) }
}

#[ test ]
fn y_flip_applied()
{
  // Two tiles differing only in r. Because tiles_tools is Y-down and the
  // compile layer flips to Y-up, increasing r should decrease screen Y.
  let spec = minimal_spec();
  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 0, 1 ), objects : vec![ "grass".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let cmds = compile( &spec, &scene, &Camera::default() );
  let sprites : Vec< _ > = cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).collect();
  assert_eq!( sprites.len(), 2 );
  let y0 = sprite_y( sprites[ 0 ] );
  let y1 = sprite_y( sprites[ 1 ] );
  assert!( y1 < y0, "r=1 should map to smaller Y than r=0 (got y0={y0}, y1={y1})" );
}

#[ test ]
fn camera_translation_shifts_sprites()
{
  let spec = minimal_spec();
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let base_cmds = compile( &spec, &scene, &Camera::default() );
  let shifted_cam = Camera { world_center : ( 100.0, 0.0 ), ..Camera::default() };
  let shifted_cmds = compile( &spec, &scene, &shifted_cam );

  let base_x = sprite_x( base_cmds.iter().find( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).unwrap() );
  let shifted_x = sprite_x( shifted_cmds.iter().find( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).unwrap() );
  assert!
  (
    ( shifted_x - ( base_x - 100.0 ) ).abs() < 1e-3,
    "camera +100x should subtract 100 from sprite x: base={base_x} shifted={shifted_x}",
  );
}

#[ test ]
fn camera_zoom_scales_transform()
{
  let spec = minimal_spec();
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 1, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let one = compile( &spec, &scene, &Camera::default() );
  let two = compile( &spec, &scene, &Camera { zoom : 2.0, ..Camera::default() } );

  // The sprite-level scale reflects camera zoom.
  let scale_one = if let RenderCommand::Sprite( s ) =
    one.iter().find( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).unwrap()
    { s.transform.scale[ 0 ] } else { panic!() };
  let scale_two = if let RenderCommand::Sprite( s ) =
    two.iter().find( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).unwrap()
    { s.transform.scale[ 0 ] } else { panic!() };
  assert!( ( scale_one - 1.0 ).abs() < 1e-3, "default zoom is 1.0, got {scale_one}" );
  assert!( ( scale_two - 2.0 ).abs() < 1e-3, "zoom=2 should set scale 2, got {scale_two}" );
}

#[ test ]
fn yasc_sorts_by_world_y()
{
  // Two objects in the same bucket with different rows; YAsc sorts them.
  let mut spec = minimal_spec();
  spec.pipeline.layers[ 0 ].sort = SortMode::YAsc;

  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 2 ), objects : vec![ "grass".into() ] },    // further north (Y-up: higher y)
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },    // origin
    ],
    ..minimal_scene_3x3()
  };
  let cmds = compile( &spec, &scene, &Camera::default() );
  let sprites : Vec< _ > = cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).collect();

  // YAsc on world-Y: after the Y-flip, r=0 has LARGER world y than r=2 (which
  // was flipped to more-negative). So YAsc emits the r=2 tile first (smaller y).
  let y_first = sprite_y( sprites[ 0 ] );
  let y_second = sprite_y( sprites[ 1 ] );
  assert!( y_first <= y_second, "YAsc order violated: first={y_first} second={y_second}" );
}

#[ test ]
fn palette_expansion_produces_same_tiles_as_explicit()
{
  let spec = minimal_spec();

  let mut palette = HashMap::new();
  palette.insert( '.', vec![ "grass".into() ] );
  let ascii_scene = Scene
  {
    tiles : Vec::new(),
    palette,
    map : vec!
    [
      ". . .".into(),
      ". . .".into(),
    ],
    ..minimal_scene_3x3()
  };

  let explicit_scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 1, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 2, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 0, 1 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 1, 1 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 2, 1 ), objects : vec![ "grass".into() ] },
    ],
    palette : HashMap::new(),
    map : Vec::new(),
    ..minimal_scene_3x3()
  };

  let ascii_cmds = compile( &spec, &ascii_scene, &Camera::default() );
  let explicit_cmds = compile( &spec, &explicit_scene, &Camera::default() );

  let ascii_sprites = ascii_cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count();
  let explicit_sprites = explicit_cmds.iter().filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) ).count();
  assert_eq!( ascii_sprites, explicit_sprites, "palette expansion must match explicit tiles" );
  assert_eq!( ascii_sprites, 6 );
}

// ────────────────────────────────────────────────────────────────────────────
// Error paths.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn rejects_non_hex_anchor()
{
  let mut spec = minimal_spec();
  spec.objects[ 0 ].anchor = Anchor::Edge;
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let err = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap_err();
  assert!
  (
    matches!( err, CompileError::UnsupportedAnchor { ref anchor, .. } if *anchor == "Edge" ),
    "expected UnsupportedAnchor/Edge, got {err:?}",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Slice 2 — Animation + Variant + PhaseOffset support.
// ────────────────────────────────────────────────────────────────────────────

fn grass_with_source( source : SpriteSource ) -> Object
{
  let mut g = grass_object();
  g.animations.get_mut( "default" ).unwrap()[ 0 ].sprite_source = source;
  g
}

#[ test ]
fn variant_hashcoord_picks_deterministically()
{
  let mut spec = minimal_spec();
  spec.objects[ 0 ] = grass_with_source
  (
    SpriteSource::Variant
    {
      variants : vec!
      [
        Variant
        {
          sprite : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ) ),
          weight : 1,
        },
        Variant
        {
          sprite : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "1".into() ) ) ),
          weight : 1,
        },
      ],
      selection : VariantSelection::HashCoord,
    }
  );

  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 7, 3 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };

  let cmds_first  = compile( &spec, &scene, &Camera::default() );
  let cmds_second = compile( &spec, &scene, &Camera::default() );

  // Two compiles of the same spec + scene + position → same sprite id.
  let spr_of = | cmds : &[ tilemap_renderer::commands::RenderCommand ] |
  {
    cmds.iter().find_map( | c |
      if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite.inner() ) }
      else { None }
    ).unwrap()
  };
  assert_eq!( spr_of( &cmds_first ), spr_of( &cmds_second ),
    "HashCoord selection must be deterministic" );
}

#[ test ]
fn variant_fixed_always_picks_that_index()
{
  let mut spec = minimal_spec();
  spec.objects[ 0 ] = grass_with_source
  (
    SpriteSource::Variant
    {
      variants : vec!
      [
        Variant
        {
          sprite : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ) ),
          weight : 1,
        },
        Variant
        {
          sprite : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "1".into() ) ) ),
          weight : 1,
        },
      ],
      selection : VariantSelection::Fixed( 1 ),
    }
  );

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprite_id = cmds.iter().find_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).unwrap();

  // Fixed(1) → frame "1" is `compiled.ids.sprite("terrain", "1")`.
  assert_eq!( Some( sprite_id ), compiled.ids.sprite( "terrain", "1" ) );
}

#[ test ]
fn variant_random_still_unsupported()
{
  let mut spec = minimal_spec();
  spec.objects[ 0 ] = grass_with_source
  (
    SpriteSource::Variant
    {
      variants : vec!
      [
        Variant
        {
          sprite : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ) ),
          weight : 1,
        },
      ],
      selection : VariantSelection::Random,
    }
  );

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let err = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap_err();
  assert!( matches!( err, CompileError::UnsupportedSource { .. } ),
    "Random variants still aren't supported; got {err:?}" );
}

#[ test ]
fn animation_frame_advances_with_time()
{
  // 4-frame animation at 10 fps — frames indexed "0".."3" in the atlas.
  let mut spec = minimal_spec();
  spec.animations.push
  (
    Animation
    {
      id : "water_flow".into(),
      timing : AnimationTiming::Regular
      {
        frames : vec!
        [
          SpriteRef( "terrain".into(), "0".into() ),
          SpriteRef( "terrain".into(), "1".into() ),
          SpriteRef( "terrain".into(), "2".into() ),
          SpriteRef( "terrain".into(), "3".into() ),
        ],
        fps : 10.0,
      },
      mode : AnimationMode::Loop,
      phase_offset : PhaseOffset::None,
    }
  );
  spec.objects[ 0 ] = grass_with_source( SpriteSource::Animation( AnimationRef( "water_flow".into() ) ) );

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };

  let cmds_t0 = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let cmds_t1 = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.15 ).unwrap();

  let spr_of = | cmds : &[ tilemap_renderer::commands::RenderCommand ] |
  {
    cmds.iter().find_map( | c |
      if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
    ).unwrap()
  };

  // t=0 → frame 0; t=0.15 s (at 10 fps = frame 1).
  assert_eq!( Some( spr_of( &cmds_t0 ) ), compiled.ids.sprite( "terrain", "0" ),
    "t=0 should show frame 0" );
  assert_eq!( Some( spr_of( &cmds_t1 ) ), compiled.ids.sprite( "terrain", "1" ),
    "t=0.15s at 10fps should show frame 1" );
}

#[ test ]
fn phase_offset_hashcoord_spreads_frames_across_tiles()
{
  let mut spec = minimal_spec();
  spec.animations.push
  (
    Animation
    {
      id : "water_flow".into(),
      timing : AnimationTiming::Regular
      {
        frames : vec!
        [
          SpriteRef( "terrain".into(), "0".into() ),
          SpriteRef( "terrain".into(), "1".into() ),
          SpriteRef( "terrain".into(), "2".into() ),
          SpriteRef( "terrain".into(), "3".into() ),
        ],
        fps : 4.0,
      },
      mode : AnimationMode::Loop,
      phase_offset : PhaseOffset::HashCoord,
    }
  );
  spec.objects[ 0 ] = grass_with_source( SpriteSource::Animation( AnimationRef( "water_flow".into() ) ) );

  // Sixteen tiles at the same global time. Phase offset should scatter them
  // across the period so at least a couple of different frames are visible.
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let tiles : Vec< Tile > =
    ( 0..16 ).map( | q | Tile { pos : ( q, 0 ), objects : vec![ "grass".into() ] } ).collect();
  let scene = Scene { tiles, ..minimal_scene_3x3() };
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();

  let sprites : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite.inner() ) } else { None }
  ).collect();

  assert!( sprites.len() >= 2, "HashCoord phase offset should spread frames across neighbouring tiles; saw {} unique", sprites.len() );
}
