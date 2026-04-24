//! Integration tests for the Slice-1 compile layer
//! (`scene_model::compile::compile_assets` + `compile_frame`).


#![ allow( clippy::min_ident_chars ) ]
// Test-only idioms: exact array comparisons and ref-in-closure patterns are
// intentional; `Default::default()` reads fine at fixture build sites;
// fixture builders sometimes exceed the 100-line heuristic.
#![ allow
(
  clippy::float_cmp,
  clippy::default_trait_access,
  clippy::redundant_closure_for_method_calls,
  clippy::needless_borrow,
  clippy::too_many_lines,
  clippy::doc_markdown,
) ]

use rustc_hash::FxHashMap as HashMap;

use tilemap_renderer::commands::RenderCommand;
use tilemap_scene::
{
  Anchor,
  Animation,
  AnimationMode,
  AnimationRef,
  AnimationTiming,
  Asset,
  AssetKind,
  AssetResolver,
  AutotileLayout,
  BlendMode,
  Bounds,
  Camera,
  CompileError,
  Condition,
  EdgeConnectedLayout,
  EdgeDirection,
  EdgeInstance,
  EdgePosition,
  FreeInstance,
  HexConfig,
  LayerBehaviour,
  NeighborBitmaskSource,
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
  Tint,
  TintRef,
  TriBlendPattern,
  Variant,
  VariantSelection,
  ViewportAnchorPoint,
  ViewportInstance,
  ViewportTiling,
  compile_assets,
  compile_frame,
};
use tilemap_renderer::assets::ImageSource;
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

// ────────────────────────────────────────────────────────────────────────────
// Fixture builders.
// ────────────────────────────────────────────────────────────────────────────

fn atlas_with_frames( columns : u32, pairs : &[ ( &str, ( u32, u32 ) ) ] ) -> AssetKind
{
  let mut frames = HashMap::default();
  for ( name, pos ) in pairs
  {
    frames.insert( ( *name ).to_string(), *pos );
  }
  AssetKind::Atlas { tile_size : ( 72, 64 ), columns, origin : ( 0, 0 ), gap : ( 0, 0 ), frames, frame_rects : HashMap::default() }
}

fn grass_object() -> Object
{
  let mut anims = HashMap::default();
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
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states : anims,
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
        kind : AssetKind::Atlas
        {
          tile_size : ( 72, 64 ),
          columns : 2,
          origin : ( 0, 0 ),
          gap : ( 0, 0 ),
          frames : HashMap::default(),
          frame_rects : HashMap::default(),
        },
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
        grid_stride : ( 72, 64 ),
      },
      layers : vec!
      [
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
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
  let mut anims = HashMap::default();
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
  spec.objects[ 0 ].states = anims;

  let compiled = compile_assets( &spec, &PathResolver ).expect( "compile" );
  let sprite = &compiled.assets.sprites[ 0 ];
  assert_eq!( sprite.region, [ 72.0, 64.0, 72.0, 64.0 ], "frame 3 at (col 1, row 1)" );
}

#[ test ]
fn compile_assets_single_kind_region_matches_size()
{
  // `Single` is "one image, one sprite" — the whole PNG is the region.
  // Any frame name resolves; we conventionally use the asset id.
  let mut spec = minimal_spec();
  spec.assets[ 0 ].kind = AssetKind::Single { size : ( 256, 128 ) };
  // Point the grass object's sprite source at the single-image asset.
  spec.objects[ 0 ].states.get_mut( "default" ).unwrap()[ 0 ].sprite_source
    = SpriteSource::Static( SpriteRef( "terrain".into(), "terrain".into() ) );

  let compiled = compile_assets( &spec, &PathResolver ).expect( "Single should resolve" );
  assert_eq!( compiled.assets.sprites.len(), 1 );
  assert_eq!( compiled.assets.sprites[ 0 ].region, [ 0.0, 0.0, 256.0, 128.0 ] );
}

#[ test ]
fn compile_assets_accepts_named_atlas_frame()
{
  // Atlas frames may be numeric (resolved via `columns`) OR declared by name
  // in `Atlas.frames`. Named frames carry their exact `( col, row )` so the
  // compile layer can produce a precise sprite region.
  let mut spec = minimal_spec();
  if let AssetKind::Atlas { frames, .. } = &mut spec.assets[ 0 ].kind
  {
    frames.insert( "grass_01".into(), ( 1, 2 ) );
  }
  spec.objects[ 0 ].states.get_mut( "default" ).unwrap()[ 0 ].sprite_source =
    SpriteSource::Static( SpriteRef( "terrain".into(), "grass_01".into() ) );
  let compiled = compile_assets( &spec, &PathResolver ).expect( "named frames should compile" );
  let id = compiled.ids.sprite( "terrain", "grass_01" ).expect( "sprite allocated" );
  let sprite = compiled.assets.sprites.iter().find( | s | s.id == id ).unwrap();
  // (col=1, row=2) × (72, 64) → x=72, y=128.
  assert_eq!( sprite.region, [ 72.0, 128.0, 72.0, 64.0 ], "region must come from manifest entry" );
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

  let mut palette = HashMap::default();
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
    palette : HashMap::default(),
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
fn rejects_multihex_anchor()
{
  // Edge / FreePos / Viewport are now supported (Slice 4). Multihex is the
  // last anchor still rejected at compile time.
  let mut spec = minimal_spec();
  spec.objects[ 0 ].anchor = Anchor::Multihex { shape : vec![ ( 0, 0 ), ( 1, 0 ) ] };
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let err = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap_err();
  assert!
  (
    matches!( err, CompileError::UnsupportedAnchor { ref anchor, .. } if *anchor == "Multihex" ),
    "expected UnsupportedAnchor/Multihex, got {err:?}",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Slice 2 — Animation + Variant + PhaseOffset support.
// ────────────────────────────────────────────────────────────────────────────

fn grass_with_source( source : SpriteSource ) -> Object
{
  let mut g = grass_object();
  g.states.get_mut( "default" ).unwrap()[ 0 ].sprite_source = source;
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
fn variant_random_deterministic_across_frames()
{
  // With a fixed `Scene.seed`, Random selection picks the same variant at
  // each (q, r) across recompilations — no flicker.
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
          sprite : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "3".into() ) ) ),
          weight : 1,
        },
      ],
      selection : VariantSelection::Random,
    }
  );

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 1, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 2, 0 ), objects : vec![ "grass".into() ] },
    ],
    seed : Some( 0xDEADBEEF ),
    ..minimal_scene_3x3()
  };

  let ids_a = sprite_ids_from( compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap() );
  let ids_b = sprite_ids_from( compile_frame( &spec, &scene, &compiled, &Camera::default(), 1.5 ).unwrap() );
  assert_eq!( ids_a, ids_b, "Random selection must be deterministic for the same seed + coord" );
}

fn sprite_ids_from( commands : Vec< RenderCommand > ) -> Vec< tilemap_renderer::types::ResourceId< tilemap_renderer::types::asset::Sprite > >
{
  commands.into_iter().filter_map( | c | match c
  {
    RenderCommand::Sprite( s ) => Some( s.sprite ),
    _ => None,
  }).collect()
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

// ────────────────────────────────────────────────────────────────────────────
// Slice 3 — NeighborBitmask / NeighborCondition / VertexCorners.
// ────────────────────────────────────────────────────────────────────────────

/// A minimal spec whose grass object is replaced by `wall_object` (an
/// autotile). Useful as a base for NeighborBitmask tests.
fn wall_spec() -> RenderSpec
{
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "walls".into(),
      path : "walls.png".into(),
      kind : AssetKind::Atlas
      {
        tile_size : ( 72, 64 ),
        columns : 8,
        origin : ( 0, 0 ),
        gap : ( 0, 0 ),
        frames : HashMap::default(),
        frame_rects : HashMap::default(),
      },
      filter : Default::default(),
      mipmap : Default::default(),
      wrap : Default::default(),
    }
  );
  let wall = Object
  {
    id : "stone_wall".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::NeighborBitmask
            {
              connects_with : vec![ "stone_wall".into() ],
              source : NeighborBitmaskSource::ByAtlas
              {
                asset : "walls".into(),
                layout : AutotileLayout::Bitmask6,
              },
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  };
  spec.objects.push( wall );
  spec
}

#[ test ]
fn autotile_isolated_cell_mask_zero()
{
  let spec = wall_spec();
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "stone_wall".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprite_id = cmds.iter().find_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).unwrap();
  // ByAtlas: isolated wall → mask 0 → sprite id allocated for frame "0" of the walls atlas.
  assert_eq!( Some( sprite_id ), compiled.ids.sprite( "walls", "0" ) );
}

#[ test ]
fn autotile_two_cell_line()
{
  // Two flat-top walls at (0,0) and (1,-1) — the latter is the N-axis neighbour (direction index 1 = NE for flat-top,
  // but SPEC §2.3 says flat-top N offset is (0,-1). So (1,-1) is NE of origin.
  // (0,0) sees neighbour at index 1 (NE) → bit 1 = 0b000010.
  // (1,-1) sees neighbour at index 4 (SW, offset (-1, +1)) → bit 4 = 0b010000.
  let spec = wall_spec();
  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0,  0 ), objects : vec![ "stone_wall".into() ] },
      Tile { pos : ( 1, -1 ), objects : vec![ "stone_wall".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprite_ids : Vec< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();
  assert_eq!( sprite_ids.len(), 2 );
  let expected_ne = compiled.ids.sprite( "walls", "2" );   // 0b000010
  let expected_sw = compiled.ids.sprite( "walls", "16" );  // 0b010000
  assert!( sprite_ids.contains( &expected_ne.unwrap() ), "no NE-facing mask 2 sprite in {sprite_ids:?}" );
  assert!( sprite_ids.contains( &expected_sw.unwrap() ), "no SW-facing mask 16 sprite in {sprite_ids:?}" );
}

#[ test ]
fn autotile_connects_with_void_at_map_edge()
{
  let mut spec = wall_spec();
  // Replace the wall object's connects_with to include "void".
  let wall = spec.objects.iter_mut().find( | o | o.id == "stone_wall" ).unwrap();
  let stack = wall.states.get_mut( "default" ).unwrap();
  if let SpriteSource::NeighborBitmask { connects_with, .. } = &mut stack[ 0 ].sprite_source
  {
    connects_with.push( "void".into() );
  }
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "stone_wall".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprite_id = cmds.iter().find_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).unwrap();
  // All 6 neighbours are void ⇒ mask = 0b111111 = 63.
  assert_eq!( Some( sprite_id ), compiled.ids.sprite( "walls", "63" ) );
}

#[ test ]
fn neighbor_condition_skirt_on_water_side()
{
  // Grass at (0,0), water at (0,1). Grass emits a skirt sprite on side S
  // (direction index 3, axial offset (0, +1)).
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "skirts".into(),
      path : "skirts.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "grass_side_s",  ( 0, 0 ) ),
          ( "grass_side_sw", ( 1, 0 ) ),
          ( "grass_side_se", ( 2, 0 ) ),
        ],
      ),
      filter : Default::default(),
      mipmap : Default::default(),
      wrap : Default::default(),
    }
  );
  // Add a water object.
  spec.objects.push( Object
  {
    id : "water".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : Some( 1 ),
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ),
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  // Rewire grass to have a skirt layer on S that triggers for water neighbours.
  let grass = spec.objects.iter_mut().find( | o | o.id == "grass" ).unwrap();
  let stack = grass.states.get_mut( "default" ).unwrap();
  stack.push( ObjectLayer
  {
    id : Some( "skirt".into() ),
    sprite_source : SpriteSource::NeighborCondition
    {
      condition : Condition::NeighborIs( vec![ "water".into() ] ),
      sides : vec![ EdgeDirection::S, EdgeDirection::SW, EdgeDirection::SE ],
      sprite_pattern : "grass_side_{dir}".into(),
      asset : "skirts".into(),
    },
    behaviour : LayerBehaviour::default(),
    z_in_object : 1,
    pipeline_layer : None,
  });

  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 0, 1 ), objects : vec![ "water".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();

  let sprite_ids : Vec< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();
  // Expect: grass base + water base + one skirt sprite (side S matches).
  // Side SW: neighbour at offset (-1, +1) = (-1, 1) — empty (void) → doesn't match NeighborIs(water).
  // Side SE: neighbour at offset (+1, 0) = (1, 0) — empty → doesn't match.
  let skirt_id = compiled.ids.sprite( "skirts", "grass_side_s" ).unwrap();
  assert!( sprite_ids.contains( &skirt_id ), "south-side skirt sprite missing; saw {sprite_ids:?}" );
}

#[ test ]
fn neighbor_condition_priority_lower_blends_grass_over_sand()
{
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "edges".into(),
      path : "edges.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "grass_edge_n",  ( 0, 0 ) ),
          ( "grass_edge_ne", ( 1, 0 ) ),
          ( "grass_edge_se", ( 2, 0 ) ),
          ( "grass_edge_s",  ( 3, 0 ) ),
          ( "grass_edge_sw", ( 4, 0 ) ),
          ( "grass_edge_nw", ( 5, 0 ) ),
          ( "sand_edge_n",   ( 0, 1 ) ),
          ( "sand_edge_ne",  ( 1, 1 ) ),
          ( "sand_edge_se",  ( 2, 1 ) ),
          ( "sand_edge_s",   ( 3, 1 ) ),
          ( "sand_edge_sw",  ( 4, 1 ) ),
          ( "sand_edge_nw",  ( 5, 1 ) ),
        ],
      ),
      filter : Default::default(),
      mipmap : Default::default(),
      wrap : Default::default(),
    }
  );
  // Grass prio 10 (already in grass_object).
  spec.objects.push( Object
  {
    id : "sand".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : Some( 8 ),
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ),
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  let grass = spec.objects.iter_mut().find( | o | o.id == "grass" ).unwrap();
  grass.states.get_mut( "default" ).unwrap().push( ObjectLayer
  {
    id : Some( "edges".into() ),
    sprite_source : SpriteSource::NeighborCondition
    {
      condition : Condition::NeighborPriorityLower,
      sides : vec![ EdgeDirection::N, EdgeDirection::NE, EdgeDirection::SE, EdgeDirection::S, EdgeDirection::SW, EdgeDirection::NW ],
      sprite_pattern : "grass_edge_{dir}".into(),
      asset : "edges".into(),
    },
    behaviour : LayerBehaviour::default(),
    z_in_object : 1,
    pipeline_layer : None,
  });
  // Symmetric sand edge layer — should NOT emit (sand has lower priority than grass).
  let sand = spec.objects.iter_mut().find( | o | o.id == "sand" ).unwrap();
  sand.states.get_mut( "default" ).unwrap().push( ObjectLayer
  {
    id : Some( "edges".into() ),
    sprite_source : SpriteSource::NeighborCondition
    {
      condition : Condition::NeighborPriorityLower,
      sides : vec![ EdgeDirection::N, EdgeDirection::NE, EdgeDirection::SE, EdgeDirection::S, EdgeDirection::SW, EdgeDirection::NW ],
      sprite_pattern : "sand_edge_{dir}".into(),
      asset : "edges".into(),
    },
    behaviour : LayerBehaviour::default(),
    z_in_object : 1,
    pipeline_layer : None,
  });

  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] },
      Tile { pos : ( 0, 1 ), objects : vec![ "sand".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();

  let sprite_ids : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();

  // grass at (0,0) with sand to S has higher priority → emits "grass_edge_s".
  let grass_edge_s = compiled.ids.sprite( "edges", "grass_edge_s" ).unwrap();
  assert!( sprite_ids.contains( &grass_edge_s ), "grass_edge_s missing; {sprite_ids:?}" );

  // sand at (0,1) with grass to N has lower priority → must NOT emit "sand_edge_n".
  let sand_edge_n = compiled.ids.sprite( "edges", "sand_edge_n" ).unwrap();
  assert!( !sprite_ids.contains( &sand_edge_n ), "sand_edge_n should NOT emit — sand has lower priority" );
}

#[ test ]
fn vertex_corners_three_way_blend()
{
  // Three tiles surrounding a vertex: grass at (0,0), sand at (1,-1), water at (0,-1).
  // These three hexes share exactly one dual-mesh triangle (by construction).
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "blends".into(),
      path : "blends.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "tri_gsw_0", ( 0, 0 ) ),
          ( "tri_gsw_1", ( 1, 0 ) ),
          ( "tri_gsw_2", ( 2, 0 ) ),
        ],
      ),
      filter : Default::default(),
      mipmap : Default::default(),
      wrap : Default::default(),
    }
  );

  // Terrains grass/sand/water.
  for ( id, prio ) in [ ( "sand", 8 ), ( "water", 5 ) ]
  {
    spec.objects.push( Object
    {
      id : id.into(),
      anchor : Anchor::Hex,
      global_layer : "terrain".into(),
      priority : Some( prio ),
      sort_y_source : Default::default(),
      pivot : ( 0.5, 0.5 ),
      default_state : "default".into(),
      states :
      {
        let mut m = HashMap::default();
        m.insert
        (
          "default".into(),
          vec!
          [
            ObjectLayer
            {
              id : None,
              sprite_source : SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ),
              behaviour : LayerBehaviour::default(),
              z_in_object : 0,
              pipeline_layer : None,
            },
          ],
        );
        m
      },
    });
  }

  // VertexCorners object — its own default animation has a single layer
  // with a pattern that matches the gss/sand/water triple.
  spec.objects.push( Object
  {
    id : "blend".into(),
    anchor : Anchor::Hex,   // anchor type of the owning object doesn't matter for VertexCorners pass
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern
                {
                  corners : ( "grass".into(), "sand".into(), "water".into() ),
                  sprite_pattern : "tri_gsw_{rot}".into(),
                  priority : 10,
                  animation : None,
                },
              ],
              asset : "blends".into(),
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  // Instantiate the blend object once on any tile so the VertexCorners pass
  // finds it during bucket emission. (Object presence is what matters, not
  // the tile — vertex sprites are global per-bucket.)
  let scene = Scene
  {
    tiles : vec!
    [
      Tile { pos : ( 0,  0 ), objects : vec![ "grass".into(), "blend".into() ] },
      Tile { pos : ( 1, -1 ), objects : vec![ "sand".into() ] },
      Tile { pos : ( 0, -1 ), objects : vec![ "water".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();

  let sprite_ids : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();

  // The triangle surrounding the shared vertex should have produced a
  // tri_gsw_<rot> sprite for some rotation in 0..3.
  let any_rot_emitted = ( 0..3 ).any( | r |
  {
    let id = compiled.ids.sprite( "blends", &format!( "tri_gsw_{r}" ) );
    id.is_some() && sprite_ids.contains( &id.unwrap() )
  });
  assert!( any_rot_emitted, "expected any rotation of tri_gsw to emit; sprite_ids = {sprite_ids:?}" );
}

#[ test ]
fn vertex_corners_wildcard_edge_fade()
{
  // An isolated tile of grass — every dual triangle has 2 void corners.
  // A wildcard pattern ("*", "*", "void") should cover each.
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "fades".into(),
      path : "fades.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "edge_fade_0", ( 0, 0 ) ),
          ( "edge_fade_1", ( 1, 0 ) ),
          ( "edge_fade_2", ( 2, 0 ) ),
        ],
      ),
      filter : Default::default(),
      mipmap : Default::default(),
      wrap : Default::default(),
    }
  );
  spec.objects.push( Object
  {
    id : "fade".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern
                {
                  corners : ( "*".into(), "*".into(), "void".into() ),
                  sprite_pattern : "edge_fade_{rot}".into(),
                  priority : 0,
                  animation : None,
                },
              ],
              asset : "fades".into(),
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });

  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into(), "fade".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let cmds = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();

  let emitted : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();

  // Six triangles around the isolated hex, all with 2 void corners → all
  // should match the wildcard fade pattern. We just assert at least one
  // edge_fade_* sprite emitted.
  let any_fade = ( 0..3 ).any( | r |
  {
    let id = compiled.ids.sprite( "fades", &format!( "edge_fade_{r}" ) );
    id.is_some() && emitted.contains( &id.unwrap() )
  });
  assert!( any_fade, "expected wildcard fade to match at least one triangle; emitted = {emitted:?}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Slice 4 — Edge / FreePos / Viewport anchors.
// ────────────────────────────────────────────────────────────────────────────

fn static_object_with_anchor( id : &str, anchor : Anchor, sprite : SpriteRef ) -> Object
{
  let mut anims = HashMap::default();
  anims.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::Static( sprite ),
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  Object
  {
    id : id.into(),
    anchor,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states : anims,
  }
}

fn sprite_commands( commands : &[ RenderCommand ] ) -> Vec< &tilemap_renderer::commands::Sprite >
{
  commands.iter().filter_map( | c | match c
  {
    RenderCommand::Sprite( s ) => Some( s ),
    _ => None,
  }).collect()
}

fn screen_space_commands( commands : &[ RenderCommand ] ) -> Vec< &tilemap_renderer::commands::Sprite >
{
  commands.iter().filter_map( | c | match c
  {
    RenderCommand::ScreenSpaceSprite( s ) => Some( s ),
    _ => None,
  }).collect()
}

#[ test ]
fn edge_instance_emits_single_sprite()
{
  // Edge declared from both sides should emit exactly one sprite.
  let mut spec = minimal_spec();
  spec.objects.push( static_object_with_anchor
  (
    "river",
    Anchor::Edge,
    SpriteRef( "terrain".into(), "0".into() ),
  ));
  let scene = Scene
  {
    tiles : Vec::new(),
    edges : vec!
    [
      EdgeInstance
      {
        at : EdgePosition { hex : ( 0, 0 ), dir : EdgeDirection::N },
        object : "river".into(),
        animation : None,
      },
      EdgeInstance
      {
        at : EdgePosition { hex : ( 0, -1 ), dir : EdgeDirection::S },
        object : "river".into(),
        animation : None,
      },
    ],
    ..minimal_scene_3x3()
  };

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprites = sprite_commands( &commands );
  assert_eq!( sprites.len(), 1, "canonicalisation should dedupe both declarations" );
}

#[ test ]
fn edge_rotation_matches_direction()
{
  use core::f32::consts::PI;
  let mut spec = minimal_spec();
  spec.objects.push( static_object_with_anchor
  (
    "river",
    Anchor::Edge,
    SpriteRef( "terrain".into(), "0".into() ),
  ));

  // Canonicalisation picks the lex-smaller hex and flips the direction as
  // needed. With hex (5,5) on flat-top, these three directions (NE, SE, S)
  // stay canonical — neighbour hex is > (5,5) in tuple order.
  let cases : &[ ( EdgeDirection, ( i32, i32 ), f32 ) ] =
  &[
    ( EdgeDirection::NE, ( 5, 5 ), PI / 3.0 ),
    ( EdgeDirection::SE, ( 5, 5 ), 2.0 * PI / 3.0 ),
    ( EdgeDirection::S,  ( 5, 5 ), PI ),
  ];

  for ( dir, at_hex, expected ) in cases
  {
    let scene = Scene
    {
      tiles : Vec::new(),
      edges : vec!
      [
        EdgeInstance
        {
          at : EdgePosition { hex : *at_hex, dir : *dir },
          object : "river".into(),
          animation : None,
        },
      ],
      ..minimal_scene_3x3()
    };
    let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
    let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
    let sprites = sprite_commands( &commands );
    assert_eq!( sprites.len(), 1 );
    assert!(
      ( sprites[ 0 ].transform.rotation - expected ).abs() < 1e-4,
      "dir {dir:?} expected rotation {expected}, got {}",
      sprites[ 0 ].transform.rotation,
    );
  }
}

#[ test ]
fn edge_connected_bitmask_isolated()
{
  // Single river edge, no neighbours → mask = 0, picks the "0" frame.
  let mut spec = minimal_spec();
  let bmsource = NeighborBitmaskSource::ByAtlas
  {
    asset : "terrain".into(),
    layout : AutotileLayout::Bitmask6,
  };
  let mut states = HashMap::default();
  states.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::EdgeConnectedBitmask
        {
          connects_with : vec![ "river".into() ],
          source : bmsource,
          layout : EdgeConnectedLayout::EdgeHex,
        },
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  spec.objects.push( Object
  {
    id : "river".into(),
    anchor : Anchor::Edge,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  });

  let scene = Scene
  {
    tiles : Vec::new(),
    edges : vec!
    [
      EdgeInstance
      {
        at : EdgePosition { hex : ( 0, 0 ), dir : EdgeDirection::N },
        object : "river".into(),
        animation : None,
      },
    ],
    ..minimal_scene_3x3()
  };

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprites = sprite_commands( &commands );
  assert_eq!( sprites.len(), 1 );
  let mask_zero_id = compiled.ids.sprite( "terrain", "0" ).unwrap();
  assert_eq!( sprites[ 0 ].sprite, mask_zero_id );
}

#[ test ]
fn free_pos_emits_at_instance_position()
{
  let mut spec = minimal_spec();
  spec.objects.push( static_object_with_anchor
  (
    "bullet",
    Anchor::FreePos,
    SpriteRef( "terrain".into(), "0".into() ),
  ));
  let scene = Scene
  {
    tiles : Vec::new(),
    free_instances : vec!
    [
      FreeInstance
      {
        pos : ( 37.5, -12.0 ),
        object : "bullet".into(),
        animation : None,
      },
    ],
    ..minimal_scene_3x3()
  };

  let camera = Camera { world_center : ( 0.0, 0.0 ), zoom : 1.0, viewport_size : ( 800, 600 ) };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &camera, 0.0 ).unwrap();
  let sprites = sprite_commands( &commands );
  assert_eq!( sprites.len(), 1 );
  // Camera project: (wx - 0) * 1 + 400 = 437.5; pivot (0.5, 0.5) over 72x64
  // sprite shifts by (-36, -32). Final: (437.5 - 36, 300 - 12 - 32) = (401.5, 256).
  assert!( ( sprites[ 0 ].transform.position[ 0 ] - 401.5 ).abs() < 1e-3 );
  assert!( ( sprites[ 0 ].transform.position[ 1 ] - 256.0 ).abs() < 1e-3 );
}

#[ test ]
fn free_pos_rejects_neighbour_aware_source()
{
  let mut spec = minimal_spec();
  let bmsource = NeighborBitmaskSource::ByAtlas
  {
    asset : "terrain".into(),
    layout : AutotileLayout::Bitmask6,
  };
  let mut states = HashMap::default();
  states.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::NeighborBitmask
        {
          connects_with : vec![ "grass".into() ],
          source : bmsource,
        },
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  spec.objects.push( Object
  {
    id : "bad_free".into(),
    anchor : Anchor::FreePos,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  });
  let scene = Scene
  {
    tiles : Vec::new(),
    free_instances : vec!
    [
      FreeInstance
      {
        pos : ( 0.0, 0.0 ),
        object : "bad_free".into(),
        animation : None,
      },
    ],
    ..minimal_scene_3x3()
  };

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let err = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap_err();
  assert!(
    matches!( err, CompileError::UnsupportedSource { .. } ),
    "expected UnsupportedSource, got {err:?}",
  );
}

#[ test ]
fn viewport_center_emits_screen_space_sprite()
{
  let mut spec = minimal_spec();
  let mut states = HashMap::default();
  states.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::ViewportTiled
        {
          content : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ) ),
          tiling : ViewportTiling::Center,
          anchor_point : ViewportAnchorPoint::TopLeft,
        },
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  spec.objects.push( Object
  {
    id : "sky".into(),
    anchor : Anchor::Viewport,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  });
  let scene = Scene
  {
    tiles : Vec::new(),
    viewport_instances : vec!
    [
      ViewportInstance { object : "sky".into(), animation : None },
    ],
    ..minimal_scene_3x3()
  };

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let screen = screen_space_commands( &commands );
  assert_eq!( screen.len(), 1 );
  assert_eq!( screen[ 0 ].transform.position, [ 0.0, 0.0 ] );
  assert_eq!( screen[ 0 ].transform.scale, [ 1.0, 1.0 ] );
}

#[ test ]
fn viewport_stretch_scales_to_viewport()
{
  let mut spec = minimal_spec();
  let mut states = HashMap::default();
  states.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::ViewportTiled
        {
          content : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ) ),
          tiling : ViewportTiling::Stretch,
          anchor_point : ViewportAnchorPoint::Center,
        },
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  spec.objects.push( Object
  {
    id : "bg".into(),
    anchor : Anchor::Viewport,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  });
  let scene = Scene
  {
    tiles : Vec::new(),
    viewport_instances : vec!
    [
      ViewportInstance { object : "bg".into(), animation : None },
    ],
    ..minimal_scene_3x3()
  };

  let camera = Camera { world_center : ( 0.0, 0.0 ), zoom : 1.0, viewport_size : ( 800, 600 ) };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &camera, 0.0 ).unwrap();
  let screen = screen_space_commands( &commands );
  assert_eq!( screen.len(), 1 );
  // Sprite is 72x64; viewport 800x600 → scale = (800/72, 600/64).
  assert!( ( screen[ 0 ].transform.scale[ 0 ] - ( 800.0 / 72.0 ) ).abs() < 1e-3 );
  assert!( ( screen[ 0 ].transform.scale[ 1 ] - ( 600.0 / 64.0 ) ).abs() < 1e-3 );
}

#[ test ]
fn viewport_repeat2d_tiles_to_cover_viewport()
{
  let mut spec = minimal_spec();
  let mut states = HashMap::default();
  states.insert
  (
    "default".into(),
    vec!
    [
      ObjectLayer
      {
        id : None,
        sprite_source : SpriteSource::ViewportTiled
        {
          content : Box::new( SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ) ),
          tiling : ViewportTiling::Repeat2D,
          anchor_point : ViewportAnchorPoint::TopLeft,
        },
        behaviour : LayerBehaviour::default(),
        z_in_object : 0,
        pipeline_layer : None,
      },
    ],
  );
  spec.objects.push( Object
  {
    id : "tiled_bg".into(),
    anchor : Anchor::Viewport,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  });
  let scene = Scene
  {
    tiles : Vec::new(),
    viewport_instances : vec!
    [
      ViewportInstance { object : "tiled_bg".into(), animation : None },
    ],
    ..minimal_scene_3x3()
  };

  // Viewport 800x600 with 72x64 tiles → ceil(800/72)+1 = 13 cols, ceil(600/64)+1 = 11 rows.
  let camera = Camera { world_center : ( 0.0, 0.0 ), zoom : 1.0, viewport_size : ( 800, 600 ) };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &camera, 0.0 ).unwrap();
  let screen = screen_space_commands( &commands );
  assert_eq!( screen.len(), 13 * 11, "expected full grid of screen-space sprites" );
}

#[ test ]
fn global_tint_multiplies_into_every_sprite()
{
  // Pipeline-level tint at full strength with a pure-red colour should mean
  // every emitted sprite has tint ≈ [1, 0, 0, 1] (white × red = red).
  let mut spec = minimal_spec();
  spec.tints.push( Tint
  {
    id : "dusk".into(),
    color : "#ff0000".into(),
    strength : 1.0,
    mode : BlendMode::Multiply,
  });
  spec.pipeline.global_tint = Some( TintRef( "dusk".into() ) );

  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };

  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprites = sprite_commands( &commands );
  assert_eq!( sprites.len(), 1 );
  let tint = sprites[ 0 ].tint;
  assert!( ( tint[ 0 ] - 1.0 ).abs() < 1e-5, "red channel: {tint:?}" );
  assert!( tint[ 1 ].abs() < 1e-5, "green channel zeroed: {tint:?}" );
  assert!( tint[ 2 ].abs() < 1e-5, "blue channel zeroed: {tint:?}" );
  assert!( ( tint[ 3 ] - 1.0 ).abs() < 1e-5 );
}

#[ test ]
fn global_tint_none_is_identity()
{
  // No global tint → sprites emit with white tint [1,1,1,1] (default).
  let spec = minimal_spec();
  let scene = Scene
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = compile_assets( &spec, &PathResolver ).expect( "assets" );
  let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 ).unwrap();
  let sprites = sprite_commands( &commands );
  assert_eq!( sprites[ 0 ].tint, [ 1.0, 1.0, 1.0, 1.0 ] );
}
