//! WebGL backend adapter implementation.
//!
//! This adapter provides hardware-accelerated rendering using WebGL via WebAssembly,
//! designed to integrate with the minwebgl crate for optimal GPU performance.

#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::missing_docs_in_private_items ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]

use crate::ports::{ RenderContext, Renderer, RendererCapabilities, RenderError, PrimitiveRenderer };
use crate::scene::Scene;
use crate::commands::{ RenderCommand, LineCommand, CurveCommand, TextCommand, TextAnchor, TilemapCommand, ParticleEmitterCommand };

/// WebGL rendering statistics for performance monitoring.
#[ derive( Debug, Clone, Default ) ]
pub struct WebGLStats
{
  /// Number of vertices rendered in current frame.
  pub vertices_rendered: usize,
  /// Number of draw calls issued in current frame.
  pub draw_calls: usize,
  /// Number of texture bindings in current frame.
  pub texture_bindings: usize,
  /// Frame rendering time in milliseconds.
  pub frame_time_ms: f32,
}

/// WebGL context state for managing GPU resources.
#[ derive( Debug ) ]
struct WebGLContext
{
  /// Canvas width in pixels.
  width: u32,
  /// Canvas height in pixels.
  height: u32,
  /// Whether context is lost and needs restoration.
  context_lost: bool,
  /// Current viewport dimensions.
  viewport: ( u32, u32, u32, u32 ),
  /// Active shader program ID.
  active_program: Option< u32 >,
  /// Vertex buffer object ID for line rendering.
  line_vbo: Option< u32 >,
  /// Vertex buffer object ID for curve rendering.
  curve_vbo: Option< u32 >,
  /// Render statistics for current frame.
  stats: WebGLStats,
}

impl Default for WebGLContext
{
  fn default() -> Self
  {
    Self
    {
      width: 800,
      height: 600,
      context_lost: false,
      viewport: ( 0, 0, 800, 600 ),
      active_program: None,
      line_vbo: None,
      curve_vbo: None,
      stats: WebGLStats::default(),
    }
  }
}

/// WebGL renderer backend adapter.
///
/// Provides hardware-accelerated 2D rendering using WebGL with efficient batching,
/// real-time performance, and GPU-based rendering capabilities.
#[ derive( Debug ) ]
pub struct WebGLRenderer
{
  /// WebGL context state.
  context: Option< WebGLContext >,
  /// Whether the renderer has been initialized.
  initialized: bool,
  /// Whether a frame is currently active.
  frame_active: bool,
  /// Current rendering context.
  render_context: Option< RenderContext >,
  /// Accumulated vertex data for batching.
  vertex_buffer: Vec< f32 >,
  /// Accumulated color data for batching.
  color_buffer: Vec< f32 >,
  /// Current batch primitive count.
  batch_count: usize,
  /// Maximum batch size for optimal performance.
  max_batch_size: usize,
  /// Whether mouse picking is enabled.
  mouse_picking_enabled: bool,
  /// Last mouse position for picking.
  last_mouse_pos: ( f32, f32 ),
}

impl WebGLRenderer
{
  /// Creates a new WebGL renderer instance.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    return Self
    {
      context: None,
      initialized: false,
      frame_active: false,
      render_context: None,
      vertex_buffer: Vec::with_capacity( 10_000 ),
      color_buffer: Vec::with_capacity( 10_000 ),
      batch_count: 0,
      max_batch_size: 1_000,
      mouse_picking_enabled: false,
      last_mouse_pos: ( 0.0, 0.0 ),
    };
  }

  /// Creates a WebGL renderer with specified canvas dimensions.
  #[ must_use ]
  #[ inline ]
  pub fn with_dimensions( width: u32, height: u32 ) -> Self
  {
    let mut renderer = Self::new();
    let mut ctx = WebGLContext::default();
    ctx.width = width;
    ctx.height = height;
    ctx.viewport = ( 0, 0, width, height );
    renderer.context = Some( ctx );
    return renderer;
  }

  /// Enables or disables mouse picking functionality.
  #[ inline ]
  pub fn set_mouse_picking_enabled( &mut self, enabled: bool )
  {
    self.mouse_picking_enabled = enabled;
  }

  /// Sets the maximum batch size for rendering optimization.
  #[ inline ]
  pub fn set_max_batch_size( &mut self, size: usize )
  {
    self.max_batch_size = size;
    self.vertex_buffer.reserve( size * 6 ); // 2D vertices + RGBA
    self.color_buffer.reserve( size * 4 );  // RGBA color values
  }

  /// Mock WebGL context initialization.
  #[ inline ]
  fn initialize_webgl_context( &mut self, render_context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    let mut ctx = WebGLContext::default();
    ctx.width = render_context.width;
    ctx.height = render_context.height;
    ctx.viewport = ( 0, 0, render_context.width, render_context.height );

    // Mock WebGL initialization steps
    // In real implementation, this would:
    // 1. Get WebGL context from canvas
    // 2. Set up viewport and initial state
    // 3. Load and compile shaders
    // 4. Create vertex buffer objects
    // 5. Set up blending and depth testing

    ctx.line_vbo = Some( 1 ); // Mock VBO ID
    ctx.curve_vbo = Some( 2 ); // Mock VBO ID
    ctx.active_program = Some( 1 ); // Mock shader program ID

    self.context = Some( ctx );
    return Ok( () );
  }

  /// Mock shader compilation and program linking.
  #[ inline ]
  fn compile_shaders( &mut self )
  {
    // Mock shader compilation
    // In real implementation, this would:
    // 1. Compile vertex shader for 2D primitives
    // 2. Compile fragment shader with color blending
    // 3. Link shader program
    // 4. Get uniform and attribute locations
    // 5. Set up texture samplers if needed

    if let Some( ref mut ctx ) = self.context
    {
      ctx.active_program = Some( 1 ); // Mock compiled program ID
    }
  }

  /// Flushes the current vertex batch to GPU.
  #[ inline ]
  fn flush_batch( &mut self )
  {
    if self.batch_count == 0
    {
      return;
    }

    if let Some( ref mut ctx ) = self.context
    {
      // Mock GPU batch rendering
      // In real implementation, this would:
      // 1. Upload vertex data to VBO
      // 2. Upload color data to VBO
      // 3. Set up vertex attribute pointers
      // 4. Issue draw call (drawArrays or drawElements)
      // 5. Update rendering statistics

      ctx.stats.vertices_rendered += self.batch_count * 2; // Lines have 2 vertices
      ctx.stats.draw_calls += 1;
      ctx.stats.texture_bindings += 0; // No textures for basic primitives
    }

    // Clear batch buffers
    self.vertex_buffer.clear();
    self.color_buffer.clear();
    self.batch_count = 0;
  }

  /// Adds a line to the current vertex batch.
  fn add_line_to_batch( &mut self, cmd: &LineCommand ) -> core::result::Result< (), RenderError >
  {
    // Check if batch is full
    if self.batch_count >= self.max_batch_size
    {
      self.flush_batch();
    }

    // Add line vertices to batch
    self.vertex_buffer.extend_from_slice( &[
      cmd.start.x, cmd.start.y,
      cmd.end.x, cmd.end.y,
    ] );

    // Add line colors to batch (start and end colors)
    self.color_buffer.extend_from_slice( &[
      cmd.style.color[ 0 ], cmd.style.color[ 1 ], cmd.style.color[ 2 ], cmd.style.color[ 3 ],
      cmd.style.color[ 0 ], cmd.style.color[ 1 ], cmd.style.color[ 2 ], cmd.style.color[ 3 ],
    ] );

    self.batch_count += 1;
    return Ok( () );
  }

  /// Tessellates a curve into line segments for GPU rendering.
  fn tessellate_curve( &mut self, cmd: &CurveCommand ) -> core::result::Result< (), RenderError >
  {
    // High-quality curve tessellation for smooth GPU rendering
    const CURVE_SEGMENTS: usize = 20; // Higher quality for GPU rendering

    for i in 0..CURVE_SEGMENTS
    {
      let t1 = i as f32 / CURVE_SEGMENTS as f32;
      let t2 = ( i + 1 ) as f32 / CURVE_SEGMENTS as f32;

      // Cubic Bezier curve calculation
      let x1 = Self::cubic_bezier( t1, cmd.start.x, cmd.control1.x, cmd.control2.x, cmd.end.x );
      let y1 = Self::cubic_bezier( t1, cmd.start.y, cmd.control1.y, cmd.control2.y, cmd.end.y );
      let x2 = Self::cubic_bezier( t2, cmd.start.x, cmd.control1.x, cmd.control2.x, cmd.end.x );
      let y2 = Self::cubic_bezier( t2, cmd.start.y, cmd.control1.y, cmd.control2.y, cmd.end.y );

      // Add curve segment to batch
      if self.batch_count >= self.max_batch_size
      {
        self.flush_batch();
      }

      self.vertex_buffer.extend_from_slice( &[ x1, y1, x2, y2 ] );
      self.color_buffer.extend_from_slice( &[
        cmd.style.color[ 0 ], cmd.style.color[ 1 ], cmd.style.color[ 2 ], cmd.style.color[ 3 ],
        cmd.style.color[ 0 ], cmd.style.color[ 1 ], cmd.style.color[ 2 ], cmd.style.color[ 3 ],
      ] );

      self.batch_count += 1;
    }

    return Ok( () );
  }

  /// Calculates cubic Bezier curve point at parameter t.
  fn cubic_bezier( t: f32, p0: f32, p1: f32, p2: f32, p3: f32 ) -> f32
  {
    let u = 1.0 - t;
    u.powi( 3 ) * p0 +
    3.0 * u.powi( 2 ) * t * p1 +
    3.0 * u * t.powi( 2 ) * p2 +
    t.powi( 3 ) * p3
  }

  /// Renders text using GPU-based texture atlas.
  fn render_text_gpu( &mut self, cmd: &TextCommand ) -> core::result::Result< (), RenderError >
  {
    // Mock GPU text rendering
    // In real implementation, this would:
    // 1. Load font atlas texture
    // 2. Calculate glyph positions and UV coordinates
    // 3. Generate quad vertices for each character
    // 4. Add text quads to batch rendering system
    // 5. Handle text anchoring and styling

    // Convert text from [u8; 64] array to string
    let text_end = cmd.text.iter().position( |&b| b == 0 ).unwrap_or( cmd.text.len() );
    let text_str = core::str::from_utf8( &cmd.text[ ..text_end ] ).unwrap_or( "<invalid>" );

    // Mock character processing
    let char_count = text_str.len();
    let char_width = 8.0; // Mock character width in pixels
    let _char_height = 12.0; // Mock character height in pixels

    // Apply text anchoring
    let mut x_offset = 0.0;
    match cmd.anchor
    {
      TextAnchor::TopLeft | TextAnchor::CenterLeft | TextAnchor::BottomLeft => {},
      TextAnchor::TopCenter | TextAnchor::Center | TextAnchor::BottomCenter =>
        x_offset = -( char_count as f32 * char_width ) / 2.0,
      TextAnchor::TopRight | TextAnchor::CenterRight | TextAnchor::BottomRight =>
        x_offset = -( char_count as f32 * char_width ),
    }

    // Mock adding character quads to batch
    for i in 0..char_count
    {
      let x = cmd.position.x + x_offset + ( i as f32 * char_width );
      let y = cmd.position.y;

      // Mock quad vertices (each character as 2 triangles = 6 vertices)
      if self.batch_count >= self.max_batch_size
      {
        self.flush_batch();
      }

      // Add mock character quad (simplified to single line for demonstration)
      self.vertex_buffer.extend_from_slice( &[ x, y, x + char_width, y ] );
      self.color_buffer.extend_from_slice( &[
        cmd.font_style.color[ 0 ], cmd.font_style.color[ 1 ], cmd.font_style.color[ 2 ], cmd.font_style.color[ 3 ],
        cmd.font_style.color[ 0 ], cmd.font_style.color[ 1 ], cmd.font_style.color[ 2 ], cmd.font_style.color[ 3 ],
      ] );

      self.batch_count += 1;
    }

    if let Some( ref mut ctx ) = self.context
    {
      ctx.stats.texture_bindings += 1; // Text rendering uses font atlas texture
    }

    return Ok( () );
  }

  /// Handles WebGL context loss and restoration.
  fn handle_context_loss( &mut self ) -> core::result::Result< (), RenderError >
  {
    if let Some( ref mut ctx ) = self.context
    {
      if ctx.context_lost
      {
        // Mock context restoration
        // In real implementation, this would:
        // 1. Detect context loss event
        // 2. Recreate all GPU resources (shaders, buffers, textures)
        // 3. Restore rendering state
        // 4. Continue rendering seamlessly

        ctx.context_lost = false;
        ctx.line_vbo = Some( 1 );
        ctx.curve_vbo = Some( 2 );
        ctx.active_program = Some( 1 );

        self.compile_shaders();
      }
    }

    return Ok( () );
  }

  /// Implements mouse picking for interactive features.
  pub fn handle_mouse_event( &mut self, x: f32, y: f32 ) -> Option< u32 >
  {
    if !self.mouse_picking_enabled
    {
      return None;
    }

    self.last_mouse_pos = ( x, y );

    // Mock mouse picking implementation
    // In real implementation, this would:
    // 1. Render scene to offscreen buffer with unique colors per primitive
    // 2. Sample pixel at mouse position
    // 3. Map color back to primitive ID
    // 4. Return picked primitive for interaction

    // Mock primitive ID based on position
    if let Some( ref ctx ) = self.context
    {
      if x >= 0.0 && y >= 0.0 && x < ctx.width as f32 && y < ctx.height as f32
      {
        Some( ( x as u32 + y as u32 ) % 1000 ) // Mock primitive ID
      }
      else
      {
        None
      }
    }
    else
    {
      None
    }
  }

  /// Gets current rendering statistics.
  #[ must_use ]
  pub fn get_stats( &self ) -> Option< WebGLStats >
  {
    self.context.as_ref().map( |ctx| ctx.stats.clone() )
  }

  /// Resets frame statistics.
  pub fn reset_stats( &mut self )
  {
    if let Some( ref mut ctx ) = self.context
    {
      ctx.stats = WebGLStats::default();
    }
  }
}

impl Default for WebGLRenderer
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl Renderer for WebGLRenderer
{
  type Output = String; // JSON stats or render info

  #[ inline ]
  fn capabilities( &self ) -> RendererCapabilities
  {
    return RendererCapabilities
    {
      backend_name: "WebGL".to_string(),
      backend_version: "2.0".to_string(),
      max_texture_size: 4_096, // Typical WebGL limit
      supports_transparency: true,
      supports_antialiasing: true,
      supports_custom_fonts: true,
      supports_particles: true,
      supports_realtime: true,
      max_scene_complexity: 100_000, // High complexity for GPU rendering
    };
  }

  fn initialize( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    self.initialize_webgl_context( context )?;
    self.compile_shaders();
    self.initialized = true;
    return Ok( () );
  }

  fn begin_frame( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if !self.initialized
    {
      return Err( RenderError::InitializationFailed( "WebGL renderer not initialized".to_string() ) );
    }

    if self.frame_active
    {
      return Err( RenderError::InvalidContext( "Frame already active".to_string() ) );
    }

    // Handle potential context loss
    self.handle_context_loss()?;

    self.frame_active = true;
    self.render_context = Some( context.clone() );

    // Clear frame statistics
    self.reset_stats();

    // Mock WebGL frame setup
    // In real implementation, this would:
    // 1. Set viewport to canvas size
    // 2. Clear color and depth buffers
    // 3. Set up initial rendering state
    // 4. Enable blending for transparency
    // 5. Set up projection matrix for 2D rendering

    if let Some( ref mut ctx ) = self.context
    {
      ctx.viewport = ( 0, 0, context.width, context.height );
      ctx.width = context.width;
      ctx.height = context.height;
    }

    return Ok( () );
  }

  fn render_scene( &mut self, scene: &Scene ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::InvalidContext( "No active frame".to_string() ) );
    }

    // Process all commands in the scene with batching
    for command in scene.commands()
    {
      match command
      {
        RenderCommand::Line( cmd ) => self.render_line( &cmd )?,
        RenderCommand::Curve( cmd ) => self.render_curve( &cmd )?,
        RenderCommand::Text( cmd ) => self.render_text( &cmd )?,
        RenderCommand::Tilemap( cmd ) => self.render_tilemap( &cmd )?,
        RenderCommand::ParticleEmitter( cmd ) => self.render_particle_emitter( &cmd )?,
        RenderCommand::Geometry2DCommand( _ ) => return Err( RenderError::UnsupportedCommand( "Geometry2DCommand".into() ) ),
        RenderCommand::SpriteCommand( _ ) => return Err( RenderError::UnsupportedCommand( "SpriteCommand".into() ) ),
      }
    }

    // Flush any remaining batched primitives
    self.flush_batch();

    return Ok( () );
  }

  fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::InvalidContext( "No active frame".to_string() ) );
    }

    // Final batch flush and frame completion
    self.flush_batch();

    // Mock WebGL frame completion
    // In real implementation, this would:
    // 1. Present rendered frame to canvas
    // 2. Swap buffers if double-buffered
    // 3. Update performance counters
    // 4. Handle any pending GPU operations

    self.frame_active = false;
    self.render_context = None;

    return Ok( () );
  }

  fn output( &self ) -> core::result::Result< Self::Output, RenderError >
  {
    // Return rendering statistics as JSON
    if let Some( stats ) = self.get_stats()
    {
      Ok( format!(
        r#"{{
  "backend": "WebGL",
  "vertices_rendered": {},
  "draw_calls": {},
  "texture_bindings": {},
  "frame_time_ms": {},
  "context_state": "{}",
  "batch_size": {}
}}"#,
        stats.vertices_rendered,
        stats.draw_calls,
        stats.texture_bindings,
        stats.frame_time_ms,
        if self.context.as_ref().is_some_and( |c| c.context_lost ) { "lost" } else { "active" },
        self.batch_count
      ) )
    }
    else
    {
      Ok( r#"{"backend": "WebGL", "status": "not_initialized"}"#.to_string() )
    }
  }

  fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
  {
    // Mock WebGL cleanup
    // In real implementation, this would:
    // 1. Delete all GPU resources (buffers, textures, shaders)
    // 2. Release WebGL context
    // 3. Clear all internal state

    self.context = None;
    self.initialized = false;
    self.frame_active = false;
    self.render_context = None;
    self.vertex_buffer.clear();
    self.color_buffer.clear();
    self.batch_count = 0;

    return Ok( () );
  }
}

impl PrimitiveRenderer for WebGLRenderer
{
  fn render_line( &mut self, cmd: &LineCommand ) -> core::result::Result< (), RenderError >
  {
    self.add_line_to_batch( cmd )
  }

  fn render_curve( &mut self, cmd: &CurveCommand ) -> core::result::Result< (), RenderError >
  {
    self.tessellate_curve( cmd )
  }

  fn render_text( &mut self, cmd: &TextCommand ) -> core::result::Result< (), RenderError >
  {
    self.render_text_gpu( cmd )
  }

  fn render_tilemap( &mut self, cmd: &TilemapCommand ) -> core::result::Result< (), RenderError >
  {
    // Mock tilemap rendering for WebGL
    // In real implementation, this would:
    // 1. Load tilemap texture atlas
    // 2. Generate instanced quads for each tile
    // 3. Use GPU instancing for high performance
    // 4. Support texture animation and variety

    let tile_count = cmd.tile_count;

    // Mock processing tiles
    for _ in 0..tile_count
    {
      if self.batch_count >= self.max_batch_size
      {
        self.flush_batch();
      }

      // Mock tile quad (simplified)
      self.vertex_buffer.extend_from_slice( &[
        cmd.position.x, cmd.position.y,
        cmd.position.x + cmd.tile_width, cmd.position.y + cmd.tile_height,
      ] );

      self.color_buffer.extend_from_slice( &[ 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0 ] );

      self.batch_count += 1;
    }

    if let Some( ref mut ctx ) = self.context
    {
      ctx.stats.texture_bindings += 1; // Tilemap uses texture atlas
    }

    return Ok( () );
  }

  fn render_particle_emitter( &mut self, cmd: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >
  {
    // Mock GPU-based particle system
    // In real implementation, this would:
    // 1. Use compute shaders for particle simulation
    // 2. Update particle positions on GPU
    // 3. Render particles using instanced rendering
    // 4. Support particle physics and effects

    let particle_count = (cmd.emission_rate * cmd.particle_lifetime).min( 1000.0 ) as usize; // Reasonable limit for demo

    for i in 0..particle_count
    {
      if self.batch_count >= self.max_batch_size
      {
        self.flush_batch();
      }

      // Mock particle position calculation
      let offset_x = ( i as f32 * 0.1 ).sin() * 10.0;
      let offset_y = ( i as f32 * 0.1 ).cos() * 10.0;

      let particle_x = cmd.position.x + offset_x;
      let particle_y = cmd.position.y + offset_y;

      // Mock particle quad
      self.vertex_buffer.extend_from_slice( &[
        particle_x, particle_y,
        particle_x + cmd.particle_size, particle_y + cmd.particle_size,
      ] );

      self.color_buffer.extend_from_slice( &[
        cmd.particle_color[ 0 ], cmd.particle_color[ 1 ], cmd.particle_color[ 2 ], cmd.particle_color[ 3 ],
        cmd.particle_color[ 0 ], cmd.particle_color[ 1 ], cmd.particle_color[ 2 ], cmd.particle_color[ 3 ],
      ] );

      self.batch_count += 1;
    }

    return Ok( () );
  }
}
