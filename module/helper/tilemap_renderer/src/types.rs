//! Core types used across the rendering engine.
//! All types here are POD (Copy, Clone, no allocations).

use core::{ fmt::Debug, marker::PhantomData };

/// Type-safe handle to a resource. The phantom type `T` prevents mixing up
/// different resource kinds at compile time.
///
/// ```ignore
/// let img : ResourceId< ImageAsset > = ResourceId::new( 0 );
/// let spr : ResourceId< SpriteAsset > = ResourceId::new( 1 );
/// // img == spr; // compile error: different types
/// ```
pub struct ResourceId< T >
{
  index : u32,
  _marker : PhantomData< T >,
}

impl< T > ResourceId< T >
{
  #[ inline ]
  pub const fn new( index : u32 ) -> Self
  {
    Self { index, _marker : PhantomData }
  }

  #[ inline ]
  pub fn inner( &self ) -> u32
  {
    self.index
  }
}

// Manual trait impls because derive doesn't work well with PhantomData generics.
impl< T > Debug for ResourceId< T > { fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result { write!( f, "ResourceId({})", self.index ) } }
impl< T > Clone for ResourceId< T > { fn clone( &self ) -> Self { *self } }
impl< T > Copy for ResourceId< T > {}
impl< T > PartialEq for ResourceId< T > { fn eq( &self, other : &Self ) -> bool { self.index == other.index } }
impl< T > Eq for ResourceId< T > {}
impl< T > core::hash::Hash for ResourceId< T > { fn hash< H : core::hash::Hasher >( &self, state : &mut H ) { self.index.hash( state ); } }
impl< T > nohash_hasher::IsEnabled for ResourceId< T > {}

/// Marker type for batch resources.
#[ derive( Debug, Clone, Copy ) ]
pub struct Batch;

// ============================================================================
// Render configuration
// ============================================================================

/// Shared renderer configuration.
/// Passed to backend constructors. Backends may ignore fields they don't support.
#[ derive( Debug, Clone, Copy ) ]
pub struct RenderConfig
{
  pub width : u32,
  pub height : u32,
  /// Antialiasing quality.
  /// SVG: `shape-rendering` attribute.
  /// GPU: MSAA sample count.
  /// Terminal: ignored.
  pub antialias : Antialias,
  /// Default background color (RGBA, 0.0–1.0).
  /// Used by `Clear` command if no explicit color given.
  pub background : [ f32; 4 ],
}

impl Default for RenderConfig
{
  fn default() -> Self
  {
    Self
    {
      width : 800,
      height : 600,
      antialias : Antialias::Default,
      background : [ 0.0, 0.0, 0.0, 1.0 ],
    }
  }
}

/// Antialiasing mode.
/// SVG: maps to `shape-rendering` CSS property.
/// GPU: maps to MSAA sample count.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum Antialias
{
  /// No antialiasing. SVG: `crispEdges`. GPU: MSAA 1x. Good for pixel art.
  None,
  /// Standard antialiasing. SVG: `auto`. GPU: MSAA 4x.
  #[ default ]
  Default,
  /// High-quality antialiasing. SVG: `geometricPrecision`. GPU: MSAA 8x.
  High,
}

// ============================================================================
// Transform
// ============================================================================

/// 2D affine transform.
#[ derive( Debug, Clone, Copy ) ]
pub struct Transform
{
  pub position : [ f32; 2 ],
  pub rotation : f32,
  pub scale : [ f32; 2 ],
  /// Skew in radians. SVG: `skewX()` / `skewY()`. GPU: added to affine matrix.
  pub skew : [ f32; 2 ],
  /// Draw order. Higher values are drawn on top. Default 0.0.
  /// SVG: backend sorts elements by depth before emitting.
  /// GPU: depth buffer or painter's algorithm sort.
  pub depth : f32,
}

impl Default for Transform
{
  fn default() -> Self
  {
    Self
    {
      position : [ 0.0, 0.0 ],
      rotation : 0.0,
      scale : [ 1.0, 1.0 ],
      skew : [ 0.0, 0.0 ],
      depth : 0.0,
    }
  }
}

// ============================================================================
// Style types
// ============================================================================

#[ derive( Debug, Clone, Copy, Default ) ]
pub enum LineCap
{
  #[ default ]
  Butt,
  Round,
  Square,
}

#[ derive( Debug, Clone, Copy, Default ) ]
pub enum LineJoin
{
  #[ default ]
  Miter,
  Round,
  Bevel,
}

/// Dash pattern. Fixed-size to stay POD (no Vec).
/// Up to 4 dash-gap pairs covers most cases.
/// SVG: `stroke-dasharray`. GPU: fragment shader or geometry expansion.
#[ derive( Debug, Clone, Copy ) ]
pub struct DashStyle
{
  /// Dash-gap pairs, zero-terminated. e.g. `[5.0, 3.0, 0.0, ...]` = "5 3".
  pub pattern : [ f32; 8 ],
  pub offset : f32,
}

impl Default for DashStyle
{
  fn default() -> Self
  {
    Self { pattern : [ 0.0; 8 ], offset : 0.0 }
  }
}

#[ derive( Debug, Clone, Copy, Default ) ]
pub enum TextAnchor
{
  #[ default ]
  TopLeft,
  TopCenter,
  TopRight,
  CenterLeft,
  Center,
  CenterRight,
  BottomLeft,
  BottomCenter,
  BottomRight,
}

#[ derive( Debug, Clone, Copy, Default ) ]
pub enum Topology
{
  #[ default ]
  TriangleList,
  TriangleStrip,
  LineList,
  LineStrip,
}

/// Texture sampling filter.
/// SVG: `image-rendering` CSS property.
/// GPU: `mag_filter` / `min_filter` on the texture sampler.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum SamplerFilter
{
  /// Nearest-neighbor: sharp pixels, no interpolation. Ideal for pixel art.
  Nearest,
  /// Bilinear interpolation: smooth scaling.
  #[ default ]
  Linear,
}

/// Blend mode for compositing.
/// SVG: `mix-blend-mode` CSS property.
/// GPU: blend state on the pipeline.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum BlendMode
{
  /// Source over (alpha blending).
  #[ default ]
  Normal,
  /// SVG: `multiply`. GPU: src * dst.
  Multiply,
  /// SVG: `screen`. GPU: 1 - (1-src)*(1-dst).
  Screen,
  /// SVG: `overlay`.
  Overlay,
  /// Additive blending. GPU: src + dst.
  Add,
}

/// Marker types for asset kinds — used as phantom type parameter in `ResourceId<T>`.
pub mod asset
{
  /// Marker for font assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Font;
  /// Marker for image assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Image;
  /// Marker for sprite assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Sprite;
  /// Marker for geometry assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Geometry;
  /// Marker for gradient assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Gradient;
  /// Marker for pattern assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Pattern;
  /// Marker for clip mask assets.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct ClipMask;
  /// Marker for path assets (e.g. text-on-path).
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Path;
}

/// Fill reference. Points to a fill definition in Assets, or solid color.
/// SVG: solid -> `fill="rgb(...)"`, gradient -> `fill="url(#grad_N)"`, pattern -> `fill="url(#pat_N)"`.
/// GPU: solid -> uniform color, gradient -> gradient shader, pattern -> texture with repeat sampler.
#[ derive( Debug, Clone, Copy ) ]
pub enum FillRef
{
  None,
  Solid( [ f32; 4 ] ),
  Gradient( ResourceId< asset::Gradient > ),
  Pattern( ResourceId< asset::Pattern > ),
}

impl Default for FillRef
{
  fn default() -> Self { FillRef::None }
}
