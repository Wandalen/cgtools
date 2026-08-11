//! SVG backend adapter.
//!
//! Generates an SVG document using SVG 1.1 structure with some SVG 2
//! attribute conventions (bare `href=` on `<use>`, `<image>`, and
//! `<textPath>` — the `xlink:` namespace is not declared) plus CSS
//! Compositing and Blending Level 1 for blend modes. All modern
//! browsers accept this mix; strict SVG 1.1 validators (Inkscape
//! pre-1.0, Apache Batik) will flag the bare `href=`. A full migration
//! to SVG 2 is planned as a separate change.
//!
//! `mix-blend-mode` is only emitted for non-normal modes; normal
//! blending is the SVG default and needs no style attribute.
//! Supports all features: paths, text, sprites, gradients, patterns,
//! clip masks, effects, blend modes, and text-on-path.

mod private
{
  use crate::assets::
  {
    Assets,
    ClipMaskAsset,
    DataType,
    GeometryAsset,
    GradientAsset,
    GradientKind,
    ImageAsset,
    ImageSource,
    PathAsset,
    PathSegment,
    PatternAsset,
    PixelFormat,
    Source,
    SpriteAsset,
  };
  use crate::backend::
  {
    Backend,
    Capabilities,
    Output,
    RenderError,
  };
  use crate::commands::
  {
    AddMeshInstance,
    AddSpriteInstance,
    ArcTo,
    BeginGroup,
    BeginPath,
    BeginText,
    BindBatch,
    Char,
    Clear,
    CreateMeshBatch,
    CreateSpriteBatch,
    CubicTo,
    DeleteBatch,
    DrawBatch,
    Effect,
    LineTo,
    Mesh,
    MeshBatchParams,
    MoveTo,
    QuadTo,
    RemoveInstance,
    RenderCommand,
    SetMeshBatchParams,
    SetMeshInstance,
    SetSpriteBatchParams,
    SetSpriteInstance,
    Sprite,
    SpriteBatchParams,
  };
  use crate::types::
  {
    Antialias,
    Batch,
    BlendMode,
    DashStyle,
    FillRef,
    LineCap,
    LineJoin,
    RenderConfig,
    ResourceId,
    SamplerFilter,
    TextAnchor,
    Topology,
    Transform,
    asset,
  };
  use core::fmt::Write as _;
  use nohash_hasher::IntMap;
  use base64::Engine as _;

  // ============================================================================
  // SVG resource handles
  // ============================================================================

  /// Internal storage for loaded SVG resources.
  struct SvgResources
  {
    /// Map of loaded images.
    images : IntMap< ResourceId< asset::Image >, SvgImage >,
    /// Map of loaded geometries.
    geometries : IntMap< ResourceId< asset::Geometry >, SvgGeometry >,
    /// Map of created batches.
    batches : IntMap< ResourceId< Batch >, SvgBatch >,
    /// Map of generated mesh definitions ( packed `geom_id` + topology ) -> `symbol_id`
    mesh_defs : IntMap< u64, String >,
  }

  impl SvgResources
  {
    fn new() -> Self
    {
      Self
      {
        images : IntMap::default(),
        geometries : IntMap::default(),
        batches : IntMap::default(),
        mesh_defs : IntMap::default(),
      }
    }

    fn image( &self, id : ResourceId< asset::Image > ) -> Option< &SvgImage >
    {
      self.images.get( &id )
    }

    fn geometry( &self, id : ResourceId< asset::Geometry > ) -> Option< &SvgGeometry >
    {
      self.geometries.get( &id )
    }

    fn batch( &self, id : ResourceId< Batch > ) -> Option< &SvgBatch >
    {
      self.batches.get( &id )
    }

    fn store_image( &mut self, id : ResourceId< asset::Image >, img : SvgImage )
    {
      self.images.insert( id, img );
    }

    fn store_geometry( &mut self, id : ResourceId< asset::Geometry >, geom : SvgGeometry )
    {
      self.geometries.insert( id, geom );
    }

    fn store_batch( &mut self, id : ResourceId< Batch >, batch : SvgBatch )
    {
      self.batches.insert( id, batch );
    }
  }

  /// Internal representation of an SVG image.
  struct SvgImage
  {
    /// Original width of the image.
    width : u32,
    /// Original height of the image.
    height : u32,
  }

  /// Internal representation of an SVG geometry.
  struct SvgGeometry
  {
    /// Flattened vertex positions [x0, y0, x1, y1, ...].
    positions : Vec< f32 >,
    /// Optional vertex indices.
    indices : Option< Vec< u32 > >,
  }

  /// Internal representation of a batch in SVG.
  enum SvgBatch
  {
    /// A sprite batch.
    Sprite
    {
      /// Instances currently in the batch.
      instances : Vec< AddSpriteInstance >,
      /// Parameters common to all instances.
      params : SpriteBatchParams,
    },
    /// A mesh batch.
    Mesh
    {
      /// Instances currently in the batch.
      instances : Vec< AddMeshInstance >,
      /// Parameters common to all instances.
      params : MeshBatchParams,
    },
  }

  // ============================================================================
  // Backend struct
  // ============================================================================

  /// SVG renderer backend.
  ///
  /// ```ignore
  /// let mut svg = SvgBackend::new( 800, 600 );
  /// svg.load_assets( &assets )?;
  /// svg.submit( &commands )?;
  /// let Output::String( doc ) = svg.output()? else { unreachable!() };
  /// ```
  ///
  /// # Known limitations
  ///
  /// - **Font assets are currently ignored.** `Assets.fonts` is accepted by
  ///   `load_assets` but no `@font-face`/`<font-face>` definitions are emitted,
  ///   and `<text>` elements carry no `font-family`. All text renders in the
  ///   SVG viewer's default font regardless of the fonts supplied.
  ///   `Capabilities::text` stays `true` because text *rendering* works —
  ///   only font *selection* is unimplemented.
  pub struct SvgBackend
  {
    config : RenderConfig,
    /// Manager for SVG string buffer with section indices.
    content : SvgContentManager,
    // -- streaming state --
    path_data : String,
    path_style : Option< BeginPath >,
    text_buf : String,
    text_style : Option< BeginText >,
    group_depth : u32,
    filter_counter : u32,
    resources : SvgResources,
    /// Currently bound batch for recording instances.
    recording_batch : Option< ResourceId< Batch > >,
    /// Offset applied to all visual elements in the SVG.
    viewport_offset : [ f32; 2 ],
    /// Scale applied to all visual elements in the SVG.
    viewport_scale : f32,
  }

  impl SvgBackend
  {
    /// Creates a new SVG backend from render config.
    #[ inline ]
    #[ must_use ]
    pub fn new( config : RenderConfig ) -> Self
    {
      Self
      {
        config,
        content : SvgContentManager::new( config.width, config.height, Self::shape_rendering_attr( config.antialias ) ),
        path_data : String::new(),
        path_style : None,
        text_buf : String::new(),
        text_style : None,
        group_depth : 0,
        filter_counter : 0,
        resources : SvgResources::new(),
        recording_batch : None,
        viewport_offset : [ 0.0, 0.0 ],
        viewport_scale : 1.0,
      }
    }

    /// Returns the current viewport offset `[x, y]`.
    #[ inline ]
    #[ must_use ]
    pub fn viewport_offset( &self ) -> [ f32; 2 ] { self.viewport_offset }

    /// Sets the viewport offset `[x, y]`.
    ///
    /// Immediately updates the top-level `<g transform>` wrapper so all already-rendered
    /// elements reflect the new position without re-submission.
    #[ inline ]
    pub fn set_viewport_offset( &mut self, offset : [ f32; 2 ] )
    {
      self.viewport_offset = offset;
      self.content.update_viewport_transform( self.viewport_offset, self.viewport_scale );
    }

    /// Returns the current viewport scale (zoom factor).
    #[ inline ]
    #[ must_use ]
    pub fn viewport_scale( &self ) -> f32 { self.viewport_scale }

    /// Sets the viewport scale (zoom factor).
    ///
    /// Immediately updates the top-level `<g transform>` wrapper so all already-rendered
    /// elements reflect the new zoom without re-submission.
    #[ inline ]
    pub fn set_viewport_scale( &mut self, scale : f32 )
    {
      self.viewport_scale = scale;
      self.content.update_viewport_transform( self.viewport_offset, self.viewport_scale );
    }

    fn shape_rendering_attr( antialias : Antialias ) -> &'static str
    {
      match antialias
      {
        Antialias::None => " shape-rendering=\"crispEdges\"",
        Antialias::Default => "",
        Antialias::High => " shape-rendering=\"geometricPrecision\"",
      }
    }

    fn color_to_svg( color : &[ f32; 4 ] ) -> String
    {
      // f32-to-u8 `as` cast saturates: values < 0.0 clamp to 0, values > 1.0 clamp to 255.
      // No explicit range check is needed; out-of-range input saturates silently.
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      let ( r, g, b, a ) =
      (
        ( color[ 0 ] * 255.0 ) as u8,
        ( color[ 1 ] * 255.0 ) as u8,
        ( color[ 2 ] * 255.0 ) as u8,
        color[ 3 ],
      );

      // Always emit rgb() — SVG 1.1 does not recognize rgba(); alpha is carried
      // via a separate *-opacity attribute generated by `opacity_attr`.
      let _ = a;
      format!( "rgb({r},{g},{b})" )
    }

    /// Produces an SVG opacity attribute (e.g. ` fill-opacity="0.5"`) for colors
    /// whose alpha is < 1.0; returns an empty string for fully opaque colors.
    /// `attr_name` selects the SVG attribute context (`fill-opacity`,
    /// `stroke-opacity`, `stop-opacity`, `flood-opacity`, `opacity`).
    fn opacity_attr( attr_name : &str, color : &[ f32; 4 ] ) -> String
    {
      let a = color[ 3 ].clamp( 0.0, 1.0 );
      if ( a - 1.0 ).abs() < f32::EPSILON { String::new() }
      else { format!( " {attr_name}=\"{a}\"" ) }
    }

    fn fill_to_svg( fill : &FillRef ) -> String
    {
      match fill
      {
        FillRef::None => "none".to_string(),
        FillRef::Solid( color ) => Self::color_to_svg( color ),
        FillRef::Gradient( id ) => format!( "url(#grad_{})", id.inner() ),
        FillRef::Pattern( id ) => format!( "url(#pat_{})", id.inner() ),
      }
    }

    fn transform_to_svg( &self, t : &Transform ) -> String
    {
      Self::transform_to_svg_static( t, self.config.height )
    }

    /// Converts a world-space [`Transform`] to an SVG `transform` attribute string.
    ///
    /// Handles the Y-up → Y-down coordinate flip only. Viewport pan/zoom is applied
    /// by the top-level `<g>` wrapper managed by [`SvgContentManager`], so it must
    /// **not** be baked into individual element transforms.
    fn transform_to_svg_static( t : &Transform, height : u32 ) -> String
    {
      let mut parts = Vec::new();

      // Y-up (0,0 = bottom-left) → SVG Y-down (0,0 = top-left)
      let pos_x = t.position[ 0 ];
      // `height` is a viewport/surface dimension in pixels; f32's 23-bit mantissa
      // only loses precision above 2^24 (16,777,216px) tall, which is not a
      // representable rendering surface, so the cast is lossless in practice.
      #[ allow( clippy::cast_precision_loss ) ]
      let pos_y = height as f32 - t.position[ 1 ];

      if pos_x != 0.0 || pos_y != 0.0
      {
        parts.push( format!( "translate({pos_x},{pos_y})" ) );
      }
      if t.rotation != 0.0
      {
        // CCW in Y-up → CW in SVG Y-down
        parts.push( format!( "rotate({})", ( -t.rotation ).to_degrees() ) );
      }
      // Always emit scale: Y-up → SVG Y-down requires negating scale Y
      parts.push( format!( "scale({},{})", t.scale[ 0 ], -t.scale[ 1 ] ) );
      if t.skew[ 0 ] != 0.0
      {
        parts.push( format!( "skewX({})", ( -t.skew[ 0 ] ).to_degrees() ) );
      }
      if t.skew[ 1 ] != 0.0
      {
        parts.push( format!( "skewY({})", ( -t.skew[ 1 ] ).to_degrees() ) );
      }

      if parts.is_empty()
      {
        String::new()
      }
      else
      {
        format!( " transform=\"{}\"", parts.join( " " ) )
      }
    }

    /// Emits a raw local transform — no viewport Y-flip.
    /// Used for instances inside an already Y-flipped `<g>` parent group.
    fn transform_to_svg_local( t : &Transform ) -> String
    {
      let mut parts = Vec::new();

      if t.position[ 0 ] != 0.0 || t.position[ 1 ] != 0.0
      {
        parts.push( format!( "translate({},{})", t.position[ 0 ], t.position[ 1 ] ) );
      }
      if t.rotation != 0.0
      {
        parts.push( format!( "rotate({})", t.rotation.to_degrees() ) );
      }
      if ( t.scale[ 0 ] - 1.0 ).abs() > f32::EPSILON || ( t.scale[ 1 ] - 1.0 ).abs() > f32::EPSILON
      {
        parts.push( format!( "scale({},{})", t.scale[ 0 ], t.scale[ 1 ] ) );
      }
      if t.skew[ 0 ] != 0.0
      {
        parts.push( format!( "skewX({})", t.skew[ 0 ].to_degrees() ) );
      }
      if t.skew[ 1 ] != 0.0
      {
        parts.push( format!( "skewY({})", t.skew[ 1 ].to_degrees() ) );
      }

      if parts.is_empty()
      {
        String::new()
      }
      else
      {
        format!( " transform=\"{}\"", parts.join( " " ) )
      }
    }

    /// Blend-mode attribute fragment. Returns an empty string for
    /// `BlendMode::Normal` because normal is the SVG default; emitting a
    /// `style="mix-blend-mode:normal"` on every element would add no
    /// information and pollute output. Non-normal modes produce the full
    /// ` style="mix-blend-mode:X"` fragment, including the leading space.
    fn blend_to_svg( blend : BlendMode ) -> &'static str
    {
      match blend
      {
        BlendMode::Normal => "",
        BlendMode::Multiply => " style=\"mix-blend-mode:multiply\"",
        BlendMode::Screen => " style=\"mix-blend-mode:screen\"",
        BlendMode::Overlay => " style=\"mix-blend-mode:overlay\"",
        BlendMode::Add => " style=\"mix-blend-mode:lighter\"",
      }
    }

    fn linecap_to_svg( cap : LineCap ) -> &'static str
    {
      match cap
      {
        LineCap::Butt => "butt",
        LineCap::Round => "round",
        LineCap::Square => "square",
      }
    }

    fn linejoin_to_svg( join : LineJoin ) -> &'static str
    {
      match join
      {
        LineJoin::Miter => "miter",
        LineJoin::Round => "round",
        LineJoin::Bevel => "bevel",
      }
    }

    fn dash_to_svg( dash : &DashStyle ) -> String
    {
      let values : Vec< String > = dash
      .pattern
      .iter()
      .take_while( | &&v | v > 0.0 )
      .map( std::string::ToString::to_string )
      .collect();

      if values.is_empty()
      {
        String::new()
      }
      else
      {
        let mut s = format!( " stroke-dasharray=\"{}\"", values.join( "," ) );
        if dash.offset != 0.0
        {
          let _ = write!( s, " stroke-dashoffset=\"{}\"", dash.offset );
        }
        s
      }
    }

    fn anchor_to_svg( anchor : TextAnchor ) -> ( &'static str, &'static str )
    {
      let h = match anchor
      {
        TextAnchor::TopLeft | TextAnchor::CenterLeft | TextAnchor::BottomLeft => "start",
        TextAnchor::TopCenter | TextAnchor::Center | TextAnchor::BottomCenter => "middle",
        TextAnchor::TopRight | TextAnchor::CenterRight | TextAnchor::BottomRight => "end",
      };
      let v = match anchor
      {
        TextAnchor::TopLeft | TextAnchor::TopCenter | TextAnchor::TopRight => "hanging",
        TextAnchor::CenterLeft | TextAnchor::Center | TextAnchor::CenterRight => "central",
        TextAnchor::BottomLeft | TextAnchor::BottomCenter | TextAnchor::BottomRight => "baseline",
      };
      ( h, v )
    }

    /// Encodes raw pixel bytes into a PNG file in memory.
    /// Returns `None` if the dimensions don't match the byte count.
    fn bitmap_to_png( bytes : &[ u8 ], width : u32, height : u32, format : PixelFormat ) -> Option< Vec< u8 > >
    {
      use image::DynamicImage;

      let dynamic = match format
      {
        PixelFormat::Rgba8 =>
          DynamicImage::ImageRgba8( image::RgbaImage::from_raw( width, height, bytes.to_vec() )? ),
        PixelFormat::Rgb8 =>
          DynamicImage::ImageRgb8( image::RgbImage::from_raw( width, height, bytes.to_vec() )? ),
        PixelFormat::Gray8 =>
          DynamicImage::ImageLuma8( image::GrayImage::from_raw( width, height, bytes.to_vec() )? ),
        PixelFormat::GrayAlpha8 =>
          DynamicImage::ImageLumaA8( image::GrayAlphaImage::from_raw( width, height, bytes.to_vec() )? ),
      };

      let mut png = Vec::new();
      // `core::io` is unstable (feature `core_io`, rust-lang/rust#154046) on this
      // toolchain's stable channel, so clippy's suggested `core::` swap does not
      // compile here; `std::io::Cursor` is the only usable path.
      #[ allow( clippy::std_instead_of_core ) ]
      dynamic.write_to( &mut std::io::Cursor::new( &mut png ), image::ImageFormat::Png ).ok()?;
      Some( png )
    }

    /// Extracts width and height from a PNG byte buffer by reading the IHDR chunk.
    /// Returns `None` if the buffer is too short or does not start with the PNG signature.
    /// Extracts (width, height) from an encoded image buffer using the `image`
    /// crate's format guesser. Supports any format the crate can decode the
    /// dimensions of — PNG, JPEG, GIF, WebP, BMP, TIFF, etc. Returns `None`
    /// when the format is unrecognized or the header is malformed.
    // `core::io` is unstable (feature `core_io`, rust-lang/rust#154046) on this
    // toolchain's stable channel, so clippy's suggested `core::` swap does not
    // compile here; `std::io::Cursor` is the only usable path. Attribute is at
    // function level because the call is this function's tail expression,
    // where item-level attributes (not statement-level) are required on stable.
    #[ allow( clippy::std_instead_of_core ) ]
    fn image_dimensions( bytes : &[ u8 ] ) -> Option< ( u32, u32 ) >
    {
      image::ImageReader::new( std::io::Cursor::new( bytes ) )
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
    }

    /// Detects the MIME type of an encoded image by inspecting its magic bytes.
    /// Falls back to `image/png` when the signature is unknown, which matches
    /// the prior behavior for well-formed PNG inputs.
    fn detect_image_mime( bytes : &[ u8 ] ) -> &'static str
    {
      if bytes.starts_with( &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] ) { return "image/png"; }
      if bytes.starts_with( &[ 0xff, 0xd8, 0xff ] ) { return "image/jpeg"; }
      if bytes.starts_with( b"GIF87a" ) || bytes.starts_with( b"GIF89a" ) { return "image/gif"; }
      if bytes.len() >= 12 && bytes.starts_with( b"RIFF" ) && &bytes[ 8..12 ] == b"WEBP" { return "image/webp"; }
      if bytes.starts_with( b"<svg" ) || bytes.starts_with( b"<?xml" ) { return "image/svg+xml"; }
      "image/png"
    }

    // Legacy PNG-only IHDR reader. Production code uses `image_dimensions` for
    // all formats; retained for its unit tests which exercise the hand-rolled
    // path as a sanity check on the `image` crate's behavior for PNG inputs.
    #[ cfg( test ) ]
    fn png_dimensions( bytes : &[ u8 ] ) -> Option< ( u32, u32 ) >
    {
      // PNG layout: 8-byte signature + 4-byte chunk length + 4-byte "IHDR" + 4-byte width + 4-byte height
      const SIG : &[ u8 ] = &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ];
      if bytes.len() < 24 || !bytes.starts_with( SIG ) { return None; }
      let w = u32::from_be_bytes( bytes[ 16..20 ].try_into().ok()? );
      let h = u32::from_be_bytes( bytes[ 20..24 ].try_into().ok()? );
      Some( ( w, h ) )
    }

    fn clip_attr( clip : Option< &ResourceId< asset::ClipMask > > ) -> String
    {
      match clip
      {
        Some( id ) => format!( " clip-path=\"url(#clip_{})\"", id.inner() ),
        None => String::new(),
      }
    }

    /// Returns the current filter id and bumps the counter.
    /// Errors on `u32::MAX` overflow — would otherwise produce duplicate filter
    /// IDs (wrapping) or panic (debug). The limit is effectively unreachable
    /// (~4B filters in one `submit`), but a clean error beats silent invalid XML.
    fn bump_filter_counter( counter : &mut u32 ) -> Result< u32, RenderError >
    {
      let id = *counter;
      *counter = counter.checked_add( 1 ).ok_or_else( ||
        RenderError::BackendError( "svg: filter_counter exhausted (u32::MAX filters in one frame)".to_string() )
      )?;
      Ok( id )
    }

    fn tint_filter_attr( &mut self, tint : &[ f32; 4 ] ) -> Result< String, RenderError >
    {
      Self::tint_filter_attr_split( tint, &mut self.content, &mut self.filter_counter )
    }

    fn tint_filter_attr_split( tint : &[ f32; 4 ], content : &mut SvgContentManager, counter : &mut u32 ) -> Result< String, RenderError >
    {
      let is_white =
        ( tint[ 0 ] - 1.0 ).abs() < f32::EPSILON
        && ( tint[ 1 ] - 1.0 ).abs() < f32::EPSILON
        && ( tint[ 2 ] - 1.0 ).abs() < f32::EPSILON
        && ( tint[ 3 ] - 1.0 ).abs() < f32::EPSILON;

      if is_white
      {
        return Ok( String::new() );
      }

      let id = Self::bump_filter_counter( counter )?;

      let filter_def = format!
      (
        "<filter id=\"tint_{}\"><feColorMatrix type=\"matrix\" values=\"{} 0 0 0 0 0 {} 0 0 0 0 0 {} 0 0 0 0 0 {} 0\"/></filter>",
        id, tint[ 0 ], tint[ 1 ], tint[ 2 ], tint[ 3 ]
      );
      content.push_frame_def( &filter_def );

      Ok( format!( " filter=\"url(#tint_{id})\"" ) )
    }

    /// Returns a fill string: `url(#mesh_tex_N)` for textured meshes, or the regular fill.
    /// Generates a `<pattern>` def for the texture if needed.
    fn texture_or_fill( &mut self, texture : Option< ResourceId< asset::Image > >, fill : &FillRef ) -> String
    {
      Self::texture_or_fill_split( texture, fill, &self.resources, &mut self.content )
    }

    fn texture_or_fill_split
    (
      texture : Option< ResourceId< asset::Image > >,
      fill : &FillRef,
      resources : &SvgResources,
      content : &mut SvgContentManager,
    ) -> String
    {
      if let Some( img_id ) = texture
        && let Some( img ) = resources.image( img_id )
          && img.width > 0 && img.height > 0
          {
            let pat_id = format!( "mesh_tex_{}", img_id.inner() );
            let pat_def = format!
            (
              "<pattern id=\"{}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\"><use href=\"#img_{}\" width=\"{}\" height=\"{}\"/></pattern>",
              pat_id, img.width, img.height, img_id.inner(), img.width, img.height
            );
            content.push_frame_def( &pat_def );
            return format!( "url(#{pat_id})" );
          }
      Self::fill_to_svg( fill )
    }

    fn segment_to_svg( seg : &PathSegment ) -> String
    {
      match seg
      {
        PathSegment::MoveTo( x, y ) => format!( "M {x} {y}" ),
        PathSegment::LineTo( x, y ) => format!( "L {x} {y}" ),
        PathSegment::QuadTo { cx, cy, x, y } => format!( "Q {cx} {cy} {x} {y}" ),
        PathSegment::CubicTo { c1x, c1y, c2x, c2y, x, y } => format!( "C {c1x} {c1y} {c2x} {c2y} {x} {y}" ),
        PathSegment::ArcTo { rx, ry, rotation, large_arc, sweep, x, y } =>
        {
          let rotation_deg = rotation.to_degrees();
          format!
          (
            "A {rx} {ry} {rotation_deg} {} {} {x} {y}",
            i32::from( *large_arc ),
            i32::from( *sweep )
          )
        }
        PathSegment::Close => "Z".to_string(),
      }
    }

    /// Flushes current path buffer into SVG.
    fn flush_path( &mut self )
    {
      let Some( style ) = self.path_style.take() else
      {
        return;
      };

      let fill = Self::fill_to_svg( &style.fill );
      let stroke = Self::color_to_svg( &style.stroke_color );
      // Alpha is emitted as SVG 1.1 *-opacity attributes (rgba() is not SVG 1.1).
      let fill_opacity = match &style.fill
      {
        FillRef::Solid( c ) => Self::opacity_attr( "fill-opacity", c ),
        _ => String::new(),
      };
      let stroke_opacity = Self::opacity_attr( "stroke-opacity", &style.stroke_color );
      let transform = self.transform_to_svg( &style.transform );
      let clip = Self::clip_attr( style.clip.as_ref() );
      let dash = Self::dash_to_svg( &style.stroke_dash );
      let blend = Self::blend_to_svg( style.blend );

      let path = format!
      (
        "<path d=\"{}\" fill=\"{}\"{} stroke=\"{}\"{} stroke-width=\"{}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}{}{}{}/>",
        self.path_data.trim(),
        fill,
        fill_opacity,
        stroke,
        stroke_opacity,
        style.stroke_width,
        Self::linecap_to_svg( style.stroke_cap ),
        Self::linejoin_to_svg( style.stroke_join ),
        dash,
        transform,
        clip,
        blend,
      );
      self.content.push_body( &path );
      self.path_data.clear();
    }

    /// Flushes current text buffer into SVG.
    fn flush_text( &mut self )
    {
      let Some( style ) = self.text_style.take() else
      {
        return;
      };

      let fill = Self::color_to_svg( &style.color );
      let fill_opacity = Self::opacity_attr( "fill-opacity", &style.color );
      let ( anchor, baseline ) = Self::anchor_to_svg( style.anchor );
      let clip = Self::clip_attr( style.clip.as_ref() );

      let t = Transform { position : style.position, ..Default::default() };
      let transform = self.transform_to_svg( &t );

      // Escape XML special chars so a character stream like '<','s','c','r','i','p','t','>'
      // cannot close the <text> element and inject arbitrary SVG markup or <script>.
      let escaped = Self::escape_xml_text( &self.text_buf );

      if let Some( path_id ) = style.along_path
      {
        let text = format!
        (
          "<text font-size=\"{}\" fill=\"{}\"{} text-anchor=\"{}\" dominant-baseline=\"{}\"{}{}>\n          <textPath href=\"#path_{}\">{}</textPath></text>",
          style.size, fill, fill_opacity, anchor, baseline, transform, clip,
          path_id.inner(), escaped,
        );
        self.content.push_body( &text );
      }
      else
      {
        let text = format!
        (
          "<text font-size=\"{}\" fill=\"{}\"{} text-anchor=\"{}\" dominant-baseline=\"{}\"{}{}>\n          {}</text>",
          style.size, fill, fill_opacity, anchor, baseline, transform, clip,
          escaped,
        );
        self.content.push_body( &text );
      }
      self.text_buf.clear();
    }

    /// Converts a filesystem path to a URI reference suitable for an SVG/HTML
    /// `href` attribute. Normalizes Windows backslashes to forward slashes and
    /// percent-encodes every byte outside the RFC 3986 unreserved set and the
    /// path-safe separator `/`. This simultaneously:
    ///
    /// - yields a valid URI reference (browsers require e.g. space → `%20`)
    /// - neutralizes attribute-injection payloads (quote, `<`, `>`, `&` are
    ///   encoded and cannot close the attribute or inject markup)
    fn path_to_href( s : &str ) -> String
    {
      use core::fmt::Write as _;
      let mut out = String::with_capacity( s.len() );
      for byte in s.bytes()
      {
        let c = byte as char;
        let safe = matches!
        (
          c,
          'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '/'
        );
        if safe
        {
          out.push( c );
        }
        else if c == '\\'
        {
          // Normalize Windows path separators to URI forward slash.
          out.push( '/' );
        }
        else
        {
          // Percent-encode as hex (uppercase per RFC 3986).
          // Writing into a String never fails; the Result is discarded.
          let _ = write!( out, "%{byte:02X}" );
        }
      }
      out
    }

    /// Escapes the five XML predefined entities so that arbitrary character
    /// content can safely be inserted as PCDATA or attribute values.
    fn escape_xml_text( s : &str ) -> String
    {
      let mut out = String::with_capacity( s.len() );
      for c in s.chars()
      {
        match c
        {
          '&'  => out.push_str( "&amp;"  ),
          '<'  => out.push_str( "&lt;"   ),
          '>'  => out.push_str( "&gt;"   ),
          '"'  => out.push_str( "&quot;" ),
          '\'' => out.push_str( "&apos;" ),
          _    => out.push( c ),
        }
      }
      out
    }

    // ---- Asset loaders ----

    fn load_gradients( &mut self, gradients : &[ GradientAsset ] )
    {
      for grad in gradients
      {
        let stops = grad.stops.iter().fold( String::new(), | mut acc, s |
        {
          let _ = write!
          (
            acc,
            "<stop offset=\"{}\" stop-color=\"{}\"{}/>",
            s.offset,
            Self::color_to_svg( &s.color ),
            Self::opacity_attr( "stop-opacity", &s.color ),
          );
          acc
        });

        let grad_type = match &grad.kind
        {
          GradientKind::Linear { .. } => "linearGradient",
          GradientKind::Radial { .. } => "radialGradient",
        };

        // `gradientUnits="userSpaceOnUse"` makes coordinates world-space (pixels),
        // matching the crate's Transform / Path API. SVG's default of
        // `objectBoundingBox` would reinterpret them as 0..1 fractions of the
        // element's bounding box and collapse typical pixel-space inputs.
        let mut grad_def = format!( "<{} id=\"grad_{}\" gradientUnits=\"userSpaceOnUse\"", grad_type, grad.id.inner() );

        match &grad.kind
        {
          GradientKind::Linear { start, end } =>
          {
            let _ = write!
            (
              grad_def,
              " x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">{}",
              start[ 0 ], start[ 1 ], end[ 0 ], end[ 1 ], stops
            );
          }
          GradientKind::Radial { center, radius, focal } =>
          {
            let _ = write!
            (
              grad_def,
              " cx=\"{}\" cy=\"{}\" r=\"{}\" fx=\"{}\" fy=\"{}\">{}",
              center[ 0 ], center[ 1 ], radius, focal[ 0 ], focal[ 1 ], stops
            );
          }
        }
        let _ = write!( grad_def, "</{grad_type}>" );
        self.content.push_asset_def( &grad_def );
      }
    }

    fn load_patterns( &mut self, patterns : &[ PatternAsset ] )
    {
      for pat in patterns
      {
        let pat_def = format!
        (
          "<pattern id=\"pat_{}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\"><use href=\"#img_{}\" width=\"{}\" height=\"{}\"/></pattern>",
          pat.id.inner(), pat.width, pat.height, pat.content.inner(), pat.width, pat.height,
        );
        self.content.push_asset_def( &pat_def );
      }
    }

    fn load_clip_masks( &mut self, clip_masks : &[ ClipMaskAsset ] )
    {
      for mask in clip_masks
      {
        let mut d = String::new();
        for seg in &mask.segments
        {
          let _ = write!( d, "{} ", Self::segment_to_svg( seg ) );
        }
        let clip_def = format!
        (
          "<clipPath id=\"clip_{}\"><path d=\"{}\"/></clipPath>",
          mask.id.inner(), d.trim()
        );
        self.content.push_asset_def( &clip_def );
      }
    }

    fn load_paths( &mut self, paths : &[ PathAsset ] )
    {
      for path in paths
      {
        let mut d = String::new();
        for seg in &path.segments
        {
          let _ = write!( d, "{} ", Self::segment_to_svg( seg ) );
        }
        let path_def = format!
        (
          "<path id=\"path_{}\" d=\"{}\"/>",
          path.id.inner(), d.trim()
        );
        self.content.push_asset_def( &path_def );
      }
    }

    /// CSS `image-rendering` fragment for a `SamplerFilter`. Empty string
    /// for `Linear` (browser default is smooth interpolation); pixelated
    /// style for `Nearest`. Prefix is a leading space so it slots into the
    /// `<image ...>` attribute list directly.
    fn filter_to_svg( filter : SamplerFilter ) -> &'static str
    {
      match filter
      {
        SamplerFilter::Linear => "",
        SamplerFilter::Nearest => " style=\"image-rendering:pixelated\"",
      }
    }

    fn load_images( &mut self, images : &[ ImageAsset ] )
    {
      // NOTE: `ImageAsset.wrap` (WrapMode::Clamp / Repeat / Mirror) is intentionally
      // ignored in the SVG backend for now.
      //
      // SVG has no native wrap-mode on `<image>` — the element draws its bitmap
      // exactly once at the given size and clamps outside. Repeat / Mirror can
      // in principle be approximated via `<pattern>` defs filled into a larger
      // `<rect>`, which is what the format's `PatternAsset` path already does
      // (see `load_patterns` below). However, applying that per-image wrapping
      // to every sprite draw call would change the command-emission pipeline in
      // ways that are out of scope for the feature that introduced
      // `WrapMode` — so for now all SVG output behaves as `Clamp` regardless of
      // the asset's declared wrap mode. GPU backends honour the field fully
      // (see `adapters/webgl.rs` → `apply_texture_wrap`).
      //
      // If / when a backend implements `Repeat` / `Mirror` here, adjust this
      // comment and update SPEC §4.1's note about SVG graceful degradation.
      for img in images
      {
        let filter = Self::filter_to_svg( img.filter );
        match &img.source
        {
          ImageSource::Bitmap { bytes, width, height, format } =>
          {
            if let Some( png ) = Self::bitmap_to_png( bytes, *width, *height, *format )
            {
              let encoded = base64::prelude::BASE64_STANDARD.encode( &png );
              let img_def = format!
              (
                "<symbol id=\"img_{}\" viewBox=\"0 0 {} {}\"><image href=\"data:image/png;base64,{}\" width=\"{}\" height=\"{}\"{}/></symbol>",
                img.id.inner(), width, height, encoded, width, height, filter
              );
              self.content.push_asset_def( &img_def );
              self.resources.store_image( img.id, SvgImage { width : *width, height : *height } );
            }
          }
          ImageSource::Encoded( bytes ) =>
          {
            // Decode dimensions for any format the `image` crate recognizes (PNG,
            // JPEG, GIF, WebP, ...) so that sprites using this sheet can render
            // with correct viewBox/use sizing.
            let ( w, h ) = Self::image_dimensions( bytes ).unwrap_or( ( 0, 0 ) );
            let mime = Self::detect_image_mime( bytes );
            let encoded = base64::prelude::BASE64_STANDARD.encode( bytes );
            // Per SVG 1.1 §11.5, `<image>` without width/height renders at 0×0.
            // Emit viewBox + explicit dimensions so `<use>` references resolve.
            // If dimensions could not be decoded (w == 0 || h == 0), fall through
            // with zero dims: `load_sprites` emits a diagnostic for that case.
            let img_def = format!
            (
              "<symbol id=\"img_{}\" viewBox=\"0 0 {} {}\"><image href=\"data:{mime};base64,{encoded}\" width=\"{}\" height=\"{}\"{}/></symbol>",
              img.id.inner(), w, h, w, h, filter
            );
            self.content.push_asset_def( &img_def );
            self.resources.store_image( img.id, SvgImage { width : w, height : h } );
          }
          ImageSource::Path( path ) =>
          {
            let href = Self::path_to_href( &path.display().to_string() );
            let img_def = format!( "<symbol id=\"img_{}\"><image href=\"{}\"{}/></symbol>", img.id.inner(), href, filter );
            self.content.push_asset_def( &img_def );
            self.resources.store_image( img.id, SvgImage { width : 0, height : 0 } );
          }
        }
      }
    }

    fn load_sprites( &mut self, sprites : &[ SpriteAsset ] )
    {
      for sprite in sprites
      {
        if let Some( sheet ) = self.resources.image( sprite.sheet )
        {
          // Zero-dim sheets come from ImageSource::Path (no file I/O performed
          // at load-assets time) or from Encoded bytes we couldn't decode.
          // Emit a warning to stderr and an HTML comment in the SVG so the
          // failure is visible rather than silent.
          if sheet.width == 0 || sheet.height == 0
          {
            eprintln!
            (
              "[tilemap_renderer:svg] warning: sprite {} references image {} with unknown dimensions — sprite will be invisible. Use ImageSource::Bitmap or Encoded with a decodable format.",
              sprite.id.inner(), sprite.sheet.inner()
            );
            let comment = format!
            (
              "<!-- sprite_{} skipped: image_{} has unknown dimensions (ImageSource::Path cannot extract without I/O) -->",
              sprite.id.inner(), sprite.sheet.inner()
            );
            self.content.push_asset_def( &comment );
            continue;
          }
          let img_def = format!
          (
            "<symbol id=\"sprite_{}\" viewBox=\"{} {} {} {}\"><use href=\"#img_{}\" width=\"{}\" height=\"{}\"/></symbol>",
            sprite.id.inner(),
            sprite.region[ 0 ], sprite.region[ 1 ], sprite.region[ 2 ], sprite.region[ 3 ],
            sprite.sheet.inner(),
            sheet.width, sheet.height
          );
          self.content.push_asset_def( &img_def );
        }
      }
    }

    fn load_geometries( &mut self, geometries : &[ GeometryAsset ] )
    {
      for geom in geometries
      {
        // `Source::Path` is read via blocking `std::fs`. On targets without a
        // filesystem (wasm32) the read fails at runtime and flows into the same
        // loud-skip diagnostics as a missing file — a stderr warning plus a
        // diagnostic HTML comment, mirroring the `ImageSource::Path` sprite
        // case above. Async `fetch()` loading is a roadmap item.
        let positions_bytes = match Self::resolve_source( &geom.positions )
        {
          Ok( bytes ) => bytes,
          Err( error ) =>
          {
            self.skip_geometry( geom.id, "positions", &error );
            continue;
          }
        };
        // `pod_collect_to_vec` copies instead of casting in place: bytes read
        // from a file carry no alignment guarantee, and `cast_slice` panics on
        // a buffer that does not happen to be 4-byte aligned.
        let positions : Vec< f32 > = bytemuck::pod_collect_to_vec( &positions_bytes );

        let indices = match &geom.indices
        {
          Some( source ) => match Self::resolve_source( source )
          {
            Ok( ibytes ) => match geom.data_type
            {
              DataType::U8  => Some( ibytes.iter().map( | &i | u32::from( i ) ).collect() ),
              DataType::U16 => Some( bytemuck::pod_collect_to_vec::< _, u16 >( &ibytes ).iter().map( | &i | u32::from( i ) ).collect() ),
              DataType::U32 => Some( bytemuck::pod_collect_to_vec( &ibytes ) ),
              DataType::F32 => None, // F32 is not a valid index type; documented in DataType::F32 doc
            },
            // A failed index source skips the whole geometry: falling back to
            // unindexed drawing would silently render different topology.
            Err( error ) =>
            {
              self.skip_geometry( geom.id, "indices", &error );
              continue;
            }
          },
          None => None,
        };

        self.resources.store_geometry( geom.id, SvgGeometry { positions, indices } );
      }
    }

    /// Resolves a geometry `Source` to owned bytes — `Bytes` verbatim, `Path`
    /// via a blocking `std::fs` read.
    fn resolve_source( source : &Source ) -> Result< std::borrow::Cow< '_, [ u8 ] >, String >
    {
      match source
      {
        Source::Bytes( bytes ) => Ok( std::borrow::Cow::Borrowed( bytes.as_slice() ) ),
        Source::Path( path ) => std::fs::read( path )
          .map( std::borrow::Cow::Owned )
          .map_err( | error | format!( "reading {} failed: {error}", path.display() ) ),
      }
    }

    /// Emits the loud-skip diagnostics for a geometry whose source could not
    /// be resolved: a stderr warning (with the error detail) plus a diagnostic
    /// HTML comment in the SVG defs. The comment interpolates only the numeric
    /// id and a static field name — never the error text, whose path content
    /// could otherwise terminate the comment early (`-->` injection).
    fn skip_geometry( &mut self, id : ResourceId< asset::Geometry >, field : &str, error : &str )
    {
      eprintln!
      (
        "[tilemap_renderer:svg] warning: geometry {} skipped — {field} source unavailable: {error}. Meshes referencing it will be absent.",
        id.inner()
      );
      let comment = format!( "<!-- geometry_{} skipped: {field} source unavailable -->", id.inner() );
      self.content.push_asset_def( &comment );
    }

    fn generate_mesh_def( &mut self, geom_id : ResourceId< asset::Geometry >, topology : Topology ) -> Option< String >
    {
      let id_u64 : u64 = u64::from( geom_id.inner() );
      let packed_key : u64 = ( id_u64 << 8 ) | u64::from( topology as u8 );

      let geom = self.resources.geometry( geom_id )?;
      let def_id = format!( "mesh_{}_{:?}", geom_id.inner(), topology );
      let mut def_content = format!( "<symbol id=\"{def_id}\" overflow=\"visible\">" );

      match topology
      {
        Topology::TriangleList =>
        {
          let idx = geom.indices.as_deref();
          let count = idx.map_or( geom.positions.len() / 2, < [ u32 ] >::len );
          for i in ( 0..count ).step_by( 3 )
          {
            let mut pts = String::new();
            let mut valid = true;
            for j in 0..3
            {
              let v_idx = idx.map_or( i + j, | v | v[ i + j ] as usize );
              let Some( &x ) = geom.positions.get( v_idx * 2 )     else { valid = false; break; };
              let Some( &y ) = geom.positions.get( v_idx * 2 + 1 ) else { valid = false; break; };
              let _ = write!( pts, "{x},{y} " );
            }
            if valid { let _ = write!( def_content, "<polygon points=\"{}\"/>", pts.trim() ); }
          }
        }
        Topology::TriangleStrip =>
        {
          let idx = geom.indices.as_deref();
          let count = idx.map_or( geom.positions.len() / 2, <[u32]>::len );
          if count < 3 { return None; }
          for i in 0..( count - 2 )
          {
            let mut pts = String::new();
            let mut valid = true;
            // Alternate winding on odd triangles to preserve consistent CCW order,
            // matching standard triangle-strip semantics (OpenGL/D3D).
            let order : [ usize; 3 ] = if i.is_multiple_of( 2 ) { [ 0, 1, 2 ] } else { [ 1, 0, 2 ] };
            for j in order
            {
              let v_idx = idx.map_or( i + j, | v | v[ i + j ] as usize );
              let Some( &x ) = geom.positions.get( v_idx * 2 )     else { valid = false; break; };
              let Some( &y ) = geom.positions.get( v_idx * 2 + 1 ) else { valid = false; break; };
              let _ = write!( pts, "{x},{y} " );
            }
            if valid { let _ = write!( def_content, "<polygon points=\"{}\"/>", pts.trim() ); }
          }
        }
        Topology::LineList | Topology::LineStrip =>
        {
          let mut pts = String::new();
          let idx = geom.indices.as_deref();
          let count = idx.map_or( geom.positions.len() / 2, <[u32]>::len );
          for i in 0..count
          {
            let v_idx = idx.map_or( i, | v | v[ i ] as usize );
            let Some( &x ) = geom.positions.get( v_idx * 2 )     else { continue; };
            let Some( &y ) = geom.positions.get( v_idx * 2 + 1 ) else { continue; };
            let _ = write!( pts, "{x},{y} " );

            if topology == Topology::LineList && ( i + 1 ).is_multiple_of( 2 )
            {
              let _ = write!( def_content, "<polyline points=\"{}\" fill=\"none\"/>", pts.trim() );
              pts.clear();
            }
          }
          if !pts.is_empty() && topology == Topology::LineStrip
          {
            let _ = write!( def_content, "<polyline points=\"{}\" fill=\"none\"/>", pts.trim() );
          }
        }
      }

      def_content.push_str( "</symbol>" );
      self.content.push_frame_def( &def_content );
      self.resources.mesh_defs.insert( packed_key, def_id.clone() );

      Some( def_id )
    }

    fn cmd_clear( &mut self, c : &Clear )
    {
      let color = Self::color_to_svg( &c.color );
      let opacity = Self::opacity_attr( "fill-opacity", &c.color );
      let rect = format!( "<rect width=\"100%\" height=\"100%\" fill=\"{color}\"{opacity}/>" );
      self.content.push_body( &rect );
    }

    fn cmd_begin_path( &mut self, bp : &BeginPath )
    {
      self.path_data.clear();
      self.path_style = Some( *bp );
    }

    fn cmd_move_to( &mut self, m : MoveTo )
    {
      let _ = write!( self.path_data, "M {} {} ", m.0, m.1 );
    }

    fn cmd_line_to( &mut self, l : LineTo )
    {
      let _ = write!( self.path_data, "L {} {} ", l.0, l.1 );
    }

    fn cmd_quad_to( &mut self, q : &QuadTo )
    {
      let _ = write!( self.path_data, "Q {} {} {} {} ", q.cx, q.cy, q.x, q.y );
    }

    fn cmd_cubic_to( &mut self, c : &CubicTo )
    {
      let _ = write!( self.path_data, "C {} {} {} {} {} {} ", c.c1x, c.c1y, c.c2x, c.c2y, c.x, c.y );
    }

    fn cmd_arc_to( &mut self, a : &ArcTo )
    {
      let _ = write!( self.path_data, "A {} {} {} {} {} {} {} ", a.rx, a.ry, a.rotation.to_degrees(), u8::from(a.large_arc), u8::from(a.sweep), a.x, a.y );
    }

    fn cmd_close_path( &mut self )
    {
      self.path_data.push_str( "Z " );
    }

    fn cmd_end_path( &mut self )
    {
      self.flush_path();
    }

    fn cmd_begin_text( &mut self, bt : &BeginText )
    {
      self.text_buf.clear();
      self.text_style = Some( *bt );
    }

    fn cmd_char( &mut self, ch : Char )
    {
      self.text_buf.push( ch.0 );
    }

    fn cmd_end_text( &mut self )
    {
      self.flush_text();
    }

    fn cmd_mesh( &mut self, m : &Mesh )
    {
      let packed_key : u64 = u64::from(m.geometry.inner()) << 8 | u64::from(m.topology as u8);
      let def_id = match self.resources.mesh_defs.get( &packed_key )
      {
        Some( id ) => id.clone(),
        None => match self.generate_mesh_def( m.geometry, m.topology )
        {
          Some( id ) => id,
          None => return,
        },
      };

      let transform = self.transform_to_svg( &m.transform );
      let fill = self.texture_or_fill( m.texture, &m.fill );
      let clip = Self::clip_attr( m.clip.as_ref() );
      let blend = Self::blend_to_svg( m.blend );

      // Cache mesh <symbol> defs across calls with different colors, so the
      // caller's color must cascade via the <use>. `stroke=fill` drives line
      // meshes (polylines in the symbol inherit this stroke); fill drives
      // polygon meshes. The 1px same-color stroke on polygons is a benign
      // side effect.
      let mesh = format!
      (
        "<use href=\"#{def_id}\" fill=\"{fill}\" stroke=\"{fill}\"{transform}{clip}{blend}/>"
      );
      self.content.push_body( &mesh );
    }

    fn cmd_sprite( &mut self, s : &Sprite ) -> Result< (), RenderError >
    {
      let transform = self.transform_to_svg( &s.transform );
      let clip = Self::clip_attr( s.clip.as_ref() );
      let blend = Self::blend_to_svg( s.blend );
      let tint = self.tint_filter_attr( &s.tint )?;
      let sprite = format!( "<use href=\"#sprite_{}\"{}{}{}{}/>", s.sprite.inner(), transform, clip, tint, blend );
      self.content.push_body( &sprite );
      Ok( () )
    }

    fn cmd_create_sprite_batch( &mut self, cb : &CreateSpriteBatch )
    {
      self.resources.store_batch( cb.batch, SvgBatch::Sprite { instances : Vec::new(), params : cb.params } );
    }

    fn cmd_create_mesh_batch( &mut self, cb : &CreateMeshBatch )
    {
      self.resources.store_batch( cb.batch, SvgBatch::Mesh { instances : Vec::new(), params : cb.params } );
    }

    fn cmd_bind_batch( &mut self, bb : BindBatch )
    {
      self.recording_batch = Some( bb.batch );
    }

    fn cmd_add_sprite_instance( &mut self, si : &AddSpriteInstance )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( SvgBatch::Sprite { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
        {
          instances.push( *si );
        }
    }

    fn cmd_add_mesh_instance( &mut self, mi : &AddMeshInstance )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( SvgBatch::Mesh { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
        {
          instances.push( *mi );
        }
    }

    fn cmd_set_sprite_instance( &mut self, si : &SetSpriteInstance )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( SvgBatch::Sprite { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
          && ( si.index as usize ) < instances.len()
          {
            instances[ si.index as usize ] = AddSpriteInstance { transform : si.transform, sprite : si.sprite, tint : si.tint };
          }
    }

    fn cmd_set_mesh_instance( &mut self, mi : &SetMeshInstance )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( SvgBatch::Mesh { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
          && ( mi.index as usize ) < instances.len()
          {
            instances[ mi.index as usize ] = AddMeshInstance { transform : mi.transform, tint : mi.tint };
          }
    }

    fn cmd_remove_instance( &mut self, ri : RemoveInstance )
    {
      if let Some( batch_id ) = self.recording_batch
      {
        match self.resources.batches.get_mut( &batch_id )
        {
          Some( SvgBatch::Sprite { instances, .. } ) =>
          {
            if ( ri.index as usize ) < instances.len() { instances.swap_remove( ri.index as usize ); }
          }
          // Collapsing into a match guard (`Some(Mesh{..}) if cond => ..`) would
          // make this arm's pattern not count toward exhaustiveness (verified:
          // E0004 "match arms with guards don't count towards exhaustivity"),
          // since `SvgBatch` has only Sprite/Mesh variants and no wildcard arm.
          #[ allow( clippy::collapsible_match ) ]
          Some( SvgBatch::Mesh { instances, .. } ) =>
          {
            if ( ri.index as usize ) < instances.len() { instances.swap_remove( ri.index as usize ); }
          }
          None => {}
        }
      }
    }

    fn cmd_set_sprite_batch_params( &mut self, sp : &SetSpriteBatchParams )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( SvgBatch::Sprite { params, .. } ) = self.resources.batches.get_mut( &batch_id )
        {
          *params = sp.params;
        }
    }

    fn cmd_set_mesh_batch_params( &mut self, mp : &SetMeshBatchParams )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( SvgBatch::Mesh { params, .. } ) = self.resources.batches.get_mut( &batch_id )
        {
          *params = mp.params;
        }
    }

    fn cmd_unbind_batch( &mut self )
    {
      self.recording_batch = None;
    }

    fn cmd_draw_batch( &mut self, db : DrawBatch ) -> Result< (), RenderError >
    {
      let height = self.config.height;

      // Lazy-generate the mesh <symbol> def before splitting borrows.
      if let Some( SvgBatch::Mesh { params, .. } ) = self.resources.batch( db.batch )
      {
        let packed_key : u64 = u64::from(params.geometry.inner()) << 8 | u64::from(params.topology as u8);
        if !self.resources.mesh_defs.contains_key( &packed_key )
        {
          let ( geom_id, topology ) = ( params.geometry, params.topology );
          self.generate_mesh_def( geom_id, topology );
        }
      }

      let resources = &self.resources;
      let content = &mut self.content;
      let filter_counter = &mut self.filter_counter;

      match resources.batch( db.batch )
      {
        Some( SvgBatch::Sprite { instances, params } ) =>
        {
          let parent_transform = Self::transform_to_svg_static( &params.transform, height );
          let clip = Self::clip_attr( params.clip.as_ref() );
          let blend = Self::blend_to_svg( params.blend );

          content.push_body( &format!( "<g{parent_transform}{clip}>" ) );
          for inst in instances
          {
            let inst_transform = Self::transform_to_svg_local( &inst.transform );
            let tint = Self::tint_filter_attr_split( &inst.tint, content, filter_counter )?;
            let sprite = format!
            (
              "<use href=\"#sprite_{}\"{}{}{}/>",
              inst.sprite.inner(), inst_transform, tint, blend
            );
            content.push_body( &sprite );
          }
          content.push_body( "</g>" );
        }
        Some( SvgBatch::Mesh { instances, params } ) =>
        {
          let packed_key : u64 = u64::from(params.geometry.inner()) << 8 | u64::from(params.topology as u8);
          if let Some( def_id ) = resources.mesh_defs.get( &packed_key )
          {
            let parent_transform = Self::transform_to_svg_static( &params.transform, height );
            let clip = Self::clip_attr( params.clip.as_ref() );
            let blend = Self::blend_to_svg( params.blend );
            let fill = Self::texture_or_fill_split( params.texture, &params.fill, resources, content );

            content.push_body( &format!( "<g{parent_transform}{clip}>" ) );
            for inst in instances
            {
              let inst_transform = Self::transform_to_svg_local( &inst.transform );
              let mesh = format!
              (
                "<use href=\"#{def_id}\" fill=\"{fill}\" stroke=\"{fill}\"{inst_transform}{blend}/>"
              );
              content.push_body( &mesh );
            }
            content.push_body( "</g>" );
          }
        }
        None => {}
      }
      Ok( () )
    }

    fn cmd_delete_batch( &mut self, db : DeleteBatch )
    {
      self.resources.batches.remove( &db.batch );
    }

    fn cmd_begin_group( &mut self, bg : &BeginGroup ) -> Result< (), RenderError >
    {
      let transform = self.transform_to_svg( &bg.transform );
      let clip = Self::clip_attr( bg.clip.as_ref() );

      let effect_attr = match &bg.effect
      {
        Some( Effect::Opacity( a ) ) => format!( " opacity=\"{a}\"" ),
        Some( Effect::Blur { radius } ) =>
        {
          let fid = Self::bump_filter_counter( &mut self.filter_counter )?;
          let def = format!( "<filter id=\"fx_{fid}\"><feGaussianBlur stdDeviation=\"{radius}\"/></filter>" );
          self.content.push_frame_def( &def );
          format!( " filter=\"url(#fx_{fid})\"" )
        }
        Some( Effect::DropShadow { dx, dy, blur, color } ) =>
        {
          let fid = Self::bump_filter_counter( &mut self.filter_counter )?;
          let c = Self::color_to_svg( color );
          let flood_opacity = Self::opacity_attr( "flood-opacity", color );
          // `feDropShadow` is an SVG 2 primitive. We emit an equivalent SVG 1.1
          // filter chain so output validates against strict 1.1 tooling:
          //   SourceAlpha -> Gaussian blur -> offset -> color via flood+composite(in)
          //   -> merge under SourceGraphic.
          // Negate dy: Y-up shadow direction → SVG Y-down.
          let def = format!
          (
            "<filter id=\"fx_{}\">\
              <feGaussianBlur in=\"SourceAlpha\" stdDeviation=\"{}\" result=\"fx_{}_blur\"/>\
              <feOffset in=\"fx_{}_blur\" dx=\"{}\" dy=\"{}\" result=\"fx_{}_offset\"/>\
              <feFlood flood-color=\"{}\"{}/>\
              <feComposite in2=\"fx_{}_offset\" operator=\"in\" result=\"fx_{}_shadow\"/>\
              <feMerge><feMergeNode in=\"fx_{}_shadow\"/><feMergeNode in=\"SourceGraphic\"/></feMerge>\
            </filter>",
            fid,
            blur, fid,
            fid, dx, -dy, fid,
            c, flood_opacity,
            fid, fid,
            fid,
          );
          self.content.push_frame_def( &def );
          format!( " filter=\"url(#fx_{fid})\"" )
        }
        Some( Effect::ColorMatrix( values ) ) =>
        {
          let fid = Self::bump_filter_counter( &mut self.filter_counter )?;
          let vals : String = values.iter().map( std::string::ToString::to_string ).collect::< Vec< _ > >().join( " " );
          let def = format!( "<filter id=\"fx_{fid}\"><feColorMatrix type=\"matrix\" values=\"{vals}\"/></filter>" );
          self.content.push_frame_def( &def );
          format!( " filter=\"url(#fx_{fid})\"" )
        }
        None => String::new(),
      };

      let group = format!( "<g{transform}{clip}{effect_attr}>" );
      self.content.push_body( &group );
      self.group_depth += 1;
      Ok( () )
    }

    fn cmd_end_group( &mut self )
    {
      // Guard against unmatched EndGroup: emitting `</g>` at depth 0
      // would produce malformed XML that some parsers reject.
      if self.group_depth > 0
      {
        self.content.push_body( "</g>" );
        self.group_depth -= 1;
      }
    }
  }

  // ============================================================================
  // Backend trait impl
  // ============================================================================

  impl Backend for SvgBackend
  {
    #[ inline ]
    fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >
    {
      self.content.clear_defs();
      self.resources = SvgResources::new();

      self.load_gradients( &assets.gradients );
      self.load_patterns( &assets.patterns );
      self.load_clip_masks( &assets.clip_masks );
      self.load_paths( &assets.paths );
      self.load_images( &assets.images );
      self.load_sprites( &assets.sprites );
      self.load_geometries( &assets.geometries );

      Ok( () )
    }

    #[ inline ]
    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      self.content.clear_frame_defs();
      self.content.clear_body();
      self.resources.mesh_defs.clear();
      self.filter_counter = 0;
      self.group_depth = 0;
      self.recording_batch = None;
      // An unmatched BeginPath / BeginText in the previous frame would
      // otherwise leak path / text accumulators into this one and the
      // next EndPath / EndText would flush stale content into the body.
      self.path_style = None;
      self.path_data.clear();
      self.text_style = None;
      self.text_buf.clear();

      for cmd in commands
      {
        match cmd
        {
          RenderCommand::Clear( c ) => self.cmd_clear( c ),
          RenderCommand::BeginPath( bp ) => self.cmd_begin_path( bp ),
          RenderCommand::MoveTo( m ) => self.cmd_move_to( *m ),
          RenderCommand::LineTo( l ) => self.cmd_line_to( *l ),
          RenderCommand::QuadTo( q ) => self.cmd_quad_to( q ),
          RenderCommand::CubicTo( c ) => self.cmd_cubic_to( c ),
          RenderCommand::ArcTo( a ) => self.cmd_arc_to( a ),
          RenderCommand::ClosePath( _ ) => self.cmd_close_path(),
          RenderCommand::EndPath( _ ) => self.cmd_end_path(),
          RenderCommand::BeginText( bt ) => self.cmd_begin_text( bt ),
          RenderCommand::Char( ch ) => self.cmd_char( *ch ),
          RenderCommand::EndText( _ ) => self.cmd_end_text(),
          RenderCommand::Mesh( m ) => self.cmd_mesh( m ),
          // `ScreenSpaceSprite` shares the `Sprite` payload — the compile
          // layer already emits screen-space coordinates, so SVG (whose
          // user-space already is screen-space) draws it via the same
          // path as a world-space sprite.
          RenderCommand::Sprite( s ) | RenderCommand::ScreenSpaceSprite( s ) => self.cmd_sprite( s )?,
          RenderCommand::CreateSpriteBatch( cb ) => self.cmd_create_sprite_batch( cb ),
          RenderCommand::CreateMeshBatch( cb ) => self.cmd_create_mesh_batch( cb ),
          RenderCommand::BindBatch( bb ) => self.cmd_bind_batch( *bb ),
          RenderCommand::AddSpriteInstance( si ) => self.cmd_add_sprite_instance( si ),
          RenderCommand::AddMeshInstance( mi ) => self.cmd_add_mesh_instance( mi ),
          RenderCommand::SetSpriteInstance( si ) => self.cmd_set_sprite_instance( si ),
          RenderCommand::SetMeshInstance( mi ) => self.cmd_set_mesh_instance( mi ),
          RenderCommand::RemoveInstance( ri ) => self.cmd_remove_instance( *ri ),
          RenderCommand::SetSpriteBatchParams( sp ) => self.cmd_set_sprite_batch_params( sp ),
          RenderCommand::SetMeshBatchParams( mp ) => self.cmd_set_mesh_batch_params( mp ),
          RenderCommand::UnbindBatch( _ ) => self.cmd_unbind_batch(),
          RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( *db )?,
          RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( *db ),
          RenderCommand::BeginGroup( bg ) => self.cmd_begin_group( bg )?,
          RenderCommand::EndGroup( _ ) => self.cmd_end_group(),
        }
      }

      Ok( () )
    }

    #[ inline ]
    fn resize( &mut self, width : u32, height : u32 )
    {
      self.config.width = width;
      self.config.height = height;
      self.content.update_header( width, height, Self::shape_rendering_attr( self.config.antialias ) );
    }

    #[ inline ]
    fn output( &self ) -> Result< Output, RenderError >
    {
      Ok( Output::String( self.content.buffer().to_string() ) )
    }

    #[ inline ]
    fn capabilities( &self ) -> Capabilities
    {
      // Note: `text: true` reflects that text rendering works, but font assets
      // (`Assets.fonts`) are currently ignored — all text renders in the SVG
      // viewer's default font. See `SvgBackend` type docs and
      // docs/feature/001_svg_backend_adapter.md ("Known gap — font selection").
      Capabilities
      {
        paths : true,
        text : true,
        meshes : true,
        sprites : true,
        batches : true,
        gradients : true,
        patterns : true,
        clip_masks : true,
        effects : true,
        blend_modes : true,
        text_on_path : true,
        max_texture_size : 0,
        supported_blend_modes : &[
          BlendMode::Normal,
          BlendMode::Multiply,
          BlendMode::Screen,
          BlendMode::Overlay,
          BlendMode::Add,
        ],
      }
    }
  }

  // ============================================================================
  // SVG Content Manager
  // ============================================================================

  /// Manages a single SVG string buffer with indexed sections to avoid full reallocations.
  #[ derive( Debug, Clone ) ]
  struct SvgContentManager
  {
    buffer : String,
    defs_start : usize,
    defs_end : usize,
    /// Byte offset of the first frame-time def inside `<defs>`.
    /// Asset defs (from `load_assets`) live before this point;
    /// frame defs (from `submit`: filters, tints, mesh symbols, mesh-tex patterns) live after.
    /// Cleared at the start of each `submit()` so defs never accumulate across frames.
    frame_defs_start : usize,
    body_start : usize,
    /// Byte offset of the viewport transform value inside the `<g transform="...">` tag.
    vp_transform_start : usize,
    /// Byte length of the current viewport transform value.
    vp_transform_len : usize,
    /// Byte offset where body elements begin (just after the opening `<g ...>`).
    elements_start : usize,
    body_end : usize,
  }

  impl SvgContentManager
  {
    const BODY_OPEN   : &'static str = "<!--framebegin-->";
    const VP_PREFIX   : &'static str = "<g transform=\"";
    const VP_SUFFIX   : &'static str = "\">";
    const BODY_CLOSE  : &'static str = "</g><!--frameend-->\n";
    const DEFS_OPEN   : &'static str = "<defs>";
    const DEFS_CLOSE  : &'static str = "</defs>\n";

    fn initial_vp_transform( offset : [ f32; 2 ], scale : f32 ) -> String
    {
      format!( "scale({scale}) translate({},{})", offset[ 0 ], -offset[ 1 ] )
    }

    /// Creates a newly formatted SVG buffer layout empty with `<defs>` and `body` sections.
    pub fn new( width : u32, height : u32, shape_rendering : &str ) -> Self
    {
      let mut buffer = String::new();

      let header = format!
      (
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" xmlns=\"http://www.w3.org/2000/svg\"{shape_rendering}>\n"
      );
      buffer.push_str( &header );

      let defs_start = buffer.len();
      buffer.push_str( Self::DEFS_OPEN );
      let frame_defs_start = buffer.len(); // right after "<defs>" — no asset defs yet
      buffer.push_str( Self::DEFS_CLOSE );
      let defs_end = buffer.len();

      let body_start = buffer.len();
      buffer.push_str( Self::BODY_OPEN );
      buffer.push_str( Self::VP_PREFIX );
      let vp_transform_start = buffer.len();
      let initial = Self::initial_vp_transform( [ 0.0, 0.0 ], 1.0 );
      let vp_transform_len = initial.len();
      buffer.push_str( &initial );
      buffer.push_str( Self::VP_SUFFIX );
      let elements_start = buffer.len();
      buffer.push_str( Self::BODY_CLOSE );
      let body_end = buffer.len();

      buffer.push_str( "</svg>\n" );

      Self
      {
        buffer,
        defs_start,
        defs_end,
        frame_defs_start,
        body_start,
        vp_transform_start,
        vp_transform_len,
        elements_start,
        body_end,
      }
    }

    /// Updates the SVG header attributes dynamically like changing width/height bounds.
    pub fn update_header( &mut self, width : u32, height : u32, shape_rendering : &str )
    {
      let header = format!
      (
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" xmlns=\"http://www.w3.org/2000/svg\"{shape_rendering}>\n"
      );
      self.buffer.replace_range( 0..self.defs_start, &header );
      let diff = header.len().cast_signed() - self.defs_start.cast_signed();

      if diff != 0
      {
        // Every offset below sits at or after the old `defs_start`, so shifting
        // each by `diff` can never underflow: the smallest resulting value is
        // `defs_start + diff`, which is exactly `header.len()` (>= 0) by
        // construction of `diff` above. `checked_add_signed` makes that
        // invariant an explicit runtime check instead of a silenced cast.
        const MSG : &str = "SvgContentManager offset shift underflowed despite the header-relative invariant";
        self.defs_start          = self.defs_start        .checked_add_signed( diff ).expect( MSG );
        self.defs_end            = self.defs_end          .checked_add_signed( diff ).expect( MSG );
        self.frame_defs_start    = self.frame_defs_start  .checked_add_signed( diff ).expect( MSG );
        self.body_start          = self.body_start        .checked_add_signed( diff ).expect( MSG );
        self.vp_transform_start  = self.vp_transform_start.checked_add_signed( diff ).expect( MSG );
        self.elements_start      = self.elements_start    .checked_add_signed( diff ).expect( MSG );
        self.body_end            = self.body_end          .checked_add_signed( diff ).expect( MSG );
      }
    }

    /// Updates the viewport pan/zoom transform on the top-level `<g>` wrapper.
    ///
    /// This modifies the single `transform` attribute in-place so all previously
    /// rendered elements immediately reflect the new viewport without re-submission.
    pub fn update_viewport_transform( &mut self, offset : [ f32; 2 ], scale : f32 )
    {
      let new_transform = Self::initial_vp_transform( offset, scale );
      let old_end = self.vp_transform_start + self.vp_transform_len;

      self.buffer.replace_range( self.vp_transform_start..old_end, &new_transform );

      {
        // `elements_start`/`body_end` both sit at or after the old transform's
        // end, so shifting by `diff` can never underflow — same reasoning as
        // `update_header` above. `checked_add_signed` turns that invariant into
        // an explicit runtime check instead of a silenced cast.
        const MSG : &str = "SvgContentManager offset shift underflowed despite the transform-relative invariant";
        let diff = new_transform.len().cast_signed() - self.vp_transform_len.cast_signed();
        self.vp_transform_len = new_transform.len();
        self.elements_start = self.elements_start.checked_add_signed( diff ).expect( MSG );
        self.body_end       = self.body_end      .checked_add_signed( diff ).expect( MSG );
      }
    }

    /// Clears the `<defs>` content scope entirely (both asset and frame defs).
    pub fn clear_defs( &mut self )
    {
      let inner_start = self.defs_start + Self::DEFS_OPEN.len();
      let inner_end   = self.defs_end   - Self::DEFS_CLOSE.len();

      self.buffer.replace_range( inner_start..inner_end, "" );
      let removed = inner_end - inner_start;

      self.defs_end           -= removed;
      self.frame_defs_start    = self.defs_start + Self::DEFS_OPEN.len();
      self.body_start         -= removed;
      self.vp_transform_start -= removed;
      self.elements_start     -= removed;
      self.body_end           -= removed;
    }

    /// Inlines an asset-time def (from `load_assets`) into the definitions section.
    ///
    /// Advances `frame_defs_start` so that the asset/frame boundary stays accurate.
    pub fn push_asset_def( &mut self, def : &str )
    {
      let insert_at = self.defs_end - Self::DEFS_CLOSE.len();
      self.buffer.insert_str( insert_at, def );

      let added = def.len();
      self.defs_end           += added;
      self.frame_defs_start   += added;
      self.body_start         += added;
      self.vp_transform_start += added;
      self.elements_start     += added;
      self.body_end           += added;
    }

    /// Inlines a frame-time def (from `submit`) into the definitions section.
    ///
    /// Does **not** advance `frame_defs_start` — these defs are cleared by
    /// [`clear_frame_defs`] at the start of each `submit()`.
    pub fn push_frame_def( &mut self, def : &str )
    {
      let insert_at = self.defs_end - Self::DEFS_CLOSE.len();
      self.buffer.insert_str( insert_at, def );

      let added = def.len();
      self.defs_end           += added;
      self.body_start         += added;
      self.vp_transform_start += added;
      self.elements_start     += added;
      self.body_end           += added;
    }

    /// Clears all frame-time defs added since the last `load_assets` call.
    ///
    /// Called at the start of each `submit()` together with `clear_body`.
    pub fn clear_frame_defs( &mut self )
    {
      let inner_end = self.defs_end - Self::DEFS_CLOSE.len();
      if inner_end <= self.frame_defs_start { return; }

      self.buffer.replace_range( self.frame_defs_start..inner_end, "" );
      let removed = inner_end - self.frame_defs_start;

      self.defs_end           -= removed;
      self.body_start         -= removed;
      self.vp_transform_start -= removed;
      self.elements_start     -= removed;
      self.body_end           -= removed;
    }

    /// Clears only the dynamic render paths payload.
    pub fn clear_body( &mut self )
    {
      let inner_end = self.body_end - Self::BODY_CLOSE.len();

      self.buffer.replace_range( self.elements_start..inner_end, "" );
      let removed = inner_end - self.elements_start;

      self.body_end -= removed;
    }

    /// Pushes SVG command sequence nodes inside the viewport wrapper.
    pub fn push_body( &mut self, content : &str )
    {
      let insert_at = self.body_end - Self::BODY_CLOSE.len();
      self.buffer.insert_str( insert_at, content );
      self.body_end += content.len();
    }

    /// Reference handle access to underlying payload SVG.
    pub fn buffer( &self ) -> &str
    {
      &self.buffer
    }
  }

  // ============================================================================
  // Tests
  // ============================================================================

  // Documented exception (task 071) to the all-tests-in-tests/ convention: the tests below
  // pin private formatting/encoding helpers -- `transform_to_svg_static`/`transform_to_svg_local`
  // (Y-flip math), `anchor_to_svg`, `path_to_href`, `png_dimensions`, `detect_image_mime`,
  // `bitmap_to_png`, and `SvgContentManager` -- none of which are in the `mod_interface`
  // exports; publishing them solely for test placement would widen the API for no caller.
  // (`image_encoded_png_stores_dimensions` additionally builds its PNG fixture via the private
  // encoder.) The adapter's public-surface behavior tests live in `tests/svg_backend_test.rs`;
  // the small driving helpers (`svg800x600`, `render`, `defs`) are intentionally present on
  // both sides -- an inline module cannot import from `tests/helpers`.
  #[ cfg( test ) ]
  mod tests
  {
    use super::*;
    use crate::backend::{ Backend, Output };
    use crate::types::{ MipmapMode, WrapMode };

    fn svg800x600() -> SvgBackend
    {
      SvgBackend::new( RenderConfig { width : 800, height : 600, ..Default::default() } )
    }

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

    fn render( svg : &SvgBackend ) -> String
    {
      match svg.output().unwrap()
      {
        Output::String( s ) => s,
        _ => panic!( "expected string output" ),
      }
    }

    fn defs( svg : &SvgBackend ) -> String
    {
      let full = render( svg );
      let start = full.find( "<defs>" ).unwrap() + "<defs>".len();
      let end = full.find( "</defs>" ).unwrap();
      full[ start..end ].to_string()
    }

    // -- transform Y-up --

    #[ test ]
    fn transform_y_up_bottom_left_origin()
    {
      // Position (0,0) in Y-up should map to SVG (0, height=600)
      let s = SvgBackend::transform_to_svg_static(
        &Transform { position : [ 0.0, 0.0 ], ..Default::default() },
        600,
      );
      assert!( s.contains( "translate(0,600)" ), "got: {s}" );
    }

    #[ test ]
    fn transform_y_up_top_right()
    {
      // Position (800,600) should map to SVG (800, 0)
      let s = SvgBackend::transform_to_svg_static(
        &Transform { position : [ 800.0, 600.0 ], ..Default::default() },
        600,
      );
      assert!( s.contains( "translate(800,0)" ), "got: {s}" );
    }

    #[ test ]
    fn transform_y_up_center()
    {
      // Position (400,300) should map to SVG (400, 300)
      let s = SvgBackend::transform_to_svg_static(
        &Transform { position : [ 400.0, 300.0 ], ..Default::default() },
        600,
      );
      assert!( s.contains( "translate(400,300)" ), "got: {s}" );
    }

    #[ test ]
    fn transform_rotation_negated()
    {
      let angle = core::f32::consts::FRAC_PI_4; // 45° CCW in Y-up
      let s = SvgBackend::transform_to_svg_static(
        &Transform { rotation : angle, ..Default::default() },
        600,
      );
      // Should emit negative degrees in SVG
      assert!( s.contains( "rotate(-45" ), "got: {s}" );
    }

    #[ test ]
    fn transform_scale_y_negated()
    {
      let s = SvgBackend::transform_to_svg_static(
        &Transform { scale : [ 2.0, 3.0 ], ..Default::default() },
        600,
      );
      // scale Y should be negated: 3.0 → -3.0
      assert!( s.contains( "scale(2,-3)" ), "got: {s}" );
    }

    #[ test ]
    fn transform_identity_scale_emits_y_flip()
    {
      // Default scale (1,1) should still emit scale(1,-1) for Y-flip
      let s = SvgBackend::transform_to_svg_static(
        &Transform::default(),
        600,
      );
      assert!( s.contains( "scale(1,-1)" ), "got: {s}" );
    }

    /// Verify that zoom=1.0 does NOT inject scale(1) noise into per-element transforms.
    #[ test ]
    fn transform_no_zoom_in_per_element_transform()
    {
      let s = SvgBackend::transform_to_svg_static(
        &Transform::default(),
        600,
      );
      // Only scale(1,-1) for Y-flip should be present; no zoom prefix
      assert!( !s.contains( "scale(1) " ), "got: {s}" );
    }

    #[ test ]
    fn transform_skew_negated()
    {
      let angle = core::f32::consts::FRAC_PI_6; // 30°
      let s = SvgBackend::transform_to_svg_static(
        &Transform { skew : [ angle, 0.0 ], ..Default::default() },
        600,
      );
      assert!( s.contains( "skewX(-30" ), "got: {s}" );
    }

    // -- local transform (for batch instances inside Y-flipped group) --

    #[ test ]
    fn local_transform_no_y_flip()
    {
      let s = SvgBackend::transform_to_svg_local( &Transform
      {
        position : [ 10.0, 20.0 ],
        rotation : 0.5,
        scale : [ 2.0, 3.0 ],
        ..Default::default()
      });
      // Position is raw, no Y-flip
      assert!( s.contains( "translate(10,20)" ), "got: {s}" );
      // Rotation is raw (positive), not negated
      let deg = 0.5_f32.to_degrees();
      assert!( s.contains( &format!( "rotate({deg})" ) ), "got: {s}" );
      // Scale is raw, no Y negation
      assert!( s.contains( "scale(2,3)" ), "got: {s}" );
    }

    // -- content manager --

    #[ test ]
    fn content_manager_push_clear_cycle()
    {
      let mut cm = SvgContentManager::new( 100, 100, "" );
      cm.push_asset_def( "<test-def/>" );
      cm.push_body( "<test-body/>" );

      let buf = cm.buffer();
      assert!( buf.contains( "<test-def/>" ) );
      assert!( buf.contains( "<test-body/>" ) );

      cm.clear_body();
      let buf = cm.buffer();
      assert!( buf.contains( "<test-def/>" ) );
      assert!( !buf.contains( "<test-body/>" ) );

      cm.clear_defs();
      let buf = cm.buffer();
      assert!( !buf.contains( "<test-def/>" ) );
    }

    // -- png_dimensions --

    /// Verifies that `png_dimensions` extracts correct width/height from valid PNG bytes.
    #[ test ]
    fn png_dimensions_valid()
    {
      // Generate a real 3×5 PNG via bitmap_to_png, then extract dimensions from its header.
      let bytes = vec![ 0u8; 3 * 5 * 4 ];
      let png = SvgBackend::bitmap_to_png( &bytes, 3, 5, PixelFormat::Rgba8 ).unwrap();
      assert_eq!( SvgBackend::png_dimensions( &png ), Some( ( 3, 5 ) ) );
    }

    /// Verifies MIME type detection from magic bytes.
    #[ test ]
    fn detect_image_mime_by_magic()
    {
      // PNG
      assert_eq!( SvgBackend::detect_image_mime( &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0 ] ), "image/png" );
      // JPEG
      assert_eq!( SvgBackend::detect_image_mime( &[ 0xff, 0xd8, 0xff, 0xe0 ] ), "image/jpeg" );
      // GIF
      assert_eq!( SvgBackend::detect_image_mime( b"GIF89a..." ), "image/gif" );
      // WebP
      let mut webp = Vec::from( *b"RIFF\0\0\0\0WEBP" );
      webp.push( 0 );
      assert_eq!( SvgBackend::detect_image_mime( &webp ), "image/webp" );
      // Unknown falls back to PNG
      assert_eq!( SvgBackend::detect_image_mime( &[ 0, 0, 0, 0 ] ), "image/png" );
    }

    /// Verifies that `path_to_href` produces a valid URI reference:
    /// spaces become %20 and Windows backslashes become forward slashes.
    #[ test ]
    fn image_path_produces_valid_uri_reference()
    {
      assert_eq!( SvgBackend::path_to_href( "images/tile set/floor.png" ), "images/tile%20set/floor.png" );
      assert_eq!( SvgBackend::path_to_href( r"images\tiles\floor.png" ), "images/tiles/floor.png" );
      assert_eq!( SvgBackend::path_to_href( "safe-name_1.2.png" ), "safe-name_1.2.png" );
      // All URI-reserved and XML-unsafe characters are percent-encoded.
      let e = SvgBackend::path_to_href( "a\"b<c>d&e#f?g%h" );
      assert!( !e.contains( '"' ) && !e.contains( '<' ) && !e.contains( '>' ) && !e.contains( '&' ), "unsafe char leaked: {e}" );
    }

    /// Verifies that a short / non-PNG buffer returns None.
    #[ test ]
    fn png_dimensions_invalid()
    {
      assert_eq!( SvgBackend::png_dimensions( &[] ), None );
      assert_eq!( SvgBackend::png_dimensions( &[ 0u8; 24 ] ), None ); // no PNG signature
    }

    /// Verifies that `load_assets` extracts PNG dimensions from `ImageSource::Encoded`
    /// so that a sprite symbol uses the correct sheet size.
    #[ test ]
    fn image_encoded_png_stores_dimensions()
    {
      let png = SvgBackend::bitmap_to_png( &[ 0u8; 8 * 4 * 4 ], 8, 4, PixelFormat::Rgba8 ).unwrap();
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Encoded( png ),
          filter : SamplerFilter::Linear,
          mipmap : MipmapMode::Off,
          wrap : WrapMode::Clamp,
        }],
        sprites : vec![ SpriteAsset
        {
          id : ResourceId::new( 0 ),
          sheet : ResourceId::new( 0 ),
          region : [ 0.0, 0.0, 4.0, 4.0 ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let d = defs( &svg );
      // The sprite symbol's <use> must reference width="8" height="4" (the sheet size)
      assert!( d.contains( "width=\"8\"" ), "defs: {d}" );
      assert!( d.contains( "height=\"4\"" ), "defs: {d}" );
    }

    const PNG_MAGIC : &[ u8 ] = &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ];

    /// Verifies that a 1×1 Rgba8 pixel buffer produces valid PNG output
    /// (starts with the PNG magic bytes).
    #[ test ]
    fn bitmap_to_png_rgba8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 255, 0, 128, 255 ], 1, 1, PixelFormat::Rgba8 );
      let bytes = png.expect( "expected Some for valid 1x1 Rgba8" );
      assert!( bytes.starts_with( PNG_MAGIC ), "not PNG: {:?}", &bytes[ ..8.min( bytes.len() ) ] );
    }

    /// Verifies that a 1×1 Rgb8 pixel buffer encodes successfully.
    #[ test ]
    fn bitmap_to_png_rgb8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 255, 0, 128 ], 1, 1, PixelFormat::Rgb8 );
      assert!( png.is_some(), "expected Some for valid 1x1 Rgb8" );
    }

    /// Verifies that a 1×1 Gray8 pixel buffer encodes successfully.
    #[ test ]
    fn bitmap_to_png_gray8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 128 ], 1, 1, PixelFormat::Gray8 );
      assert!( png.is_some(), "expected Some for valid 1x1 Gray8" );
    }

    /// Verifies that a 1×1 `GrayAlpha8` pixel buffer encodes successfully.
    #[ test ]
    fn bitmap_to_png_gray_alpha8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 128, 255 ], 1, 1, PixelFormat::GrayAlpha8 );
      assert!( png.is_some(), "expected Some for valid 1x1 GrayAlpha8" );
    }

    /// Verifies that mismatched dimensions (too few bytes for the declared size) return None.
    #[ test ]
    fn bitmap_to_png_dimension_mismatch_returns_none()
    {
      // 2×2 Rgba8 needs 16 bytes; supplying only 4 must return None
      let png = SvgBackend::bitmap_to_png( &[ 255, 0, 0, 255 ], 2, 2, PixelFormat::Rgba8 );
      assert!( png.is_none(), "expected None for undersized buffer" );
    }

    // anchor_to_svg — 9 variants (private method, must stay inline)

    #[ test ]
    fn anchor_top_left()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::TopLeft );
      assert_eq!( h, "start" );
      assert_eq!( v, "hanging" );
    }

    #[ test ]
    fn anchor_top_center()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::TopCenter );
      assert_eq!( h, "middle" );
      assert_eq!( v, "hanging" );
    }

    #[ test ]
    fn anchor_top_right()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::TopRight );
      assert_eq!( h, "end" );
      assert_eq!( v, "hanging" );
    }

    #[ test ]
    fn anchor_center_left()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::CenterLeft );
      assert_eq!( h, "start" );
      assert_eq!( v, "central" );
    }

    #[ test ]
    fn anchor_center()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::Center );
      assert_eq!( h, "middle" );
      assert_eq!( v, "central" );
    }

    #[ test ]
    fn anchor_center_right()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::CenterRight );
      assert_eq!( h, "end" );
      assert_eq!( v, "central" );
    }

    #[ test ]
    fn anchor_bottom_left()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::BottomLeft );
      assert_eq!( h, "start" );
      assert_eq!( v, "baseline" );
    }

    #[ test ]
    fn anchor_bottom_center()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::BottomCenter );
      assert_eq!( h, "middle" );
      assert_eq!( v, "baseline" );
    }

    #[ test ]
    fn anchor_bottom_right()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::BottomRight );
      assert_eq!( h, "end" );
      assert_eq!( v, "baseline" );
    }
  }
}

mod_interface::mod_interface!
{
  own use SvgBackend;
}
