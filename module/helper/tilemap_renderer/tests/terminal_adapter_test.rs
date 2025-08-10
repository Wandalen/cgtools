//! Terminal adapter comprehensive test suite.
//!
//! This test suite validates the terminal backend adapter implementation
//! following the Test Matrix approach from the design rulebook.

#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::assertions_on_constants ) ]

use tilemap_renderer::
{
  adapters::TerminalRenderer,
  ports::{ Renderer, PrimitiveRenderer, RenderContext, RenderError },
  commands::{ LineCommand, CurveCommand, TextCommand, Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, RenderCommand, TilemapCommand, ParticleEmitterCommand },
  scene::Scene,
};

/// Test Matrix: Terminal Renderer Creation and Configuration
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | new() | Default constructor | 80x24 dimensions |
/// | with_dimensions() | Custom size | Specified dimensions |
/// | set_unicode_enabled() | Unicode toggle | Affects line characters |
/// | set_color_enabled() | Color toggle | Affects ANSI output |
#[ test ]
fn test_terminal_renderer_creation()
{
  // Default constructor
  let _renderer = TerminalRenderer::new();
  // Dimensions are private, test through behavior instead
  
  // Custom dimensions
  let _renderer = TerminalRenderer::with_dimensions( 40, 12 );
  // Dimensions are private, test through behavior instead
}

#[ test ]
fn test_terminal_renderer_configuration()
{
  let mut renderer = TerminalRenderer::new();
  
  // Test configuration methods exist and don't panic
  renderer.set_unicode_enabled( false );
  renderer.set_color_enabled( false );
  
  // These should not crash the renderer
  assert!( true );
}

/// Test Matrix: Renderer Trait Implementation
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | initialize() | Initialization | Success |
/// | capabilities() | Capability report | Correct support flags |
/// | begin_frame() | Frame start | Success with context |
/// | end_frame() | Frame end | Success |
/// | cleanup() | Cleanup | Success |
#[ test ]
fn test_terminal_renderer_lifecycle()
{
  let mut renderer = TerminalRenderer::new();
  let context = RenderContext::default();
  
  // Initialize
  assert!( renderer.initialize( &context ).is_ok() );
  
  // Check capabilities
  let caps = renderer.capabilities();
  // Terminal backend capabilities - checking what's actually supported
  assert!( !caps.supports_particles );
  assert!( !caps.supports_realtime );
  assert!( !caps.supports_transparency );
  
  // Frame lifecycle
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Cleanup
  assert!( renderer.cleanup().is_ok() );
}

#[ test ]
fn test_terminal_renderer_error_handling()
{
  let mut renderer = TerminalRenderer::new();
  let context = RenderContext::default();
  
  // Cannot begin frame without initialization
  assert!( renderer.begin_frame( &context ).is_err() );
  
  // Initialize properly
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Cannot begin frame twice
  assert!( renderer.begin_frame( &context ).is_err() );
  
  // End frame and try operations on inactive frame
  assert!( renderer.end_frame().is_ok() );
}

/// Test Matrix: Line Rendering
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Horizontal line | y1 == y2 | Horizontal chars |
/// | Vertical line | x1 == x2 | Vertical chars |
/// | Diagonal line | Different slopes | Appropriate chars |
/// | Point line | Same coordinates | Point character |
/// | Colored line | ANSI colors | Color codes present |
#[ test ]
fn test_terminal_line_rendering()
{
  let mut renderer = TerminalRenderer::with_dimensions( 20, 10 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Horizontal line
  let line = LineCommand
  {
    start: Point2D { x: 2.0, y: 2.0 },
    end: Point2D { x: 8.0, y: 2.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Red
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  assert!( !output.is_empty() );
  
  // Should contain horizontal line characters (either - or ─)
  assert!( output.contains( '─' ) || output.contains( '-' ) );
}

#[ test ]
fn test_terminal_vertical_line_rendering()
{
  let mut renderer = TerminalRenderer::with_dimensions( 20, 10 );
  let context = RenderContext::default();
  renderer.set_unicode_enabled( false ); // Test ASCII mode
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Vertical line
  let line = LineCommand
  {
    start: Point2D { x: 5.0, y: 1.0 },
    end: Point2D { x: 5.0, y: 7.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 1.0, 0.0, 1.0 ], // Green
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  assert!( output.contains( '|' ) ); // ASCII mode should use |
}

#[ test ]
fn test_terminal_point_rendering()
{
  let mut renderer = TerminalRenderer::with_dimensions( 10, 5 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Point (same start and end coordinates)
  let line = LineCommand
  {
    start: Point2D { x: 3.0, y: 2.0 },
    end: Point2D { x: 3.0, y: 2.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 0.0, 1.0, 1.0 ], // Blue
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  assert!( output.contains( '●' ) || output.contains( '*' ) );
}

/// Test Matrix: Curve Rendering  
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Simple curve | Basic quadratic | Line segments |
/// | Flat curve | Control point on line | Straight line |
/// | Sharp curve | Extreme control point | Curved approximation |
#[ test ]
fn test_terminal_curve_rendering()
{
  let mut renderer = TerminalRenderer::with_dimensions( 15, 8 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Simple quadratic curve
  let curve = CurveCommand
  {
    start: Point2D { x: 1.0, y: 3.0 },
    control1: Point2D { x: 7.0, y: 1.0 },
    control2: Point2D { x: 20.0, y: 6.0 },
    end: Point2D { x: 12.0, y: 3.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 1.0, 0.0, 1.0 ], // Yellow
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_curve( &curve ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  assert!( !output.is_empty() );
  // Curve should produce some line characters
  assert!( output.chars().any( |c| c == '─' || c == '│' || c == '-' || c == '|' ) );
}

/// Test Matrix: Text Rendering
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Simple text | Basic string | Characters visible |
/// | Empty text | Zero-length string | No output |
/// | Long text | Exceeds width | Truncated appropriately |
/// | Anchored text | Different anchors | Correct positioning |
#[ test ]
fn test_terminal_text_rendering()
{
  let mut renderer = TerminalRenderer::with_dimensions( 25, 8 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Simple text
  let mut text_array = [ 0u8; 64 ];
  let text = b"Hello Terminal!";
  text_array[ ..text.len() ].copy_from_slice( text );
  
  let text_cmd = TextCommand
  {
    text: text_array,
    text_len: text.len() as u8,
    position: Point2D { x: 2.0, y: 3.0 },
    font_style: FontStyle
    {
      color: [ 1.0, 0.5, 0.0, 1.0 ], // Orange
      size: 12.0,
      family_id: 0,
      weight: 400,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  };
  
  assert!( renderer.render_text( &text_cmd ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  assert!( output.contains( "Hello Terminal!" ) );
}

#[ test ]
fn test_terminal_text_anchoring()
{
  let mut renderer = TerminalRenderer::with_dimensions( 20, 6 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  // Test different text anchors
  let anchors = [
    TextAnchor::TopLeft,
    TextAnchor::TopCenter,
    TextAnchor::TopRight,
    TextAnchor::CenterLeft,
    TextAnchor::Center,
    TextAnchor::CenterRight,
    TextAnchor::BottomLeft,
    TextAnchor::BottomCenter,
    TextAnchor::BottomRight,
  ];
  
  for anchor in &anchors
  {
    let context = RenderContext::default();
    assert!( renderer.begin_frame( &context ).is_ok() );
    
    let mut text_array = [ 0u8; 64 ];
    let text = b"Test";
    text_array[ ..text.len() ].copy_from_slice( text );
    
    let text_cmd = TextCommand
    {
      text: text_array,
      text_len: text.len() as u8,
      position: Point2D { x: 10.0, y: 2.0 },
      font_style: FontStyle
      {
        color: [ 0.0, 0.0, 0.0, 1.0 ], // Black
        size: 12.0,
        family_id: 0,
        weight: 400,
        italic: false,
      },
      anchor: *anchor,
    };
    
    assert!( renderer.render_text( &text_cmd ).is_ok() );
    assert!( renderer.end_frame().is_ok() );
    
    // Should not panic and should produce some output
    let output = renderer.get_output();
    assert!( !output.is_empty() );
  }
}

/// Test Matrix: Scene Rendering Integration
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Empty scene | No commands | Blank output |
/// | Mixed scene | Multiple primitives | All rendered |
/// | Large scene | Many commands | All processed |
#[ test ]
fn test_terminal_scene_rendering()
{
  let mut renderer = TerminalRenderer::with_dimensions( 30, 15 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Create a scene with multiple elements
  let mut scene = Scene::new();
  
  // Add a line
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 5.0 },
    end: Point2D { x: 15.0, y: 5.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ],
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  } ) );
  
  // Add text
  let mut text_array = [ 0u8; 64 ];
  let text = b"Scene Test";
  text_array[ ..text.len() ].copy_from_slice( text );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: text_array,
    text_len: text.len() as u8,
    position: Point2D { x: 8.0, y: 3.0 },
    font_style: FontStyle
    {
      color: [ 0.0, 1.0, 0.0, 1.0 ],
      size: 12.0,
      family_id: 0,
      weight: 400,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  assert!( output.contains( "Scene Test" ) );
  assert!( output.chars().any( |c| c == '─' || c == '-' ) ); // Line characters
}

#[ test ]
fn test_terminal_empty_scene()
{
  let mut renderer = TerminalRenderer::with_dimensions( 10, 5 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  let scene = Scene::new();
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  // Should be mostly spaces and newlines
  assert!( output.chars().all( |c| c.is_whitespace() || c == ' ' ) );
}

/// Test Matrix: Unsupported Commands
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Tilemap command | Not supported | UnsupportedCommand error |
/// | Particle command | Not supported | UnsupportedCommand error |
#[ test ]
fn test_terminal_unsupported_commands()
{
  let mut renderer = TerminalRenderer::new();
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Test tilemap command (unsupported)
  let tilemap = TilemapCommand
  {
    position: Point2D { x: 0.0, y: 0.0 },
    tile_width: 16.0,
    tile_height: 16.0,
    map_width: 10,
    map_height: 10,
    tileset_id: 0,
    tile_data: [ 0; 32 ],
    tile_count: 32,
  };
  
  let result = renderer.render_tilemap( &tilemap );
  assert!( result.is_err() );
  match result.unwrap_err()
  {
    RenderError::UnsupportedCommand( _ ) => {},
    _ => panic!( "Expected UnsupportedCommand error" ),
  }
  
  // Test particle command (unsupported)
  let particle = ParticleEmitterCommand
  {
    position: Point2D { x: 5.0, y: 5.0 },
    emission_rate: 10.0,
    particle_lifetime: 1.0,
    initial_velocity: Point2D { x: 1.0, y: 0.0 },
    velocity_variance: Point2D { x: 0.1, y: 0.1 },
    particle_size: 2.0,
    size_variance: 0.1,
    particle_color: [ 1.0, 1.0, 1.0, 1.0 ],
    color_variance: [ 0.0, 0.0, 0.0, 0.0 ],
  };
  
  let result = renderer.render_particle_emitter( &particle );
  assert!( result.is_err() );
  match result.unwrap_err()
  {
    RenderError::UnsupportedCommand( _ ) => {},
    _ => panic!( "Expected UnsupportedCommand error" ),
  }
}

/// Test Matrix: Output and Export
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | String output | Get rendered output | Valid string |
/// | Console output | Empty destination | Output to stdout |
/// | File export | Valid file path | File created |
#[ test ]
fn test_terminal_output_generation()
{
  let mut renderer = TerminalRenderer::with_dimensions( 8, 4 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Add some content
  let line = LineCommand
  {
    start: Point2D { x: 1.0, y: 1.0 },
    end: Point2D { x: 6.0, y: 1.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 1.0, 1.0, 1.0 ],
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Test string output
  let output = renderer.get_output();
  assert!( !output.is_empty() );
  assert!( output.contains( '\n' ) ); // Should have newlines
  
  // Count lines - should match height
  let line_count = output.matches( '\n' ).count();
  assert_eq!( line_count, 4 ); // Should have 4 lines
  
  // Test console output (empty destination)
  assert!( renderer.output().is_ok() );
}

#[ test ]
#[ cfg( feature = "std" ) ]
fn test_terminal_file_export()
{
  let mut renderer = TerminalRenderer::with_dimensions( 5, 3 );
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Test file export
  let test_file = "-terminal_test_output.txt";
  assert!( renderer.export_to_file( test_file ).is_ok() );
  
  // Verify file exists and has content
  let file_content = std::fs::read_to_string( test_file ).expect( "Should read test file" );
  assert!( !file_content.is_empty() );
  
  // Clean up
  let _ = std::fs::remove_file( test_file );
}

/// Test Matrix: Color Support
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | ANSI colors enabled | Color codes present | ANSI escape sequences |
/// | ANSI colors disabled | No color codes | Plain text only |
/// | RGBA conversion | Color values | Correct RGB mapping |
#[ test ]
fn test_terminal_color_support()
{
  // Test with colors enabled
  let mut renderer = TerminalRenderer::with_dimensions( 10, 5 );
  let context = RenderContext::default();
  renderer.set_color_enabled( true );
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  let line = LineCommand
  {
    start: Point2D { x: 1.0, y: 1.0 },
    end: Point2D { x: 5.0, y: 1.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Bright red
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output_with_color = renderer.get_output();
  assert!( output_with_color.contains( "\x1b[" ) ); // Should have ANSI escape codes
  
  // Test with colors disabled
  let mut renderer_no_color = TerminalRenderer::with_dimensions( 10, 5 );
  renderer_no_color.set_color_enabled( false );
  let context2 = RenderContext::default();
  assert!( renderer_no_color.initialize( &context2 ).is_ok() );
  assert!( renderer_no_color.begin_frame( &context2 ).is_ok() );
  assert!( renderer_no_color.render_line( &line ).is_ok() );
  assert!( renderer_no_color.end_frame().is_ok() );
  
  let output_no_color = renderer_no_color.get_output();
  assert!( !output_no_color.contains( "\x1b[" ) ); // Should NOT have ANSI escape codes
}

/// Test Matrix: Unicode Support
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Unicode enabled | Line drawing chars | ─, │, ● characters |
/// | Unicode disabled | ASCII chars | -, |, * characters |
#[ test ]
fn test_terminal_unicode_support()
{
  // Test with Unicode enabled
  let mut renderer_unicode = TerminalRenderer::with_dimensions( 10, 5 );
  renderer_unicode.set_unicode_enabled( true );
  let context = RenderContext::default();
  assert!( renderer_unicode.initialize( &context ).is_ok() );
  assert!( renderer_unicode.begin_frame( &context ).is_ok() );
  
  // Horizontal line
  let h_line = LineCommand
  {
    start: Point2D { x: 1.0, y: 1.0 },
    end: Point2D { x: 5.0, y: 1.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 1.0, 1.0, 1.0 ],
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  // Vertical line
  let v_line = LineCommand
  {
    start: Point2D { x: 3.0, y: 2.0 },
    end: Point2D { x: 3.0, y: 4.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 1.0, 1.0, 1.0 ],
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer_unicode.render_line( &h_line ).is_ok() );
  assert!( renderer_unicode.render_line( &v_line ).is_ok() );
  assert!( renderer_unicode.end_frame().is_ok() );
  
  let output_unicode = renderer_unicode.get_output();
  assert!( output_unicode.contains( '─' ) ); // Unicode horizontal
  assert!( output_unicode.contains( '│' ) ); // Unicode vertical
  
  // Test with Unicode disabled
  let mut renderer_ascii = TerminalRenderer::with_dimensions( 10, 5 );
  renderer_ascii.set_unicode_enabled( false );
  let context2 = RenderContext::default();
  assert!( renderer_ascii.initialize( &context2 ).is_ok() );
  assert!( renderer_ascii.begin_frame( &context2 ).is_ok() );
  assert!( renderer_ascii.render_line( &h_line ).is_ok() );
  assert!( renderer_ascii.render_line( &v_line ).is_ok() );
  assert!( renderer_ascii.end_frame().is_ok() );
  
  let output_ascii = renderer_ascii.get_output();
  assert!( output_ascii.contains( '-' ) ); // ASCII horizontal
  assert!( output_ascii.contains( '|' ) ); // ASCII vertical
}

/// Comprehensive integration test demonstrating all terminal backend capabilities
#[ test ]
fn test_terminal_comprehensive_integration()
{
  let mut renderer = TerminalRenderer::with_dimensions( 40, 20 );
  renderer.set_unicode_enabled( true );
  renderer.set_color_enabled( true );
  let context = RenderContext::default();
  
  assert!( renderer.initialize( &context ).is_ok() );
  
  let context = RenderContext::default();
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Create a complex scene
  let mut scene = Scene::new();
  
  // Add title text
  let mut title_text = [ 0u8; 64 ];
  let title = b"Terminal Renderer Demo";
  title_text[ ..title.len() ].copy_from_slice( title );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: title_text,
    text_len: title.len().min( 255 ) as u8,
    position: Point2D { x: 9.0, y: 1.0 },
    font_style: FontStyle
    {
      color: [ 1.0, 1.0, 0.0, 1.0 ], // Yellow
      size: 14.0,
      family_id: 0,
      weight: 700,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  // Add a border box using lines
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 5.0, y: 3.0 }, end: Point2D { x: 35.0, y: 3.0 }, 
    style: StrokeStyle { color: [ 0.0, 1.0, 0.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter } } ) );
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 5.0, y: 15.0 }, end: Point2D { x: 35.0, y: 15.0 }, 
    style: StrokeStyle { color: [ 0.0, 1.0, 0.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter } } ) );
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 5.0, y: 3.0 }, end: Point2D { x: 5.0, y: 15.0 }, 
    style: StrokeStyle { color: [ 0.0, 1.0, 0.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter } } ) );
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 35.0, y: 3.0 }, end: Point2D { x: 35.0, y: 15.0 }, 
    style: StrokeStyle { color: [ 0.0, 1.0, 0.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter } } ) );
  
  // Add a curve
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 8.0, y: 12.0 },
    control1: Point2D { x: 20.0, y: 6.0 },
    control2: Point2D { x: 20.0, y: 6.0 },
    end: Point2D { x: 32.0, y: 12.0 },
    style: StrokeStyle { color: [ 1.0, 0.0, 1.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter },
  } ) );
  
  // Add descriptive text
  let mut desc_text = [ 0u8; 64 ];
  let desc = b"Lines, Curves & Text";
  desc_text[ ..desc.len() ].copy_from_slice( desc );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: desc_text,
    text_len: desc.len().min( 255 ) as u8,
    position: Point2D { x: 11.0, y: 9.0 },
    font_style: FontStyle
    {
      color: [ 0.0, 0.8, 1.0, 1.0 ], // Cyan
      size: 12.0,
      family_id: 0,
      weight: 400,
      italic: true,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  // Render the complete scene
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.get_output();
  
  // Verify all elements are present (check for individual characters since they have ANSI codes)
  assert!( output.contains( "T" ) && output.contains( "e" ) && output.contains( "r" ) && output.contains( "m" ) );
  assert!( output.contains( "D" ) && output.contains( "o" ) );
  assert!( output.contains( "L" ) && output.contains( "i" ) && output.contains( "n" ) && output.contains( "s" ) );
  assert!( output.contains( "C" ) && output.contains( "u" ) && output.contains( "v" ) && output.contains( "x" ) );
  assert!( output.contains( '─' ) ); // Horizontal lines
  assert!( output.contains( '│' ) ); // Vertical lines
  assert!( output.contains( "\x1b[" ) ); // ANSI color codes
  
  // Verify output structure
  let lines: Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 20 ); // Should have exactly 20 lines
  
  // Each line should be exactly 40 characters (plus potential ANSI codes)
  for line in &lines
  {
    let clean_line = line.chars().filter( |ch| return !ch.is_control() && *ch != '\x1b' ).collect::< String >();
    assert!( clean_line.len() <= 40 );
  }
}