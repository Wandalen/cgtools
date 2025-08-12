//! Test suite for rendering port traits and error handling.
//!
//! ## Test Matrix for Ports Module
//!
//! ### Test Factors:
//! - **Trait Implementation**: Mock renderer implementations, trait method coverage
//! - **Error Handling**: RenderError types, error propagation, Display formatting
//! - **Capability Discovery**: Renderer capabilities, command support detection
//! - **Context Management**: RenderContext validation, default values
//! - **Scene Validation**: Scene complexity limits, command support checking

#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::format_push_string ) ]
#![ allow( clippy::single_char_lifetime_names ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::uninlined_format_args ) ]

use tilemap_renderer as the_module;
use the_module::ports::*;
use the_module::scene::*;
use the_module::commands::*;

/// Mock renderer implementation for testing.
struct MockRenderer
{
  capabilities : RendererCapabilities,
  initialized : bool,
  frame_active : bool,
  output_data : String,
}

impl MockRenderer
{
  fn new( backend_name: &str ) -> Self
  {
    let mut capabilities = RendererCapabilities::default();
    capabilities.backend_name = backend_name.to_string();
    capabilities.backend_version = "1.0.0".to_string();
    capabilities.max_texture_size = 1024;
    capabilities.supports_transparency = true;
    capabilities.supports_antialiasing = true;
    capabilities.supports_custom_fonts = false;
    capabilities.supports_particles = false;
    capabilities.supports_realtime = false;
    capabilities.max_scene_complexity = 100;
    
    Self
    {
      capabilities,
      initialized: false,
      frame_active: false,
      output_data: String::new(),
    }
  }
  
  fn with_limited_complexity( mut self, max_complexity: usize ) -> Self
  {
    self.capabilities.max_scene_complexity = max_complexity;
    self
  }
}

impl Renderer for MockRenderer
{
  type Output = String;
  
  fn capabilities( &self ) -> RendererCapabilities
  {
    self.capabilities.clone()
  }
  
  fn initialize( &mut self, _context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if self.initialized
    {
      return Err( RenderError::InitializationFailed( "Already initialized".to_string() ) );
    }
    self.initialized = true;
    Ok( () )
  }
  
  fn begin_frame( &mut self, _context: &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if !self.initialized
    {
      return Err( RenderError::InvalidContext( "Not initialized".to_string() ) );
    }
    if self.frame_active
    {
      return Err( RenderError::RenderFailed( "Frame already active".to_string() ) );
    }
    self.frame_active = true;
    self.output_data.clear();
    self.output_data.push_str( "Frame started\n" );
    Ok( () )
  }
  
  fn render_scene( &mut self, scene: &Scene ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    
    self.validate_scene( scene )?;
    
    self.output_data.push_str( &format!( "Rendering {} commands\n", scene.len() ) );
    
    for command in scene.commands()
    {
      match command
      {
        RenderCommand::Line( _ ) => self.output_data.push_str( "Line rendered\n" ),
        RenderCommand::Curve( _ ) => self.output_data.push_str( "Curve rendered\n" ),
        RenderCommand::Text( _ ) => self.output_data.push_str( "Text rendered\n" ),
        RenderCommand::Tilemap( _ ) => return Err( RenderError::UnsupportedCommand( "Tilemap".to_string() ) ),
        RenderCommand::ParticleEmitter( _ ) => return Err( RenderError::UnsupportedCommand( "ParticleEmitter".to_string() ) ),
        _ => return Err( RenderError::UnsupportedCommand( "Unknown".to_string() ) ),
      }
    }
    
    Ok( () )
  }
  
  fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }
    self.frame_active = false;
    self.output_data.push_str( "Frame ended\n" );
    Ok( () )
  }
  
  fn output( &self ) -> core::result::Result< Self::Output, RenderError >
  {
    if self.frame_active
    {
      return Err( RenderError::OutputError( "Frame still active".to_string() ) );
    }
    if !self.initialized
    {
      return Err( RenderError::OutputError( "Not initialized".to_string() ) );
    }
    Ok( self.output_data.clone() )
  }
  
  fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
  {
    self.initialized = false;
    self.frame_active = false;
    self.output_data.clear();
    Ok( () )
  }
  
  fn supports_tilemaps( &self ) -> bool
  {
    false
  }
  
  fn supports_particles( &self ) -> bool
  {
    false
  }
}

/// Mock primitive renderer implementation for testing.
struct MockPrimitiveRenderer
{
  rendered_commands : Vec< String >,
}

impl MockPrimitiveRenderer
{
  fn new() -> Self
  {
    Self
    {
      rendered_commands: Vec::new(),
    }
  }
}

impl PrimitiveRenderer for MockPrimitiveRenderer
{
  fn render_line( &mut self, command: &LineCommand ) -> core::result::Result< (), RenderError >
  {
    self.rendered_commands.push( format!(
      "Line: ({},{}) -> ({},{})",
      command.start.x, command.start.y,
      command.end.x, command.end.y
    ) );
    Ok( () )
  }
  
  fn render_curve( &mut self, command: &CurveCommand ) -> core::result::Result< (), RenderError >
  {
    self.rendered_commands.push( format!(
      "Curve: ({},{}) -> ({},{})",
      command.start.x, command.start.y,
      command.end.x, command.end.y
    ) );
    Ok( () )
  }
  
  fn render_text( &mut self, command: &TextCommand ) -> core::result::Result< (), RenderError >
  {
    self.rendered_commands.push( format!(
      "Text: '{}' at ({},{})",
      command.text(),
      command.position.x, command.position.y
    ) );
    Ok( () )
  }
  
  fn render_tilemap( &mut self, _command: &TilemapCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::FeatureNotImplemented( "Tilemap rendering".to_string() ) )
  }
  
  fn render_particle_emitter( &mut self, _command: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::FeatureNotImplemented( "Particle rendering".to_string() ) )
  }
}

/// Tests basic renderer capabilities.
/// Test Focus: Capability discovery and reporting
#[ test ]
fn test_renderer_capabilities()
{
  let renderer = MockRenderer::new( "TestBackend" );
  let caps = renderer.capabilities();
  
  assert_eq!( caps.backend_name, "TestBackend" );
  assert_eq!( caps.backend_version, "1.0.0" );
  assert_eq!( caps.max_texture_size, 1024 );
  assert!( caps.supports_transparency );
  assert!( caps.supports_antialiasing );
  assert!( !caps.supports_custom_fonts );
  assert!( !caps.supports_particles );
  assert_eq!( caps.max_scene_complexity, 100 );
}

/// Tests renderer initialization lifecycle.
/// Test Focus: Proper initialization state management
#[ test ]
fn test_renderer_initialization()
{
  let mut renderer = MockRenderer::new( "TestBackend" );
  let context = RenderContext::default();
  
  // Should initialize successfully
  assert!( renderer.initialize( &context ).is_ok() );
  
  // Double initialization should fail
  assert!( renderer.initialize( &context ).is_err() );
  
  // Cleanup should reset state
  assert!( renderer.cleanup().is_ok() );
  assert!( renderer.initialize( &context ).is_ok() );
}

/// Tests complete rendering lifecycle.
/// Test Focus: Frame rendering workflow
#[ test ]
fn test_rendering_lifecycle()
{
  let mut renderer = MockRenderer::new( "TestBackend" );
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  // Add a simple line command
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  // Complete rendering lifecycle
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  assert!( output.contains( "Frame started" ) );
  assert!( output.contains( "Rendering 1 commands" ) );
  assert!( output.contains( "Line rendered" ) );
  assert!( output.contains( "Frame ended" ) );
}

/// Tests command support detection.
/// Test Focus: Capability-based command filtering
#[ test ]
fn test_command_support_detection()
{
  let renderer = MockRenderer::new( "TestBackend" );
  
  let line_cmd = RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } );
  
  let tilemap_cmd = RenderCommand::Tilemap( TilemapCommand::new(
    Point2D::default(),
    32.0, 32.0, 2, 2, 0,
    &[ 1, 2, 3, 4 ]
  ) );
  
  assert!( renderer.supports_command( &line_cmd ) );
  assert!( !renderer.supports_command( &tilemap_cmd ) );
  
  assert!( renderer.supports_lines() );
  assert!( renderer.supports_curves() );
  assert!( renderer.supports_text() );
  assert!( !renderer.supports_tilemaps() );
  assert!( !renderer.supports_particles() );
}

/// Tests scene validation with complexity limits.
/// Test Focus: Scene complexity checking
#[ test ]
fn test_scene_complexity_validation()
{
  let renderer = MockRenderer::new( "TestBackend" ).with_limited_complexity( 3 );
  let mut scene = Scene::new();
  
  // Add commands within limit
  for _ in 0..3
  {
    scene.add( RenderCommand::Line( LineCommand
    {
      start: Point2D::default(),
      end: Point2D { x: 10.0, y: 10.0 },
      style: StrokeStyle::default(),
    } ) );
  }
  
  assert!( renderer.validate_scene( &scene ).is_ok() );
  
  // Add one more command to exceed limit
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 20.0, y: 20.0 },
    style: StrokeStyle::default(),
  } ) );
  
  let result = renderer.validate_scene( &scene );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::ComplexityLimitExceeded ) );
}

/// Tests scene validation with unsupported commands.
/// Test Focus: Command support validation
#[ test ]
fn test_unsupported_command_validation()
{
  let renderer = MockRenderer::new( "TestBackend" );
  let mut scene = Scene::new();
  
  // Add supported command
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  // Add unsupported command
  scene.add( RenderCommand::Tilemap( TilemapCommand::new(
    Point2D::default(),
    32.0, 32.0, 2, 2, 0,
    &[ 1, 2, 3, 4 ]
  ) ) );
  
  let result = renderer.validate_scene( &scene );
  assert!( result.is_err() );
  
  if let Err( RenderError::UnsupportedCommand( cmd ) ) = result
  {
    assert_eq!( cmd, "Tilemap" );
  }
  else
  {
    panic!( "Expected UnsupportedCommand error" );
  }
}

/// Tests error handling during rendering.
/// Test Focus: Error propagation and state management
#[ test ]
fn test_rendering_error_handling()
{
  let mut renderer = MockRenderer::new( "TestBackend" );
  let context = RenderContext::default();
  let scene = Scene::new();
  
  // Begin frame without initialization should fail
  let result = renderer.begin_frame( &context );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::InvalidContext( _ ) ) );
  
  // Initialize and try to begin frame twice
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  let result = renderer.begin_frame( &context );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::RenderFailed( _ ) ) );
  
  // End frame and try to render without active frame
  assert!( renderer.end_frame().is_ok() );
  let result = renderer.render_scene( &scene );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::RenderFailed( _ ) ) );
}

/// Tests primitive renderer implementation.
/// Test Focus: PrimitiveRenderer trait functionality
#[ test ]
fn test_primitive_renderer()
{
  let mut primitive_renderer = MockPrimitiveRenderer::new();
  
  let line_cmd = LineCommand
  {
    start: Point2D { x: 1.0, y: 2.0 },
    end: Point2D { x: 3.0, y: 4.0 },
    style: StrokeStyle::default(),
  };
  
  let text_cmd = TextCommand::new(
    "Hello", 
    Point2D { x: 10.0, y: 20.0 },
    FontStyle::default(),
    TextAnchor::Center
  );
  
  assert!( primitive_renderer.render_line( &line_cmd ).is_ok() );
  assert!( primitive_renderer.render_text( &text_cmd ).is_ok() );
  
  assert_eq!( primitive_renderer.rendered_commands.len(), 2 );
  assert!( primitive_renderer.rendered_commands[ 0 ].contains( "Line: (1,2) -> (3,4)" ) );
  assert!( primitive_renderer.rendered_commands[ 1 ].contains( "Text: 'Hello' at (10,20)" ) );
}

/// Tests render command dispatching.
/// Test Focus: Command dispatch to primitive renderers
#[ test ]
fn test_render_command_dispatching()
{
  let mut primitive_renderer = MockPrimitiveRenderer::new();
  
  let line_command = RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 5.0, y: 5.0 },
    style: StrokeStyle::default(),
  } );
  
  let tilemap_command = RenderCommand::Tilemap( TilemapCommand::new(
    Point2D::default(),
    16.0, 16.0, 1, 1, 0,
    &[ 1 ]
  ) );
  
  // Supported command should work
  assert!( primitive_renderer.render_command( &line_command ).is_ok() );
  
  // Unsupported command should return error
  let result = primitive_renderer.render_command( &tilemap_command );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::FeatureNotImplemented( _ ) ) );
}

/// Tests RenderError display formatting.
/// Test Focus: Error message formatting and Display trait
#[ test ]
fn test_render_error_display()
{
  let errors = vec![
    RenderError::RenderFailed( "test failure".to_string() ),
    RenderError::UnsupportedCommand( "TestCommand".to_string() ),
    RenderError::InvalidContext( "invalid params".to_string() ),
    RenderError::InitializationFailed( "init error".to_string() ),
    RenderError::ResourceAllocationFailed( "out of memory".to_string() ),
    RenderError::OutputError( "write failed".to_string() ),
    RenderError::ComplexityLimitExceeded,
    RenderError::FeatureNotImplemented( "feature_x".to_string() ),
  ];
  
  for error in errors
  {
    let display_str = format!( "{}", error );
    assert!( !display_str.is_empty() );
    assert!( display_str.len() > 5 ); // Sanity check for meaningful messages
  }
}

/// Tests default render context values.
/// Test Focus: RenderContext default implementation
#[ test ]
fn test_render_context_defaults()
{
  let context = RenderContext::default();
  
  assert_eq!( context.width, 800 );
  assert_eq!( context.height, 600 );
  assert_eq!( context.background_color, [ 1.0, 1.0, 1.0, 1.0 ] ); // White
  assert!( context.clear_background );
  assert_eq!( context.viewport_offset.x, 0.0 );
  assert_eq!( context.viewport_offset.y, 0.0 );
  assert_eq!( context.viewport_scale, 1.0 );
}

/// Tests default renderer capabilities.
/// Test Focus: RendererCapabilities default implementation
#[ test ]
fn test_renderer_capabilities_defaults()
{
  let caps = RendererCapabilities::default();
  
  assert_eq!( caps.backend_name, "Unknown" );
  assert_eq!( caps.backend_version, "0.0.0" );
  assert_eq!( caps.max_texture_size, 0 );
  assert!( !caps.supports_transparency );
  assert!( !caps.supports_antialiasing );
  assert!( !caps.supports_custom_fonts );
  assert!( !caps.supports_particles );
  assert!( !caps.supports_realtime );
  assert_eq!( caps.max_scene_complexity, 1000 );
}

/// Tests RenderError equality and cloning.
/// Test Focus: RenderError Clone and PartialEq traits
#[ test ]
fn test_render_error_clone_and_equality()
{
  let error1 = RenderError::RenderFailed( "test".to_string() );
  let error2 = error1.clone();
  let error3 = RenderError::ComplexityLimitExceeded;
  
  assert_eq!( error1, error2 );
  assert_ne!( error1, error3 );
}

/// Tests async renderer trait default implementation.
/// Test Focus: AsyncRenderer trait default methods
#[ tokio::test ]
async fn test_async_renderer_defaults()
{
  struct TestAsyncRenderer
  {
    mock : MockRenderer,
  }
  
  impl Renderer for TestAsyncRenderer
  {
    type Output = String;
    
    fn capabilities( &self ) -> RendererCapabilities
    {
      self.mock.capabilities()
    }
    
    fn initialize( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
    {
      self.mock.initialize( context )
    }
    
    fn begin_frame( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >
    {
      self.mock.begin_frame( context )
    }
    
    fn render_scene( &mut self, scene: &Scene ) -> core::result::Result< (), RenderError >
    {
      self.mock.render_scene( scene )
    }
    
    fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
    {
      self.mock.end_frame()
    }
    
    fn output( &self ) -> core::result::Result< Self::Output, RenderError >
    {
      self.mock.output()
    }
    
    fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
    {
      self.mock.cleanup()
    }
  }
  
  impl AsyncRenderer for TestAsyncRenderer {}
  
  let mut async_renderer = TestAsyncRenderer
  {
    mock: MockRenderer::new( "AsyncTest" ),
  };
  
  let context = RenderContext::default();
  let scene = Scene::new();
  
  // Test async methods use default implementations
  assert!( async_renderer.initialize( &context ).is_ok() );
  assert!( async_renderer.begin_frame( &context ).is_ok() );
  assert!( async_renderer.render_scene_async( &scene ).await.is_ok() );
  assert!( async_renderer.end_frame().is_ok() );
  
  let output = async_renderer.output_async().await.unwrap();
  assert!( !output.is_empty() );
}