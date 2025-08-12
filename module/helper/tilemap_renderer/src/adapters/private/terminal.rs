//! Terminal backend adapter implementation.
//!
//! This adapter renders scenes as ASCII art suitable for terminal display,
//! supporting configurable dimensions, Unicode line drawing, and ANSI colors.

#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( unused_imports ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::struct_excessive_bools ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::manual_abs_diff ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::match_same_arms ) ]

use crate::ports::{ RenderContext, Renderer, RendererCapabilities, RenderError, PrimitiveRenderer };
use crate::scene::Scene;
use crate::commands::{ RenderCommand, LineCommand, CurveCommand, TextCommand, TextAnchor, TilemapCommand, ParticleEmitterCommand };

/// Terminal renderer backend adapter.
///
/// Renders scenes as ASCII art with configurable output dimensions,
/// Unicode line drawing characters, and ANSI color support.
#[ derive( Debug, Clone ) ]
pub struct TerminalRenderer
{
  /// Output buffer as 2D character grid.
  buffer : Vec< Vec< char > >,
  /// ANSI color buffer for each character.
  color_buffer : Vec< Vec< String > >,
  /// Output dimensions (width, height).
  dimensions : ( usize, usize ),
  /// Whether Unicode line drawing is enabled.
  unicode_enabled : bool,
  /// Whether ANSI colors are enabled.
  color_enabled : bool,
  /// Current rendering state.
  initialized : bool,
  /// Whether a frame is currently active.
  frame_active : bool,
  /// Rendering context for the current frame.
  context : Option< RenderContext >,
}

impl TerminalRenderer
{
  /// Creates a new terminal renderer with default dimensions.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self::with_dimensions( 80, 24 )
  }

  /// Creates a new terminal renderer with specified dimensions.
  #[ must_use ]  
  pub fn with_dimensions( width: usize, height: usize ) -> Self
  {
    let mut renderer = Self
    {
      buffer: vec![ vec![ ' '; width ]; height ],
      color_buffer: vec![ vec![ String::new(); width ]; height ],
      dimensions: ( width, height ),
      unicode_enabled: true,
      color_enabled: true,
      initialized: false,
      frame_active: false,
      context: None,
    };
    renderer.clear_buffer();
    renderer
  }

  /// Sets whether Unicode line drawing characters are enabled.
  pub fn set_unicode_enabled( &mut self, enabled: bool )
  {
    self.unicode_enabled = enabled;
  }

  /// Sets whether ANSI color support is enabled.  
  pub fn set_color_enabled( &mut self, enabled: bool )
  {
    self.color_enabled = enabled;
  }

  /// Clears the output buffer.
  fn clear_buffer( &mut self )
  {
    for row in &mut self.buffer
    {
      for cell in row
      {
        *cell = ' ';
      }
    }
    for row in &mut self.color_buffer
    {
      for cell in row
      {
        cell.clear();
      }
    }
  }

  /// Sets a character at the specified position with optional color.
  fn set_char_at( &mut self, x: usize, y: usize, ch: char, color: Option< &str > )
  {
    if x < self.dimensions.0 && y < self.dimensions.1
    {
      self.buffer[ y ][ x ] = ch;
      if let Some( color_code ) = color
      {
        if self.color_enabled
        {
          self.color_buffer[ y ][ x ] = color_code.to_string();
        }
      }
    }
  }

  /// Draws a line between two points using ASCII characters.
  fn draw_line( &mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Option< &str > )
  {
    let x1_i = x1 as usize;
    let y1_i = y1 as usize;  
    let x2_i = x2 as usize;
    let y2_i = y2 as usize;

    // Simple line drawing using Bresenham-style algorithm
    let dx = if x2_i >= x1_i { x2_i - x1_i } else { x1_i - x2_i };
    let dy = if y2_i >= y1_i { y2_i - y1_i } else { y1_i - y2_i };

    if dx == 0 && dy == 0
    {
      // Single point
      let ch = if self.unicode_enabled { '●' } else { '*' };
      self.set_char_at( x1_i, y1_i, ch, color );
      return;
    }

    if dx > dy
    {
      // Horizontal-ish line
      let ch = if self.unicode_enabled { '─' } else { '-' };
      let start_x = x1_i.min( x2_i );
      let end_x = x1_i.max( x2_i );
      let y = if dx > 0 { 
        y1_i + ( ( y2_i as isize - y1_i as isize ) * ( start_x as isize - x1_i as isize ) / ( x2_i as isize - x1_i as isize ).max( 1 ) ) as usize
      } else { 
        y1_i 
      };
      
      for x in start_x..=end_x
      {
        self.set_char_at( x, y, ch, color );
      }
    }
    else
    {
      // Vertical-ish line
      let ch = if self.unicode_enabled { '│' } else { '|' };
      let start_y = y1_i.min( y2_i );
      let end_y = y1_i.max( y2_i );
      let x = if dy > 0 && y2_i != y1_i {
        let dx_calc = ( x2_i as isize - x1_i as isize ) * ( start_y as isize - y1_i as isize ) / ( y2_i as isize - y1_i as isize );
        ( x1_i as isize + dx_calc ).max( 0 ).min( self.dimensions.0 as isize - 1 ) as usize
      } else {
        x1_i
      };

      for y in start_y..=end_y  
      {
        self.set_char_at( x, y, ch, color );
      }
    }
  }

  /// Approximates a curve using line segments.
  fn draw_curve( &mut self, cmd: &CurveCommand, color: Option< &str > )
  {
    // Simple curve approximation with multiple line segments
    const SEGMENTS: usize = 10;
    
    for i in 0..SEGMENTS
    {
      let t1 = i as f32 / SEGMENTS as f32;
      let t2 = ( i + 1 ) as f32 / SEGMENTS as f32;
      
      // Cubic Bezier curve calculation using control1 (ignoring control2 for simplicity)
      let x1 = ( 1.0 - t1 ).powi( 2 ) * cmd.start.x + 
                2.0 * ( 1.0 - t1 ) * t1 * cmd.control1.x + 
                t1.powi( 2 ) * cmd.end.x;
      let y1 = ( 1.0 - t1 ).powi( 2 ) * cmd.start.y + 
                2.0 * ( 1.0 - t1 ) * t1 * cmd.control1.y + 
                t1.powi( 2 ) * cmd.end.y;
                
      let x2 = ( 1.0 - t2 ).powi( 2 ) * cmd.start.x + 
                2.0 * ( 1.0 - t2 ) * t2 * cmd.control1.x + 
                t2.powi( 2 ) * cmd.end.x;
      let y2 = ( 1.0 - t2 ).powi( 2 ) * cmd.start.y + 
                2.0 * ( 1.0 - t2 ) * t2 * cmd.control1.y + 
                t2.powi( 2 ) * cmd.end.y;

      self.draw_line( x1, y1, x2, y2, color );
    }
  }

  /// Draws text at the specified position.
  fn draw_text( &mut self, cmd: &TextCommand, color: Option< &str > )
  {
    // Convert text from [u8; 64] array to string
    let text_end = cmd.text.iter().position( |&b| b == 0 ).unwrap_or( cmd.text.len() );
    let text_str = core::str::from_utf8( &cmd.text[ ..text_end ] ).unwrap_or( "<invalid>" );
    
    let mut x = cmd.position.x as usize;
    let y = cmd.position.y as usize;

    // Apply text anchoring
    let text_width = text_str.len();
    match cmd.anchor
    {
      TextAnchor::TopLeft => {}, // No adjustment needed
      TextAnchor::TopCenter => x = x.saturating_sub( text_width / 2 ),
      TextAnchor::TopRight => x = x.saturating_sub( text_width ),
      TextAnchor::CenterLeft => {}, // No horizontal adjustment
      TextAnchor::Center => x = x.saturating_sub( text_width / 2 ),
      TextAnchor::CenterRight => x = x.saturating_sub( text_width ),
      TextAnchor::BottomLeft => {}, // No adjustment needed
      TextAnchor::BottomCenter => x = x.saturating_sub( text_width / 2 ),
      TextAnchor::BottomRight => x = x.saturating_sub( text_width ),
    }

    // Draw each character
    for ( i, ch ) in text_str.chars().enumerate()
    {
      self.set_char_at( x + i, y, ch, color );
    }
  }

  /// Converts RGBA color to ANSI escape sequence.
  fn rgba_to_ansi( color: &[ f32; 4 ] ) -> String
  {
    // Convert to RGB 8-bit values  
    let r = ( color[ 0 ] * 255.0 ) as u8;
    let g = ( color[ 1 ] * 255.0 ) as u8;
    let b = ( color[ 2 ] * 255.0 ) as u8;
    
    // Use 24-bit ANSI color code
    format!( "\x1b[38;2;{};{};{}m", r, g, b )
  }

  /// Gets the terminal output as a string.
  #[ must_use ]
  pub fn get_output( &self ) -> String
  {
    let mut result = String::new();
    
    for y in 0..self.dimensions.1
    {
      for x in 0..self.dimensions.0
      {
        if self.color_enabled && !self.color_buffer[ y ][ x ].is_empty()
        {
          result.push_str( &self.color_buffer[ y ][ x ] );
        }
        result.push( self.buffer[ y ][ x ] );
        if self.color_enabled && !self.color_buffer[ y ][ x ].is_empty()
        {
          result.push_str( "\x1b[0m" ); // Reset color
        }
      }
      result.push( '\n' );
    }
    
    result
  }

  /// Exports output to a text file.
  pub fn export_to_file( &self, path: &str ) -> core::result::Result< (), RenderError >
  {
    let content = self.get_output();
    #[ cfg( feature = "std" ) ]
    {
      std::fs::write( path, content ).map_err( |e| RenderError::OutputError( format!( "Failed to write file: {}", e ) ) )?;
    }
    #[ cfg( not( feature = "std" ) ) ]
    {
      // For no_std environments, we can't write files directly
      return Err( RenderError::OutputError( "File export not available in no_std".to_string() ) );
    }
    Ok( () )
  }

  /// Outputs to console or file based on destination.
  pub fn output_to< T: AsRef< str > >( &self, destination: T ) -> core::result::Result< (), RenderError >
  {
    let dest_str = destination.as_ref();
    if dest_str.is_empty()
    {
      // Output to stdout/console
      #[ cfg( feature = "std" ) ]
      {
        print!( "{}", self.get_output() );
      }
      Ok( () )
    }
    else
    {
      // Export to file
      self.export_to_file( dest_str )
    }
  }
}

impl Default for TerminalRenderer
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl Renderer for TerminalRenderer
{
  type Output = String;

  fn capabilities( &self ) -> RendererCapabilities
  {
    RendererCapabilities
    {
      backend_name: "Terminal".to_string(),
      backend_version: "1.0.0".to_string(),
      max_texture_size: 0,
      supports_transparency: false,
      supports_antialiasing: false,
      supports_custom_fonts: false,
      supports_particles: false,
      supports_realtime: false,
      max_scene_complexity: 10000,
    }
  }

  fn initialize( &mut self, _context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    self.initialized = true;
    Ok( () )
  }

  fn begin_frame( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if !self.initialized
    {
      return Err( RenderError::InitializationFailed( "Renderer not initialized".to_string() ) );
    }
    if self.frame_active
    {
      return Err( RenderError::InvalidContext( "Frame already active".to_string() ) );
    }

    self.frame_active = true;
    self.context = Some( context.clone() );
    self.clear_buffer();
    Ok( () )
  }

  fn render_scene( &mut self, scene: &Scene ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::InvalidContext( "No active frame".to_string() ) );
    }

    for command in scene.commands()
    {
      match command
      {
        RenderCommand::Line( cmd ) => self.render_line( &cmd )?,
        RenderCommand::Curve( cmd ) => self.render_curve( &cmd )?,
        RenderCommand::Text( cmd ) => self.render_text( &cmd )?,
        RenderCommand::Tilemap( cmd ) => self.render_tilemap( &cmd )?,
        RenderCommand::ParticleEmitter( cmd ) => self.render_particle_emitter( &cmd )?,
      }
    }

    Ok( () )
  }

  fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::InvalidContext( "No active frame".to_string() ) );
    }

    self.frame_active = false;
    self.context = None;
    Ok( () )
  }

  fn output( &self ) -> core::result::Result< Self::Output, RenderError >
  {
    Ok( self.get_output() )
  }

  fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
  {
    self.initialized = false;
    self.frame_active = false;
    self.context = None;
    self.clear_buffer();
    Ok( () )
  }
}

impl PrimitiveRenderer for TerminalRenderer
{
  fn render_line( &mut self, cmd: &LineCommand ) -> core::result::Result< (), RenderError >
  {
    let color = if self.color_enabled
    {
      Some( Self::rgba_to_ansi( &cmd.style.color ) )
    }
    else
    {
      None
    };

    self.draw_line( cmd.start.x, cmd.start.y, cmd.end.x, cmd.end.y, color.as_deref() );
    Ok( () )
  }

  fn render_curve( &mut self, cmd: &CurveCommand ) -> core::result::Result< (), RenderError >
  {
    let color = if self.color_enabled
    {
      Some( Self::rgba_to_ansi( &cmd.style.color ) )
    }
    else
    {
      None
    };

    self.draw_curve( cmd, color.as_deref() );
    Ok( () )
  }

  fn render_text( &mut self, cmd: &TextCommand ) -> core::result::Result< (), RenderError >
  {
    let color = if self.color_enabled
    {
      Some( Self::rgba_to_ansi( &cmd.font_style.color ) )
    }
    else
    {
      None
    };

    self.draw_text( cmd, color.as_deref() );
    Ok( () )
  }

  fn render_tilemap( &mut self, _cmd: &TilemapCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::UnsupportedCommand( "Tilemap rendering not supported in terminal backend".to_string() ) )
  }

  fn render_particle_emitter( &mut self, _cmd: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::UnsupportedCommand( "Particle rendering not supported in terminal backend".to_string() ) )
  }
}