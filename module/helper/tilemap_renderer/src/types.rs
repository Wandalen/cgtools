//! Core types used across the rendering engine.
//! All types here are POD (Copy, Clone, no allocations).

/// Handle to a resource stored in Assets. POD, Copy.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub struct ResourceId( pub u32 );

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

/// Fill reference. Points to a fill definition in Assets, or solid color.
/// SVG: solid -> `fill="rgb(...)"`, gradient -> `fill="url(#grad_N)"`, pattern -> `fill="url(#pat_N)"`.
/// GPU: solid -> uniform color, gradient -> gradient shader, pattern -> texture with repeat sampler.
#[ derive( Debug, Clone, Copy, Default ) ]
pub enum FillRef
{
  #[ default ]
  None,
  Solid( [ f32; 4 ] ),
  /// References a gradient or pattern stored in Assets.
  Asset( ResourceId ),
}
