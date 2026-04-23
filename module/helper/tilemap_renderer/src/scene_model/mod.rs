//! Compositional declarative scene format for 2D tile-based games.
//!
//! This module implements the format described in `scene_model/SPEC.md` (v0.2.0).
//! It provides serde-compatible data types for describing a render spec and a
//! scene, plus a compile layer that turns them into a stream of
//! [`crate::commands::RenderCommand`]s consumable by existing backends
//! (`SvgBackend`, `WebGlBackend`).
//!
//! # One-minute overview
//!
//! Three primitives — that's the entire vocabulary:
//!
//! - [`Object`] — a renderable class (grass, water, knight, village). Has an
//!   [`Anchor`] (how it attaches to the world) and named animation stacks.
//! - [`ObjectLayer`] — one textured strip of an object. Combines a
//!   [`SpriteSource`] (what to draw) with a [`LayerBehaviour`] (tint, blend,
//!   effects).
//! - [`Anchor`] — hex cell / edge / dual-mesh vertex / multihex / world
//!   pixel / viewport. Determines what "position" means and what neighbour
//!   context is visible to the layer.
//!
//! There is no "class hierarchy" — "terrain", "unit", "overlay" are just
//! names users give their [`Object`]s. The compile layer treats them
//! uniformly.
//!
//! `Scene` describes where instances of declared objects live; [`RenderSpec`]
//! declares the objects themselves along with sampling parameters.
//!
//! # Quick start — in code, no RON required
//!
//! `Scene` and `RenderSpec` are plain structs. You can load them from RON
//! (or JSON) via serde, or construct them in memory from any source — your
//! game's own state, a JSON map loader, an ECS query. The compile pipeline
//! doesn't care.
//!
//! ```ignore
//! use std::collections::HashMap;
//! use tilemap_renderer::
//! {
//!   backend::Backend,
//!   types::RenderConfig,
//!   adapters::SvgBackend,
//!   scene_model::
//!   {
//!     Anchor, Asset, AssetKind, Bounds, Camera, HexConfig, LayerBehaviour,
//!     Object, ObjectLayer, PathResolver, PipelineLayer, RenderPipeline,
//!     RenderSpec, Scene, SortMode, SpriteRef, SpriteSource, Tile,
//!     TilingStrategy, compile_assets, compile_frame,
//!   },
//! };
//!
//! // 1. Declare a spec — one asset, one object.
//! let mut grass_anim = HashMap::new();
//! grass_anim.insert
//! (
//!   "default".into(),
//!   vec!
//!   [
//!     ObjectLayer
//!     {
//!       id : None,
//!       sprite_source : SpriteSource::Static( SpriteRef( "terrain".into(), "0".into() ) ),
//!       behaviour : LayerBehaviour::default(),
//!       z_in_object : 0,
//!       pipeline_layer : None,
//!     },
//!   ],
//! );
//!
//! let spec = RenderSpec
//! {
//!   version : "0.2.0".into(),
//!   assets : vec!
//!   [
//!     Asset
//!     {
//!       id : "terrain".into(),
//!       path : "assets/terrain.png".into(),
//!       kind : AssetKind::Atlas
//!       {
//!         tile_size : ( 72, 64 ),
//!         columns : 8,
//!         frames : HashMap::new(),
//!       },
//!       filter : Default::default(),
//!       mipmap : Default::default(),
//!       wrap : Default::default(),
//!     },
//!   ],
//!   tints : Vec::new(),
//!   animations : Vec::new(),
//!   effects : Vec::new(),
//!   objects : vec!
//!   [
//!     Object
//!     {
//!       id : "grass".into(),
//!       anchor : Anchor::Hex,
//!       global_layer : "terrain".into(),
//!       priority : Some( 10 ),
//!       sort_y_source : Default::default(),
//!       default_animation : "default".into(),
//!       animations : grass_anim,
//!     },
//!   ],
//!   pipeline : RenderPipeline
//!   {
//!     hex : HexConfig
//!     {
//!       tiling : TilingStrategy::HexFlatTop,
//!       cell_size : ( 72, 64 ),
//!     },
//!     layers : vec![ PipelineLayer { id : "terrain".into(), sort : SortMode::None, tint_mask : None } ],
//!     global_tint : None,
//!     viewport_size : None,
//!   },
//! };
//!
//! // 2. Populate a scene in code — no RON file involved.
//! let mut scene = Scene::new( Bounds { min : ( 0, 0 ), max : ( 5, 5 ) } );
//! for r in 0..5
//! {
//!   for q in 0..5
//!   {
//!     scene.tiles.push( Tile { pos : ( q, r ), objects : vec![ "grass".into() ] } );
//!   }
//! }
//!
//! // 3. Compile assets once, then compile a frame per tick.
//! let compiled = compile_assets( &spec, &PathResolver )?;
//! let camera = Camera { world_center : ( 0.0, 0.0 ), zoom : 1.0, viewport_size : ( 800, 600 ) };
//! let commands = compile_frame( &spec, &scene, &compiled, &camera, 0.0 )?;
//!
//! // 4. Hand off to any backend.
//! let mut backend = SvgBackend::new( RenderConfig { width : 800, height : 600, ..Default::default() } );
//! backend.load_assets( &compiled.assets )?;
//! backend.submit( &commands )?;
//! # Ok::< (), Box< dyn std::error::Error > >( () )
//! ```
//!
//! # Sprite source cookbook
//!
//! Nine [`SpriteSource`] variants covering the common idioms. Leaf sources
//! produce one sprite; composite sources emit based on grid context.
//!
//! ## `Static` — a fixed sprite
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ SpriteRef, SpriteSource };
//! SpriteSource::Static( SpriteRef( "terrain".into(), "grass_01".into() ) )
//! # ;
//! ```
//!
//! ## `Variant` — weighted random-looking variations
//!
//! Pick one entry per tile based on its coordinate. Neighbouring cells look
//! different without mirroring — ideal for breaking up large terrain blocks.
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ SpriteRef, SpriteSource, Variant, VariantSelection };
//! SpriteSource::Variant
//! {
//!   variants : vec!
//!   [
//!     Variant { sprite : Box::new( SpriteSource::Static( SpriteRef( "t".into(), "0".into() ) ) ), weight : 5 },
//!     Variant { sprite : Box::new( SpriteSource::Static( SpriteRef( "t".into(), "1".into() ) ) ), weight : 2 },
//!     Variant { sprite : Box::new( SpriteSource::Static( SpriteRef( "t".into(), "2".into() ) ) ), weight : 1 },
//!   ],
//!   selection : VariantSelection::HashCoord,   // deterministic per (q, r)
//! }
//! # ;
//! ```
//!
//! ## `Animation` — time-based frame cycling
//!
//! Declare the animation once in `RenderSpec.animations[]`, reference by id
//! from the layer. Use `phase_offset: HashCoord` so neighbouring tiles
//! animate out of sync.
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ AnimationRef, SpriteSource };
//! SpriteSource::Animation( AnimationRef( "water_flow".into() ) )
//! # ;
//! ```
//!
//! ## `NeighborBitmask` — autotile (walls, fences)
//!
//! Classical 6-bit neighbour mask → sprite lookup. Two flavours:
//!
//! - `ByAtlas` — sprite at atlas index = mask value (simplest setup; author
//!   64 sprites in mask order).
//! - `ByMapping` — explicit `mask → leaf_source` map with a fallback; lets
//!   you point specific masks at animated sprites or variants.
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ AutotileLayout, NeighborBitmaskSource, SpriteSource };
//! SpriteSource::NeighborBitmask
//! {
//!   connects_with : vec![ "stone_wall".into() ],
//!   source : NeighborBitmaskSource::ByAtlas
//!   {
//!     asset : "walls_atlas".into(),
//!     layout : AutotileLayout::Bitmask6,
//!   },
//! }
//! # ;
//! ```
//!
//! ## `NeighborCondition` — skirts and Wesnoth-style edge blends
//!
//! Per-side conditional emission. A single layer can emit zero to
//! `len(sides)` sprites. Two common uses:
//!
//! - **Skirts** (3D side faces): draw a "side" sprite on the S / SW / SE
//!   sides when the neighbour is water or off-map.
//! - **Edge blends**: draw a terrain-overhang sprite on any side where the
//!   neighbour's priority is lower than ours.
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ Condition, EdgeDirection, SpriteSource };
//! SpriteSource::NeighborCondition
//! {
//!   condition : Condition::NeighborPriorityLower,
//!   sides : vec!
//!   [
//!     EdgeDirection::N, EdgeDirection::NE, EdgeDirection::SE,
//!     EdgeDirection::S, EdgeDirection::SW, EdgeDirection::NW,
//!   ],
//!   sprite_pattern : "grass_edge_{dir}".into(),
//!   asset : "edges".into(),
//! }
//! # ;
//! ```
//!
//! The `{dir}` placeholder is substituted with lowercase direction names
//! (`"n"`, `"ne"`, `"se"`, `"s"`, `"sw"`, `"nw"`, `"e"`, `"w"`). Each
//! substituted name must exist in the atlas's `frames` manifest.
//!
//! ## `VertexCorners` — dual-mesh triangle blending
//!
//! Every vertex of the hex grid where three hexes meet can emit a blend
//! sprite based on the sorted tuple of corner terrain ids. Wildcards
//! (`"*"`) let a single pattern match several triple shapes.
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ SpriteSource, TriBlendPattern };
//! SpriteSource::VertexCorners
//! {
//!   patterns : vec!
//!   [
//!     TriBlendPattern
//!     {
//!       corners : ( "grass".into(), "sand".into(), "water".into() ),
//!       sprite_pattern : "tri_gsw_{rot}".into(),
//!       priority : 10,
//!       animation : None,
//!     },
//!     TriBlendPattern
//!     {
//!       corners : ( "*".into(), "*".into(), "void".into() ),
//!       sprite_pattern : "edge_fade_{rot}".into(),
//!       priority : 0,
//!       animation : None,
//!     },
//!   ],
//!   asset : "transitions".into(),
//! }
//! # ;
//! ```
//!
//! ## `External` — sprite supplied by game code at runtime
//!
//! Use when the sprite depends on state the format doesn't model (custom
//! unit equipment, dynamic portraits). The game calls `set_sprite(instance,
//! slot, sprite_ref)` each frame. Runtime plumbing lands with the stateful
//! `Renderer` in a later slice.
//!
//! # Integrating with your game
//!
//! ## The scene is a snapshot of *what to draw*, nothing else
//!
//! [`Scene`] deliberately carries no game mechanics — no HP, no AI state, no
//! inventory. Game logic lives in your own structs. Each frame (or when
//! game state changes), project the visible slice of your world into a
//! `Scene`:
//!
//! ```ignore
//! # use tilemap_renderer::scene_model::{ Bounds, Entity, Scene, Tile };
//! # struct Game { units : Vec< Unit > }
//! # struct Unit { q : i32, r : i32, kind : String, owner : u32, facing : tilemap_renderer::scene_model::EdgeDirection }
//! # impl Unit { fn current_state( &self ) -> String { "idle".into() } }
//! fn scene_for_render( game : &Game ) -> Scene
//! {
//!   let mut scene = Scene::new( Bounds::unbounded() );
//!   // terrain would come from game.map here...
//!   for unit in &game.units
//!   {
//!     scene.entities.push( Entity
//!     {
//!       at : ( unit.q, unit.r ),
//!       object : unit.kind.clone(),
//!       owner : unit.owner,
//!       animation : Some( unit.current_state() ),
//!       facing : Some( unit.facing ),
//!     });
//!   }
//!   scene
//! }
//! ```
//!
//! Equivalently you can maintain a `Scene` alongside your game state and
//! mutate it in place (`scene.entities[i].at = new_pos`) — avoids
//! re-allocating on every frame for large levels.
//!
//! ## Loading from RON
//!
//! When you have authored specs and scenes in RON, use [`RenderSpec::load`]
//! and [`Scene::load`]. The loader validates on parse (currently a
//! skeleton — full rule set lands incrementally) and returns typed errors.
//!
//! # Mapping back to the format doc
//!
//! For the normative specification (field-by-field semantics, rule
//! specificity, the exact hash function, and the full list of edge cases),
//! see `scene_model/SPEC.md` in this crate's source tree.
//!
//! # Current implementation status
//!
//! The compile layer supports:
//!
//! - Hex anchor + Static / Animation / Variant / NeighborBitmask /
//!   NeighborCondition / VertexCorners sources.
//! - Loop / PingPong / OneShot animation modes with regular, from-sheet, or
//!   irregular-timing frames.
//! - Camera translate + uniform zoom.
//! - Pipeline buckets with None / YAsc / YDesc sorting, per-layer
//!   bucket overrides.
//! - ASCII palette expansion for RON-authored maps.
//!
//! Not yet wired into compile (returns typed `CompileError` when used):
//!
//! - `Edge` / `Multihex` / `FreePos` / `Viewport` anchors.
//! - `EdgeConnectedBitmask` and `ViewportTiled` sprite sources.
//! - `TintBehaviour::Flat` / `Masked`, `Effects`, team-colour resolution.
//! - `External` sprite source runtime plumbing.
//! - Stateful `Renderer` with `spawn` / `despawn` / `set_animation` API
//!   and `SpriteBatch` optimisation.
//!
//! These land in follow-up slices.

// Submodules — each uses its own `mod_interface!` to tag its public items.
// Items are re-exported explicitly by name below (rather than with glob
// imports) because `mod_interface!` also defines scope-marker submodules
// (`own`, `orphan`, `exposed`, `prelude`) in every sub-file; a glob re-export
// would pull them all in and produce ambiguous-re-export warnings. The
// sub-module paths stay available for callers that want disambiguation.

pub mod anchor;
pub mod compile;
pub mod coords;
pub mod error;
pub mod hash;
pub mod layer;
pub mod load;
pub mod object;
pub mod pipeline;
pub mod resource;
pub mod scene;
pub mod source;
pub mod spec;
pub mod validate;

pub use anchor::{ Anchor, EdgeDirection, SortYSource };
pub use coords::{ Axial, Even, Flat, FlatSided, FlatTopped, HexCoord, Odd, Offset, Pixel, Pointy, TriCoord };
pub use error::{ LoadError, ValidationError };
pub use hash::{ hash_coord, hash_str };
pub use layer::{ LayerBehaviour, MaskTint, ObjectLayer, TintBehaviour };
pub use object::Object;
pub use pipeline::{ HexConfig, PipelineLayer, RenderPipeline, SortMode, TilingStrategy };
pub use resource::
{
  Animation,
  AnimationMode,
  AnimationRef,
  AnimationTiming,
  Asset,
  AssetKind,
  Axis,
  BlendMode,
  Effect,
  EffectKind,
  EffectRef,
  PhaseOffset,
  SheetLayout,
  SpriteRef,
  TimedFrame,
  Tint,
  TintRef,
};
pub use scene::
{
  Bounds,
  EdgeInstance,
  EdgePosition,
  Entity,
  FreeInstance,
  MultihexInstance,
  Player,
  Scene,
  SceneMeta,
  Tile,
  ViewportInstance,
};
pub use source::
{
  AutotileLayout,
  Condition,
  EdgeConnectedLayout,
  NeighborBitmaskSource,
  SpriteSource,
  TriBlendPattern,
  Variant,
  VariantSelection,
  ViewportAnchorPoint,
  ViewportTiling,
};
pub use spec::RenderSpec;
pub use validate::Validate;

// Phase-2 compile-to-commands layer.
pub use compile::
{
  AssetResolver,
  Camera,
  CompileError,
  CompiledAssets,
  IdMap,
  PathResolver,
  compile_assets,
  compile_frame,
  hex_to_world_pixel_flat,
  hex_to_world_pixel_pointy,
  resolve_animation_frame,
};
