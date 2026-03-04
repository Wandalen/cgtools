//! Asset definitions — resources loaded before rendering.
//!
//! Assets CAN allocate (Vec, PathBuf, etc.) unlike commands.
//! They are loaded once via `Backend::load_assets()` and referenced
//! from commands by `ResourceId`.

use std::path::PathBuf;
use crate::types::*;

// ============================================================================
// Asset container
// ============================================================================

/// Collection of all resources needed for rendering.
/// Loaded into a backend once before submitting commands.
pub struct Assets
{
  pub fonts : Vec< FontAsset >,
  pub images : Vec< ImageAsset >,
  pub sprites : Vec< SpriteAsset >,
  pub geometries : Vec< GeometryAsset >,
  pub gradients : Vec< GradientAsset >,
  pub patterns : Vec< PatternAsset >,
  pub clip_masks : Vec< ClipMaskAsset >,
  /// Named paths that can be referenced (e.g. for text-on-path).
  pub paths : Vec< PathAsset >,
}

// ============================================================================
// Individual asset types
// ============================================================================

pub struct FontAsset
{
  pub id : ResourceId,
  pub source : PathBuf,
}

pub struct ImageAsset
{
  pub id : ResourceId,
  pub source : Source,
  /// Sampling filter for this image.
  /// SVG: `image-rendering: pixelated` (Nearest) vs `auto` (Linear).
  /// GPU: sampler mag/min filter.
  pub filter : SamplerFilter,
}

/// A rectangular region within a loaded image (sprite sheet support).
/// SVG: `<symbol viewBox="x y w h"><use href="#sheet" .../></symbol>`.
/// GPU: UV coordinates mapped to the sub-rectangle within the texture atlas.
pub struct SpriteAsset
{
  pub id : ResourceId,
  /// The source image (sprite sheet) this sprite is cut from.
  pub sheet : ResourceId,
  /// Region within the sheet: `[x, y, width, height]` in pixels.
  pub region : [ f32; 4 ],
}

pub struct GeometryAsset
{
  pub id : ResourceId,
  pub source : Source,
  pub data_type : DataType,
}

/// Gradient definition.
/// SVG: `<linearGradient>` / `<radialGradient>` in `<defs>`.
/// GPU: uploaded as a 1D texture or evaluated analytically in shader.
pub struct GradientAsset
{
  pub id : ResourceId,
  pub kind : GradientKind,
  pub stops : Vec< GradientStop >,
}

#[ derive( Debug, Clone, Copy ) ]
pub struct GradientStop
{
  /// Position along gradient, 0.0 to 1.0.
  pub offset : f32,
  pub color : [ f32; 4 ],
}

#[ derive( Debug, Clone, Copy ) ]
pub enum GradientKind
{
  Linear
  {
    start : [ f32; 2 ],
    end : [ f32; 2 ],
  },
  Radial
  {
    center : [ f32; 2 ],
    radius : f32,
    /// Focal point, can equal center.
    focal : [ f32; 2 ],
  },
}

/// A repeating tile pattern.
/// SVG: `<pattern>` in `<defs>` containing an `<image>` or shape.
/// GPU: texture with `AddressMode::Repeat` sampler.
pub struct PatternAsset
{
  pub id : ResourceId,
  /// The image or geometry to tile.
  pub content : ResourceId,
  pub width : f32,
  pub height : f32,
}

/// A clip mask — a shape that limits rendering to its interior.
/// SVG: `<clipPath>` in `<defs>`, elements use `clip-path="url(#...)"`.
/// GPU: draw clip shape into stencil buffer, enable stencil test for content.
pub struct ClipMaskAsset
{
  pub id : ResourceId,
  pub segments : Vec< PathSegment >,
}

/// Stored path (e.g. for text-on-path references).
pub struct PathAsset
{
  pub id : ResourceId,
  pub segments : Vec< PathSegment >,
}

/// Path segment for use in Assets.
#[ derive( Debug, Clone, Copy ) ]
pub enum PathSegment
{
  MoveTo( f32, f32 ),
  LineTo( f32, f32 ),
  QuadTo { cx : f32, cy : f32, x : f32, y : f32 },
  CubicTo { c1x : f32, c1y : f32, c2x : f32, c2y : f32, x : f32, y : f32 },
  ArcTo { rx : f32, ry : f32, rotation : f32, large_arc : bool, sweep : bool, x : f32, y : f32 },
  Close,
}

// ============================================================================
// Supporting types
// ============================================================================

pub enum Source
{
  Path( PathBuf ),
  Bytes( Vec< u8 > ),
}

pub enum DataType
{
  U8,
  U16,
  U32,
  F32,
}
