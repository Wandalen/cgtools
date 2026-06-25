//! Opaque/transparent two-pass split — SPEC §8.6.
//!
//! When any `PipelineLayer` sets `opaque: true` the renderer splits the frame:
//! opaque buckets draw first, front-to-back (topmost pipeline layer first) with
//! depth writes on; transparent buckets draw after in painter's order with depth
//! writes off — except an `occlude_overlap` bucket, which keeps writes on for its
//! own draws. Depth-write toggling rides the `SetDepthWrite` render command, and
//! each bucket's sprites are pinned to `pipeline_index / bucket_count`. With no
//! opaque layer the renderer takes the original single-pass path and emits no
//! `SetDepthWrite` at all.

#![ allow( clippy::min_ident_chars ) ]
#![ allow
(
  clippy::default_trait_access,
  clippy::too_many_lines,
  clippy::float_cmp,
) ]

extern crate alloc;
use alloc::sync::Arc;
use rustc_hash::FxHashMap as HashMap;

use tilemap_renderer::commands::RenderCommand;
use tilemap_scene::
{
  Anchor,
  Asset,
  AssetKind,
  Camera,
  HexConfig,
  LayerBehaviour,
  Object,
  ObjectLayer,
  PathResolver,
  PipelineLayer,
  Placement,
  Renderer,
  RenderPipeline,
  RenderSpec,
  Scene,
  SortMode,
  SpriteRef,
  SpriteSource,
  TilingStrategy,
};
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

mod common;

// ────────────────────────────────────────────────────────────────────────────
// Fixtures — three objects, one per pipeline bucket (`bg`, `mid`, `top`), all
// drawn from one shared atlas. Per-layer `opaque` / `occlude_overlap` flags are
// supplied by each test so a spec can be built with exactly the pass behaviour
// under test.
// ────────────────────────────────────────────────────────────────────────────

fn static_layer( frame : &str ) -> ObjectLayer
{
  ObjectLayer
  {
    id : None,
    sprite_source : SpriteSource::Static( SpriteRef { asset : "sheet".into(), frame : frame.into() } ),
    behaviour : LayerBehaviour::default(),
    z_in_object : 0,
    pipeline_layer : None,
  }
}

fn atlas_asset() -> Asset
{
  Asset
  {
    id : "sheet".into(),
    path : "sheet.png".into(),
    kind : AssetKind::Atlas
    {
      tile_size : ( 72, 64 ),
      columns : 4,
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
  }
}

fn object( id : &str, frame : &str, bucket : &str ) -> Object
{
  let mut states = HashMap::default();
  states.insert( "default".into(), vec![ static_layer( frame ) ] );
  Object
  {
    id : id.into(),
    anchor : Anchor::Hex,
    global_layer : bucket.into(),
    priority : None,
    sort_y_source : Default::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states,
  }
}

fn layer( id : &str, opaque : bool, occlude_overlap : bool ) -> PipelineLayer
{
  PipelineLayer
  {
    id : id.into(),
    sort : SortMode::None,
    tint_mask : None,
    alpha_clip : if opaque { 0.5 } else { 0.0 },
    occlude_overlap,
    opaque,
  }
}

/// Build a three-bucket spec. `flags[i] = (opaque, occlude_overlap)` for bucket
/// `i` in declared order: `bg` (index 0), `mid` (1), `top` (2).
fn build_spec( flags : [ ( bool, bool ); 3 ] ) -> RenderSpec
{
  RenderSpec
  {
    version : "0.2.0".into(),
    assets : vec![ atlas_asset() ],
    tints : Vec::new(),
    animations : Vec::new(),
    effects : Vec::new(),
    objects : vec!
    [
      object( "bg", "0", "bg" ),
      object( "mid", "1", "mid" ),
      object( "top", "2", "top" ),
    ],
    pipeline : RenderPipeline
    {
      hex : HexConfig { tiling : TilingStrategy::HexFlatTop, grid_stride : ( 72, 64 ) },
      layers : vec!
      [
        layer( "bg", flags[ 0 ].0, flags[ 0 ].1 ),
        layer( "mid", flags[ 1 ].0, flags[ 1 ].1 ),
        layer( "top", flags[ 2 ].0, flags[ 2 ].1 ),
      ],
      global_tint : None,
      viewport_size : None,
      clear_color : None,
    },
  }
}

fn spawn_one_per_bucket( scene : &mut Scene )
{
  for ( id, r ) in [ ( "bg", 0 ), ( "mid", 1 ), ( "top", 2 ) ]
  {
    let obj = scene.object( id ).unwrap();
    scene.spawn( obj, Placement::Hex { q : 0, r } );
  }
}

/// Depth-write toggles in command order — `true`/`false` per `SetDepthWrite`.
fn depth_write_sequence( cmds : &[ RenderCommand ] ) -> Vec< bool >
{
  cmds.iter()
    .filter_map( | c | if let RenderCommand::SetDepthWrite( s ) = c { Some( s.enabled ) } else { None } )
    .collect()
}

// ────────────────────────────────────────────────────────────────────────────
// 1. With opaque layers present, the frame is bracketed by `SetDepthWrite`:
//    enable for the opaque pass, disable for the transparent pass, restore at
//    the end. Opaque buckets are pinned front-to-back (topmost layer nearest).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn opaque_layers_bracket_passes_and_pin_depth_front_to_back()
{
  // bg + mid opaque, top transparent.
  let spec = build_spec( [ ( true, false ), ( true, false ), ( false, false ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // Opaque pass turns writes on, transparent pass turns them off, then the
  // renderer restores them to true for the next frame's opaque pass.
  assert_eq!
  (
    depth_write_sequence( &cmds ), vec![ true, false, true ],
    "expected enable (opaque) → disable (transparent) → restore",
  );

  // Three buckets, one sprite each. Opaque pass is front-to-back: the topmost
  // opaque layer (`mid`, index 1) draws before the lower one (`bg`, index 0);
  // the transparent `top` (index 2) draws last.
  let sprites = common::flat_sprites( &cmds );
  assert_eq!( sprites.len(), 3, "one sprite per bucket" );

  let depths : Vec< f32 > = sprites.iter().map( | s | s.transform.depth ).collect();
  assert_eq!
  (
    depths, vec![ 1.0 / 3.0, 0.0, 2.0 / 3.0 ],
    "opaque pass front-to-back (mid before bg), transparent `top` last; \
     each depth is pipeline_index / bucket_count",
  );
}

// ────────────────────────────────────────────────────────────────────────────
// 2. No opaque layer → original single-pass path, no `SetDepthWrite` emitted
//    and no depth pinning (every sprite keeps the default depth 0).
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn no_opaque_layer_takes_single_pass_path()
{
  let spec = build_spec( [ ( false, false ), ( false, false ), ( false, false ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  assert!
  (
    depth_write_sequence( &cmds ).is_empty(),
    "single-pass path must not emit any SetDepthWrite",
  );

  // Painter's order, no depth pinning.
  let depths : Vec< f32 > = common::flat_sprites( &cmds ).iter().map( | s | s.transform.depth ).collect();
  assert_eq!( depths, vec![ 0.0, 0.0, 0.0 ], "no opaque pass ⇒ depths untouched" );
}

// ────────────────────────────────────────────────────────────────────────────
// 3. An `occlude_overlap` transparent bucket keeps depth writes ON for its own
//    draws while the surrounding transparent pass has them off. Walk the
//    flattened stream tracking the live depth-write state at each sprite.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn occlude_overlap_keeps_depth_writes_in_transparent_pass()
{
  // bg opaque; mid plain transparent; top transparent + occlude_overlap.
  let spec = build_spec( [ ( true, false ), ( false, false ), ( false, true ) ] );
  let mut renderer = Renderer::new( &spec, &PathResolver ).expect( "renderer" );
  let mut scene = Scene::new( Arc::new( spec ) );
  spawn_one_per_bucket( &mut scene );

  let cmds = renderer.render( &scene, &Camera::default() ).expect( "render" ).to_vec();

  // Walk the flattened stream, tracking the depth-write state in force at each
  // Sprite. `SetDepthWrite` passes through the flattener unchanged.
  let flat = common::flatten_to_sprites( &cmds );
  let mut write = true; // backend default
  let mut by_depth : Vec< ( f32, bool ) > = Vec::new();
  for c in &flat
  {
    match c
    {
      RenderCommand::SetDepthWrite( s ) => write = s.enabled,
      RenderCommand::Sprite( s ) => by_depth.push( ( s.transform.depth, write ) ),
      _ => {},
    }
  }

  by_depth.sort_by( | a, b | a.0.partial_cmp( &b.0 ).unwrap() );
  // depth 0 = bg (opaque, writes on), 1/3 = mid (transparent, writes off),
  // 2/3 = top (occlude_overlap, writes back on).
  assert_eq!
  (
    by_depth,
    vec![ ( 0.0, true ), ( 1.0 / 3.0, false ), ( 2.0 / 3.0, true ) ],
    "opaque draws with writes on, plain transparent with writes off, \
     occlude_overlap restores writes for its own draw",
  );
}
