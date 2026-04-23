//! Render pipeline — ordered pipeline buckets, tiling strategy, global tint.
//!
//! See SPEC §8. The pipeline names the z-buckets objects draw into, the sort
//! mode within each bucket, and the tiling strategy used for neighbour / vertex
//! queries.

mod private
{
  use serde::{ Deserialize, Serialize };
  use crate::scene_model::resource::TintRef;

  /// Top-level pipeline declaration embedded in [`crate::scene_model::spec::RenderSpec`].
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct RenderPipeline
  {
    /// Grid geometry.
    pub hex : HexConfig,
    /// Pipeline z-buckets, rendered in declaration order (bottom to top).
    pub layers : Vec< PipelineLayer >,
    /// Optional global tint multiplied into every draw call. See SPEC §8.4.
    #[ serde( default ) ]
    pub global_tint : Option< TintRef >,
    /// Viewport size in screen pixels (width, height). Optional — when absent,
    /// backends derive it from the window / canvas.
    #[ serde( default ) ]
    pub viewport_size : Option< ( u32, u32 ) >,
  }

  /// One pipeline bucket — a named z-layer objects can route draw calls into.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct PipelineLayer
  {
    /// Bucket id. Referenced by `Object.global_layer` and
    /// `ObjectLayer.pipeline_layer`.
    pub id : String,
    /// Sort mode applied to draw calls gathered in this bucket.
    #[ serde( default ) ]
    pub sort : SortMode,
    /// Optional per-bucket tint mask applied between per-object tints and
    /// the global tint. See SPEC §12.1 step 4.
    #[ serde( default ) ]
    pub tint_mask : Option< TintRef >,
  }

  /// Sort mode for a [`PipelineLayer`]. See SPEC §8.2.
  #[ derive( Debug, Clone, Copy, Default, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum SortMode
  {
    /// Draw calls keep their emission order. Deterministic for static scenes.
    #[ default ]
    None,
    /// Sort by screen Y ascending — objects lower on screen draw on top.
    YAsc,
    /// Sort by screen Y descending.
    YDesc,
  }

  /// Hex / grid geometry configuration. See SPEC §2.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  pub struct HexConfig
  {
    /// Which tiling strategy the scene uses.
    pub tiling : TilingStrategy,
    /// Full bounding-box size of one cell in pixels `( width, height )`.
    pub cell_size : ( u32, u32 ),
  }

  /// Tiling strategy — determines neighbour ordering, dual-mesh shape, and
  /// pixel-conversion. See SPEC §2.1.
  ///
  /// Version 0.2.0 implements the two hex variants; the square variants are
  /// reserved and rejected at load time with
  /// [`crate::scene_model::error::ValidationError::UnsupportedTiling`].
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum TilingStrategy
  {
    /// Flat-top hex — six neighbours, dual-mesh triangles.
    HexFlatTop,
    /// Pointy-top hex — six neighbours, dual-mesh triangles.
    HexPointyTop,
    /// 4-neighbour square grid (reserved, not implemented).
    Square4,
    /// 8-neighbour square grid (reserved, not implemented).
    Square8,
  }
}

mod_interface::mod_interface!
{
  own use RenderPipeline;
  own use PipelineLayer;
  own use SortMode;
  own use HexConfig;
  own use TilingStrategy;
}
