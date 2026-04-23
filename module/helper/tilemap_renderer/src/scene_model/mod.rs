//! Compositional declarative scene format for 2D tile-based games.
//!
//! This module implements the format described in `scene_model/SPEC.md` (v0.2.0).
//! It provides serde-deserialisable data types for two RON files — `render_spec.ron`
//! (rendering rules: objects, layers, animations, pipeline) and `scene.ron`
//! (per-level data: tiles, entities, viewport instances) — plus a loading API
//! and a validation skeleton.
//!
//! Rendering itself (compilation to [`crate::commands::RenderCommand`]) is not
//! part of this phase; this module produces validated data structures that a
//! later compile-to-commands layer will consume.
//!
//! # Coordinates
//!
//! Grid coordinates are re-exported from `tiles_tools`:
//! - Hex cells use axial `( q, r )` via [`HexCoord`].
//! - Dual-mesh triangles use tri-axial `( a, b, c )` via [`TriCoord`].
//!
//! # Overview
//!
//! Three primitives power the format:
//! - [`Object`] — a renderable entity class declared in `render_spec`.
//! - [`ObjectLayer`] — one textured strip of an object with an independent
//!   sprite source and behaviour.
//! - [`Anchor`] — how an object is attached to the world (hex / edge /
//!   vertex / multihex / free world point / viewport).
//!
//! Scenes reference declared objects by id and place instances of them on the
//! grid or in viewport / free space (see [`Scene`]).
//!
//! # Example
//!
//! ```ignore
//! use tilemap_renderer::scene_model::{ RenderSpec, Scene };
//!
//! let spec = RenderSpec::load( "game/render_spec.ron" )?;
//! let scene = Scene::load( "game/levels/tutorial.ron" )?;
//! // next phase: compile (spec, scene) into RenderCommand stream
//! # Ok::< (), Box< dyn std::error::Error > >( () )
//! ```

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
