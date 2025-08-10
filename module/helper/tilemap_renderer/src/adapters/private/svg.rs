//! SVG backend adapter implementation.
//!
//! This adapter renders scenes to SVG format, providing scalable vector output
//! suitable for web display, printing, and vector graphics workflows.

#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::format_push_string ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]

use crate::ports::{ RenderContext, Renderer, RendererCapabilities, RenderError, PrimitiveRenderer };
use crate::scene::Scene;
use crate::commands::{ LineCap, LineJoin, RenderCommand, LineCommand, CurveCommand, TextCommand, TextAnchor, TilemapCommand, ParticleEmitterCommand };

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
}

impl SvgRenderer
{
  /// Creates a new SVG renderer instance.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      svg_content: String::new(),
      initialized: false,
      frame_active: false,
      context: None,
    }
  }
  
  /// Converts a color array to SVG color string.
  fn color_to_svg( color: &[ f32; 4 ] ) -> String
  {
    let r = ( color[ 0 ] * 255.0 ) as u8;
    let g = ( color[ 1 ] * 255.0 ) as u8;
    let b = ( color[ 2 ] * 255.0 ) as u8;
    let a = color[ 3 ];
    
    if ( a - 1.0 ).abs() < f32::EPSILON
    {
      format!( "rgb({r},{g},{b})" )
    }
    else
    {
      format!( "rgba({r},{g},{b},{a})" )
    }
  }
  
  /// Converts line cap style to SVG stroke-linecap.
  fn line_cap_to_svg( cap: LineCap ) -> &'static str
  {
    match cap
    {
      LineCap::Butt => "butt",
      LineCap::Round => "round", 
      LineCap::Square => "square",
    }
  }
  
  /// Converts line join style to SVG stroke-linejoin.
  fn line_join_to_svg( join: LineJoin ) -> &'static str
  {
    match join
    {
      LineJoin::Miter => "miter",
      LineJoin::Round => "round",
      LineJoin::Bevel => "bevel",
    }
  }
  
  /// Resolves font family from font family ID.
  /// For now, this is a simple mapping. In a full implementation,
  /// this would lookup from a font registry.
  fn resolve_font_family( family_id: u32 ) -> &'static str
  {
    match family_id
    {
      0 => "Arial",
      1 => "Times New Roman", 
      2 => "Courier New",
      3 => "Helvetica",
      4 => "Georgia",
      _ => "sans-serif", // Fallback
    }
  }
}

impl Default for SvgRenderer
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl Renderer for SvgRenderer
{
  type Output = String;
  
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
    caps
  }
  
  /// Initializes the SVG renderer with the given context.
  ///
  /// # Errors
  /// Returns `InitializationFailed` if the renderer cannot be initialized.
  fn initialize( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if self.initialized
    {
      return Err( RenderError::InitializationFailed( "SVG renderer already initialized".to_string() ) );
    }
    
    self.context = Some( context.clone() );
    self.initialized = true;
    Ok( () )
  }
  
  /// Begins a new rendering frame.
  ///
  /// # Errors
  /// Returns `InvalidContext` if the context is invalid or `RenderFailed` if frame setup fails.
  fn begin_frame( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if !self.initialized
    {
      return Err( RenderError::InvalidContext( "SVG renderer not initialized".to_string() ) );
    }
    
    if self.frame_active
    {
      return Err( RenderError::RenderFailed( "Frame already active".to_string() ) );
    }
    
    self.context = Some( context.clone() );
    self.frame_active = true;
    self.svg_content.clear();
    
    // Start SVG document
    self.svg_content.push_str( &format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">
"#,
      context.width, context.height, context.width, context.height
    ) );
    
    // Add background if needed
    if context.clear_background
    {
      let bg_color = Self::color_to_svg( &context.background_color );
      self.svg_content.push_str( &format!(
        r#"  <rect width="100%" height="100%" fill="{bg_color}"/>
"#
      ) );
    }
    
    Ok( () )
  }
  
  /// Renders a complete scene to the output.
  ///
  /// # Errors
  /// Returns various errors including `UnsupportedCommand`, `ComplexityLimitExceeded`, or `RenderFailed`.
  fn render_scene( &mut self, scene: &Scene ) -> core::result::Result< (), RenderError >
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
      }
    }
    
    Ok( () )
  }
  
  /// Ends the current rendering frame and finalizes the output.
  ///
  /// # Errors
  /// Returns `RenderFailed` if frame finalization fails.
  fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    // Close SVG document
    self.svg_content.push_str( "</svg>\n" );
    
    self.frame_active = false;
    Ok( () )
  }
  
  /// Retrieves the rendered output.
  ///
  /// # Errors
  /// Returns `OutputError` if the output cannot be retrieved.
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
    
    Ok( self.svg_content.clone() )
  }
  
  /// Performs cleanup and releases resources.
  ///
  /// # Errors
  /// Returns `RenderFailed` if cleanup operations fail.
  fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
  {
    self.initialized = false;
    self.frame_active = false;
    self.svg_content.clear();
    self.context = None;
    Ok( () )
  }
  
  fn supports_tilemaps( &self ) -> bool
  {
    false // SVG adapter doesn't support tilemaps
  }
  
  fn supports_particles( &self ) -> bool
  {
    false // SVG adapter doesn't support particle effects
  }
}

impl PrimitiveRenderer for SvgRenderer
{
  /// Renders a line command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if line rendering fails.
  fn render_line( &mut self, command: &LineCommand ) -> core::result::Result< (), RenderError >
  {
    let stroke_color = Self::color_to_svg( &command.style.color );
    let line_cap = Self::line_cap_to_svg( command.style.cap_style );
    let line_join = Self::line_join_to_svg( command.style.join_style );
    
    self.svg_content.push_str( &format!(
      r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"/>
"#,
      command.start.x, command.start.y,
      command.end.x, command.end.y,
      stroke_color, command.style.width,
      line_cap, line_join
    ) );
    
    Ok( () )
  }
  
  /// Renders a curve command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if curve rendering fails.
  fn render_curve( &mut self, command: &CurveCommand ) -> core::result::Result< (), RenderError >
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
    
    self.svg_content.push_str( &format!(
      r#"  <path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"/>
"#,
      path_data, stroke_color, command.style.width,
      line_cap, line_join
    ) );
    
    Ok( () )
  }
  
  /// Renders a text command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if text rendering fails.
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
    
    self.svg_content.push_str( &format!(
      r#"  <text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="{}" dominant-baseline="{}">{}</text>
"#,
      command.position.x, command.position.y,
      Self::resolve_font_family( command.font_style.family_id ), command.font_style.size,
      fill_color, text_anchor, dominant_baseline,
      text_content
    ) );
    
    Ok( () )
  }
  
  /// Renders a tilemap command.
  ///
  /// # Errors
  /// Returns `FeatureNotImplemented` if tilemap rendering is not supported.
  fn render_tilemap( &mut self, _command: &TilemapCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::FeatureNotImplemented( "Tilemap rendering not supported in SVG backend".to_string() ) )
  }
  
  /// Renders a particle emitter command.
  ///
  /// # Errors
  /// Returns `FeatureNotImplemented` if particle rendering is not supported.
  fn render_particle_emitter( &mut self, _command: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::FeatureNotImplemented( "Particle rendering not supported in SVG backend".to_string() ) )
  }
}