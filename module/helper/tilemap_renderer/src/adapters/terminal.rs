//! Terminal backend adapter.
//!
//! Renders commands as ASCII/Unicode art with optional ANSI colors.
//! Supports paths (Bresenham approximation) and text (direct placement).

use crate::assets::Assets;
use crate::backend::*;
use crate::commands::*;
use crate::types::RenderConfig;

/// Terminal renderer backend.
///
/// ```ignore
/// let config = RenderConfig { width: 120, height: 40, ..Default::default() };
/// let mut term = TerminalBackend::new( config );
/// term.load_assets( &assets )?;
/// term.submit( &commands )?;
/// let Output::String( text ) = term.output()? else { unreachable!() };
/// print!( "{text}" );
/// ```
pub struct TerminalBackend
{
  width : usize,
  height : usize,
  /// Flat character buffer, row-major.
  buffer : Vec< char >,
  /// ANSI RGB color per cell, row-major.
  colors : Vec< Option< [ u8; 3 ] > >,
  /// Whether to use Unicode line drawing characters.
  pub unicode : bool,
  /// Whether to emit ANSI color codes.
  pub ansi_color : bool,
  // -- streaming state --
  path_active : bool,
  path_cursor : ( f32, f32 ),
  path_color : [ f32; 4 ],
}

impl TerminalBackend
{
  /// Creates a new terminal backend from render config.
  #[ must_use ]
  pub fn new( config : RenderConfig ) -> Self
  {
    let width = config.width as usize;
    let height = config.height as usize;
    Self
    {
      width,
      height,
      buffer : vec![ ' '; width * height ],
      colors : vec![ None; width * height ],
      unicode : true,
      ansi_color : true,
      path_active : false,
      path_cursor : ( 0.0, 0.0 ),
      path_color : [ 1.0, 1.0, 1.0, 1.0 ],
    }
  }

  fn clear_buffer( &mut self )
  {
    self.buffer.fill( ' ' );
    self.colors.fill( None );
  }

  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  fn set_pixel( &mut self, x : f32, y : f32, ch : char, color : &[ f32; 4 ] )
  {
    let xi = x as usize;
    let yi = y as usize;
    if xi < self.width && yi < self.height
    {
      let idx = yi * self.width + xi;
      self.buffer[ idx ] = ch;
      if self.ansi_color
      {
        self.colors[ idx ] = Some
        ([
          ( color[ 0 ] * 255.0 ) as u8,
          ( color[ 1 ] * 255.0 ) as u8,
          ( color[ 2 ] * 255.0 ) as u8,
        ]);
      }
    }
  }

  /// Bresenham line drawing.
  #[ allow( clippy::cast_possible_truncation, clippy::cast_possible_wrap ) ]
  fn draw_line( &mut self, x0 : f32, y0 : f32, x1 : f32, y1 : f32, color : &[ f32; 4 ] )
  {
    let ch = if self.unicode { '*' } else { '*' };
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let x_end = x1 as i32;
    let y_end = y1 as i32;
    let dx = ( x_end - x ).abs();
    let dy = -( y_end - y ).abs();
    let sx = if x < x_end { 1 } else { -1 };
    let sy = if y < y_end { 1 } else { -1 };
    let mut err = dx + dy;

    loop
    {
      self.set_pixel( x as f32, y as f32, ch, color );
      if x == x_end && y == y_end { break; }
      let e2 = 2 * err;
      if e2 >= dy
      {
        err += dy;
        x += sx;
      }
      if e2 <= dx
      {
        err += dx;
        y += sy;
      }
    }
  }
}

impl Backend for TerminalBackend
{
  fn load_assets( &mut self, _assets : &Assets ) -> Result< (), RenderError >
  {
    // Terminal ignores most assets.
    Ok( () )
  }

  fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    for cmd in commands
    {
      match cmd
      {
        RenderCommand::Clear( _ ) =>
        {
          self.clear_buffer();
        }

        RenderCommand::BeginPath( bp ) =>
        {
          self.path_active = true;
          self.path_cursor = ( 0.0, 0.0 );
          self.path_color = bp.stroke_color;
        }
        RenderCommand::MoveTo( m ) =>
        {
          self.path_cursor = ( m.0, m.1 );
        }
        RenderCommand::LineTo( l ) =>
        {
          let ( cx, cy ) = self.path_cursor;
          let color = self.path_color;
          self.draw_line( cx, cy, l.0, l.1, &color );
          self.path_cursor = ( l.0, l.1 );
        }
        RenderCommand::QuadTo( q ) =>
        {
          // Approximate: just draw line to endpoint
          let ( cx, cy ) = self.path_cursor;
          let color = self.path_color;
          self.draw_line( cx, cy, q.x, q.y, &color );
          self.path_cursor = ( q.x, q.y );
        }
        RenderCommand::CubicTo( c ) =>
        {
          let ( cx, cy ) = self.path_cursor;
          let color = self.path_color;
          self.draw_line( cx, cy, c.x, c.y, &color );
          self.path_cursor = ( c.x, c.y );
        }
        RenderCommand::ArcTo( a ) =>
        {
          let ( cx, cy ) = self.path_cursor;
          let color = self.path_color;
          self.draw_line( cx, cy, a.x, a.y, &color );
          self.path_cursor = ( a.x, a.y );
        }
        RenderCommand::ClosePath( _ ) | RenderCommand::EndPath( _ ) =>
        {
          self.path_active = false;
        }

        RenderCommand::BeginText( bt ) =>
        {
          // Text will be placed character by character
          self.path_cursor = ( bt.position[ 0 ], bt.position[ 1 ] );
          self.path_color = bt.color;
        }
        RenderCommand::Char( ch ) =>
        {
          let ( x, y ) = self.path_cursor;
          let color = self.path_color;
          self.set_pixel( x, y, ch.0, &color );
          self.path_cursor.0 += 1.0;
        }
        RenderCommand::EndText( _ ) => {}

        // Unsupported in terminal
        RenderCommand::Mesh( _ ) => {}
        RenderCommand::Sprite( _ ) => {}
        RenderCommand::BeginInstancedMesh( _ ) => {}
        RenderCommand::Instance( _ ) => {}
        RenderCommand::EndInstancedMesh( _ ) => {}
        RenderCommand::BeginInstancedSprite( _ ) => {}
        RenderCommand::SpriteInstance( _ ) => {}
        RenderCommand::EndInstancedSprite( _ ) => {}
        RenderCommand::BeginRecordBatch( _ ) => {}
        RenderCommand::EndRecordBatch( _ ) => {}
        RenderCommand::BeginGroup( _ ) => {}
        RenderCommand::EndGroup( _ ) => {}
      }
    }

    Ok( () )
  }

  fn resize( &mut self, width : u32, height : u32 )
  {
    self.width = width as usize;
    self.height = height as usize;
    self.buffer = vec![ ' '; self.width * self.height ];
    self.colors = vec![ None; self.width * self.height ];
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    let mut result = String::with_capacity( ( self.width + 1 ) * self.height );

    for y in 0..self.height
    {
      for x in 0..self.width
      {
        let idx = y * self.width + x;
        if let Some( [ r, g, b ] ) = self.colors[ idx ]
        {
          result.push_str( &format!( "\x1b[38;2;{r};{g};{b}m" ) );
          result.push( self.buffer[ idx ] );
          result.push_str( "\x1b[0m" );
        }
        else
        {
          result.push( self.buffer[ idx ] );
        }
      }
      result.push( '\n' );
    }

    Ok( Output::String( result ) )
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities
    {
      paths : true,
      text : true,
      meshes : false,
      sprites : false,
      instancing : false,
      gradients : false,
      patterns : false,
      clip_masks : false,
      effects : false,
      blend_modes : false,
      text_on_path : false,
      max_texture_size : 0,
    }
  }
}
