//! Asset definitions — resources loaded before rendering.
//!
//! Assets CAN allocate (`Vec`, `PathBuf`, etc.) unlike commands.
//! They are loaded once via `Backend::load_assets()` and referenced
//! from commands by `ResourceId<T>`.

mod private
{
  use std::path::PathBuf;
  use nohash_hasher::IntSet;
  use crate::types::{ ResourceId, SamplerFilter, asset };

  // ============================================================================
  // Asset container
  // ============================================================================

  /// Collection of all resources needed for rendering.
  /// Loaded into a backend once before submitting commands.
  #[ derive( Debug ) ]
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
  #[ derive( Debug, error_tools::Error ) ]
  pub enum ValidationError
  {
    /// Two assets of the same type share the same [`ResourceId`].
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
    #[ inline ]
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      match self
      {
        Self::DuplicateId { kind, index } => write!( f, "duplicate {kind} id: {index}" ),
      }
    }
  }

  impl Assets
  {
    /// Validates that no two assets of the same type share a [`ResourceId`].
    /// Returns all errors found (does not stop at first).
    #[ inline ]
    #[ must_use ]
    pub fn validate( &self ) -> Vec< ValidationError >
    {
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

      let mut errors = Vec::new();

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

  /// Helper trait to extract [`ResourceId`] from asset structs.
  trait HasId< T >
  {
    fn resource_id( &self ) -> ResourceId< T >;
  }

  impl HasId< asset::Font > for FontAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Font > { self.id }
  }

  impl HasId< asset::Image > for ImageAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Image > { self.id }
  }

  impl HasId< asset::Sprite > for SpriteAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Sprite > { self.id }
  }

  impl HasId< asset::Geometry > for GeometryAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Geometry > { self.id }
  }

  impl HasId< asset::Gradient > for GradientAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Gradient > { self.id }
  }

  impl HasId< asset::Pattern > for PatternAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Pattern > { self.id }
  }

  impl HasId< asset::ClipMask > for ClipMaskAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::ClipMask > { self.id }
  }

  impl HasId< asset::Path > for PathAsset
  {
    #[ inline ]
    fn resource_id( &self ) -> ResourceId< asset::Path > { self.id }
  }

  // ============================================================================
  // Individual asset types
  // ============================================================================

  /// A font loaded from a file path.
  #[ derive( Debug ) ]
  pub struct FontAsset
  {
    /// Unique resource identifier.
    pub id : ResourceId< asset::Font >,
    /// Path to the font file.
    pub source : PathBuf,
  }

  /// An image asset with source and sampling configuration.
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
  pub struct ClipMaskAsset
  {
    /// Unique resource identifier.
    pub id : ResourceId< asset::ClipMask >,
    /// Path segments defining the clip shape.
    pub segments : Vec< PathSegment >,
  }

  /// Stored path (e.g. for text-on-path references).
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
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
  #[ derive( Debug ) ]
  pub enum Source
  {
    /// File path to load data from.
    Path( PathBuf ),
    /// Raw byte data in memory.
    Bytes( Vec< u8 > ),
  }

  /// Primitive data type for geometry buffers.
  #[ derive( Debug ) ]
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

}

mod_interface::mod_interface!
{
  own use Assets;
  own use ValidationError;
  own use FontAsset;
  own use ImageAsset;
  own use SpriteAsset;
  own use GeometryAsset;
  own use GradientAsset;
  own use GradientStop;
  own use GradientKind;
  own use PatternAsset;
  own use ClipMaskAsset;
  own use PathAsset;
  own use PathSegment;
  own use ImageSource;
  own use PixelFormat;
  own use Source;
  own use DataType;
}
