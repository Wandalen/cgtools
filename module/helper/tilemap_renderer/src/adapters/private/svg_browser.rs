//! SVG-in-browser backend adapter with interactivity.
//!
//! This adapter extends the SVG backend to generate interactive SVG content
//! suitable for web deployment with JavaScript event handling, hover effects,
//! and runtime interactivity.

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_raw_string_hashes ) ]
#![ allow( clippy::struct_excessive_bools ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::match_same_arms ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]

use crate::
{
  ports::{ Renderer, PrimitiveRenderer, RenderContext, RendererCapabilities, RenderError },
  commands::{ LineCommand, CurveCommand, TextCommand, TilemapCommand, ParticleEmitterCommand },
  scene::Scene,
};

/// Interactive SVG renderer for browser deployment.
/// 
/// Generates SVG content with embedded JavaScript for mouse events,
/// hover effects, selection, and runtime interactivity.
#[ derive( Clone ) ]
pub struct SvgBrowserRenderer
{
  /// Accumulated SVG elements with interactive features
  elements: Vec< String >,
  
  /// JavaScript event handlers
  event_handlers: Vec< String >,
  
  /// CSS styles for interactivity
  styles: Vec< String >,
  
  /// Viewport dimensions
  width: u32,
  height: u32,
  
  /// Background color
  background_color: [ f32; 4 ],
  
  /// Mouse picking enabled flag
  mouse_picking_enabled: bool,
  
  /// Hover effects enabled flag
  hover_effects_enabled: bool,
  
  /// Animation support enabled flag
  animation_enabled: bool,
  
  /// Current element ID counter for unique identification
  element_id_counter: u32,
  
  /// Initialization state
  is_initialized: bool,
  
  /// Frame active state
  frame_active: bool,
}

impl SvgBrowserRenderer
{
  /// Creates a new interactive SVG browser renderer with default settings.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    Self
    {
      elements: Vec::new(),
      event_handlers: Vec::new(),
      styles: Vec::new(),
      width: 800,
      height: 600,
      background_color: [ 1.0, 1.0, 1.0, 1.0 ],
      mouse_picking_enabled: true,
      hover_effects_enabled: true,
      animation_enabled: true,
      element_id_counter: 0,
      is_initialized: false,
      frame_active: false,
    }
  }
  
  /// Creates a new interactive SVG browser renderer with specified dimensions.
  #[ must_use ]
  #[ inline ]
  pub fn with_dimensions( width: u32, height: u32 ) -> Self
  {
    let mut renderer = Self::new();
    renderer.width = width;
    renderer.height = height;
    renderer
  }
  
  /// Enables or disables mouse picking functionality.
  #[ inline ]
  pub fn set_mouse_picking_enabled( &mut self, enabled: bool )
  {
    self.mouse_picking_enabled = enabled;
  }
  
  /// Enables or disables hover effects.
  #[ inline ]
  pub fn set_hover_effects_enabled( &mut self, enabled: bool )
  {
    self.hover_effects_enabled = enabled;
  }
  
  /// Enables or disables animation support.
  #[ inline ]
  pub fn set_animation_enabled( &mut self, enabled: bool )
  {
    self.animation_enabled = enabled;
  }
  
  /// Generates a unique element ID.
  fn generate_element_id( &mut self ) -> String
  {
    self.element_id_counter += 1;
    format!( "element_{}", self.element_id_counter )
  }
  
  /// Converts RGBA color to CSS hex format.
  fn color_to_hex( color: [ f32; 4 ] ) -> String
  {
    format!(
      "#{:02x}{:02x}{:02x}",
      ( color[ 0 ] * 255.0 ) as u8,
      ( color[ 1 ] * 255.0 ) as u8,
      ( color[ 2 ] * 255.0 ) as u8
    )
  }
  
  /// Converts RGBA color to CSS rgba format with alpha.
  fn color_to_rgba( color: [ f32; 4 ] ) -> String
  {
    format!(
      "rgba({}, {}, {}, {})",
      ( color[ 0 ] * 255.0 ) as u8,
      ( color[ 1 ] * 255.0 ) as u8,
      ( color[ 2 ] * 255.0 ) as u8,
      color[ 3 ]
    )
  }
  
  /// Adds hover effect CSS for an element.
  fn add_hover_effect( &mut self, element_id: &str, base_color: [ f32; 4 ] )
  {
    if self.hover_effects_enabled
    {
      // Create brighter hover color
      let hover_color = [
        ( base_color[ 0 ] * 1.2 ).min( 1.0 ),
        ( base_color[ 1 ] * 1.2 ).min( 1.0 ),
        ( base_color[ 2 ] * 1.2 ).min( 1.0 ),
        base_color[ 3 ]
      ];
      
      self.styles.push( format!(
        "#{element_id}:hover {{ stroke: {}; cursor: pointer; }}",
        Self::color_to_rgba( hover_color ),
        element_id = element_id
      ) );
    }
  }
  
  /// Adds click event handler for an element.
  fn add_click_handler( &mut self, element_id: &str )
  {
    if self.mouse_picking_enabled
    {
      self.event_handlers.push( format!(
        "document.getElementById('{}').addEventListener('click', function(e) {{ 
          console.log('Clicked element: {}');
          e.target.setAttribute('stroke-width', '3');
          setTimeout(() => e.target.setAttribute('stroke-width', '2'), 200);
        }});",
        element_id, element_id
      ) );
    }
  }
  
  /// Generates the complete HTML document with embedded SVG and interactivity.
  fn generate_html_document( &self ) -> String
  {
    let background_color = Self::color_to_hex( self.background_color );
    
    let mut html = format!(
      r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Interactive SVG Renderer</title>
  <style>
    body {{
      margin: 0;
      padding: 20px;
      font-family: Arial, sans-serif;
      background-color: #f0f0f0;
    }}
    .svg-container {{
      background-color: white;
      border: 1px solid #ccc;
      border-radius: 4px;
      padding: 10px;
      display: inline-block;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }}
    svg {{
      background-color: {background_color};
    }}
"#,
      background_color = background_color
    );
    
    // Add custom styles
    for style in &self.styles
    {
      html.push_str( "    " );
      html.push_str( style );
      html.push( '\n' );
    }
    
    html.push_str( r#"  </style>
</head>
<body>
  <div class="svg-container">
    <svg width=""# );
    html.push_str( &self.width.to_string() );
    html.push_str( r#"" height=""# );
    html.push_str( &self.height.to_string() );
    html.push_str( r#"" xmlns="http://www.w3.org/2000/svg">
"# );
    
    // Add all SVG elements
    for element in &self.elements
    {
      html.push_str( "      " );
      html.push_str( element );
      html.push( '\n' );
    }
    
    html.push_str( r#"    </svg>
  </div>
  
  <script>
    // Interactive SVG functionality
    console.log('Interactive SVG renderer loaded');
    
"# );
    
    // Add event handlers
    for handler in &self.event_handlers
    {
      html.push_str( "    " );
      html.push_str( handler );
      html.push( '\n' );
    }
    
    if self.animation_enabled
    {
      html.push_str( r#"    
    // Animation utilities
    function animateElement(elementId, property, fromValue, toValue, duration) {
      const element = document.getElementById(elementId);
      const startTime = Date.now();
      
      function animate() {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        const currentValue = fromValue + (toValue - fromValue) * progress;
        element.setAttribute(property, currentValue);
        
        if (progress < 1) {
          requestAnimationFrame(animate);
        }
      }
      
      animate();
    }
"# );
    }
    
    html.push_str( r#"  </script>
</body>
</html>
"# );
    
    html
  }
}

impl Default for SvgBrowserRenderer
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

impl Renderer for SvgBrowserRenderer
{
  type Output = String;
  
  #[ inline ]
  fn initialize( &mut self, context: &RenderContext ) -> Result< (), RenderError >
  {
    self.width = context.width;
    self.height = context.height;
    self.background_color = context.background_color;
    
    // Clear previous content
    self.elements.clear();
    self.event_handlers.clear();
    self.styles.clear();
    self.element_id_counter = 0;
    
    self.is_initialized = true;
    Ok( () )
  }
  
  #[ inline ]
  fn begin_frame( &mut self, _context: &RenderContext ) -> Result< (), RenderError >
  {
    if !self.is_initialized
    {
      return Err( RenderError::InitializationFailed( "Renderer not initialized".to_string() ) );
    }
    
    if self.frame_active
    {
      return Err( RenderError::RenderFailed( "Frame already active".to_string() ) );
    }
    
    self.frame_active = true;
    Ok( () )
  }
  
  #[ inline ]
  fn end_frame( &mut self ) -> Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    self.frame_active = false;
    Ok( () )
  }
  
  fn render_scene( &mut self, scene: &Scene ) -> Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    // Validate scene before rendering
    self.validate_scene( scene )?;
    
    for command in scene.commands()
    {
      match command
      {
        crate::commands::RenderCommand::Line( line_cmd ) => self.render_line( line_cmd )?,
        crate::commands::RenderCommand::Curve( curve_cmd ) => self.render_curve( curve_cmd )?,
        crate::commands::RenderCommand::Text( text_cmd ) => self.render_text( text_cmd )?,
        crate::commands::RenderCommand::Tilemap( tilemap_cmd ) => self.render_tilemap( tilemap_cmd )?,
        crate::commands::RenderCommand::ParticleEmitter( particle_cmd ) => self.render_particle_emitter( particle_cmd )?,
      }
    }
    
    Ok( () )
  }
  
  #[ inline ]
  fn capabilities( &self ) -> RendererCapabilities
  {
    RendererCapabilities
    {
      backend_name: "SVG-Browser".to_string(),
      backend_version: "1.0".to_string(),
      supports_transparency: true,
      supports_antialiasing: true,
      supports_custom_fonts: true,
      supports_particles: false, // Not implemented for SVG
      supports_realtime: true,   // Interactive updates
      max_texture_size: 0,       // Vector graphics
      max_scene_complexity: 50_000,
    }
  }
  
  fn output( &self ) -> Result< Self::Output, RenderError >
  {
    Ok( self.generate_html_document() )
  }
  
  #[ inline ]
  fn cleanup( &mut self ) -> Result< (), RenderError >
  {
    self.elements.clear();
    self.event_handlers.clear();
    self.styles.clear();
    self.element_id_counter = 0;
    self.is_initialized = false;
    self.frame_active = false;
    Ok( () )
  }
}

impl PrimitiveRenderer for SvgBrowserRenderer
{
  fn render_line( &mut self, command: &LineCommand ) -> Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    let element_id = self.generate_element_id();
    let color = Self::color_to_rgba( command.style.color );
    
    let line_element = format!(
      r#"<line id="{}" x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" stroke-opacity="{}" stroke-linecap="{}" stroke-linejoin="{}"/>"#,
      element_id,
      command.start.x,
      command.start.y,
      command.end.x,
      command.end.y,
      color,
      command.style.width,
      command.style.color[ 3 ],
      match command.style.cap_style
      {
        crate::commands::LineCap::Butt => "butt",
        crate::commands::LineCap::Round => "round",
        crate::commands::LineCap::Square => "square",
      },
      match command.style.join_style
      {
        crate::commands::LineJoin::Miter => "miter",
        crate::commands::LineJoin::Round => "round",
        crate::commands::LineJoin::Bevel => "bevel",
      }
    );
    
    self.elements.push( line_element );
    self.add_hover_effect( &element_id, command.style.color );
    self.add_click_handler( &element_id );
    
    Ok( () )
  }
  
  fn render_curve( &mut self, command: &CurveCommand ) -> Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    let element_id = self.generate_element_id();
    let color = Self::color_to_rgba( command.style.color );
    
    let path_data = format!(
      "M {} {} C {} {}, {} {}, {} {}",
      command.start.x, command.start.y,
      command.control1.x, command.control1.y,
      command.control2.x, command.control2.y,
      command.end.x, command.end.y
    );
    
    let curve_element = format!(
      r#"<path id="{}" d="{}" stroke="{}" stroke-width="{}" stroke-opacity="{}" fill="none" stroke-linecap="{}" stroke-linejoin="{}"/>"#,
      element_id,
      path_data,
      color,
      command.style.width,
      command.style.color[ 3 ],
      match command.style.cap_style
      {
        crate::commands::LineCap::Butt => "butt",
        crate::commands::LineCap::Round => "round",
        crate::commands::LineCap::Square => "square",
      },
      match command.style.join_style
      {
        crate::commands::LineJoin::Miter => "miter",
        crate::commands::LineJoin::Round => "round",
        crate::commands::LineJoin::Bevel => "bevel",
      }
    );
    
    self.elements.push( curve_element );
    self.add_hover_effect( &element_id, command.style.color );
    self.add_click_handler( &element_id );
    
    Ok( () )
  }
  
  fn render_text( &mut self, command: &TextCommand ) -> Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    let element_id = self.generate_element_id();
    let color = Self::color_to_rgba( command.font_style.color );
    
    // Extract text from fixed-size array
    let text_slice = &command.text[ ..command.text_len as usize ];
    let text_string = core::str::from_utf8( text_slice )
      .map_err( |_| RenderError::RenderFailed( "Invalid UTF-8 text".to_string() ) )?;
    
    // Calculate text anchor position
    let ( anchor_x, anchor_y ) = match command.anchor
    {
      crate::commands::TextAnchor::TopLeft => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::TopCenter => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::TopRight => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::CenterLeft => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::Center => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::CenterRight => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::BottomLeft => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::BottomCenter => ( command.position.x, command.position.y ),
      crate::commands::TextAnchor::BottomRight => ( command.position.x, command.position.y ),
    };
    
    let text_anchor = match command.anchor
    {
      crate::commands::TextAnchor::TopLeft |
      crate::commands::TextAnchor::CenterLeft |
      crate::commands::TextAnchor::BottomLeft => "start",
      crate::commands::TextAnchor::TopCenter |
      crate::commands::TextAnchor::Center |
      crate::commands::TextAnchor::BottomCenter => "middle",
      crate::commands::TextAnchor::TopRight |
      crate::commands::TextAnchor::CenterRight |
      crate::commands::TextAnchor::BottomRight => "end",
    };
    
    let text_element = format!(
      r#"<text id="{}" x="{}" y="{}" fill="{}" fill-opacity="{}" font-size="{}" font-weight="{}" font-style="{}" text-anchor="{}">{}</text>"#,
      element_id,
      anchor_x,
      anchor_y,
      color,
      command.font_style.color[ 3 ],
      command.font_style.size,
      command.font_style.weight,
      if command.font_style.italic { "italic" } else { "normal" },
      text_anchor,
      text_string
    );
    
    self.elements.push( text_element );
    
    // Add hover effect for text with different styling
    if self.hover_effects_enabled
    {
      self.styles.push( format!(
        "#{element_id}:hover {{ fill: #ff6600; cursor: pointer; }}",
        element_id = element_id
      ) );
    }
    
    self.add_click_handler( &element_id );
    
    Ok( () )
  }
  
  fn render_tilemap( &mut self, _command: &TilemapCommand ) -> Result< (), RenderError >
  {
    Err( RenderError::UnsupportedCommand( "Tilemap rendering not implemented for SVG-Browser backend".to_string() ) )
  }
  
  fn render_particle_emitter( &mut self, _command: &ParticleEmitterCommand ) -> Result< (), RenderError >
  {
    Err( RenderError::UnsupportedCommand( "Particle emitter rendering not implemented for SVG-Browser backend".to_string() ) )
  }
  
}