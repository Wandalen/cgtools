//! Asset definitions — resources loaded before rendering.
//!
//! Assets CAN allocate (Vec, PathBuf, etc.) unlike commands.
//! They are loaded once via `Backend::load_assets()` and referenced
//! from commands by `ResourceId<T>`.

use std::path::PathBuf;
use nohash_hasher::IntSet;
use crate::types::{ ResourceId, SamplerFilter, asset };

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
// Validation
// ============================================================================

/// Error found during asset validation.
#[ derive( Debug ) ]
pub enum ValidationError
{
  /// Two assets of the same type share the same ResourceId.
  DuplicateId { kind : &'static str, index : u32 },
}

impl core::fmt::Display for ValidationError
{
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::DuplicateId { kind, index } =>
        write!( f, "duplicate {} id: {}", kind, index ),
    }
  }
}

impl Assets
{
  /// Validates that no two assets of the same type share a ResourceId.
  /// Returns all errors found (does not stop at first).
  pub fn validate( &self ) -> Vec< ValidationError >
  {
    let mut errors = Vec::new();

    fn check_dups< T >( items : &[ impl HasId< T > ], kind : &'static str, errors : &mut Vec< ValidationError > )
    {
      let mut seen = IntSet::default();
      for item in items
      {
        if !seen.insert( item.resource_id().inner() )
        {
          errors.push( ValidationError::DuplicateId { kind, index : item.resource_id().inner() } );
        }
      }
    }

    check_dups::< asset::Font >( &self.fonts, "font", &mut errors );
    check_dups::< asset::Image >( &self.images, "image", &mut errors );
    check_dups::< asset::Sprite >( &self.sprites, "sprite", &mut errors );
    check_dups::< asset::Geometry >( &self.geometries, "geometry", &mut errors );
    check_dups::< asset::Gradient >( &self.gradients, "gradient", &mut errors );
    check_dups::< asset::Pattern >( &self.patterns, "pattern", &mut errors );
    check_dups::< asset::ClipMask >( &self.clip_masks, "clip_mask", &mut errors );
    check_dups::< asset::Path >( &self.paths, "path", &mut errors );

    errors
  }
}

/// Helper trait to extract ResourceId from asset structs.
trait HasId< T >
{
  fn resource_id( &self ) -> ResourceId< T >;
}

impl HasId< asset::Font > for FontAsset { fn resource_id( &self ) -> ResourceId< asset::Font > { self.id } }
impl HasId< asset::Image > for ImageAsset { fn resource_id( &self ) -> ResourceId< asset::Image > { self.id } }
impl HasId< asset::Sprite > for SpriteAsset { fn resource_id( &self ) -> ResourceId< asset::Sprite > { self.id } }
impl HasId< asset::Geometry > for GeometryAsset { fn resource_id( &self ) -> ResourceId< asset::Geometry > { self.id } }
impl HasId< asset::Gradient > for GradientAsset { fn resource_id( &self ) -> ResourceId< asset::Gradient > { self.id } }
impl HasId< asset::Pattern > for PatternAsset { fn resource_id( &self ) -> ResourceId< asset::Pattern > { self.id } }
impl HasId< asset::ClipMask > for ClipMaskAsset { fn resource_id( &self ) -> ResourceId< asset::ClipMask > { self.id } }
impl HasId< asset::Path > for PathAsset { fn resource_id( &self ) -> ResourceId< asset::Path > { self.id } }

// ============================================================================
// Individual asset types
// ============================================================================

pub struct FontAsset
{
  pub id : ResourceId< asset::Font >,
  pub source : PathBuf,
}

pub struct ImageAsset
{
  pub id : ResourceId< asset::Image >,
  pub source : ImageSource,
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
  pub id : ResourceId< asset::Sprite >,
  /// The source image (sprite sheet) this sprite is cut from.
  pub sheet : ResourceId< asset::Image >,
  /// Region within the sheet: `[x, y, width, height]` in pixels.
  pub region : [ f32; 4 ],
}

pub struct GeometryAsset
{
  pub id : ResourceId< asset::Geometry >,
  /// Vertex positions (flat: [x0, y0, x1, y1, ...]).
  pub positions : Source,
  /// Optional UV coordinates (flat: [u0, v0, u1, v1, ...]).
  /// GPU: used for texture mapping in fragment shader.
  /// SVG: ignored — SVG uses pattern fill which tiles by bounding box.
  pub uvs : Option< Source >,
  /// Optional vertex indices for indexed drawing.
  pub indices : Option< Source >,
  pub data_type : DataType,
}

/// Gradient definition.
/// SVG: `<linearGradient>` / `<radialGradient>` in `<defs>`.
/// GPU: uploaded as a 1D texture or evaluated analytically in shader.
pub struct GradientAsset
{
  pub id : ResourceId< asset::Gradient >,
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
  pub id : ResourceId< asset::Pattern >,
  /// The image to tile.
  pub content : ResourceId< asset::Image >,
  pub width : f32,
  pub height : f32,
}

/// A clip mask — a shape that limits rendering to its interior.
/// SVG: `<clipPath>` in `<defs>`, elements use `clip-path="url(#...)"`.
/// GPU: draw clip shape into stencil buffer, enable stencil test for content.
pub struct ClipMaskAsset
{
  pub id : ResourceId< asset::ClipMask >,
  pub segments : Vec< PathSegment >,
}

/// Stored path (e.g. for text-on-path references).
pub struct PathAsset
{
  pub id : ResourceId< asset::Path >,
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

/// Image data source.
pub enum ImageSource
{
  /// Path to image file — backend decodes (PNG, JPEG, etc.).
  Path( PathBuf ),
  /// Encoded image in memory (PNG, JPEG, etc.) — backend decodes.
  Encoded( Vec< u8 > ),
  /// Raw pixel data — ready to upload directly.
  Bitmap
  {
    bytes : Vec< u8 >,
    width : u32,
    height : u32,
    format : PixelFormat,
  },
}

/// Pixel format for raw bitmap data.
#[ derive( Debug, Clone, Copy ) ]
pub enum PixelFormat
{
  /// 4 bytes per pixel: red, green, blue, alpha.
  Rgba8,
  /// 3 bytes per pixel: red, green, blue.
  Rgb8,
  /// 1 byte per pixel: grayscale.
  Gray8,
  /// 2 bytes per pixel: grayscale + alpha.
  GrayAlpha8,
}

/// Generic data source for geometry buffers.
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
