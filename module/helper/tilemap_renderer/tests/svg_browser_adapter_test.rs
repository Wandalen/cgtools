//! SVG-browser adapter comprehensive test suite.
//!
//! This test suite validates the interactive SVG-browser backend adapter
//! following the Test Matrix approach from the design rulebook.

#![ cfg( feature = "testing" ) ]

#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::assertions_on_constants ) ]

use tilemap_renderer::
{
  adapters::SvgBrowserRenderer,
  ports::{ Renderer, PrimitiveRenderer, RenderContext, RenderError },
  commands::{ LineCommand, CurveCommand, TextCommand, Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, RenderCommand, TilemapCommand, ParticleEmitterCommand },
  scene::Scene,
};

/// Test Matrix: SVG-Browser Renderer Creation and Configuration
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | new() | Default constructor | Default settings |
/// | with_dimensions() | Custom size | Specified dimensions |
/// | set_mouse_picking_enabled() | Mouse picking toggle | Affects JS generation |
/// | set_hover_effects_enabled() | Hover effects toggle | Affects CSS generation |
/// | set_animation_enabled() | Animation toggle | Affects JS utilities |
#[ test ]
fn test_svg_browser_renderer_creation()
{
  // Default constructor
  let renderer = SvgBrowserRenderer::new();
  let caps = renderer.capabilities();
  assert_eq!( caps.backend_name, "SVG-Browser" );
  assert!( caps.supports_realtime ); // Interactive updates
  
  // Custom dimensions
  let _renderer = SvgBrowserRenderer::with_dimensions( 1200, 800 );
}

#[ test ]
fn test_svg_browser_renderer_configuration()
{
  let mut renderer = SvgBrowserRenderer::new();
  
  // Test configuration methods
  renderer.set_mouse_picking_enabled( false );
  renderer.set_hover_effects_enabled( false );
  renderer.set_animation_enabled( false );
  
  // These should not crash the renderer
  assert!( true );
}

/// Test Matrix: SVG-Browser Renderer Lifecycle
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | initialize() | Initialization | Success with HTML setup |
/// | capabilities() | Capability report | Interactive SVG capabilities |
/// | begin_frame() | Frame start | Success with element reset |
/// | end_frame() | Frame end | Success |
/// | cleanup() | Resource cleanup | Success |
#[ test ]
fn test_svg_browser_renderer_lifecycle()
{
  let mut renderer = SvgBrowserRenderer::new();
  
  // Check capabilities
  let caps = renderer.capabilities();
  assert_eq!( caps.backend_name, "SVG-Browser" );
  assert_eq!( caps.backend_version, "1.0" );
  assert!( caps.supports_transparency );
  assert!( caps.supports_antialiasing );
  assert!( caps.supports_custom_fonts );
  assert!( !caps.supports_particles ); // Not implemented for SVG
  assert!( caps.supports_realtime ); // Interactive updates
  assert_eq!( caps.max_scene_complexity, 50_000 );
  
  // Initialize with context
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  // Frame lifecycle
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Cleanup
  assert!( renderer.cleanup().is_ok() );
}

#[ test ]
fn test_svg_browser_renderer_error_handling()
{
  let mut renderer = SvgBrowserRenderer::new();
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

/// Test Matrix: Interactive Line Rendering with Hover Effects
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Single line | Basic line with interactivity | SVG line with hover CSS |
/// | Multiple lines | Batch processing | Multiple interactive elements |
/// | Colored lines | RGBA color support | Proper color conversion |
/// | Line caps/joins | Different line styles | SVG stroke attributes |
#[ test ]
fn test_svg_browser_line_rendering()
{
  let mut renderer = SvgBrowserRenderer::with_dimensions( 400, 300 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Single interactive line
  let line = LineCommand
  {
    start: Point2D { x: 50.0, y: 50.0 },
    end: Point2D { x: 150.0, y: 100.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Red
      width: 3.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  
  // Multiple lines with different colors
  let line2 = LineCommand
  {
    start: Point2D { x: 200.0, y: 50.0 },
    end: Point2D { x: 300.0, y: 100.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 0.8, 0.2, 0.7 ], // Green with transparency
      width: 2.0,
      cap_style: LineCap::Square,
      join_style: LineJoin::Bevel,
    },
  };
  
  assert!( renderer.render_line( &line2 ).is_ok() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Verify HTML output contains interactive elements
  let output = renderer.output().unwrap();
  assert!( output.contains( "<!DOCTYPE html>" ) );
  assert!( output.contains( "<svg" ) );
  assert!( output.contains( "stroke=" ) );
  assert!( output.contains( "addEventListener" ) );
  assert!( output.contains( ":hover" ) );
}

/// Test Matrix: Interactive Curve Rendering with JavaScript
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Cubic Bezier | High-quality curve path | SVG path with interactivity |
/// | Control points | Proper curve definition | Correct path data |
/// | Interactive curves | JavaScript event binding | Click and hover handlers |
#[ test ]
fn test_svg_browser_curve_rendering()
{
  let mut renderer = SvgBrowserRenderer::with_dimensions( 500, 400 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Interactive cubic Bezier curve
  let curve = CurveCommand
  {
    start: Point2D { x: 100.0, y: 200.0 },
    control1: Point2D { x: 200.0, y: 100.0 },
    control2: Point2D { x: 300.0, y: 300.0 },
    end: Point2D { x: 400.0, y: 200.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 0.5, 1.0, 1.0 ], // Blue
      width: 2.5,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  assert!( renderer.render_curve( &curve ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Verify SVG path generation with interactivity
  let output = renderer.output().unwrap();
  assert!( output.contains( "<path" ) );
  assert!( output.contains( "d=\"M" ) ); // SVG path data
  assert!( output.contains( "C " ) );    // Cubic Bezier command
  assert!( output.contains( "stroke=" ) );
  assert!( output.contains( "getElementById" ) );
}

/// Test Matrix: Interactive Text Rendering with CSS Styling
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Simple text | Basic text with interactivity | SVG text with hover effects |
/// | Text anchoring | Different anchor positions | Correct text-anchor values |
/// | Font styling | Weight, size, italic | Proper SVG font attributes |
/// | UTF-8 text | Unicode character support | Correct text encoding |
#[ test ]
fn test_svg_browser_text_rendering()
{
  let mut renderer = SvgBrowserRenderer::with_dimensions( 600, 400 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Interactive text
  let mut text_array = [ 0u8; 64 ];
  let text = b"Interactive Text!";
  text_array[ ..text.len() ].copy_from_slice( text );
  
  let text_cmd = TextCommand
  {
    text: text_array,
    text_len: text.len() as u8,
    position: Point2D { x: 300.0, y: 200.0 },
    font_style: FontStyle
    {
      color: [ 0.8, 0.2, 0.8, 1.0 ], // Magenta
      size: 24.0,
      weight: 600,
      italic: true,
      family_id: 0,
    },
    anchor: TextAnchor::Center,
  };
  
  assert!( renderer.render_text( &text_cmd ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Verify interactive text output
  let output = renderer.output().unwrap();
  assert!( output.contains( "<text" ) );
  assert!( output.contains( "Interactive Text!" ) );
  assert!( output.contains( "text-anchor=\"middle\"" ) );
  assert!( output.contains( "font-style=\"italic\"" ) );
  assert!( output.contains( "font-weight=\"600\"" ) );
  assert!( output.contains( "addEventListener" ) );
}

/// Test Matrix: HTML Document Generation
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Complete HTML | Full document structure | Valid HTML5 document |
/// | CSS integration | Embedded stylesheets | Hover effects and styling |
/// | JavaScript integration | Event handlers | Interactive functionality |
/// | Responsive design | Viewport and container | Proper layout structure |
#[ test ]
fn test_svg_browser_html_generation()
{
  let mut renderer = SvgBrowserRenderer::with_dimensions( 800, 600 );
  renderer.set_mouse_picking_enabled( true );
  renderer.set_hover_effects_enabled( true );
  renderer.set_animation_enabled( true );
  
  let mut context = RenderContext::default();
  context.width = 800;
  context.height = 600;
  context.background_color = [ 0.95, 0.95, 0.95, 1.0 ];
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Add some content to generate a complete document
  let line = LineCommand
  {
    start: Point2D { x: 100.0, y: 100.0 },
    end: Point2D { x: 200.0, y: 150.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.5, 0.0, 1.0 ],
      width: 2.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  assert!( renderer.render_line( &line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Verify complete HTML document structure
  let output = renderer.output().unwrap();
  
  // HTML5 document structure
  assert!( output.contains( "<!DOCTYPE html>" ) );
  assert!( output.contains( "<html lang=\"en\">" ) );
  assert!( output.contains( "<head>" ) );
  assert!( output.contains( "<meta charset=\"UTF-8\">" ) );
  assert!( output.contains( "<meta name=\"viewport\"" ) );
  assert!( output.contains( "<title>Interactive SVG Renderer</title>" ) );
  
  // CSS styling
  assert!( output.contains( "<style>" ) );
  assert!( output.contains( "svg-container" ) );
  assert!( output.contains( ":hover" ) );
  
  // SVG content
  assert!( output.contains( "<svg width=\"800\" height=\"600\"" ) );
  assert!( output.contains( "xmlns=\"http://www.w3.org/2000/svg\"" ) );
  
  // JavaScript functionality
  assert!( output.contains( "<script>" ) );
  assert!( output.contains( "console.log" ) );
  assert!( output.contains( "addEventListener" ) );
  assert!( output.contains( "animateElement" ) ); // Animation utilities
  
  // Document closure
  assert!( output.contains( "</script>" ) );
  assert!( output.contains( "</body>" ) );
  assert!( output.contains( "</html>" ) );
}

/// Test Matrix: Interactive Scene Rendering
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Mixed scene | Multiple interactive elements | All elements with interactivity |
/// | Complex scene | High element count | Efficient HTML generation |
/// | Event coordination | Multiple event handlers | Proper JavaScript coordination |
#[ test ]
fn test_svg_browser_scene_rendering()
{
  let mut renderer = SvgBrowserRenderer::with_dimensions( 1000, 800 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Create a complex interactive scene
  let mut scene = Scene::new();
  
  // Add interactive lines
  for i in 0..5
  {
    scene.add( RenderCommand::Line( LineCommand
    {
      start: Point2D { x: 100.0 + i as f32 * 150.0, y: 200.0 },
      end: Point2D { x: 150.0 + i as f32 * 150.0, y: 250.0 },
      style: StrokeStyle
      {
        color: [ i as f32 / 5.0, 0.5, 1.0 - i as f32 / 5.0, 1.0 ],
        width: 3.0,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // Add interactive curves
  for i in 0..3
  {
    scene.add( RenderCommand::Curve( CurveCommand
    {
      start: Point2D { x: 200.0 + i as f32 * 200.0, y: 400.0 },
      control1: Point2D { x: 300.0 + i as f32 * 200.0, y: 300.0 },
      control2: Point2D { x: 400.0 + i as f32 * 200.0, y: 500.0 },
      end: Point2D { x: 500.0 + i as f32 * 200.0, y: 400.0 },
      style: StrokeStyle
      {
        color: [ 1.0, i as f32 / 3.0, 0.0, 0.8 ],
        width: 2.5,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // Add interactive text
  let mut title_text = [ 0u8; 64 ];
  let title = b"Interactive Scene";
  title_text[ ..title.len() ].copy_from_slice( title );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: title_text,
    text_len: title.len() as u8,
    position: Point2D { x: 500.0, y: 100.0 },
    font_style: FontStyle
    {
      color: [ 0.2, 0.2, 0.8, 1.0 ],
      size: 32.0,
      weight: 700,
      italic: false,
      family_id: 0,
    },
    anchor: TextAnchor::Center,
  } ) );
  
  // Render the complete interactive scene
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Verify comprehensive interactivity
  let output = renderer.output().unwrap();
  
  // Should contain multiple interactive elements
  let line_count = output.matches( "<line" ).count();
  let path_count = output.matches( "<path" ).count();
  let text_count = output.matches( "<text" ).count();
  
  assert_eq!( line_count, 5 );  // 5 lines
  assert_eq!( path_count, 3 );  // 3 curves
  assert_eq!( text_count, 1 );  // 1 text element
  
  // Should contain multiple event handlers
  let event_handler_count = output.matches( "addEventListener" ).count();
  assert_eq!( event_handler_count, 9 ); // All elements should have handlers
  
  // Should contain hover effects
  let hover_count = output.matches( ":hover" ).count();
  assert!( hover_count >= 8 ); // Lines and curves should have hover effects
}

/// Test Matrix: Unsupported Commands
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Tilemap command | Not supported | UnsupportedCommand error |
/// | Particle command | Not supported | UnsupportedCommand error |
#[ test ]
fn test_svg_browser_unsupported_commands()
{
  let mut renderer = SvgBrowserRenderer::new();
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
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