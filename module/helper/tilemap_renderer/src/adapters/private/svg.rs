//! SVG backend adapter implementation.
//!
//! This adapter renders scenes to SVG format, providing scalable vector output
//! suitable for web display, printing, and vector graphics workflows.

use base64::Engine;
use rustc_hash::FxHashMap;

use crate::ports::
{
  RenderContext,
  Renderer,
  RendererCapabilities,
  RenderError,
  PrimitiveRenderer
};
use crate::scene::Scene;
use crate::commands::
{
  CurveCommand,
  LineCap,
  LineCommand,
  LineJoin,
  ParticleEmitterCommand,
  RenderCommand,
  TextAnchor,
  TextCommand,
  TilemapCommand,
  Transform2D
};

/// SVG renderer backend adapter.
///
/// Renders scenes to SVG (Scalable Vector Graphics) format with comprehensive
/// support for lines, curves, text, and basic shapes.
#[ derive( Debug, Clone ) ]
pub struct SvgRenderer
{
  /// SVG content buffer.
  svg_content : String,
  /// Current rendering state.
  initialized : bool,
  /// Whether a frame is currently active.
  frame_active : bool,
  /// Rendering context for the current frame.
  context : Option< RenderContext >,
  images : FxHashMap< String, ( u32, u32 ) >
}

impl SvgRenderer
{
  /// Creates a new SVG renderer instance.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    return Self
    {
      svg_content : String::new(),
      initialized : false,
      frame_active : false,
      context : None,
      images : FxHashMap::default()
    };
  }

  /// Converts a color array to SVG color string.
  #[ inline ]
  fn color_to_svg( color : &[ f32; 4 ] ) -> String
  {
    let r = ( color[ 0 ] * 255.0 ) as u8;
    let g = ( color[ 1 ] * 255.0 ) as u8;
    let b = ( color[ 2 ] * 255.0 ) as u8;
    let a = color[ 3 ];

    if ( a - 1.0 ).abs() < f32::EPSILON
    {
      return format!( "rgb({r},{g},{b})" );
    }
    else
    {
      return format!( "rgba({r},{g},{b},{a})" );
    }
  }

  /// Converts line cap style to SVG stroke-linecap.
  #[ inline ]
  fn line_cap_to_svg( cap : LineCap ) -> &'static str
  {
    match cap
    {
      LineCap::Butt => return "butt",
      LineCap::Round => return "round",
      LineCap::Square => return "square",
    }
  }

  /// Converts line join style to SVG stroke-linejoin.
  #[ inline ]
  fn line_join_to_svg( join : LineJoin ) -> &'static str
  {
    match join
    {
      LineJoin::Miter => return "miter",
      LineJoin::Round => return "round",
      LineJoin::Bevel => return "bevel",
    }
  }

  /// Resolves font family from font family ID.
  /// For now, this is a simple mapping. In a full implementation,
  /// this would lookup from a font registry.
  #[ inline ]
  fn resolve_font_family( family_id : u32 ) -> &'static str
  {
    match family_id
    {
      0 => return "Arial",
      1 => return "Times New Roman",
      2 => return "Courier New",
      3 => return "Helvetica",
      4 => return "Georgia",
      _ => return "sans-serif", // Fallback
    }
  }

  #[ inline ]
  pub fn render_geometry( &mut self, points : &[ f32 ], mut transform : Transform2D, style : GeometryStyle )
  {
    if points.len() < 3
    {
      // Not a valid polygon, but we can just ignore it.
      return;
    }

    transform.position[ 1 ] = -transform.position[ 1 ];
    let ctx = self.context.unwrap();
    transform.position[ 0 ] += ctx.viewport_offset.x + ctx.width as f32 / 2.0;
    transform.position[ 1 ] += -ctx.viewport_offset.y + ctx.height as f32 / 2.0;
    transform.rotation = transform.rotation.to_degrees();
    let zoom = self.context.unwrap().viewport_scale;
    // Convert the vector of points into an SVG-compatible string
    let mut points_str = String::with_capacity( points.len() * 2 );
    for c in points.chunks_exact( 2 )
    {
      points_str.push_str( &format!( "{},{} ", c[ 0 ], c[ 1 ] ) );
    }
    points_str.pop();

    let fill = style.fill_color.map_or( "none".to_string(), | c | Self::color_to_svg( &c ) );

    let stroke = style.stroke_color.map_or( "none".to_string(), | c | Self::color_to_svg( &c ) );

    self.push_frame_content
    (
      &format!
      (
        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}" transform="scale=({}) translate({}, {}) rotate({}) scale({}, {})"/>"#,
        points_str, fill, stroke, style.stroke_width, zoom,
        transform.position[ 0 ], transform.position[ 1 ],
        transform.rotation,
        transform.scale[ 0 ], transform.scale[ 1 ]
      )
    );
  }

  pub fn load_image( &mut self, bytes : &[ u8 ], width : u32, height : u32, format : ImageFormat, id : &str )
  {
    let img = base64::prelude::BASE64_STANDARD.encode( bytes );
    let def = format!
    (
      r#"<defs><symbol id="{id}" viewBox="-{} -{} {width} {height}">
      <image href="data:image/{};base64,{img}" x="-{}" y="-{}" width="{width}" height="{height}"/>
      </sybmbol></defs>"#,
      width / 2, height / 2, format.as_ref(), width / 2, height / 2
    );
    let begin_index = self.svg_content.find( "<!--framebegin-->" ).expect( "Renderer is not initialized" );
    self.svg_content.insert_str( begin_index, &def );
    _ = self.images.insert( id.to_string(), ( width, height ) );
  }

  pub fn render_image( &mut self, id : &str, mut transform : Transform2D )
  {
    transform.position[ 1 ] = -transform.position[ 1 ];
    let ctx = self.context.unwrap();
    transform.position[ 0 ] += ctx.viewport_offset.x + ctx.width as f32 / 2.0;
    transform.position[ 1 ] += -ctx.viewport_offset.y + ctx.height as f32 / 2.0;
    transform.rotation = transform.rotation.to_degrees();
    let zoom = self.context.unwrap().viewport_scale;
    let ( width, height ) = self.images[ id ];
    let s = format!
    (
      r#"<g transform="scale({}) translate({}, {}) rotate({}) scale({}, {})"><use href="\#{id} width="{width}" height="{height}"/></g>"#,
      zoom,
      transform.position[ 0 ],
      transform.position[ 1 ],
      transform.rotation,
      transform.scale[ 0 ],
      transform.scale[ 1 ],
    );

    self.push_frame_content( &s );
  }

  fn push_frame_content( &mut self, s : &str )
  {
    let idx = self.svg_content.find( "<!--framebegin-->" ).expect( "Renderer is not initialized" );
    self.svg_content.insert_str(idx + "<!--framebegin-->".len(), s );
  }

  pub fn clear( &mut self )
  {
    let begin_index = self.svg_content.find( "<!--framebegin-->" ).expect( "Renderer is not initialized" )
    + "<!--framebegin-->".len();
    let end_index = self.svg_content.find( "<!--frameend-->" ).expect( "Renderer is not initialized" );
    self.svg_content.replace_range( begin_index..end_index, "" );
  }
}

impl Default for SvgRenderer
{
  #[ inline ]
  fn default() -> Self
  {
    return Self::new();
  }
}

impl Renderer for SvgRenderer
{
  type Output = String;

  #[ inline ]
  fn capabilities( &self ) -> RendererCapabilities
  {
    let mut caps = RendererCapabilities::default();
    caps.backend_name = "SVG".to_string();
    caps.backend_version = "1.0".to_string();
    caps.max_texture_size = 0; // SVG doesn't have texture size limits
    caps.supports_transparency = true;
    caps.supports_antialiasing = true; // SVG browsers handle this
    caps.supports_custom_fonts = true;
    caps.supports_particles = false; // Not implemented for SVG
    caps.supports_realtime = false; // SVG is static
    caps.max_scene_complexity = 10000; // Large scenes supported
    return caps;
  }

  /// Initializes the SVG renderer with the given context.
  ///
  /// # Errors
  /// Returns `InitializationFailed` if the renderer cannot be initialized.
  #[ inline ]
  fn initialize( &mut self, context : &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if self.initialized
    {
      return Err( RenderError::InitializationFailed( "SVG renderer already initialized".to_string() ) );
    }

    self.context = Some( context.clone() );
    self.initialized = true;

    // self.context = Some( context.clone() );
    self.frame_active = true;
    self.svg_content.clear();

    // Start SVG document
    self.svg_content.push_str
    (
      &format!
      (
        r#"<?xml version="1.0" encoding="UTF-8"?> <svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
        context.width, context.height, context.width, context.height
      )
    );

    // Add background if needed
    if context.clear_background
    {
      let bg_color = Self::color_to_svg( &context.background_color );
      self.svg_content.push_str
      (
        &format!
        (
          r#"<rect width="100%" height="100%" fill="{bg_color}"/>"#
        )
      );
    }

    self.svg_content.push_str( "<!--framebegin--><!--frameend-->" );
    self.svg_content.push_str( "</svg>\n" );
    // self.svg_content.push_str( "" );

    return Ok( () );
  }

  /// Begins a new rendering frame.
  ///
  /// # Errors
  /// Returns `InvalidContext` if the context is invalid or `RenderFailed` if frame setup fails.
  #[ inline ]
  fn begin_frame( &mut self, context : &RenderContext ) -> core::result::Result< (), RenderError >
  {
    // if !self.initialized
    // {
    //   return Err( RenderError::InvalidContext( "SVG renderer not initialized".to_string() ) );
    // }

    // if self.frame_active
    // {
    //   return Err( RenderError::RenderFailed( "Frame already active".to_string() ) );
    // }

    // self.context = Some( context.clone() );
    // self.frame_active = true;
    // self.svg_content.clear();

    // // Start SVG document
    // self.svg_content.push_str
    // (
    //   &format!
    //   (
    //     r#"<?xml version="1.0" encoding="UTF-8"?> <svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
    //     context.width, context.height, context.width, context.height
    //   )
    // );

    // // Add background if needed
    // if context.clear_background
    // {
    //   let bg_color = Self::color_to_svg( &context.background_color );
    //   self.svg_content.push_str
    //   (
    //     &format!
    //     (
    //       r#"<rect width="100%" height="100%" fill="{bg_color}"/>"#
    //     )
    //   );
    // }

    return Ok( () );
  }

  /// Renders a complete scene to the output.
  ///
  /// # Errors
  /// Returns various errors including `UnsupportedCommand`, `ComplexityLimitExceeded`, or `RenderFailed`.
  #[ inline ]
  fn render_scene( &mut self, scene : &Scene ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }

    self.validate_scene( scene )?;

    for command in scene.commands()
    {
      match command
      {
        RenderCommand::Line( line_cmd ) => self.render_line( line_cmd )?,
        RenderCommand::Curve( curve_cmd ) => self.render_curve( curve_cmd )?,
        RenderCommand::Text( text_cmd ) => self.render_text( text_cmd )?,
        RenderCommand::Tilemap( _ ) => return Err( RenderError::UnsupportedCommand( "Tilemap".to_string() ) ),
        RenderCommand::ParticleEmitter( _ ) => return Err( RenderError::UnsupportedCommand( "ParticleEmitter".to_string() ) ),
        RenderCommand::Geometry2DCommand( _ ) => return Err( RenderError::UnsupportedCommand( "Geometry2DCommand".into() ) ),
        RenderCommand::SpriteCommand( _ ) => return Err( RenderError::UnsupportedCommand( "SpriteCommand".into() ) ),
      }
    }

    return Ok( () );
  }

  /// Ends the current rendering frame and finalizes the output.
  ///
  /// # Errors
  /// Returns `RenderFailed` if frame finalization fails.
  #[ inline ]
  fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }

    // Close SVG document
    // self.svg_content.push_str( "</svg>\n" );

    // self.frame_active = false;
    return Ok( () );
  }

  /// Retrieves the rendered output.
  ///
  /// # Errors
  /// Returns `OutputError` if the output cannot be retrieved.
  #[ inline ]
  fn output( &self ) -> core::result::Result< Self::Output, RenderError >
  {
    if self.frame_active
    {
      return Err( RenderError::OutputError( "Frame still active".to_string() ) );
    }

    if !self.initialized
    {
      return Err( RenderError::OutputError( "Renderer not initialized".to_string() ) );
    }

    return Ok( self.svg_content.clone() );
  }

  /// Performs cleanup and releases resources.
  ///
  /// # Errors
  /// Returns `RenderFailed` if cleanup operations fail.
  #[ inline ]
  fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
  {
    self.initialized = false;
    self.frame_active = false;
    self.svg_content.clear();
    self.context = None;
    return Ok( () );
  }

  #[ inline ]
  fn supports_tilemaps( &self ) -> bool
  {
    return false; // SVG adapter doesn't support tilemaps
  }

  #[ inline ]
  fn supports_particles( &self ) -> bool
  {
    return false; // SVG adapter doesn't support particle effects
  }
}

impl PrimitiveRenderer for SvgRenderer
{
  /// Renders a line command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if line rendering fails.
  #[ inline ]
  fn render_line( &mut self, command : &LineCommand ) -> core::result::Result< (), RenderError >
  {
    let stroke_color = Self::color_to_svg( &command.style.color );
    let line_cap = Self::line_cap_to_svg( command.style.cap_style );
    let line_join = Self::line_join_to_svg( command.style.join_style );

    self.push_frame_content( &format!(
      r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"/>"#,
      command.start.x, command.start.y,
      command.end.x, command.end.y,
      stroke_color, command.style.width,
      line_cap, line_join
    ) );

    return Ok( () );
  }

  /// Renders a curve command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if curve rendering fails.
  #[ inline ]
  fn render_curve( &mut self, command : &CurveCommand ) -> core::result::Result< (), RenderError >
  {
    let stroke_color = Self::color_to_svg( &command.style.color );
    let line_cap = Self::line_cap_to_svg( command.style.cap_style );
    let line_join = Self::line_join_to_svg( command.style.join_style );

    // SVG cubic bezier curve
    let path_data = format!(
      "M {} {} C {} {}, {} {}, {} {}",
      command.start.x, command.start.y,
      command.control1.x, command.control1.y,
      command.control2.x, command.control2.y,
      command.end.x, command.end.y
    );

    self.push_frame_content
    (
      &format!
      (
        r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"/>"#,
        path_data, stroke_color, command.style.width,
        line_cap, line_join
      )
    );

    return Ok( () );
  }

  /// Renders a text command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if text rendering fails.
  #[ inline ]
  fn render_text( &mut self, command: &TextCommand ) -> core::result::Result< (), RenderError >
  {
    let fill_color = Self::color_to_svg( &command.font_style.color );
    let text_content = command.text();

    // Convert text anchor to SVG text-anchor
    let text_anchor = match command.anchor
    {
      TextAnchor::TopLeft | TextAnchor::CenterLeft | TextAnchor::BottomLeft => "start",
      TextAnchor::TopCenter | TextAnchor::Center | TextAnchor::BottomCenter => "middle",
      TextAnchor::TopRight | TextAnchor::CenterRight | TextAnchor::BottomRight => "end",
    };

    // Convert vertical alignment to dominant-baseline
    let dominant_baseline = match command.anchor
    {
      TextAnchor::TopLeft | TextAnchor::TopCenter | TextAnchor::TopRight => "hanging",
      TextAnchor::CenterLeft | TextAnchor::Center | TextAnchor::CenterRight => "central",
      TextAnchor::BottomLeft | TextAnchor::BottomCenter | TextAnchor::BottomRight => "baseline",
    };

    self.push_frame_content( &format!(
      r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="{}" dominant-baseline="{}">{}</text>"#,
      command.position.x, command.position.y,
      Self::resolve_font_family( command.font_style.family_id ), command.font_style.size,
      fill_color, text_anchor, dominant_baseline,
      text_content
    ) );

    return Ok( () );
  }

  /// Renders a tilemap command.
  ///
  /// # Errors
  /// Returns `FeatureNotImplemented` if tilemap rendering is not supported.
  #[ inline ]
  fn render_tilemap( &mut self, _command: &TilemapCommand ) -> core::result::Result< (), RenderError >
  {
    return Err( RenderError::FeatureNotImplemented( "Tilemap rendering not supported in SVG backend".to_string() ) );
  }

  /// Renders a particle emitter command.
  ///
  /// # Errors
  /// Returns `FeatureNotImplemented` if particle rendering is not supported.
  #[ inline ]
  fn render_particle_emitter( &mut self, _command: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >
  {
    return Err( RenderError::FeatureNotImplemented( "Particle rendering not supported in SVG backend".to_string() ) );
  }
}

#[ derive( Debug, Clone, Copy ) ]
pub struct GeometryStyle
{
  pub fill_color : Option< [ f32; 4 ] >,
  pub stroke_color : Option< [ f32; 4 ] >,
  pub stroke_width : f32,
}

#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum ImageFormat
{
  Png,
  Jpeg,
}

impl AsRef< str > for ImageFormat
{
  fn as_ref( &self ) -> &str
  {
    match self
    {
      ImageFormat::Png => "png",
      ImageFormat::Jpeg => "jpeg",
    }
  }
}
