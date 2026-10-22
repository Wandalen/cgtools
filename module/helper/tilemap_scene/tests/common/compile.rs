//! Compile-pipeline fixtures shared by the `*_compile_test.rs` integration
//! files: a one-object spec, a 3×3 scene of it, and render helpers that
//! flatten the batch stream back to per-sprite commands.

extern crate alloc;

use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;
use tilemap_renderer::commands::RenderCommand;
use tilemap_scene::
{
  Anchor,
  Asset,
  AssetKind,
  Bounds,
  Camera,
  CompileError,
  HexConfig,
  LayerBehaviour,
  MipmapMode,
  Object,
  ObjectLayer,
  PathResolver,
  PipelineLayer,
  Renderer,
  RenderPipeline,
  RenderSpec,
  SamplerFilter,
  Scene,
  SceneSnapshot,
  SortMode,
  SortYSource,
  SpriteRef,
  SpriteSource,
  Tile,
  TilingStrategy,
  TriBlendPattern,
  WrapMode,
};

/// An `Atlas` kind with 72×64 tiles, `columns` columns and the named
/// frames `pairs` ( name → ( col, row ) ).
pub fn atlas_with_frames( columns : u32, pairs : &[ ( &str, ( u32, u32 ) ) ] ) -> AssetKind
{
  let mut frames = HashMap::default();
  for ( name, pos ) in pairs
  {
    frames.insert( ( *name ).to_string(), *pos );
  }
  AssetKind::Atlas { tile_size : ( 72, 64 ), columns, origin : ( 0, 0 ), gap : ( 0, 0 ), frames, frame_rects : HashMap::default(), image_size : None }
}

/// The `grass` object: a `Hex` anchor on the `terrain` layer with
/// `priority: 10` and one `Static( terrain:0 )` layer.
pub fn grass_object() -> Object
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
        sprite_source : SpriteSource::Static( SpriteRef { asset : "terrain".into(), frame : "0".into() } ),
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
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states : anims,
  }
}

/// A flat-top spec with one `terrain` atlas, the [`grass_object`] and one
/// `terrain` pipeline layer.
pub fn minimal_spec() -> RenderSpec
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
        PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None, alpha_clip : 0.0, occlude_overlap : false, opaque : false },
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  }
}

/// A 3×3 scene with `grass` on every tile.
pub fn minimal_scene_3x3() -> SceneSnapshot
{
  let mut scene = SceneSnapshot::new( Bounds { min : ( 0, 0 ), max : ( 2, 2 ) } );
  for r in 0..3
  {
    for q in 0..3
    {
      scene.tiles.push( Tile { pos : ( q, r ), objects : vec![ "grass".into() ] } );
    }
  }
  scene
}

/// Renders `snap` after ticking its clock to `t`, flattened to per-sprite
/// commands.
pub fn at_time_compile( spec : &RenderSpec, snap : &SceneSnapshot, camera : &Camera, t : f32 ) -> Vec< RenderCommand >
{
  let mut renderer = Renderer::new( spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::from_snapshot( snap, Arc::new( spec.clone() ) ).expect( "scene" );
  scene.tick( t );
  let raw = renderer.render( &scene, camera ).expect( "render" );
  super::commands_to_sprites( raw )
}

/// Try to render. Returns a `Result` so error-path tests can assert on
/// the specific [`CompileError`] variant.
pub fn try_compile( spec : &RenderSpec, snap : &SceneSnapshot, camera : &Camera ) -> Result< Vec< RenderCommand >, CompileError >
{
  let mut renderer = Renderer::new( spec, &PathResolver )?;
  let scene = Scene::from_snapshot( snap, Arc::new( spec.clone() ) ).expect( "snap valid" );
  let raw = renderer.render( &scene, camera )?;
  Ok( super::commands_to_sprites( raw ) )
}

/// The world-space `Sprite` commands in `commands`.
pub fn sprite_commands( commands : &[ RenderCommand ] ) -> Vec< &tilemap_renderer::commands::Sprite >
{
  commands.iter().filter_map( | c | match c
  {
    RenderCommand::Sprite( s ) => Some( s ),
    _ => None,
  }).collect()
}

/// Build the single-terrain dual-grid spec used by the orient_to_grid tests:
/// one `hexagon` object that both marks terrain (`priority`) and carries the
/// `VertexCorners` layer (`orient_to_grid: true`) routing the canonical
/// full/edge/corner frames.
pub fn dual_orient_spec() -> RenderSpec
{
  let mut spec = minimal_spec();
  spec.pipeline.hex.grid_stride = ( 96, 111 );
  spec.objects.clear();
  // Pre-baked oriented frames: full has 2 (▲/▽), edge and corner have 6 each
  // (the void / lone-hex points in any of 6 directions). Positions are
  // irrelevant to these tests — only that each frame exists in the atlas.
  let frames : Vec< ( &str, ( u32, u32 ) ) > = vec!
  [
    ( "dual_full_0", ( 0, 0 ) ), ( "dual_full_1", ( 1, 0 ) ),
    ( "dual_edge_0", ( 2, 0 ) ), ( "dual_edge_1", ( 3, 0 ) ),
    ( "dual_edge_2", ( 0, 1 ) ), ( "dual_edge_3", ( 1, 1 ) ),
    ( "dual_edge_4", ( 2, 1 ) ), ( "dual_edge_5", ( 3, 1 ) ),
    ( "dual_corner_0", ( 0, 2 ) ), ( "dual_corner_1", ( 1, 2 ) ),
    ( "dual_corner_2", ( 2, 2 ) ), ( "dual_corner_3", ( 3, 2 ) ),
    ( "dual_corner_4", ( 0, 3 ) ), ( "dual_corner_5", ( 1, 3 ) ),
  ];
  spec.assets.push
  (
    Asset
    {
      id : "dual".into(),
      path : "dual.png".into(),
      kind : atlas_with_frames( 4, &frames ),
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
      premultiplied : false,
    }
  );
  spec.objects.push( Object
  {
    id : "hexagon".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : Some( 10 ),
    sort_y_source : SortYSource::default(),
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
                TriBlendPattern { corners : ( "hexagon".into(), "hexagon".into(), "hexagon".into() ), sprite_pattern : "dual_full_{rot}".into(),   priority : 30, animation : None },
                TriBlendPattern { corners : ( "hexagon".into(), "hexagon".into(), "void".into() ),    sprite_pattern : "dual_edge_{rot}".into(),   priority : 20, animation : None },
                TriBlendPattern { corners : ( "hexagon".into(), "void".into(), "void".into() ),       sprite_pattern : "dual_corner_{rot}".into(), priority : 10, animation : None },
              ],
              asset : "dual".into(),
              orient_to_grid : true,
              corner_source : None,
              offset : None,
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
  spec
}
