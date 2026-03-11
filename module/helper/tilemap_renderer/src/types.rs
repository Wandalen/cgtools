//! Core types used across the rendering engine.
//! All types here are POD (Copy, Clone, no allocations).
//!
//! ## Coordinate system
//!
//! The renderer uses a **Y-up** coordinate system:
//! - `(0, 0)` is the bottom-left corner of the viewport.
//! - `(width, height)` is the top-right corner.
//! - Positive rotation is counter-clockwise.
//!
//! Backend adapters that target Y-down surfaces (e.g. SVG, Canvas2D)
//! flip the Y axis internally.

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
  /// Index into the resource storage.
  index : u32,
  _marker : PhantomData< T >,
}

impl< T > ResourceId< T >
{
  /// Creates a new resource handle from a raw index.
  #[ inline ]
  pub const fn new( index : u32 ) -> Self
  {
    Self { index, _marker : PhantomData }
  }

  /// Returns the raw index value.
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
  /// Viewport width in pixels.
  pub width : u32,
  /// Viewport height in pixels.
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
#[ derive( Debug, Clone, Copy, Default, PartialEq, Eq ) ]
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
  /// Translation offset `[ x, y ]`.
  pub position : [ f32; 2 ],
  /// Rotation angle in radians (counter-clockwise).
  pub rotation : f32,
  /// Scale factors `[ sx, sy ]`.
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

impl Transform
{
  /// Column-major 3x3 affine matrix.
  pub fn to_mat3( &self ) -> [ f32; 9 ]
  {
    let cos_r = self.rotation.cos();
    let sin_r = self.rotation.sin();
    let sx = self.scale[ 0 ];
    let sy = self.scale[ 1 ];
    let skx = self.skew[ 0 ].tan();
    let sky = self.skew[ 1 ].tan();

    let m00 = ( cos_r + sin_r * sky ) * sx;
    let m10 = ( sin_r - cos_r * sky ) * sx;
    let m01 = ( cos_r * skx - sin_r ) * sy;
    let m11 = ( sin_r * skx + cos_r ) * sy;

    [
      m00,                 m10,                 0.0,
      m01,                 m11,                 0.0,
      self.position[ 0 ],  self.position[ 1 ],  1.0,
    ]
  }
}

// ============================================================================
// Style types
// ============================================================================

/// Line cap style for stroke endpoints.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum LineCap
{
  /// Flat cap flush with the endpoint.
  #[ default ]
  Butt,
  /// Semicircular cap extending past the endpoint.
  Round,
  /// Rectangular cap extending past the endpoint.
  Square,
}

/// Line join style for stroke corners.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum LineJoin
{
  /// Sharp corner join.
  #[ default ]
  Miter,
  /// Rounded corner join.
  Round,
  /// Beveled (flat) corner join.
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
  /// Starting offset into the dash pattern.
  pub offset : f32,
}

impl Default for DashStyle
{
  fn default() -> Self
  {
    Self { pattern : [ 0.0; 8 ], offset : 0.0 }
  }
}

/// Anchor point for text placement.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum TextAnchor
{
  /// Top-left corner.
  #[ default ]
  TopLeft,
  /// Top edge, horizontally centered.
  TopCenter,
  /// Top-right corner.
  TopRight,
  /// Left edge, vertically centered.
  CenterLeft,
  /// Dead center.
  Center,
  /// Right edge, vertically centered.
  CenterRight,
  /// Bottom-left corner.
  BottomLeft,
  /// Bottom edge, horizontally centered.
  BottomCenter,
  /// Bottom-right corner.
  BottomRight,
}

/// Primitive topology for vertex data.
#[ derive( Debug, Clone, Copy, Default, PartialEq, Eq ) ]
pub enum Topology
{
  /// Independent triangles (every 3 vertices).
  #[ default ]
  TriangleList,
  /// Connected triangle strip.
  TriangleStrip,
  /// Independent line segments (every 2 vertices).
  LineList,
  /// Connected line strip.
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
  /// No fill.
  None,
  /// Solid RGBA color.
  Solid( [ f32; 4 ] ),
  /// Reference to a gradient asset.
  Gradient( ResourceId< asset::Gradient > ),
  /// Reference to a pattern asset.
  Pattern( ResourceId< asset::Pattern > ),
}

impl Default for FillRef
{
  fn default() -> Self { FillRef::None }
}

// ============================================================================
// Tests
// ============================================================================

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn resource_id_type_safety()
  {
    let a : ResourceId< asset::Image > = ResourceId::new( 5 );
    let b : ResourceId< asset::Image > = ResourceId::new( 5 );
    let c : ResourceId< asset::Image > = ResourceId::new( 7 );
    assert_eq!( a, b );
    assert_ne!( a, c );
    assert_eq!( a.inner(), 5 );
  }

  #[ test ]
  fn resource_id_debug()
  {
    let id : ResourceId< asset::Sprite > = ResourceId::new( 42 );
    assert_eq!( format!( "{:?}", id ), "ResourceId(42)" );
  }

  #[ test ]
  fn transform_default_is_identity()
  {
    let t = Transform::default();
    assert_eq!( t.position, [ 0.0, 0.0 ] );
    assert_eq!( t.rotation, 0.0 );
    assert_eq!( t.scale, [ 1.0, 1.0 ] );
    assert_eq!( t.skew, [ 0.0, 0.0 ] );
    assert_eq!( t.depth, 0.0 );
  }

  #[ test ]
  fn to_mat3_identity()
  {
    let t = Transform::default();
    let m = t.to_mat3();
    let expected =
    [
      1.0, 0.0, 0.0,
      0.0, 1.0, 0.0,
      0.0, 0.0, 1.0,
    ];
    for ( a, b ) in m.iter().zip( expected.iter() )
    {
      assert!( ( a - b ).abs() < 1e-6, "expected {} got {}", b, a );
    }
  }

  #[ test ]
  fn to_mat3_translation()
  {
    let t = Transform { position : [ 10.0, 20.0 ], ..Default::default() };
    let m = t.to_mat3();
    // Column-major: translation is in last column (indices 6, 7)
    assert!( ( m[ 6 ] - 10.0 ).abs() < 1e-6 );
    assert!( ( m[ 7 ] - 20.0 ).abs() < 1e-6 );
    // Rotation/scale part should be identity
    assert!( ( m[ 0 ] - 1.0 ).abs() < 1e-6 );
    assert!( ( m[ 4 ] - 1.0 ).abs() < 1e-6 );
  }

  #[ test ]
  fn to_mat3_scale()
  {
    let t = Transform { scale : [ 2.0, 3.0 ], ..Default::default() };
    let m = t.to_mat3();
    assert!( ( m[ 0 ] - 2.0 ).abs() < 1e-6 );
    assert!( ( m[ 4 ] - 3.0 ).abs() < 1e-6 );
    assert!( ( m[ 1 ] ).abs() < 1e-6 );
    assert!( ( m[ 3 ] ).abs() < 1e-6 );
  }

  #[ test ]
  fn to_mat3_rotation_90()
  {
    let t = Transform { rotation : core::f32::consts::FRAC_PI_2, ..Default::default() };
    let m = t.to_mat3();
    // 90° CCW: cos=0, sin=1
    // m00=cos=0, m10=sin=1, m01=-sin=-1, m11=cos=0
    assert!( m[ 0 ].abs() < 1e-6, "m00={}", m[ 0 ] );
    assert!( ( m[ 1 ] - 1.0 ).abs() < 1e-6, "m10={}", m[ 1 ] );
    assert!( ( m[ 3 ] + 1.0 ).abs() < 1e-6, "m01={}", m[ 3 ] );
    assert!( m[ 4 ].abs() < 1e-6, "m11={}", m[ 4 ] );
  }

  #[ test ]
  fn render_config_default()
  {
    let c = RenderConfig::default();
    assert_eq!( c.width, 800 );
    assert_eq!( c.height, 600 );
    assert_eq!( c.antialias, Antialias::Default );
    assert_eq!( c.background, [ 0.0, 0.0, 0.0, 1.0 ] );
  }
}
