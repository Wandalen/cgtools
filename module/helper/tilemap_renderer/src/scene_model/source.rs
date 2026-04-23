//! Sprite sources — rules that pick a concrete sprite / frame for a layer.
//!
//! Sources split into two kinds (see SPEC §5):
//!
//! - **Leaf sources** — [`SpriteSource::Static`], [`SpriteSource::Variant`],
//!   [`SpriteSource::Animation`], [`SpriteSource::External`]. Each produces a
//!   single sprite without inspecting grid context beyond the object's own
//!   position. Leaves compose freely inside composites.
//! - **Composite sources** — [`SpriteSource::NeighborBitmask`],
//!   [`SpriteSource::NeighborCondition`], [`SpriteSource::VertexCorners`],
//!   [`SpriteSource::EdgeConnectedBitmask`], [`SpriteSource::ViewportTiled`].
//!   These inspect neighbours, vertex corners, or the viewport and route
//!   through leaf sources in their inner value slots.
//!
//! The leaf / composite distinction is not encoded in the type system — one
//! big `SpriteSource` enum keeps the serde schema flat and ergonomic in RON.
//! Composite-inside-composite nesting is an error caught at validation time
//! (see [`crate::scene_model::error::ValidationError::IllegalSourceNesting`]).

mod private
{
  use serde::{ Deserialize, Serialize };
  use std::collections::HashMap;
  use crate::scene_model::anchor::EdgeDirection;
  use crate::scene_model::resource::{ SpriteRef, AnimationRef };

  /// A sprite-selection rule for a layer. See module-level docs and SPEC §5.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum SpriteSource
  {
    // ─── Leaf sources ───────────────────────────────────────────────────────

    /// A fixed sprite reference. SPEC §5.1.
    Static( SpriteRef ),
    /// A weighted list of alternative sub-sources, one chosen per instance.
    /// SPEC §5.2.
    Variant
    {
      /// Candidate sub-sources with weights.
      variants : Vec< Variant >,
      /// Selection strategy (hashed coord / random / fixed index).
      selection : VariantSelection,
    },
    /// Plays a declared animation (see [`crate::scene_model::resource::Animation`]). SPEC §5.3.
    Animation( AnimationRef ),
    /// Sprite supplied at runtime by game code via a named slot. SPEC §5.8.
    External
    {
      /// Slot name game code writes into (e.g. `"body"`, `"weapon"`).
      slot : String,
    },

    // ─── Composite sources ──────────────────────────────────────────────────

    /// Autotile via hex-cell neighbour bitmask. SPEC §5.4.
    NeighborBitmask
    {
      /// Object ids that count as "connected" when computing the bitmask.
      connects_with : Vec< String >,
      /// How the bitmask maps to a sprite.
      source : NeighborBitmaskSource,
    },
    /// Per-side conditional sprite emission — skirts, Wesnoth edge blends. SPEC §5.5.
    NeighborCondition
    {
      /// Condition evaluated per side.
      condition : Condition,
      /// Sides to test (subset of the anchor's direction set).
      sides : Vec< EdgeDirection >,
      /// Sprite name template with `{dir}` placeholder.
      sprite_pattern : String,
      /// Asset id hosting the patterned sprites.
      asset : String,
    },
    /// Dual-mesh triangle lookup by sorted corner tuple. SPEC §5.6.
    VertexCorners
    {
      /// Matching rules, ordered by declaration; matching uses
      /// specificity → priority → declaration order (SPEC §9).
      patterns : Vec< TriBlendPattern >,
      /// Asset id hosting the patterned sprites.
      asset : String,
    },
    /// Autotile via edge-endpoint connectivity bitmask (rivers, edge roads). SPEC §5.9.
    EdgeConnectedBitmask
    {
      /// Object ids that count as "connected" on neighbour edges.
      connects_with : Vec< String >,
      /// How the 4-bit mask maps to a sprite.
      source : NeighborBitmaskSource,
      /// Bit-layout convention for the mask.
      layout : EdgeConnectedLayout,
    },
    /// Viewport-anchored tiled / stretched / centred image. SPEC §5.7.
    ViewportTiled
    {
      /// Inner leaf source providing the texture (usually `Static` or `Animation`).
      content : Box< SpriteSource >,
      /// How the texture is laid out inside the viewport.
      tiling : ViewportTiling,
      /// Anchor point when the layer is not tiled in a given axis.
      anchor_point : ViewportAnchorPoint,
    },
  }

  // ─── Leaf-source helpers ──────────────────────────────────────────────────

  /// One entry in a [`SpriteSource::Variant`] list.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Variant
  {
    /// Sub-source producing this variant's sprite (a leaf source).
    pub sprite : Box< SpriteSource >,
    /// Selection weight. Higher = more likely.
    pub weight : u32,
  }

  /// How a [`SpriteSource::Variant`] picks one of its entries per instance.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, Default ) ]
  #[ non_exhaustive ]
  pub enum VariantSelection
  {
    /// Detrministic hash of the instance's grid coordinate (default).
    #[ default ]
    HashCoord,
    /// Random at scene load, fixed for the lifetime of the scene.
    Random,
    /// Force a specific entry index.
    Fixed( usize ),
  }

  // ─── Composite-source helpers ─────────────────────────────────────────────

  /// Condition grammar for [`SpriteSource::NeighborCondition`]. SPEC §5.5.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum Condition
  {
    /// Neighbour cell's object id is in the given list.
    NeighborIs( Vec< String > ),
    /// Neighbour is off-map (a `"void"` cell).
    NoNeighbor,
    /// Current cell's object has strictly higher `priority` than the neighbour's.
    NeighborPriorityLower,
    /// Any sub-condition matches.
    AnyOf( Vec< Condition > ),
    /// Every sub-condition matches.
    AllOf( Vec< Condition > ),
    /// Inverts the sub-condition.
    Not( Box< Condition > ),
  }

  /// Bitmask-lookup strategy for [`SpriteSource::NeighborBitmask`] and
  /// [`SpriteSource::EdgeConnectedBitmask`].
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum NeighborBitmaskSource
  {
    /// Explicit mask → leaf-source mapping with a fallback. See SPEC §5.4 Option A.
    ByMapping
    {
      /// Bitmask values mapped to leaf sources (`Static` / `Variant` / `Animation`).
      mapping : HashMap< u8, SpriteSource >,
      /// Used when the computed mask isn't in `mapping`.
      fallback : Box< SpriteSource >,
    },
    /// Sprite index = bitmask value in a pre-arranged atlas. SPEC §5.4 Option B.
    ByAtlas
    {
      /// Asset id hosting the bitmask-indexed atlas.
      asset : String,
      /// Which bitmask width the atlas expects.
      layout : AutotileLayout,
    },
  }

  /// Width of a [`NeighborBitmaskSource::ByAtlas`].
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum AutotileLayout
  {
    /// 6-bit hex neighbour mask (flat-top or pointy-top). 64 entries.
    Bitmask6,
  }

  /// Bit-layout convention for [`SpriteSource::EdgeConnectedBitmask`].
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum EdgeConnectedLayout
  {
    /// 4-bit hex edge mask: ccw / cw bits at each of the edge's two endpoints.
    /// See SPEC §5.9 for the exact bit positions.
    EdgeHex,
  }

  /// One matching rule inside [`SpriteSource::VertexCorners`].
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct TriBlendPattern
  {
    /// Canonicalised, sorted triple of corner object ids. Use `"*"` for
    /// single-slot wildcards.
    pub corners : ( String, String, String ),
    /// Sprite name template with `{rot}` placeholder (rotation 0..2).
    pub sprite_pattern : String,
    /// Match priority; higher wins ties of equal specificity.
    #[ serde( default ) ]
    pub priority : i32,
    /// Optional animation that plays on the matched vertex.
    #[ serde( default ) ]
    pub animation : Option< AnimationRef >,
  }

  /// How a [`SpriteSource::ViewportTiled`] lays its inner content across the viewport.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum ViewportTiling
  {
    /// Tile in both axes.
    Repeat2D,
    /// Tile in X, anchored in Y.
    RepeatX,
    /// Tile in Y, anchored in X.
    RepeatY,
    /// Stretch to fill viewport (may distort aspect).
    Stretch,
    /// Scale to fit viewport preserving aspect (may letterbox).
    Fit,
    /// Draw once at the anchor point at native size.
    Center,
  }

  /// Anchor point inside the viewport for non-tiled axes of [`ViewportTiling`].
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, Default ) ]
  #[ non_exhaustive ]
  pub enum ViewportAnchorPoint
  {
    /// Top-left.
    TopLeft,
    /// Top centre.
    TopCenter,
    /// Top-right.
    TopRight,
    /// Middle-left.
    CenterLeft,
    /// Viewport centre (default).
    #[ default ]
    Center,
    /// Middle-right.
    CenterRight,
    /// Bottom-left.
    BottomLeft,
    /// Bottom centre — common for parallax mountain lines.
    BottomCenter,
    /// Bottom-right.
    BottomRight,
  }
}

mod_interface::mod_interface!
{
  own use SpriteSource;
  own use Variant;
  own use VariantSelection;
  own use Condition;
  own use NeighborBitmaskSource;
  own use AutotileLayout;
  own use EdgeConnectedLayout;
  own use TriBlendPattern;
  own use ViewportTiling;
  own use ViewportAnchorPoint;
}
