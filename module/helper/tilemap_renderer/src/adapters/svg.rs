//! SVG backend adapter.
//!
//! Generates a complete SVG 1.1 document from render commands.
//! Supports all features: paths, text, sprites, gradients, patterns,
//! clip masks, effects, blend modes, and text-on-path.

mod private
{
  use crate::assets::*;
  use crate::backend::*;
  use crate::commands::*;
  use crate::types::*;
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
    #[ must_use ]
    pub fn new( config : RenderConfig ) -> Self
    {
      Self
      {
        config,
        content : SvgContentManager::new( config.width, config.height, Self::shape_rendering_attr( &config.antialias ) ),
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
    #[must_use]
    pub fn viewport_offset( &self ) -> [ f32; 2 ] { self.viewport_offset }

    /// Sets the viewport offset `[x, y]`.
    ///
    /// Immediately updates the top-level `<g transform>` wrapper so all already-rendered
    /// elements reflect the new position without re-submission.
    pub fn set_viewport_offset( &mut self, offset : [ f32; 2 ] )
    {
      self.viewport_offset = offset;
      self.content.update_viewport_transform( self.viewport_offset, self.viewport_scale );
    }

    /// Returns the current viewport scale (zoom factor).
    #[must_use]
    pub fn viewport_scale( &self ) -> f32 { self.viewport_scale }

    /// Sets the viewport scale (zoom factor).
    ///
    /// Immediately updates the top-level `<g transform>` wrapper so all already-rendered
    /// elements reflect the new zoom without re-submission.
    pub fn set_viewport_scale( &mut self, scale : f32 )
    {
      self.viewport_scale = scale;
      self.content.update_viewport_transform( self.viewport_offset, self.viewport_scale );
    }

    fn shape_rendering_attr( antialias : &Antialias ) -> &'static str
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

    fn blend_to_svg( blend : &BlendMode ) -> &'static str
    {
      match blend
      {
        BlendMode::Normal => "normal",
        BlendMode::Multiply => "multiply",
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::Add => "lighter",
      }
    }

    fn linecap_to_svg( cap : &LineCap ) -> &'static str
    {
      match cap
      {
        LineCap::Butt => "butt",
        LineCap::Round => "round",
        LineCap::Square => "square",
      }
    }

    fn linejoin_to_svg( join : &LineJoin ) -> &'static str
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

    fn anchor_to_svg( anchor : &TextAnchor ) -> ( &'static str, &'static str )
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
    fn bitmap_to_png( bytes : &[ u8 ], width : u32, height : u32, format : &PixelFormat ) -> Option< Vec< u8 > >
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
      dynamic.write_to( &mut std::io::Cursor::new( &mut png ), image::ImageFormat::Png ).ok()?;
      Some( png )
    }

    /// Extracts width and height from a PNG byte buffer by reading the IHDR chunk.
    /// Returns `None` if the buffer is too short or does not start with the PNG signature.
    /// Extracts (width, height) from an encoded image buffer using the `image`
    /// crate's format guesser. Supports any format the crate can decode the
    /// dimensions of — PNG, JPEG, GIF, WebP, BMP, TIFF, etc. Returns `None`
    /// when the format is unrecognized or the header is malformed.
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
    #[ allow( dead_code ) ]
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
      let blend = Self::blend_to_svg( &style.blend );

      let path = format!
      (
        "<path d=\"{}\" fill=\"{}\"{} stroke=\"{}\"{} stroke-width=\"{}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}{}{} style=\"mix-blend-mode:{}\"/>",
        self.path_data.trim(),
        fill,
        fill_opacity,
        stroke,
        stroke_opacity,
        style.stroke_width,
        Self::linecap_to_svg( &style.stroke_cap ),
        Self::linejoin_to_svg( &style.stroke_join ),
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
      let ( anchor, baseline ) = Self::anchor_to_svg( &style.anchor );
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
          out.push_str( &format!( "%{byte:02X}" ) );
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

        let mut grad_def = format!( "<{} id=\"grad_{}\"", grad_type, grad.id.inner() );

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

    fn load_images( &mut self, images : &[ ImageAsset ] )
    {
      for img in images
      {
        match &img.source
        {
          ImageSource::Bitmap { bytes, width, height, format } =>
          {
            if let Some( png ) = Self::bitmap_to_png( bytes, *width, *height, format )
            {
              let encoded = base64::prelude::BASE64_STANDARD.encode( &png );
              let img_def = format!
              (
                "<symbol id=\"img_{}\" viewBox=\"0 0 {} {}\"><image href=\"data:image/png;base64,{}\" width=\"{}\" height=\"{}\"/></symbol>",
                img.id.inner(), width, height, encoded, width, height
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
              "<symbol id=\"img_{}\" viewBox=\"0 0 {} {}\"><image href=\"data:{mime};base64,{encoded}\" width=\"{}\" height=\"{}\"/></symbol>",
              img.id.inner(), w, h, w, h
            );
            self.content.push_asset_def( &img_def );
            self.resources.store_image( img.id, SvgImage { width : w, height : h } );
          }
          ImageSource::Path( path ) =>
          {
            let href = Self::path_to_href( &path.display().to_string() );
            let img_def = format!( "<symbol id=\"img_{}\"><image href=\"{}\"/></symbol>", img.id.inner(), href );
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
        // TODO: Source::Path geometries are silently skipped for now.
        // Future: load via std::fs on native or fetch() on wasm32, then re-invoke
        // store_geometry. Until then callers must resolve paths to Source::Bytes
        // before calling load_assets.
        if let Source::Bytes( bytes ) = &geom.positions
        {
          let positions : &[ f32 ] = bytemuck::cast_slice( bytes );
          let indices = if let Some( Source::Bytes( ibytes ) ) = &geom.indices
          {
            match geom.data_type
            {
              DataType::U8  => Some( ibytes.iter().map( | &i | u32::from( i ) ).collect() ),
              DataType::U16 => Some( bytemuck::cast_slice::< _, u16 >( ibytes ).iter().map( | &i | u32::from( i ) ).collect() ),
              DataType::U32 => Some( bytemuck::cast_slice::< _, u32 >( ibytes ).to_vec() ),
              DataType::F32 => None, // F32 is not a valid index type; documented in DataType::F32 doc
            }
          }
          else { None };

          self.resources.store_geometry( geom.id, SvgGeometry { positions : positions.to_vec(), indices } );
        }
      }
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
            let order : [ usize; 3 ] = if i % 2 == 0 { [ 0, 1, 2 ] } else { [ 1, 0, 2 ] };
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

            if topology == Topology::LineList && ( i + 1 ) % 2 == 0
            {
              let _ = write!( def_content, "<polyline points=\"{}\" fill=\"none\" stroke=\"currentColor\"/>", pts.trim() );
              pts.clear();
            }
          }
          if !pts.is_empty() && topology == Topology::LineStrip
          {
            let _ = write!( def_content, "<polyline points=\"{}\" fill=\"none\" stroke=\"currentColor\"/>", pts.trim() );
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

    fn cmd_move_to( &mut self, m : &MoveTo )
    {
      let _ = write!( self.path_data, "M {} {} ", m.0, m.1 );
    }

    fn cmd_line_to( &mut self, l : &LineTo )
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

    fn cmd_char( &mut self, ch : &Char )
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
      let blend = Self::blend_to_svg( &m.blend );

      let mesh = format!
      (
        "<use href=\"#{def_id}\" fill=\"{fill}\"{transform}{clip} style=\"mix-blend-mode:{blend}\"/>"
      );
      self.content.push_body( &mesh );
    }

    fn cmd_sprite( &mut self, s : &Sprite ) -> Result< (), RenderError >
    {
      let transform = self.transform_to_svg( &s.transform );
      let clip = Self::clip_attr( s.clip.as_ref() );
      let blend = Self::blend_to_svg( &s.blend );
      let tint = self.tint_filter_attr( &s.tint )?;
      let sprite = format!( "<use href=\"#sprite_{}\"{}{}{} style=\"mix-blend-mode:{}\"/>", s.sprite.inner(), transform, clip, tint, blend );
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

    fn cmd_bind_batch( &mut self, bb : &BindBatch )
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
            instances[ mi.index as usize ] = AddMeshInstance { transform : mi.transform };
          }
    }

    fn cmd_remove_instance( &mut self, ri : &RemoveInstance )
    {
      if let Some( batch_id ) = self.recording_batch
      {
        match self.resources.batches.get_mut( &batch_id )
        {
          Some( SvgBatch::Sprite { instances, .. } ) =>
          {
            if ( ri.index as usize ) < instances.len() { instances.swap_remove( ri.index as usize ); }
          }
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

    fn cmd_draw_batch( &mut self, db : &DrawBatch ) -> Result< (), RenderError >
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
          let blend = Self::blend_to_svg( &params.blend );

          content.push_body( &format!( "<g{parent_transform}{clip}>" ) );
          for inst in instances
          {
            let inst_transform = Self::transform_to_svg_local( &inst.transform );
            let tint = Self::tint_filter_attr_split( &inst.tint, content, filter_counter )?;
            let sprite = format!
            (
              "<use href=\"#sprite_{}\"{}{} style=\"mix-blend-mode:{}\"/>",
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
            let blend = Self::blend_to_svg( &params.blend );
            let fill = Self::texture_or_fill_split( params.texture, &params.fill, resources, content );

            content.push_body( &format!( "<g{parent_transform}{clip}>" ) );
            for inst in instances
            {
              let inst_transform = Self::transform_to_svg_local( &inst.transform );
              let mesh = format!
              (
                "<use href=\"#{def_id}\" fill=\"{fill}\"{inst_transform} style=\"mix-blend-mode:{blend}\"/>"
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

    fn cmd_delete_batch( &mut self, db : &DeleteBatch )
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
          // Negate dy: Y-up shadow direction → SVG Y-down
          let def = format!
          (
            "<filter id=\"fx_{}\"><feDropShadow dx=\"{}\" dy=\"{}\" stdDeviation=\"{}\" flood-color=\"{}\"{}/></filter>",
            fid, dx, -dy, blur, c, flood_opacity
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

    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      self.content.clear_frame_defs();
      self.content.clear_body();
      self.resources.mesh_defs.clear();
      self.filter_counter = 0;
      self.group_depth = 0;
      self.recording_batch = None;

      for cmd in commands
      {
        match cmd
        {
          RenderCommand::Clear( c ) => self.cmd_clear( c ),
          RenderCommand::BeginPath( bp ) => self.cmd_begin_path( bp ),
          RenderCommand::MoveTo( m ) => self.cmd_move_to( m ),
          RenderCommand::LineTo( l ) => self.cmd_line_to( l ),
          RenderCommand::QuadTo( q ) => self.cmd_quad_to( q ),
          RenderCommand::CubicTo( c ) => self.cmd_cubic_to( c ),
          RenderCommand::ArcTo( a ) => self.cmd_arc_to( a ),
          RenderCommand::ClosePath( _ ) => self.cmd_close_path(),
          RenderCommand::EndPath( _ ) => self.cmd_end_path(),
          RenderCommand::BeginText( bt ) => self.cmd_begin_text( bt ),
          RenderCommand::Char( ch ) => self.cmd_char( ch ),
          RenderCommand::EndText( _ ) => self.cmd_end_text(),
          RenderCommand::Mesh( m ) => self.cmd_mesh( m ),
          RenderCommand::Sprite( s ) => self.cmd_sprite( s )?,
          RenderCommand::CreateSpriteBatch( cb ) => self.cmd_create_sprite_batch( cb ),
          RenderCommand::CreateMeshBatch( cb ) => self.cmd_create_mesh_batch( cb ),
          RenderCommand::BindBatch( bb ) => self.cmd_bind_batch( bb ),
          RenderCommand::AddSpriteInstance( si ) => self.cmd_add_sprite_instance( si ),
          RenderCommand::AddMeshInstance( mi ) => self.cmd_add_mesh_instance( mi ),
          RenderCommand::SetSpriteInstance( si ) => self.cmd_set_sprite_instance( si ),
          RenderCommand::SetMeshInstance( mi ) => self.cmd_set_mesh_instance( mi ),
          RenderCommand::RemoveInstance( ri ) => self.cmd_remove_instance( ri ),
          RenderCommand::SetSpriteBatchParams( sp ) => self.cmd_set_sprite_batch_params( sp ),
          RenderCommand::SetMeshBatchParams( mp ) => self.cmd_set_mesh_batch_params( mp ),
          RenderCommand::UnbindBatch( _ ) => self.cmd_unbind_batch(),
          RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( db )?,
          RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( db ),
          RenderCommand::BeginGroup( bg ) => self.cmd_begin_group( bg )?,
          RenderCommand::EndGroup( _ ) => self.cmd_end_group(),
        }
      }

      Ok( () )
    }

    fn resize( &mut self, width : u32, height : u32 )
    {
      self.config.width = width;
      self.config.height = height;
      self.content.update_header( width, height, Self::shape_rendering_attr( &self.config.antialias ) );
    }

    fn output( &self ) -> Result< Output, RenderError >
    {
      Ok( Output::String( self.content.buffer().to_string() ) )
    }

    fn capabilities( &self ) -> Capabilities
    {
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
      let diff = header.len() as isize - self.defs_start as isize;

      #[ allow( clippy::cast_sign_loss ) ]
      if diff != 0
      {
        self.defs_start          = ( self.defs_start          as isize + diff ) as usize;
        self.defs_end            = ( self.defs_end            as isize + diff ) as usize;
        self.frame_defs_start    = ( self.frame_defs_start    as isize + diff ) as usize;
        self.body_start          = ( self.body_start          as isize + diff ) as usize;
        self.vp_transform_start  = ( self.vp_transform_start  as isize + diff ) as usize;
        self.elements_start      = ( self.elements_start      as isize + diff ) as usize;
        self.body_end            = ( self.body_end            as isize + diff ) as usize;
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

      #[ allow( clippy::cast_sign_loss ) ]
      {
        let diff = new_transform.len() as isize - self.vp_transform_len as isize;
        self.vp_transform_len = new_transform.len();
        self.elements_start = ( self.elements_start as isize + diff ) as usize;
        self.body_end       = ( self.body_end       as isize + diff ) as usize;
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

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;
    use crate::backend::{ Backend, Output };

    // -- helpers --

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

    fn body( svg : &SvgBackend ) -> String
    {
      let full = render( svg );
      // The frame body is wrapped in a viewport <g transform="...">...</g>.
      // Return the inner content so tests don't need to know about the wrapper.
      let frame_start = full.find( "<!--framebegin-->" ).unwrap() + "<!--framebegin-->".len();
      let frame_end   = full.find( "<!--frameend-->" ).unwrap();
      let frame = &full[ frame_start..frame_end ];
      // Strip the opening <g ...> tag and trailing </g>
      let inner_start = frame.find( '>' ).map_or( 0, | i | i + 1 );
      let inner_end   = frame.rfind( "</" ).unwrap_or( frame.len() );
      frame[ inner_start..inner_end ].to_string()
    }

    fn defs( svg : &SvgBackend ) -> String
    {
      let full = render( svg );
      let start = full.find( "<defs>" ).unwrap() + "<defs>".len();
      let end = full.find( "</defs>" ).unwrap();
      full[ start..end ].to_string()
    }

    // -- clear --

    #[ test ]
    fn clear_emits_rect()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[ RenderCommand::Clear( Clear { color : [ 1.0, 0.0, 0.0, 1.0 ] } ) ] ).unwrap();
      let b = body( &svg );
      assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {b}" );
      assert!( b.contains( "width=\"100%\"" ) );
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

    /// Zoom is now applied via the viewport `<g>` wrapper, not per-element.
    /// Verify that `set_viewport_scale` updates the wrapper transform.
    #[ test ]
    fn viewport_zoom_updates_wrapper()
    {
      let mut svg = svg800x600();
      svg.set_viewport_scale( 2.0 );
      let full = svg.content.buffer().to_string();
      assert!( full.contains( "scale(2)" ), "wrapper: {full}" );
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

    /// Viewport offset is now applied via the `<g>` wrapper, not per-element.
    /// `set_viewport_offset` should update the wrapper transform attribute.
    #[ test ]
    fn viewport_offset_updates_wrapper()
    {
      let mut svg = svg800x600();
      svg.set_viewport_offset( [ 10.0, 20.0 ] );
      let full = svg.content.buffer().to_string();
      // In the wrapper: offset Y is negated (Y-up → SVG Y-down flip)
      assert!( full.contains( "translate(10,-20)" ), "wrapper: {full}" );
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

    // -- path --

    #[ test ]
    fn path_emits_svg_path()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginPath( BeginPath
        {
          transform : Transform::default(),
          fill : FillRef::Solid( [ 0.0, 0.0, 1.0, 1.0 ] ),
          stroke_color : [ 1.0, 1.0, 1.0, 1.0 ],
          stroke_width : 2.0,
          stroke_cap : LineCap::Round,
          stroke_join : LineJoin::Round,
          stroke_dash : DashStyle::default(),
          blend : BlendMode::Normal,
          clip : None,
        }),
        RenderCommand::MoveTo( MoveTo( 10.0, 20.0 ) ),
        RenderCommand::LineTo( LineTo( 100.0, 200.0 ) ),
        RenderCommand::ClosePath( ClosePath ),
        RenderCommand::EndPath( EndPath ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "<path" ), "body: {b}" );
      assert!( b.contains( "M 10 20" ), "body: {b}" );
      assert!( b.contains( "L 100 200" ), "body: {b}" );
      assert!( b.contains( 'Z' ), "body: {b}" );
      assert!( b.contains( "fill=\"rgb(0,0,255)\"" ), "body: {b}" );
      assert!( b.contains( "stroke-linecap=\"round\"" ), "body: {b}" );
      assert!( b.contains( "stroke-linejoin=\"round\"" ), "body: {b}" );
    }

    // -- image loading viewBox --

    #[ test ]
    fn image_viewbox_origin_zero()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap { bytes : vec![ 0u8; 64 * 32 * 4 ], width : 64, height : 32, format : PixelFormat::Rgba8 },
          filter : SamplerFilter::Linear,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();

      let d = defs( &svg );
      // Should use "0 0 w h" viewBox, not center-origin
      assert!( d.contains( "viewBox=\"0 0 64 32\"" ), "defs: {d}" );
      // Should not have negative offsets
      assert!( !d.contains( "x=\"-" ), "defs: {d}" );
      assert!( !d.contains( "y=\"-" ), "defs: {d}" );
    }

    // -- sprite tint --

    #[ test ]
    fn sprite_white_tint_no_filter()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
          filter : SamplerFilter::Linear,
        }],
        sprites : vec![ SpriteAsset
        {
          id : ResourceId::new( 0 ),
          sheet : ResourceId::new( 0 ),
          region : [ 0.0, 0.0, 16.0, 16.0 ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      svg.submit( &[
        RenderCommand::Sprite( Sprite
        {
          transform : Transform::default(),
          sprite : ResourceId::new( 0 ),
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
          blend : BlendMode::Normal,
          clip : None,
        }),
      ]).unwrap();

      let b = body( &svg );
      assert!( !b.contains( "filter=" ), "white tint should not create filter, body: {b}" );
    }

    #[ test ]
    fn sprite_colored_tint_creates_filter()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
          filter : SamplerFilter::Linear,
        }],
        sprites : vec![ SpriteAsset
        {
          id : ResourceId::new( 0 ),
          sheet : ResourceId::new( 0 ),
          region : [ 0.0, 0.0, 16.0, 16.0 ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      svg.submit( &[
        RenderCommand::Sprite( Sprite
        {
          transform : Transform::default(),
          sprite : ResourceId::new( 0 ),
          tint : [ 1.0, 0.0, 0.0, 1.0 ],
          blend : BlendMode::Normal,
          clip : None,
        }),
      ]).unwrap();

      let b = body( &svg );
      let d = defs( &svg );
      assert!( b.contains( "filter=\"url(#tint_0)\"" ), "body: {b}" );
      assert!( d.contains( "<filter id=\"tint_0\">" ), "defs: {d}" );
      assert!( d.contains( "feColorMatrix" ), "defs: {d}" );
    }

    #[ test ]
    fn two_tinted_sprites_get_distinct_filter_ids()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
          filter : SamplerFilter::Linear,
        }],
        sprites : vec![ SpriteAsset
        {
          id : ResourceId::new( 0 ),
          sheet : ResourceId::new( 0 ),
          region : [ 0.0, 0.0, 16.0, 16.0 ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let s = Sprite
      {
        transform : Transform::default(),
        sprite : ResourceId::new( 0 ),
        tint : [ 1.0, 0.0, 0.0, 1.0 ],
        blend : BlendMode::Normal,
        clip : None,
      };
      svg.submit( &[ RenderCommand::Sprite( s ), RenderCommand::Sprite( Sprite { tint : [ 0.0, 1.0, 0.0, 1.0 ], ..s } ) ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "url(#tint_0)" ), "body: {b}" );
      assert!( b.contains( "url(#tint_1)" ), "body: {b}" );
    }

    // -- batch lifecycle --

    #[ test ]
    fn sprite_batch_create_draw()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 32, height : 32, format : PixelFormat::Rgba8 },
          filter : SamplerFilter::Linear,
        }],
        sprites : vec![ SpriteAsset
        {
          id : ResourceId::new( 0 ),
          sheet : ResourceId::new( 0 ),
          region : [ 0.0, 0.0, 32.0, 32.0 ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();

      let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
      svg.submit( &[
        RenderCommand::CreateSpriteBatch( CreateSpriteBatch
        {
          batch : batch_id,
          params : SpriteBatchParams
          {
            transform : Transform::default(),
            sheet : ResourceId::new( 0 ),
            blend : BlendMode::Normal,
            clip : None,
          },
        }),
        RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
        RenderCommand::AddSpriteInstance( AddSpriteInstance
        {
          transform : Transform { position : [ 10.0, 20.0 ], ..Default::default() },
          sprite : ResourceId::new( 0 ),
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
        }),
        RenderCommand::AddSpriteInstance( AddSpriteInstance
        {
          transform : Transform { position : [ 50.0, 60.0 ], ..Default::default() },
          sprite : ResourceId::new( 0 ),
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
        }),
        RenderCommand::UnbindBatch( UnbindBatch ),
        RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
      ]).unwrap();

      let b = body( &svg );
      // Should have a group wrapper
      assert!( b.contains( "<g" ), "body: {b}" );
      assert!( b.contains( "</g>" ), "body: {b}" );
      // Should have two sprite instances with local transforms
      assert_eq!( b.matches( "#sprite_0" ).count(), 2, "body: {b}" );
      // Local transforms should use raw positions (no Y-flip)
      assert!( b.contains( "translate(10,20)" ), "body: {b}" );
      assert!( b.contains( "translate(50,60)" ), "body: {b}" );
    }

    // -- mesh batch --

    #[ test ]
    fn mesh_batch_create_draw()
    {
      let mut svg = svg800x600();
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
      let assets = Assets
      {
        geometries : vec![ GeometryAsset
        {
          id : ResourceId::new( 0 ),
          positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
          uvs : None,
          indices : None,
          data_type : DataType::U16,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();

      let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
      svg.submit( &[
        RenderCommand::CreateMeshBatch( CreateMeshBatch
        {
          batch : batch_id,
          params : MeshBatchParams
          {
            transform : Transform::default(),
            geometry : ResourceId::new( 0 ),
            fill : FillRef::Solid( [ 0.0, 1.0, 0.0, 1.0 ] ),
            texture : None,
            topology : Topology::TriangleList,
            blend : BlendMode::Normal,
            clip : None,
          },
        }),
        RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
        RenderCommand::AddMeshInstance( AddMeshInstance
        {
          transform : Transform { position : [ 5.0, 10.0 ], ..Default::default() },
        }),
        RenderCommand::UnbindBatch( UnbindBatch ),
        RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "<g" ), "body: {b}" );
      assert!( b.contains( "fill=\"rgb(0,255,0)\"" ), "body: {b}" );
      assert!( b.contains( "translate(5,10)" ), "body: {b}" );
    }

    // -- batch instance update and remove --

    #[ test ]
    fn batch_set_and_remove_instance()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();

      let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
      // First submit: create batch with 2 instances
      svg.submit( &[
        RenderCommand::CreateSpriteBatch( CreateSpriteBatch
        {
          batch : batch_id,
          params : SpriteBatchParams
          {
            transform : Transform::default(),
            sheet : ResourceId::new( 0 ),
            blend : BlendMode::Normal,
            clip : None,
          },
        }),
        RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
        RenderCommand::AddSpriteInstance( AddSpriteInstance
        {
          transform : Transform { position : [ 1.0, 2.0 ], ..Default::default() },
          sprite : ResourceId::new( 0 ),
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
        }),
        RenderCommand::AddSpriteInstance( AddSpriteInstance
        {
          transform : Transform { position : [ 3.0, 4.0 ], ..Default::default() },
          sprite : ResourceId::new( 0 ),
          tint : [ 1.0, 1.0, 1.0, 1.0 ],
        }),
        RenderCommand::UnbindBatch( UnbindBatch ),
      ]).unwrap();

      // Second submit: remove first instance, draw
      svg.submit( &[
        RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
        RenderCommand::RemoveInstance( RemoveInstance { index : 0 } ),
        RenderCommand::UnbindBatch( UnbindBatch ),
        RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
      ]).unwrap();

      let b = body( &svg );
      // Should have only 1 instance (the one at 3,4)
      assert_eq!( b.matches( "#sprite_0" ).count(), 1, "body: {b}" );
      assert!( b.contains( "translate(3,4)" ), "body: {b}" );
      assert!( !b.contains( "translate(1,2)" ), "body: {b}" );
    }

    // -- delete batch --

    #[ test ]
    fn delete_batch()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();

      let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
      svg.submit( &[
        RenderCommand::CreateSpriteBatch( CreateSpriteBatch
        {
          batch : batch_id,
          params : SpriteBatchParams
          {
            transform : Transform::default(),
            sheet : ResourceId::new( 0 ),
            blend : BlendMode::Normal,
            clip : None,
          },
        }),
        RenderCommand::DeleteBatch( DeleteBatch { batch : batch_id } ),
        RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
      ]).unwrap();

      let b = body( &svg );
      // Draw after delete should produce nothing
      assert!( !b.contains( "<g" ), "body: {b}" );
    }

    // -- effects --

    #[ test ]
    fn effect_blur()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginGroup( BeginGroup
        {
          transform : Transform::default(),
          clip : None,
          effect : Some( Effect::Blur { radius : 5.0 } ),
        }),
        RenderCommand::EndGroup( EndGroup ),
      ]).unwrap();

      let b = body( &svg );
      let d = defs( &svg );
      assert!( b.contains( "filter=\"url(#fx_0)\"" ), "body: {b}" );
      assert!( d.contains( "feGaussianBlur" ), "defs: {d}" );
      assert!( d.contains( "stdDeviation=\"5\"" ), "defs: {d}" );
    }

    #[ test ]
    fn effect_drop_shadow_y_flipped()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginGroup( BeginGroup
        {
          transform : Transform::default(),
          clip : None,
          effect : Some( Effect::DropShadow { dx : 2.0, dy : 3.0, blur : 4.0, color : [ 0.0, 0.0, 0.0, 0.5 ] } ),
        }),
        RenderCommand::EndGroup( EndGroup ),
      ]).unwrap();

      let d = defs( &svg );
      assert!( d.contains( "feDropShadow" ), "defs: {d}" );
      assert!( d.contains( "dx=\"2\"" ), "defs: {d}" );
      // dy should be negated: 3.0 → -3.0
      assert!( d.contains( "dy=\"-3\"" ), "defs: {d}" );
    }

    #[ test ]
    fn effect_color_matrix()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      let mut values = [ 0.0f32; 20 ];
      values[ 0 ] = 1.0; // r->r
      values[ 6 ] = 1.0; // g->g
      values[ 12 ] = 1.0; // b->b
      values[ 18 ] = 1.0; // a->a
      svg.submit( &[
        RenderCommand::BeginGroup( BeginGroup
        {
          transform : Transform::default(),
          clip : None,
          effect : Some( Effect::ColorMatrix( values ) ),
        }),
        RenderCommand::EndGroup( EndGroup ),
      ]).unwrap();

      let d = defs( &svg );
      assert!( d.contains( "feColorMatrix" ), "defs: {d}" );
      assert!( d.contains( "type=\"matrix\"" ), "defs: {d}" );
    }

    #[ test ]
    fn effect_opacity()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginGroup( BeginGroup
        {
          transform : Transform::default(),
          clip : None,
          effect : Some( Effect::Opacity( 0.5 ) ),
        }),
        RenderCommand::EndGroup( EndGroup ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "opacity=\"0.5\"" ), "body: {b}" );
    }

    // -- groups --

    #[ test ]
    fn nested_groups()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginGroup( BeginGroup { transform : Transform::default(), clip : None, effect : None } ),
        RenderCommand::BeginGroup( BeginGroup { transform : Transform::default(), clip : None, effect : None } ),
        RenderCommand::EndGroup( EndGroup ),
        RenderCommand::EndGroup( EndGroup ),
      ]).unwrap();

      let b = body( &svg );
      assert_eq!( b.matches( "<g" ).count(), 2 );
      assert_eq!( b.matches( "</g>" ).count(), 2 );
    }

    #[ test ]
    fn unmatched_end_group_does_not_emit_closing_tag()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[ RenderCommand::EndGroup( EndGroup ) ]).unwrap();

      let b = body( &svg );
      assert_eq!( b.matches( "</g>" ).count(), 0, "unmatched EndGroup should not emit </g>: {b}" );
    }

    // -- geometry mesh --

    #[ test ]
    fn mesh_triangle_list()
    {
      let mut svg = svg800x600();
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
      let assets = Assets
      {
        geometries : vec![ GeometryAsset
        {
          id : ResourceId::new( 0 ),
          positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
          uvs : None,
          indices : None,
          data_type : DataType::U16,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();

      svg.submit( &[
        RenderCommand::Mesh( Mesh
        {
          transform : Transform::default(),
          geometry : ResourceId::new( 0 ),
          fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
          texture : None,
          topology : Topology::TriangleList,
          blend : BlendMode::Normal,
          clip : None,
        }),
      ]).unwrap();

      let b = body( &svg );
      let d = defs( &svg );
      assert!( d.contains( "<polygon" ), "defs: {d}" );
      assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {b}" );
    }

    /// Verifies that DataType::U8 index buffers are correctly loaded and used
    /// so geometry with U8 indices renders polygons rather than being silently dropped.
    #[ test ]
    fn geometry_u8_indices_loaded()
    {
      let mut svg = svg800x600();
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
      let indices_u8 : &[ u8 ] = &[ 0, 1, 2 ];
      let assets = Assets
      {
        geometries : vec![ GeometryAsset
        {
          id : ResourceId::new( 0 ),
          positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
          uvs : None,
          indices : Some( Source::Bytes( indices_u8.to_vec() ) ),
          data_type : DataType::U8,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      svg.submit( &[
        RenderCommand::Mesh( Mesh
        {
          transform : Transform::default(),
          geometry : ResourceId::new( 0 ),
          fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
          texture : None,
          topology : Topology::TriangleList,
          blend : BlendMode::Normal,
          clip : None,
        }),
      ]).unwrap();

      let d = defs( &svg );
      assert!( d.contains( "<polygon" ), "U8 indices not used — polygon missing from defs: {d}" );
    }

    /// Verifies that out-of-bounds indices in geometry do not cause a panic.
    /// The out-of-range polygon is silently skipped; valid polygons still render.
    #[ test ]
    fn geometry_oob_index_no_panic()
    {
      let mut svg = svg800x600();
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ]; // 3 vertices
      // Triangle 0: valid (0,1,2). Triangle 1: index 99 is out of bounds.
      let indices : Vec< u32 > = vec![ 0, 1, 2, 0, 1, 99 ];
      let assets = Assets
      {
        geometries : vec![ GeometryAsset
        {
          id : ResourceId::new( 0 ),
          positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
          uvs : None,
          indices : Some( Source::Bytes( bytemuck::cast_slice( &indices ).to_vec() ) ),
          data_type : DataType::U32,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      // Must not panic
      svg.submit( &[
        RenderCommand::Mesh( Mesh
        {
          transform : Transform::default(),
          geometry : ResourceId::new( 0 ),
          fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
          texture : None,
          topology : Topology::TriangleList,
          blend : BlendMode::Normal,
          clip : None,
        }),
      ]).unwrap();

      let d = defs( &svg );
      // The valid first triangle should still appear
      assert!( d.contains( "<polygon" ), "valid polygon missing from defs: {d}" );
    }

    fn mesh_svg( topology : Topology, positions : &[ f32 ] ) -> ( String, String )
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        geometries : vec![ GeometryAsset
        {
          id : ResourceId::new( 0 ),
          positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
          uvs : None,
          indices : None,
          data_type : DataType::U16,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      svg.submit( &[
        RenderCommand::Mesh( Mesh
        {
          transform : Transform::default(),
          geometry : ResourceId::new( 0 ),
          fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
          texture : None,
          topology,
          blend : BlendMode::Normal,
          clip : None,
        }),
      ]).unwrap();
      ( body( &svg ), defs( &svg ) )
    }

    /// TriangleStrip with 4 vertices produces 2 triangles (n − 2 = 2 polygons).
    #[ test ]
    fn mesh_triangle_strip_polygon_count()
    {
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 0.0, 100.0, 100.0, 100.0 ];
      let ( _b, d ) = mesh_svg( Topology::TriangleStrip, positions );
      assert_eq!( d.matches( "<polygon" ).count(), 2, "defs: {d}" );
    }

    /// TriangleStrip with exactly 3 vertices produces exactly 1 triangle.
    #[ test ]
    fn mesh_triangle_strip_min_count()
    {
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
      let ( _b, d ) = mesh_svg( Topology::TriangleStrip, positions );
      assert_eq!( d.matches( "<polygon" ).count(), 1, "defs: {d}" );
    }

    /// TriangleStrip alternates winding on odd triangles — for strip v0..v3,
    /// triangle 0 is (v0,v1,v2) and triangle 1 is (v2,v1,v3), preserving CCW order
    /// (swapping the first two would flip winding; the second triangle in a raw
    /// strip is (v1,v2,v3) which has opposite winding from (v0,v1,v2)).
    #[ test ]
    fn mesh_triangle_strip_alternates_winding()
    {
      // Four distinct vertices so we can identify the emitted order.
      let positions : &[ f32 ] = &[ 0.0, 0.0, 10.0, 0.0, 0.0, 10.0, 10.0, 10.0 ];
      let ( _b, d ) = mesh_svg( Topology::TriangleStrip, positions );
      // First triangle: v0,v1,v2 => "0,0 10,0 0,10"
      assert!( d.contains( "points=\"0,0 10,0 0,10\"" ), "first tri wrong: {d}" );
      // Second triangle: order swapped to v2,v1,v3 => "0,10 10,0 10,10"
      assert!( d.contains( "points=\"0,10 10,0 10,10\"" ), "second tri winding not alternated: {d}" );
      // Raw (un-alternated) order would have been v1,v2,v3 => "10,0 0,10 10,10" — ensure it's absent.
      assert!( !d.contains( "points=\"10,0 0,10 10,10\"" ), "strip emitted raw order: {d}" );
    }

    /// TriangleStrip with fewer than 3 vertices produces no geometry — degenerate input is silently skipped.
    #[ test ]
    fn mesh_triangle_strip_degenerate_no_output()
    {
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0 ]; // 2 vertices
      let ( b, _d ) = mesh_svg( Topology::TriangleStrip, positions );
      // No <use> in body — the mesh def was not created
      assert!( !b.contains( "<use" ), "body: {b}" );
    }

    /// LineList with 4 vertices (2 pairs) produces 2 `<polyline>` elements.
    #[ test ]
    fn mesh_line_list_even()
    {
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 100.0, 200.0, 0.0, 300.0, 100.0 ];
      let ( _b, d ) = mesh_svg( Topology::LineList, positions );
      assert_eq!( d.matches( "<polyline" ).count(), 2, "defs: {d}" );
    }

    /// LineList with 3 vertices (odd) emits only 1 `<polyline>` — the trailing vertex is ignored.
    #[ test ]
    fn mesh_line_list_odd_vertex_count()
    {
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 100.0, 200.0, 0.0 ];
      let ( _b, d ) = mesh_svg( Topology::LineList, positions );
      assert_eq!( d.matches( "<polyline" ).count(), 1, "defs: {d}" );
    }

    /// LineStrip with 4 vertices produces a single `<polyline>` with all points.
    #[ test ]
    fn mesh_line_strip_single_polyline()
    {
      let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 100.0, 100.0, 0.0, 100.0 ];
      let ( _b, d ) = mesh_svg( Topology::LineStrip, positions );
      assert_eq!( d.matches( "<polyline" ).count(), 1, "defs: {d}" );
    }

    // -- resize --

    #[ test ]
    fn resize_updates_viewbox()
    {
      let mut svg = svg800x600();
      svg.resize( 1024, 768 );
      let full = render( &svg );
      assert!( full.contains( "width=\"1024\"" ), "full: {full}" );
      assert!( full.contains( "height=\"768\"" ), "full: {full}" );
      assert!( full.contains( "viewBox=\"0 0 1024 768\"" ), "full: {full}" );
    }

    // -- capabilities --

    #[ test ]
    fn capabilities_all_true()
    {
      let svg = svg800x600();
      let caps = svg.capabilities();
      assert!( caps.paths );
      assert!( caps.text );
      assert!( caps.meshes );
      assert!( caps.sprites );
      assert!( caps.batches );
      assert!( caps.gradients );
      assert!( caps.patterns );
      assert!( caps.clip_masks );
      assert!( caps.effects );
      assert!( caps.blend_modes );
      assert!( caps.text_on_path );
      assert_eq!( caps.max_texture_size, 0 );
    }

    // -- blend modes --

    #[ test ]
    fn blend_mode_multiply()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginPath( BeginPath
        {
          transform : Transform::default(),
          fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
          stroke_color : [ 0.0, 0.0, 0.0, 0.0 ],
          stroke_width : 0.0,
          stroke_cap : LineCap::Butt,
          stroke_join : LineJoin::Miter,
          stroke_dash : DashStyle::default(),
          blend : BlendMode::Multiply,
          clip : None,
        }),
        RenderCommand::MoveTo( MoveTo( 0.0, 0.0 ) ),
        RenderCommand::EndPath( EndPath ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "mix-blend-mode:multiply" ), "body: {b}" );
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
      let png = SvgBackend::bitmap_to_png( &bytes, 3, 5, &PixelFormat::Rgba8 ).unwrap();
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

    /// Verifies that an ImageSource::Path containing XML-special characters
    /// cannot break out of the href attribute and inject event handlers like
    /// onload="alert(1)". Filenames with double-quotes are legal on Linux.
    #[ test ]
    fn image_path_escapes_attribute_injection()
    {
      let mut svg = svg800x600();
      let malicious = r#"foo" onload="alert(1)"#;
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Path( std::path::PathBuf::from( malicious ) ),
          filter : SamplerFilter::Linear,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let d = defs( &svg );
      // Raw unescaped injection must not appear.
      assert!( !d.contains( r#"onload="alert(1)""# ), "event handler leaked: {d}" );
      // Percent-encoded form must appear (quote => %22).
      assert!( d.contains( "%22" ), "expected percent-encoded quote in href: {d}" );
    }

    /// Verifies that path_to_href produces a valid URI reference:
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

    /// Verifies that a sprite referencing an ImageSource::Path sheet
    /// (which has unknown dimensions) is skipped and a diagnostic HTML
    /// comment is emitted instead of producing an invisible sprite symbol.
    #[ test ]
    fn sprite_on_path_sheet_is_skipped_with_comment()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Path( "does_not_matter.png".into() ),
          filter : SamplerFilter::Linear,
        }],
        sprites : vec![ SpriteAsset
        {
          id : ResourceId::new( 7 ),
          sheet : ResourceId::new( 0 ),
          region : [ 0.0, 0.0, 4.0, 4.0 ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let d = defs( &svg );
      // No sprite_7 symbol was emitted.
      assert!( !d.contains( "id=\"sprite_7\"" ), "sprite should be skipped: {d}" );
      // A diagnostic comment was emitted instead.
      assert!( d.contains( "sprite_7 skipped" ), "diagnostic comment missing: {d}" );
    }

    /// Verifies that JPEG-encoded bytes produce a `data:image/jpeg` URI.
    #[ test ]
    fn image_encoded_jpeg_emits_jpeg_mime()
    {
      let jpeg_bytes : Vec< u8 > = vec![ 0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46 ];
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Encoded( jpeg_bytes ),
          filter : SamplerFilter::Linear,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let d = defs( &svg );
      assert!( d.contains( "data:image/jpeg;base64," ), "defs: {d}" );
      assert!( !d.contains( "data:image/png;base64," ), "should not emit PNG mime: {d}" );
    }

    /// Verifies that a short / non-PNG buffer returns None.
    #[ test ]
    fn png_dimensions_invalid()
    {
      assert_eq!( SvgBackend::png_dimensions( &[] ), None );
      assert_eq!( SvgBackend::png_dimensions( &[ 0u8; 24 ] ), None ); // no PNG signature
    }

    /// Verifies that load_assets extracts PNG dimensions from ImageSource::Encoded
    /// so that a sprite symbol uses the correct sheet size.
    #[ test ]
    fn image_encoded_png_stores_dimensions()
    {
      let png = SvgBackend::bitmap_to_png( &vec![ 0u8; 8 * 4 * 4 ], 8, 4, &PixelFormat::Rgba8 ).unwrap();
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Encoded( png ),
          filter : SamplerFilter::Linear,
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

    // -- bitmap_to_png --

    const PNG_MAGIC : &[ u8 ] = &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ];

    /// Verifies that a 1×1 Rgba8 pixel buffer produces valid PNG output
    /// (starts with the PNG magic bytes).
    #[ test ]
    fn bitmap_to_png_rgba8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 255, 0, 128, 255 ], 1, 1, &PixelFormat::Rgba8 );
      let bytes = png.expect( "expected Some for valid 1x1 Rgba8" );
      assert!( bytes.starts_with( PNG_MAGIC ), "not PNG: {:?}", &bytes[ ..8.min( bytes.len() ) ] );
    }

    /// Verifies that a 1×1 Rgb8 pixel buffer encodes successfully.
    #[ test ]
    fn bitmap_to_png_rgb8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 255, 0, 128 ], 1, 1, &PixelFormat::Rgb8 );
      assert!( png.is_some(), "expected Some for valid 1x1 Rgb8" );
    }

    /// Verifies that a 1×1 Gray8 pixel buffer encodes successfully.
    #[ test ]
    fn bitmap_to_png_gray8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 128 ], 1, 1, &PixelFormat::Gray8 );
      assert!( png.is_some(), "expected Some for valid 1x1 Gray8" );
    }

    /// Verifies that a 1×1 GrayAlpha8 pixel buffer encodes successfully.
    #[ test ]
    fn bitmap_to_png_gray_alpha8_valid()
    {
      let png = SvgBackend::bitmap_to_png( &[ 128, 255 ], 1, 1, &PixelFormat::GrayAlpha8 );
      assert!( png.is_some(), "expected Some for valid 1x1 GrayAlpha8" );
    }

    /// Verifies that mismatched dimensions (too few bytes for the declared size) return None.
    #[ test ]
    fn bitmap_to_png_dimension_mismatch_returns_none()
    {
      // 2×2 Rgba8 needs 16 bytes; supplying only 4 must return None
      let png = SvgBackend::bitmap_to_png( &[ 255, 0, 0, 255 ], 2, 2, &PixelFormat::Rgba8 );
      assert!( png.is_none(), "expected None for undersized buffer" );
    }

    /// End-to-end: load a 2×2 Rgba8 Bitmap image asset and verify that `<defs>`
    /// contains a `data:image/png;base64,` URI — the full encode path ran.
    #[ test ]
    fn image_bitmap_emits_png_data_uri()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap
          {
            bytes : vec![ 255u8; 2 * 2 * 4 ],
            width : 2,
            height : 2,
            format : PixelFormat::Rgba8,
          },
          filter : SamplerFilter::Linear,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let d = defs( &svg );
      assert!( d.contains( "data:image/png;base64," ), "defs: {d}" );
    }

    /// When the byte buffer is too small for the declared dimensions,
    /// bitmap_to_png returns None and no image def is emitted.
    #[ test ]
    fn image_bitmap_bad_dimensions_emits_nothing()
    {
      let mut svg = svg800x600();
      let assets = Assets
      {
        images : vec![ ImageAsset
        {
          id : ResourceId::new( 0 ),
          source : ImageSource::Bitmap
          {
            bytes : vec![ 255u8; 4 ], // too small for 4×4
            width : 4,
            height : 4,
            format : PixelFormat::Rgba8,
          },
          filter : SamplerFilter::Linear,
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      let d = defs( &svg );
      assert!( !d.contains( "data:image/png;base64," ), "expected no image def, defs: {d}" );
    }

    // -- text rendering --

    fn begin_text_cmd( anchor : TextAnchor, position : [ f32; 2 ] ) -> RenderCommand
    {
      RenderCommand::BeginText( BeginText
      {
        font : ResourceId::new( 0 ),
        size : 16.0,
        color : [ 1.0, 1.0, 1.0, 1.0 ],
        anchor,
        position,
        along_path : None,
        clip : None,
      })
    }

    /// Verifies that BeginText / Char / EndText produces a `<text>` element
    /// containing the submitted characters.
    #[ test ]
    fn text_basic_flow_emits_text_element()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        begin_text_cmd( TextAnchor::TopLeft, [ 10.0, 20.0 ] ),
        RenderCommand::Char( Char( 'H' ) ),
        RenderCommand::Char( Char( 'i' ) ),
        RenderCommand::EndText( EndText ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "<text" ), "body: {b}" );
      assert!( b.contains( "Hi" ), "body: {b}" );
    }

    /// Verifies SVG 1.1 conformance: translucent colors emit `rgb()` plus a
    /// separate `*-opacity` attribute, never the CSS-Color-Level-4 `rgba()`
    /// notation (which Inkscape / strict SVG parsers may reject).
    #[ test ]
    fn color_emits_svg11_rgb_plus_opacity_not_rgba()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[ RenderCommand::Clear( Clear { color : [ 1.0, 0.0, 0.0, 0.5 ] } ) ] ).unwrap();
      let b = body( &svg );
      assert!( !b.contains( "rgba(" ), "rgba() notation leaked (not SVG 1.1): {b}" );
      assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "expected rgb() fill: {b}" );
      assert!( b.contains( "fill-opacity=\"0.5\"" ), "expected fill-opacity attr: {b}" );
    }

    /// Opaque colors (alpha = 1.0) emit no opacity attribute at all.
    #[ test ]
    fn opaque_color_omits_opacity_attribute()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[ RenderCommand::Clear( Clear { color : [ 0.0, 1.0, 0.0, 1.0 ] } ) ] ).unwrap();
      let b = body( &svg );
      assert!( b.contains( "fill=\"rgb(0,255,0)\"" ), "expected opaque rgb: {b}" );
      assert!( !b.contains( "fill-opacity" ), "opaque color should not emit opacity attr: {b}" );
    }

    /// Verifies that XML-special characters in the Char stream are escaped
    /// so they cannot break out of the <text> element and inject markup.
    #[ test ]
    fn text_escapes_xml_special_characters()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      let injection = "</text><script>x</script>";
      let mut cmds : Vec< RenderCommand > = vec![ begin_text_cmd( TextAnchor::TopLeft, [ 0.0, 0.0 ] ) ];
      cmds.extend( injection.chars().map( | c | RenderCommand::Char( Char( c ) ) ) );
      cmds.push( RenderCommand::EndText( EndText ) );
      svg.submit( &cmds ).unwrap();

      let b = body( &svg );
      // The raw injection must NOT appear — the </text> and <script> tags must be escaped.
      assert!( !b.contains( "</text><script>" ), "injection not escaped: {b}" );
      assert!( !b.contains( "<script>" ), "script tag leaked: {b}" );
      // The escaped form must be present.
      assert!( b.contains( "&lt;/text&gt;&lt;script&gt;" ), "expected escaped form: {b}" );
    }

    /// Verifies that EndText without BeginText is silently ignored (no panic, no output).
    #[ test ]
    fn text_end_without_begin_is_noop()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[ RenderCommand::EndText( EndText ) ] ).unwrap();
      assert!( !body( &svg ).contains( "<text" ) );
    }

    /// Verifies font-size is emitted in the `<text>` element.
    #[ test ]
    fn text_emits_font_size()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        RenderCommand::BeginText( BeginText
        {
          font : ResourceId::new( 0 ),
          size : 24.0,
          color : [ 0.0, 0.0, 0.0, 1.0 ],
          anchor : TextAnchor::Center,
          position : [ 0.0, 0.0 ],
          along_path : None,
          clip : None,
        }),
        RenderCommand::Char( Char( 'A' ) ),
        RenderCommand::EndText( EndText ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "font-size=\"24\"" ), "body: {b}" );
    }

    // anchor_to_svg — 9 variants (private method, must stay inline)

    #[ test ]
    fn anchor_top_left()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::TopLeft );
      assert_eq!( h, "start" );
      assert_eq!( v, "hanging" );
    }

    #[ test ]
    fn anchor_top_center()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::TopCenter );
      assert_eq!( h, "middle" );
      assert_eq!( v, "hanging" );
    }

    #[ test ]
    fn anchor_top_right()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::TopRight );
      assert_eq!( h, "end" );
      assert_eq!( v, "hanging" );
    }

    #[ test ]
    fn anchor_center_left()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::CenterLeft );
      assert_eq!( h, "start" );
      assert_eq!( v, "central" );
    }

    #[ test ]
    fn anchor_center()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::Center );
      assert_eq!( h, "middle" );
      assert_eq!( v, "central" );
    }

    #[ test ]
    fn anchor_center_right()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::CenterRight );
      assert_eq!( h, "end" );
      assert_eq!( v, "central" );
    }

    #[ test ]
    fn anchor_bottom_left()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::BottomLeft );
      assert_eq!( h, "start" );
      assert_eq!( v, "baseline" );
    }

    #[ test ]
    fn anchor_bottom_center()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::BottomCenter );
      assert_eq!( h, "middle" );
      assert_eq!( v, "baseline" );
    }

    #[ test ]
    fn anchor_bottom_right()
    {
      let ( h, v ) = SvgBackend::anchor_to_svg( &TextAnchor::BottomRight );
      assert_eq!( h, "end" );
      assert_eq!( v, "baseline" );
    }

    /// Verifies that anchor attributes from BeginText are written into the `<text>` element.
    #[ test ]
    fn text_anchor_attrs_in_output()
    {
      let mut svg = svg800x600();
      svg.load_assets( &empty_assets() ).unwrap();
      svg.submit( &[
        begin_text_cmd( TextAnchor::BottomRight, [ 0.0, 0.0 ] ),
        RenderCommand::Char( Char( 'X' ) ),
        RenderCommand::EndText( EndText ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "text-anchor=\"end\"" ), "body: {b}" );
      assert!( b.contains( "dominant-baseline=\"baseline\"" ), "body: {b}" );
    }

    /// Verifies that text with `along_path` produces a `<textPath href="#path_N">` element.
    #[ test ]
    fn text_along_path_emits_text_path()
    {
      use crate::assets::{ PathAsset, PathSegment };

      let mut svg = svg800x600();
      let assets = Assets
      {
        paths : vec![ PathAsset
        {
          id : ResourceId::new( 3 ),
          segments : vec![ PathSegment::MoveTo( 0.0, 0.0 ), PathSegment::LineTo( 200.0, 0.0 ) ],
        }],
        ..empty_assets()
      };
      svg.load_assets( &assets ).unwrap();
      svg.submit( &[
        RenderCommand::BeginText( BeginText
        {
          font : ResourceId::new( 0 ),
          size : 12.0,
          color : [ 0.0, 0.0, 0.0, 1.0 ],
          anchor : TextAnchor::Center,
          position : [ 0.0, 0.0 ],
          along_path : Some( ResourceId::new( 3 ) ),
          clip : None,
        }),
        RenderCommand::Char( Char( 'A' ) ),
        RenderCommand::Char( Char( 'B' ) ),
        RenderCommand::EndText( EndText ),
      ]).unwrap();

      let b = body( &svg );
      assert!( b.contains( "<textPath" ), "body: {b}" );
      assert!( b.contains( "href=\"#path_3\"" ), "body: {b}" );
      assert!( b.contains( "AB" ), "body: {b}" );
    }
  }
}

mod_interface::mod_interface!
{
  own use SvgBackend;
}
