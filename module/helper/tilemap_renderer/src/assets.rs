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
  /// Loaded font assets.
  pub fonts : Vec< FontAsset >,
  /// Loaded image assets.
  pub images : Vec< ImageAsset >,
  /// Sprite (sub-image) assets.
  pub sprites : Vec< SpriteAsset >,
  /// Geometry (mesh) assets.
  pub geometries : Vec< GeometryAsset >,
  /// Gradient fill assets.
  pub gradients : Vec< GradientAsset >,
  /// Tiling pattern assets.
  pub patterns : Vec< PatternAsset >,
  /// Clip mask assets.
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
  DuplicateId
  {
    /// Asset type name.
    kind : &'static str,
    /// Duplicate resource index.
    index : u32,
  },
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

/// A font loaded from a file path.
pub struct FontAsset
{
  /// Unique resource identifier.
  pub id : ResourceId< asset::Font >,
  /// Path to the font file.
  pub source : PathBuf,
}

/// An image asset with source and sampling configuration.
pub struct ImageAsset
{
  /// Unique resource identifier.
  pub id : ResourceId< asset::Image >,
  /// Where to load the image data from.
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
  /// Unique resource identifier.
  pub id : ResourceId< asset::Sprite >,
  /// The source image (sprite sheet) this sprite is cut from.
  pub sheet : ResourceId< asset::Image >,
  /// Region within the sheet: `[x, y, width, height]` in pixels.
  pub region : [ f32; 4 ],
}

/// Mesh geometry with positions, UVs, and indices.
pub struct GeometryAsset
{
  /// Unique resource identifier.
  pub id : ResourceId< asset::Geometry >,
  /// Vertex positions (flat: [x0, y0, x1, y1, ...]).
  pub positions : Source,
  /// Optional UV coordinates (flat: [u0, v0, u1, v1, ...]).
  /// GPU: used for texture mapping in fragment shader.
  /// SVG: ignored — SVG uses pattern fill which tiles by bounding box.
  pub uvs : Option< Source >,
  /// Optional vertex indices for indexed drawing.
  pub indices : Option< Source >,
  /// Primitive data type for index/position buffers.
  pub data_type : DataType,
}

/// Gradient definition.
/// SVG: `<linearGradient>` / `<radialGradient>` in `<defs>`.
/// GPU: uploaded as a 1D texture or evaluated analytically in shader.
pub struct GradientAsset
{
  /// Unique resource identifier.
  pub id : ResourceId< asset::Gradient >,
  /// Linear or radial gradient parameters.
  pub kind : GradientKind,
  /// Color stops along the gradient.
  pub stops : Vec< GradientStop >,
}

/// A single color stop in a gradient.
#[ derive( Debug, Clone, Copy ) ]
pub struct GradientStop
{
  /// Position along gradient, 0.0 to 1.0.
  pub offset : f32,
  /// RGBA color at this stop.
  pub color : [ f32; 4 ],
}

/// Shape of a gradient (linear or radial).
#[ derive( Debug, Clone, Copy ) ]
pub enum GradientKind
{
  /// Linear gradient between two points.
  Linear
  {
    /// Start point [x, y].
    start : [ f32; 2 ],
    /// End point [x, y].
    end : [ f32; 2 ],
  },
  /// Radial gradient from center outward.
  Radial
  {
    /// Center point [x, y].
    center : [ f32; 2 ],
    /// Outer radius.
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
  /// Unique resource identifier.
  pub id : ResourceId< asset::Pattern >,
  /// The image to tile.
  pub content : ResourceId< asset::Image >,
  /// Tile width in pixels.
  pub width : f32,
  /// Tile height in pixels.
  pub height : f32,
}

/// A clip mask — a shape that limits rendering to its interior.
/// SVG: `<clipPath>` in `<defs>`, elements use `clip-path="url(#...)"`.
/// GPU: draw clip shape into stencil buffer, enable stencil test for content.
pub struct ClipMaskAsset
{
  /// Unique resource identifier.
  pub id : ResourceId< asset::ClipMask >,
  /// Path segments defining the clip shape.
  pub segments : Vec< PathSegment >,
}

/// Stored path (e.g. for text-on-path references).
pub struct PathAsset
{
  /// Unique resource identifier.
  pub id : ResourceId< asset::Path >,
  /// Path segments defining the curve.
  pub segments : Vec< PathSegment >,
}

/// Path segment for use in Assets.
#[ derive( Debug, Clone, Copy ) ]
pub enum PathSegment
{
  /// Move pen to position (x, y).
  MoveTo( f32, f32 ),
  /// Draw line to position (x, y).
  LineTo( f32, f32 ),
  /// Quadratic Bezier curve to (x, y) with control point (cx, cy).
  QuadTo
  {
    /// Control point X.
    cx : f32,
    /// Control point Y.
    cy : f32,
    /// End point X.
    x : f32,
    /// End point Y.
    y : f32,
  },
  /// Cubic Bezier curve to (x, y) with two control points.
  CubicTo
  {
    /// First control point X.
    c1x : f32,
    /// First control point Y.
    c1y : f32,
    /// Second control point X.
    c2x : f32,
    /// Second control point Y.
    c2y : f32,
    /// End point X.
    x : f32,
    /// End point Y.
    y : f32,
  },
  /// Elliptical arc to (x, y).
  ArcTo
  {
    /// Ellipse X radius.
    rx : f32,
    /// Ellipse Y radius.
    ry : f32,
    /// Ellipse rotation in radians.
    rotation : f32,
    /// Use the large arc sweep.
    large_arc : bool,
    /// Sweep direction flag.
    sweep : bool,
    /// End point X.
    x : f32,
    /// End point Y.
    y : f32,
  },
  /// Close the current sub-path.
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
    /// Raw pixel bytes.
    bytes : Vec< u8 >,
    /// Image width in pixels.
    width : u32,
    /// Image height in pixels.
    height : u32,
    /// Pixel layout.
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
  /// File path to load data from.
  Path( PathBuf ),
  /// Raw byte data in memory.
  Bytes( Vec< u8 > ),
}

/// Primitive data type for geometry buffers.
pub enum DataType
{
  /// Unsigned 8-bit integer.
  U8,
  /// Unsigned 16-bit integer.
  U16,
  /// Unsigned 32-bit integer.
  U32,
  /// 32-bit floating point.
  F32,
}

// ============================================================================
// Tests
// ============================================================================

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use crate::types::ResourceId;

  fn empty_assets() -> Assets
  {
    Assets
    {
      fonts : vec![],
      images : vec![],
      sprites : vec![],
      geometries : vec![],
      gradients : vec![],
      patterns : vec![],
      clip_masks : vec![],
      paths : vec![],
    }
  }

  #[ test ]
  fn validate_empty_assets()
  {
    let a = empty_assets();
    assert!( a.validate().is_empty() );
  }

  #[ test ]
  fn validate_no_duplicates()
  {
    let a = Assets
    {
      images : vec![
        ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
        ImageAsset { id : ResourceId::new( 1 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
      ],
      ..empty_assets()
    };
    assert!( a.validate().is_empty() );
  }

  #[ test ]
  fn validate_duplicate_image_ids()
  {
    let a = Assets
    {
      images : vec![
        ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
        ImageAsset { id : ResourceId::new( 5 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
      ],
      ..empty_assets()
    };
    let errors = a.validate();
    assert_eq!( errors.len(), 1 );
    let msg = format!( "{}", errors[ 0 ] );
    assert!( msg.contains( "image" ) );
    assert!( msg.contains( "5" ) );
  }

  #[ test ]
  fn validate_duplicate_geometry_ids()
  {
    let a = Assets
    {
      geometries : vec![
        GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
        GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
      ],
      ..empty_assets()
    };
    let errors = a.validate();
    assert_eq!( errors.len(), 1 );
  }

  #[ test ]
  fn validate_duplicates_across_types_ok()
  {
    // Same index in different asset types is fine
    let a = Assets
    {
      images : vec![
        ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
      ],
      geometries : vec![
        GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::U16 },
      ],
      ..empty_assets()
    };
    assert!( a.validate().is_empty() );
  }

  #[ test ]
  fn validate_multiple_duplicate_types()
  {
    let a = Assets
    {
      images : vec![
        ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
        ImageAsset { id : ResourceId::new( 0 ), source : ImageSource::Encoded( vec![] ), filter : crate::types::SamplerFilter::Linear },
      ],
      sprites : vec![
        SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 0 ), region : [ 0.0; 4 ] },
        SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 0 ), region : [ 0.0; 4 ] },
      ],
      ..empty_assets()
    };
    let errors = a.validate();
    assert_eq!( errors.len(), 2 );
  }

  #[ test ]
  fn validate_gradient_duplicates()
  {
    let stop = GradientStop { offset : 0.0, color : [ 1.0, 1.0, 1.0, 1.0 ] };
    let a = Assets
    {
      gradients : vec![
        GradientAsset { id : ResourceId::new( 0 ), kind : GradientKind::Linear { start : [ 0.0, 0.0 ], end : [ 1.0, 1.0 ] }, stops : vec![ stop ] },
        GradientAsset { id : ResourceId::new( 0 ), kind : GradientKind::Linear { start : [ 0.0, 0.0 ], end : [ 1.0, 1.0 ] }, stops : vec![ stop ] },
      ],
      ..empty_assets()
    };
    assert_eq!( a.validate().len(), 1 );
  }

  #[ test ]
  fn validate_clip_mask_duplicates()
  {
    let a = Assets
    {
      clip_masks : vec![
        ClipMaskAsset { id : ResourceId::new( 0 ), segments : vec![] },
        ClipMaskAsset { id : ResourceId::new( 0 ), segments : vec![] },
      ],
      ..empty_assets()
    };
    assert_eq!( a.validate().len(), 1 );
  }

  #[ test ]
  fn validate_path_duplicates()
  {
    let a = Assets
    {
      paths : vec![
        PathAsset { id : ResourceId::new( 3 ), segments : vec![] },
        PathAsset { id : ResourceId::new( 3 ), segments : vec![] },
      ],
      ..empty_assets()
    };
    assert_eq!( a.validate().len(), 1 );
  }
}
