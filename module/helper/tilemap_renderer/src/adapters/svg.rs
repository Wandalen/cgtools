//! SVG backend adapter.
//!
//! Generates a complete SVG 1.1 document from render commands.
//! Supports all features: paths, text, sprites, gradients, patterns,
//! clip masks, effects, blend modes, and text-on-path.

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
  /// Map of generated mesh definitions ( packed geom_id + topology ) -> symbol_id
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
  pub viewport_offset : [ f32; 2 ],
  /// Scale applied to all visual elements in the SVG.
  pub viewport_scale : f32,
}

impl SvgBackend
{
  /// Creates a new SVG backend from render config.
  #[ must_use ]
  pub fn new( config : RenderConfig ) -> Self
  {
    Self
    {
      config : config.clone(),
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
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    let ( r, g, b, a ) =
    (
      ( color[ 0 ] * 255.0 ) as u8,
      ( color[ 1 ] * 255.0 ) as u8,
      ( color[ 2 ] * 255.0 ) as u8,
      color[ 3 ],
    );

    if ( a - 1.0 ).abs() < f32::EPSILON
    {
      format!( "rgb({r},{g},{b})" )
    }
    else
    {
      format!( "rgba({r},{g},{b},{a})" )
    }
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
    Self::transform_to_svg_static( t, self.config.width, self.config.height, self.viewport_offset, self.viewport_scale )
  }

  fn transform_to_svg_static( t : &Transform, _width : u32, height : u32, offset : [ f32; 2 ], zoom : f32 ) -> String
  {
    let mut parts = Vec::new();

    // Apply viewport offset
    let mut pos = t.position;
    pos[ 0 ] += offset[ 0 ];
    pos[ 1 ] += offset[ 1 ];

    // Y-up (0,0 = bottom-left) → SVG Y-down (0,0 = top-left)
    pos[ 1 ] = height as f32 - pos[ 1 ];

    if zoom != 1.0
    {
      parts.push( format!( "scale({})", zoom ) );
    }

    if pos[ 0 ] != 0.0 || pos[ 1 ] != 0.0
    {
      parts.push( format!( "translate({},{})", pos[ 0 ], pos[ 1 ] ) );
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
    if t.scale[ 0 ] != 1.0 || t.scale[ 1 ] != 1.0
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
    .map( | v | v.to_string() )
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

  fn clip_attr( clip : &Option< ResourceId< asset::ClipMask > > ) -> String
  {
    match clip
    {
      Some( id ) => format!( " clip-path=\"url(#clip_{})\"", id.inner() ),
      None => String::new(),
    }
  }

  fn tint_filter_attr( &mut self, tint : &[ f32; 4 ] ) -> String
  {
    Self::tint_filter_attr_split( tint, &mut self.content, &mut self.filter_counter )
  }

  fn tint_filter_attr_split( tint : &[ f32; 4 ], content : &mut SvgContentManager, counter : &mut u32 ) -> String
  {
    let is_white =
      ( tint[ 0 ] - 1.0 ).abs() < f32::EPSILON
      && ( tint[ 1 ] - 1.0 ).abs() < f32::EPSILON
      && ( tint[ 2 ] - 1.0 ).abs() < f32::EPSILON
      && ( tint[ 3 ] - 1.0 ).abs() < f32::EPSILON;

    if is_white
    {
      return String::new();
    }

    let id = *counter;
    *counter += 1;

    let filter_def = format!
    (
      "<filter id=\"tint_{}\"><feColorMatrix type=\"matrix\" values=\"{} 0 0 0 0 0 {} 0 0 0 0 0 {} 0 0 0 0 0 {} 0\"/></filter>",
      id, tint[ 0 ], tint[ 1 ], tint[ 2 ], tint[ 3 ]
    );
    content.push_def( &filter_def );

    format!( " filter=\"url(#tint_{})\"", id )
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
    {
      if let Some( img ) = resources.image( img_id )
      {
        if img.width > 0 && img.height > 0
        {
          let pat_id = format!( "mesh_tex_{}", img_id.inner() );
          let pat_def = format!
          (
            "<pattern id=\"{}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\"><use href=\"#img_{}\" width=\"{}\" height=\"{}\"/></pattern>",
            pat_id, img.width, img.height, img_id.inner(), img.width, img.height
          );
          content.push_def( &pat_def );
          return format!( "url(#{})", pat_id );
        }
      }
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
        format!
        (
          "A {rx} {ry} {rotation} {} {} {x} {y}",
          if *large_arc { 1 } else { 0 },
          if *sweep { 1 } else { 0 }
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
    let transform = self.transform_to_svg( &style.transform );
    let clip = Self::clip_attr( &style.clip );
    let dash = Self::dash_to_svg( &style.stroke_dash );
    let blend = Self::blend_to_svg( &style.blend );

    let path = format!
    (
      "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}{}{} style=\"mix-blend-mode:{}\"/>",
      self.path_data.trim(),
      fill,
      stroke,
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
    let ( anchor, baseline ) = Self::anchor_to_svg( &style.anchor );
    let clip = Self::clip_attr( &style.clip );

    let mut t = Transform::default();
    t.position = style.position;
    let transform = self.transform_to_svg( &t );

    if let Some( path_id ) = style.along_path
    {
      let text = format!
      (
        "<text font-size=\"{}\" fill=\"{}\" text-anchor=\"{}\" dominant-baseline=\"{}\"{}{}>\n          <textPath href=\"#path_{}\">{}</textPath></text>",
        style.size, fill, anchor, baseline, transform, clip,
        path_id.inner(), self.text_buf,
      );
      self.content.push_body( &text );
    }
    else
    {
      let text = format!
      (
        "<text font-size=\"{}\" fill=\"{}\" text-anchor=\"{}\" dominant-baseline=\"{}\"{}{}>\n          {}</text>",
        style.size, fill, anchor, baseline, transform, clip,
        self.text_buf,
      );
      self.content.push_body( &text );
    }
    self.text_buf.clear();
  }

  // ---- Asset loaders ----

  fn load_gradients( &mut self, gradients : &[ GradientAsset ] )
  {
    for grad in gradients
    {
      let stops : String = grad
      .stops
      .iter()
      .map( | s |
      {
        format!
        (
          "<stop offset=\"{}\" stop-color=\"{}\"/>",
          s.offset,
          Self::color_to_svg( &s.color )
        )
      })
      .collect();

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
      let _ = write!( grad_def, "</{}>", grad_type );
      self.content.push_def( &grad_def );
    }
  }

  fn load_patterns( &mut self, patterns : &[ PatternAsset ] )
  {
    for pat in patterns
    {
      let pat_def = format!
      (
        "<pattern id=\"pat_{}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\"><use href=\"#img_{}\"/></pattern>",
        pat.id.inner(), pat.width, pat.height, pat.content.inner(),
      );
      self.content.push_def( &pat_def );
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
      self.content.push_def( &clip_def );
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
      self.content.push_def( &path_def );
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
          let encoded = base64::prelude::BASE64_STANDARD.encode( bytes );
          let mime = match format
          {
            PixelFormat::Rgba8 | PixelFormat::Rgb8 => "image/png",
            _ => "image/png",
          };
          let img_def = format!
          (
            "<symbol id=\"img_{}\" viewBox=\"0 0 {} {}\"><image href=\"data:{};base64,{}\" width=\"{}\" height=\"{}\"/></symbol>",
            img.id.inner(),
            width, height,
            mime, encoded,
            width, height
          );
          self.content.push_def( &img_def );
          self.resources.store_image( img.id, SvgImage { width : *width, height : *height } );
        }
        ImageSource::Encoded( bytes ) =>
        {
          let encoded = base64::prelude::BASE64_STANDARD.encode( bytes );
          let img_def = format!( "<symbol id=\"img_{}\"><image href=\"data:image/png;base64,{}\"/></symbol>", img.id.inner(), encoded );
          self.content.push_def( &img_def );
          self.resources.store_image( img.id, SvgImage { width : 0, height : 0 } );
        }
        ImageSource::Path( path ) =>
        {
          let img_def = format!( "<symbol id=\"img_{}\"><image href=\"{}\"/></symbol>", img.id.inner(), path.display() );
          self.content.push_def( &img_def );
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
        let img_def = format!
        (
          "<symbol id=\"sprite_{}\" viewBox=\"{} {} {} {}\"><use href=\"#img_{}\" width=\"{}\" height=\"{}\"/></symbol>",
          sprite.id.inner(),
          sprite.region[ 0 ], sprite.region[ 1 ], sprite.region[ 2 ], sprite.region[ 3 ],
          sprite.sheet.inner(),
          sheet.width, sheet.height
        );
        self.content.push_def( &img_def );
      }
    }
  }

  fn load_geometries( &mut self, geometries : &[ GeometryAsset ] )
  {
    for geom in geometries
    {
      if let Source::Bytes( bytes ) = &geom.positions
      {
        let positions : &[ f32 ] = bytemuck::cast_slice( bytes );
        let indices = if let Some( Source::Bytes( ibytes ) ) = &geom.indices
        {
          match geom.data_type
          {
            DataType::U16 => Some( bytemuck::cast_slice::< _, u16 >( ibytes ).iter().map( | &i | i as u32 ).collect() ),
            DataType::U32 => Some( bytemuck::cast_slice::< _, u32 >( ibytes ).to_vec() ),
            _ => None,
          }
        }
        else { None };

        self.resources.store_geometry( geom.id, SvgGeometry { positions : positions.to_vec(), indices } );

        // Pre-generate all topologies for this geometry upfront
        for topology in [ Topology::TriangleList, Topology::TriangleStrip, Topology::LineList, Topology::LineStrip ]
        {
          self.generate_mesh_def( geom.id, topology );
        }
      }
    }
  }

  fn generate_mesh_def( &mut self, geom_id : ResourceId< asset::Geometry >, topology : Topology ) -> Option< String >
  {
    let id_u64 : u64 = geom_id.inner() as u64;
    let packed_key : u64 = ( id_u64 << 8 ) | ( topology as u8 as u64 );

    let geom = self.resources.geometry( geom_id )?;
    let def_id = format!( "mesh_{}_{:?}", geom_id.inner(), topology );
    let mut def_content = format!( "<symbol id=\"{}\" overflow=\"visible\">", def_id );

    match topology
    {
      Topology::TriangleList =>
      {
        let idx = geom.indices.as_ref().map( | v | v.as_slice() );
        let count = idx.map_or( geom.positions.len() / 2, | v | v.len() );
        for i in ( 0..count ).step_by( 3 )
        {
          let mut pts = String::new();
          for j in 0..3
          {
            let v_idx = idx.map_or( i + j, | v | v[ i + j ] as usize );
            let x = geom.positions[ v_idx * 2 ];
            let y = geom.positions[ v_idx * 2 + 1 ];
            let _ = write!( pts, "{},{} ", x, y );
          }
          let _ = write!( def_content, "<polygon points=\"{}\"/>", pts.trim() );
        }
      }
      Topology::TriangleStrip =>
      {
        let idx = geom.indices.as_ref().map( | v | v.as_slice() );
        let count = idx.map_or( geom.positions.len() / 2, | v | v.len() );
        for i in 0..( count - 2 )
        {
          let mut pts = String::new();
          for j in 0..3
          {
            let v_idx = idx.map_or( i + j, | v | v[ i + j ] as usize );
            let x = geom.positions[ v_idx * 2 ];
            let y = geom.positions[ v_idx * 2 + 1 ];
            let _ = write!( pts, "{},{} ", x, y );
          }
          let _ = write!( def_content, "<polygon points=\"{}\"/>", pts.trim() );
        }
      }
      Topology::LineList | Topology::LineStrip =>
      {
        let mut pts = String::new();
        let idx = geom.indices.as_ref().map( | v | v.as_slice() );
        let count = idx.map_or( geom.positions.len() / 2, | v | v.len() );
        for i in 0..count
        {
          let v_idx = idx.map_or( i, | v | v[ i ] as usize );
          let x = geom.positions[ v_idx * 2 ];
          let y = geom.positions[ v_idx * 2 + 1 ];
          let _ = write!( pts, "{},{} ", x, y );

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
    self.content.push_def( &def_content );
    self.resources.mesh_defs.insert( packed_key, def_id.clone() );

    Some( def_id )
  }

  fn cmd_clear( &mut self, c : &Clear )
  {
    let color = Self::color_to_svg( &c.color );
    let rect = format!( "<rect width=\"100%\" height=\"100%\" fill=\"{color}\"/>" );
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
    let _ = write!( self.path_data, "A {} {} {} {} {} {} {} ", a.rx, a.ry, a.rotation, a.large_arc as u8, a.sweep as u8, a.x, a.y );
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
    let packed_key : u64 = ( m.geometry.inner() as u64 ) << 8 | ( m.topology as u8 as u64 );
    let def_id = match self.resources.mesh_defs.get( &packed_key )
    {
      Some( id ) => id.clone(),
      None => return,
    };

    let transform = self.transform_to_svg( &m.transform );
    let fill = self.texture_or_fill( m.texture, &m.fill );
    let clip = Self::clip_attr( &m.clip );
    let blend = Self::blend_to_svg( &m.blend );

    let mesh = format!
    (
      "<use href=\"#{}\" fill=\"{}\"{}{} style=\"mix-blend-mode:{}\"/>",
      def_id, fill, transform, clip, blend
    );
    self.content.push_body( &mesh );
  }

  fn cmd_sprite( &mut self, s : &Sprite )
  {
    let transform = self.transform_to_svg( &s.transform );
    let clip = Self::clip_attr( &s.clip );
    let blend = Self::blend_to_svg( &s.blend );
    let tint = self.tint_filter_attr( &s.tint );
    let sprite = format!( "<use href=\"#sprite_{}\"{}{}{} style=\"mix-blend-mode:{}\"/>", s.sprite.inner(), transform, clip, tint, blend );
    self.content.push_body( &sprite );
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
    {
      if let Some( SvgBatch::Sprite { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
      {
        instances.push( *si );
      }
    }
  }

  fn cmd_add_mesh_instance( &mut self, mi : &AddMeshInstance )
  {
    if let Some( batch_id ) = self.recording_batch
    {
      if let Some( SvgBatch::Mesh { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
      {
        instances.push( *mi );
      }
    }
  }

  fn cmd_set_sprite_instance( &mut self, si : &SetSpriteInstance )
  {
    if let Some( batch_id ) = self.recording_batch
    {
      if let Some( SvgBatch::Sprite { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
      {
        if ( si.index as usize ) < instances.len()
        {
          instances[ si.index as usize ] = AddSpriteInstance { transform : si.transform, sprite : si.sprite, tint : si.tint };
        }
      }
    }
  }

  fn cmd_set_mesh_instance( &mut self, mi : &SetMeshInstance )
  {
    if let Some( batch_id ) = self.recording_batch
    {
      if let Some( SvgBatch::Mesh { instances, .. } ) = self.resources.batches.get_mut( &batch_id )
      {
        if ( mi.index as usize ) < instances.len()
        {
          instances[ mi.index as usize ] = AddMeshInstance { transform : mi.transform };
        }
      }
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
          if ( ri.index as usize ) < instances.len() { instances.remove( ri.index as usize ); }
        }
        Some( SvgBatch::Mesh { instances, .. } ) =>
        {
          if ( ri.index as usize ) < instances.len() { instances.remove( ri.index as usize ); }
        }
        None => {}
      }
    }
  }

  fn cmd_set_sprite_batch_params( &mut self, sp : &SetSpriteBatchParams )
  {
    if let Some( batch_id ) = self.recording_batch
    {
      if let Some( SvgBatch::Sprite { params, .. } ) = self.resources.batches.get_mut( &batch_id )
      {
        *params = sp.params;
      }
    }
  }

  fn cmd_set_mesh_batch_params( &mut self, mp : &SetMeshBatchParams )
  {
    if let Some( batch_id ) = self.recording_batch
    {
      if let Some( SvgBatch::Mesh { params, .. } ) = self.resources.batches.get_mut( &batch_id )
      {
        *params = mp.params;
      }
    }
  }

  fn cmd_unbind_batch( &mut self )
  {
    self.recording_batch = None;
  }

  fn cmd_draw_batch( &mut self, db : &DrawBatch )
  {
    let width = self.config.width;
    let height = self.config.height;
    let offset = self.viewport_offset;
    let zoom = self.viewport_scale;

    let resources = &self.resources;
    let content = &mut self.content;
    let filter_counter = &mut self.filter_counter;

    match resources.batch( db.batch )
    {
      Some( SvgBatch::Sprite { instances, params } ) =>
      {
        let parent_transform = Self::transform_to_svg_static( &params.transform, width, height, offset, zoom );
        let clip = Self::clip_attr( &params.clip );
        let blend = Self::blend_to_svg( &params.blend );

        content.push_body( &format!( "<g{}{}>", parent_transform, clip ) );
        for inst in instances
        {
          let inst_transform = Self::transform_to_svg_local( &inst.transform );
          let tint = Self::tint_filter_attr_split( &inst.tint, content, filter_counter );
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
        let packed_key : u64 = ( params.geometry.inner() as u64 ) << 8 | ( params.topology as u8 as u64 );
        if let Some( def_id ) = resources.mesh_defs.get( &packed_key )
        {
          let parent_transform = Self::transform_to_svg_static( &params.transform, width, height, offset, zoom );
          let clip = Self::clip_attr( &params.clip );
          let blend = Self::blend_to_svg( &params.blend );
          let fill = Self::texture_or_fill_split( params.texture, &params.fill, resources, content );

          content.push_body( &format!( "<g{}{}>", parent_transform, clip ) );
          for inst in instances
          {
            let inst_transform = Self::transform_to_svg_local( &inst.transform );
            let mesh = format!
            (
              "<use href=\"#{}\" fill=\"{}\"{} style=\"mix-blend-mode:{}\"/>",
              def_id, fill, inst_transform, blend
            );
            content.push_body( &mesh );
          }
          content.push_body( "</g>" );
        }
      }
      None => {}
    }
  }

  fn cmd_delete_batch( &mut self, db : &DeleteBatch )
  {
    self.resources.batches.remove( &db.batch );
  }

  fn cmd_begin_group( &mut self, bg : &BeginGroup )
  {
    let transform = self.transform_to_svg( &bg.transform );
    let clip = Self::clip_attr( &bg.clip );

    let effect_attr = match &bg.effect
    {
      Some( Effect::Opacity( a ) ) => format!( " opacity=\"{}\"", a ),
      Some( Effect::Blur { radius } ) =>
      {
        let fid = self.filter_counter;
        self.filter_counter += 1;
        let def = format!( "<filter id=\"fx_{}\"><feGaussianBlur stdDeviation=\"{}\"/></filter>", fid, radius );
        self.content.push_def( &def );
        format!( " filter=\"url(#fx_{})\"", fid )
      }
      Some( Effect::DropShadow { dx, dy, blur, color } ) =>
      {
        let fid = self.filter_counter;
        self.filter_counter += 1;
        let c = Self::color_to_svg( color );
        // Negate dy: Y-up shadow direction → SVG Y-down
        let def = format!
        (
          "<filter id=\"fx_{}\"><feDropShadow dx=\"{}\" dy=\"{}\" stdDeviation=\"{}\" flood-color=\"{}\"/></filter>",
          fid, dx, -dy, blur, c
        );
        self.content.push_def( &def );
        format!( " filter=\"url(#fx_{})\"", fid )
      }
      Some( Effect::ColorMatrix( values ) ) =>
      {
        let fid = self.filter_counter;
        self.filter_counter += 1;
        let vals : String = values.iter().map( | v | v.to_string() ).collect::< Vec< _ > >().join( " " );
        let def = format!( "<filter id=\"fx_{}\"><feColorMatrix type=\"matrix\" values=\"{}\"/></filter>", fid, vals );
        self.content.push_def( &def );
        format!( " filter=\"url(#fx_{})\"", fid )
      }
      None => String::new(),
    };

    let group = format!( "<g{}{}{}>", transform, clip, effect_attr );
    self.content.push_body( &group );
    self.group_depth += 1;
  }

  fn cmd_end_group( &mut self )
  {
    self.content.push_body( "</g>" );
    self.group_depth = self.group_depth.saturating_sub( 1 );
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
    self.content.clear_body();
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
        RenderCommand::Sprite( s ) => self.cmd_sprite( s ),
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
        RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( db ),
        RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( db ),
        RenderCommand::BeginGroup( bg ) => self.cmd_begin_group( bg ),
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
pub struct SvgContentManager
{
  buffer : String,
  defs_start : usize,
  defs_end : usize,
  body_start : usize,
  body_end : usize,
}

impl SvgContentManager
{
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
    buffer.push_str( "<defs>" );
    buffer.push_str( "</defs>\n" );
    let defs_end = buffer.len();

    let body_start = buffer.len();
    buffer.push_str( "<!--framebegin-->" );
    buffer.push_str( "<!--frameend-->\n" );
    let body_end = buffer.len();

    buffer.push_str( "</svg>\n" );

    Self
    {
      buffer,
      defs_start,
      defs_end,
      body_start,
      body_end,
    }
  }

  /// Updates the SVG header attributes dynamically like changing width/height bounds
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
      self.defs_start = ( self.defs_start as isize + diff ) as usize;
      self.defs_end = ( self.defs_end as isize + diff ) as usize;
      self.body_start = ( self.body_start as isize + diff ) as usize;
      self.body_end = ( self.body_end as isize + diff ) as usize;
    }
  }

  /// Clears the `<defs>` content scope entirely
  pub fn clear_defs( &mut self )
  {
    let inner_start = self.defs_start + "<defs>".len();
    let inner_end = self.defs_end - "</defs>\n".len();

    self.buffer.replace_range( inner_start..inner_end, "" );
    let removed = inner_end - inner_start;

    self.defs_end -= removed;
    self.body_start -= removed;
    self.body_end -= removed;
  }

  /// Inlines element into the definitions section
  pub fn push_def( &mut self, def : &str )
  {
    let insert_at = self.defs_end - "</defs>\n".len();
    self.buffer.insert_str( insert_at, def );

    let added = def.len();
    self.defs_end += added;
    self.body_start += added;
    self.body_end += added;
  }

  /// Clears only the dynamic render paths payload
  pub fn clear_body( &mut self )
  {
    let inner_start = self.body_start + "<!--framebegin-->".len();
    let inner_end = self.body_end - "<!--frameend-->\n".len();

    self.buffer.replace_range( inner_start..inner_end, "" );
    let removed = inner_end - inner_start;

    self.body_end -= removed;
  }

  /// Pushes SVG command sequence nodes inside the frame block
  pub fn push_body( &mut self, content : &str )
  {
    let insert_at = self.body_end - "<!--frameend-->\n".len();
    self.buffer.insert_str( insert_at, content );
    self.body_end += content.len();
  }

  /// Reference handle access to underlying payload SVG
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
    // Extract between <!--framebegin--> and <!--frameend-->
    let start = full.find( "<!--framebegin-->" ).unwrap() + "<!--framebegin-->".len();
    let end = full.find( "<!--frameend-->" ).unwrap();
    full[ start..end ].to_string()
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
    assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {}", b );
    assert!( b.contains( "width=\"100%\"" ) );
  }

  // -- transform Y-up --

  #[ test ]
  fn transform_y_up_bottom_left_origin()
  {
    // Position (0,0) in Y-up should map to SVG (0, height=600)
    let s = SvgBackend::transform_to_svg_static(
      &Transform { position : [ 0.0, 0.0 ], ..Default::default() },
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    assert!( s.contains( "translate(0,600)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_y_up_top_right()
  {
    // Position (800,600) should map to SVG (800, 0)
    let s = SvgBackend::transform_to_svg_static(
      &Transform { position : [ 800.0, 600.0 ], ..Default::default() },
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    assert!( s.contains( "translate(800,0)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_y_up_center()
  {
    // Position (400,300) should map to SVG (400, 300)
    let s = SvgBackend::transform_to_svg_static(
      &Transform { position : [ 400.0, 300.0 ], ..Default::default() },
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    assert!( s.contains( "translate(400,300)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_rotation_negated()
  {
    let angle = core::f32::consts::FRAC_PI_4; // 45° CCW in Y-up
    let s = SvgBackend::transform_to_svg_static(
      &Transform { rotation : angle, ..Default::default() },
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    // Should emit negative degrees in SVG
    assert!( s.contains( "rotate(-45" ), "got: {}", s );
  }

  #[ test ]
  fn transform_scale_y_negated()
  {
    let s = SvgBackend::transform_to_svg_static(
      &Transform { scale : [ 2.0, 3.0 ], ..Default::default() },
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    // scale Y should be negated: 3.0 → -3.0
    assert!( s.contains( "scale(2,-3)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_identity_scale_emits_y_flip()
  {
    // Default scale (1,1) should still emit scale(1,-1) for Y-flip
    let s = SvgBackend::transform_to_svg_static(
      &Transform::default(),
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    assert!( s.contains( "scale(1,-1)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_zoom()
  {
    let s = SvgBackend::transform_to_svg_static(
      &Transform::default(),
      800, 600, [ 0.0, 0.0 ], 2.0,
    );
    assert!( s.contains( "scale(2)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_no_zoom_when_1()
  {
    let s = SvgBackend::transform_to_svg_static(
      &Transform::default(),
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    // Should not contain scale(1) for zoom — only scale(1,-1) for Y-flip
    assert!( !s.contains( "scale(1) " ), "got: {}", s );
  }

  #[ test ]
  fn transform_viewport_offset()
  {
    // Offset (10, 20): position becomes (10, 20), then Y-flip → (10, 600-20=580)
    let s = SvgBackend::transform_to_svg_static(
      &Transform::default(),
      800, 600, [ 10.0, 20.0 ], 1.0,
    );
    assert!( s.contains( "translate(10,580)" ), "got: {}", s );
  }

  #[ test ]
  fn transform_skew_negated()
  {
    let angle = core::f32::consts::FRAC_PI_6; // 30°
    let s = SvgBackend::transform_to_svg_static(
      &Transform { skew : [ angle, 0.0 ], ..Default::default() },
      800, 600, [ 0.0, 0.0 ], 1.0,
    );
    assert!( s.contains( "skewX(-30" ), "got: {}", s );
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
    assert!( s.contains( "translate(10,20)" ), "got: {}", s );
    // Rotation is raw (positive), not negated
    let deg = 0.5_f32.to_degrees();
    assert!( s.contains( &format!( "rotate({})", deg ) ), "got: {}", s );
    // Scale is raw, no Y negation
    assert!( s.contains( "scale(2,3)" ), "got: {}", s );
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
    assert!( b.contains( "<path" ), "body: {}", b );
    assert!( b.contains( "M 10 20" ), "body: {}", b );
    assert!( b.contains( "L 100 200" ), "body: {}", b );
    assert!( b.contains( "Z" ), "body: {}", b );
    assert!( b.contains( "fill=\"rgb(0,0,255)\"" ), "body: {}", b );
    assert!( b.contains( "stroke-linecap=\"round\"" ), "body: {}", b );
    assert!( b.contains( "stroke-linejoin=\"round\"" ), "body: {}", b );
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
        source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 64, height : 32, format : PixelFormat::Rgba8 },
        filter : SamplerFilter::Linear,
      }],
      ..empty_assets()
    };
    svg.load_assets( &assets ).unwrap();

    let d = defs( &svg );
    // Should use "0 0 w h" viewBox, not center-origin
    assert!( d.contains( "viewBox=\"0 0 64 32\"" ), "defs: {}", d );
    // Should not have negative offsets
    assert!( !d.contains( "x=\"-" ), "defs: {}", d );
    assert!( !d.contains( "y=\"-" ), "defs: {}", d );
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
    assert!( !b.contains( "filter=" ), "white tint should not create filter, body: {}", b );
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
    assert!( b.contains( "filter=\"url(#tint_0)\"" ), "body: {}", b );
    assert!( d.contains( "<filter id=\"tint_0\">" ), "defs: {}", d );
    assert!( d.contains( "feColorMatrix" ), "defs: {}", d );
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
    assert!( b.contains( "<g" ), "body: {}", b );
    assert!( b.contains( "</g>" ), "body: {}", b );
    // Should have two sprite instances with local transforms
    assert_eq!( b.matches( "#sprite_0" ).count(), 2, "body: {}", b );
    // Local transforms should use raw positions (no Y-flip)
    assert!( b.contains( "translate(10,20)" ), "body: {}", b );
    assert!( b.contains( "translate(50,60)" ), "body: {}", b );
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
    assert!( b.contains( "<g" ), "body: {}", b );
    assert!( b.contains( "fill=\"rgb(0,255,0)\"" ), "body: {}", b );
    assert!( b.contains( "translate(5,10)" ), "body: {}", b );
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
    assert_eq!( b.matches( "#sprite_0" ).count(), 1, "body: {}", b );
    assert!( b.contains( "translate(3,4)" ), "body: {}", b );
    assert!( !b.contains( "translate(1,2)" ), "body: {}", b );
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
    assert!( !b.contains( "<g" ), "body: {}", b );
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
    assert!( b.contains( "filter=\"url(#fx_0)\"" ), "body: {}", b );
    assert!( d.contains( "feGaussianBlur" ), "defs: {}", d );
    assert!( d.contains( "stdDeviation=\"5\"" ), "defs: {}", d );
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
    assert!( d.contains( "feDropShadow" ), "defs: {}", d );
    assert!( d.contains( "dx=\"2\"" ), "defs: {}", d );
    // dy should be negated: 3.0 → -3.0
    assert!( d.contains( "dy=\"-3\"" ), "defs: {}", d );
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
    assert!( d.contains( "feColorMatrix" ), "defs: {}", d );
    assert!( d.contains( "type=\"matrix\"" ), "defs: {}", d );
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
    assert!( b.contains( "opacity=\"0.5\"" ), "body: {}", b );
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
    assert!( d.contains( "<polygon" ), "defs: {}", d );
    assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {}", b );
  }

  // -- resize --

  #[ test ]
  fn resize_updates_viewbox()
  {
    let mut svg = svg800x600();
    svg.resize( 1024, 768 );
    let full = render( &svg );
    assert!( full.contains( "width=\"1024\"" ), "full: {}", full );
    assert!( full.contains( "height=\"768\"" ), "full: {}", full );
    assert!( full.contains( "viewBox=\"0 0 1024 768\"" ), "full: {}", full );
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
    assert!( b.contains( "mix-blend-mode:multiply" ), "body: {}", b );
  }

  // -- content manager --

  #[ test ]
  fn content_manager_push_clear_cycle()
  {
    let mut cm = SvgContentManager::new( 100, 100, "" );
    cm.push_def( "<test-def/>" );
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
}
