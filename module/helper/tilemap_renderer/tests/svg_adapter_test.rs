//! Test suite for SVG backend adapter.
//!
//! ## Test Matrix for SVG Adapter
//!
//! ### Test Factors:
//! - **Renderer Lifecycle**: Initialization, frame rendering, cleanup
//! - **Primitive Rendering**: Lines, curves, text output format correctness
//! - **SVG Format**: Valid XML structure, proper element attributes
//! - **Color Conversion**: RGBA to SVG color format conversion
//! - **Text Rendering**: Font family resolution, anchor positioning
//! - **Error Handling**: Unsupported commands, invalid states

#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::float_cmp ) ]

use tilemap_renderer::
{
  adapters::SvgRenderer,
  ports::{ Renderer, RenderContext, RenderError },
  scene::Scene,
  commands::{ Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, RenderCommand, LineCommand, CurveCommand, TextCommand, TilemapCommand },
};

/// Tests SVG renderer capabilities.
/// Test Focus: Backend capability reporting
#[ test ]
fn test_svg_renderer_capabilities()
{
  let renderer = SvgRenderer::new();
  let caps = renderer.capabilities();
  
  assert_eq!( caps.backend_name, "SVG" );
  assert_eq!( caps.backend_version, "1.0" );
  assert!( caps.supports_transparency );
  assert!( caps.supports_antialiasing );
  assert!( caps.supports_custom_fonts );
  assert!( !caps.supports_particles );
  assert!( !caps.supports_realtime );
  assert_eq!( caps.max_scene_complexity, 10000 );
}

/// Tests basic SVG rendering lifecycle.
/// Test Focus: Complete rendering workflow
#[ test ]
fn test_svg_rendering_lifecycle()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  // Add a simple line
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 10.0, y: 20.0 },
    end: Point2D { x: 100.0, y: 80.0 },
    style: StrokeStyle::default(),
  } ) );
  
  // Complete rendering lifecycle
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify SVG structure
  assert!( output.contains( "<?xml version=\"1.0\" encoding=\"UTF-8\"?>" ) );
  assert!( output.contains( "<svg width=\"800\" height=\"600\"" ) );
  assert!( output.contains( "<line x1=\"10\" y1=\"20\" x2=\"100\" y2=\"80\"" ) );
  assert!( output.contains( "</svg>" ) );
}

/// Tests line rendering output format.
/// Test Focus: SVG line element generation
#[ test ]
fn test_svg_line_rendering()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 10.0 },
    end: Point2D { x: 15.0, y: 25.0 },
    style: StrokeStyle
    {
      width: 2.5,
      color: [ 1.0, 0.0, 0.5, 0.8 ],
      cap_style: LineCap::Round,
      join_style: LineJoin::Bevel,
    },
  } ) );
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify line attributes
  assert!( output.contains( "x1=\"5\" y1=\"10\" x2=\"15\" y2=\"25\"" ) );
  assert!( output.contains( "stroke-width=\"2.5\"" ) );
  assert!( output.contains( "stroke=\"rgba(255,0,127,0.8)\"" ) );
  assert!( output.contains( "stroke-linecap=\"round\"" ) );
  assert!( output.contains( "stroke-linejoin=\"bevel\"" ) );
}

/// Tests curve rendering output format.
/// Test Focus: SVG path element generation for curves
#[ test ]
fn test_svg_curve_rendering()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    control1: Point2D { x: 25.0, y: 25.0 },
    control2: Point2D { x: 75.0, y: 25.0 },
    end: Point2D { x: 100.0, y: 0.0 },
    style: StrokeStyle
    {
      width: 1.5,
      color: [ 0.0, 1.0, 0.0, 1.0 ],
      cap_style: LineCap::Square,
      join_style: LineJoin::Miter,
    },
  } ) );
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify path data
  assert!( output.contains( "d=\"M 0 0 C 25 25, 75 25, 100 0\"" ) );
  assert!( output.contains( "fill=\"none\"" ) );
  assert!( output.contains( "stroke=\"rgb(0,255,0)\"" ) );
  assert!( output.contains( "stroke-width=\"1.5\"" ) );
  assert!( output.contains( "stroke-linecap=\"square\"" ) );
  assert!( output.contains( "stroke-linejoin=\"miter\"" ) );
}

/// Tests text rendering output format.
/// Test Focus: SVG text element generation
#[ test ]
fn test_svg_text_rendering()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Text( TextCommand::new(
    "Hello SVG",
    Point2D { x: 50.0, y: 100.0 },
    FontStyle
    {
      size: 16.0,
      color: [ 0.2, 0.4, 0.8, 1.0 ],
      weight: 600,
      italic: false,
      family_id: 1, // Times New Roman
    },
    TextAnchor::Center
  ) ) );
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify text attributes
  assert!( output.contains( "x=\"50\" y=\"100\"" ) );
  assert!( output.contains( "font-family=\"Times New Roman\"" ) );
  assert!( output.contains( "font-size=\"16\"" ) );
  assert!( output.contains( "fill=\"rgb(51,102,204)\"" ) );
  assert!( output.contains( "text-anchor=\"middle\"" ) );
  assert!( output.contains( "dominant-baseline=\"central\"" ) );
  assert!( output.contains( ">Hello SVG</text>" ) );
}

/// Tests color conversion functionality.
/// Test Focus: RGBA to SVG color format conversion
#[ test ]
fn test_color_conversion()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  // Test full opacity (RGB format)
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.5, 0.25, 1.0 ],
      ..StrokeStyle::default()
    },
  } ) );
  
  // Test partial transparency (RGBA format)
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 20.0, y: 20.0 },
    end: Point2D { x: 30.0, y: 30.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 0.0, 1.0, 0.5 ],
      ..StrokeStyle::default()
    },
  } ) );
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify color formats
  assert!( output.contains( "stroke=\"rgb(255,127,63)\"" ) ); // Full opacity -> RGB
  assert!( output.contains( "stroke=\"rgba(0,0,255,0.5)\"" ) ); // Partial opacity -> RGBA
}

/// Tests font family resolution.
/// Test Focus: Font ID to family name mapping
#[ test ]
fn test_font_family_resolution()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  // Test different font family IDs
  let font_tests = vec![
    ( 0, "Arial" ),
    ( 1, "Times New Roman" ),
    ( 2, "Courier New" ),
    ( 3, "Helvetica" ),
    ( 4, "Georgia" ),
    ( 999, "sans-serif" ), // Unknown ID -> fallback
  ];
  
  for ( family_id, expected_family ) in font_tests
  {
    scene.clear();
    scene.add( RenderCommand::Text( TextCommand::new(
      "Test",
      Point2D::default(),
      FontStyle
      {
        family_id,
        ..FontStyle::default()
      },
      TextAnchor::TopLeft
    ) ) );
    
    renderer.cleanup().unwrap();
    assert!( renderer.initialize( &context ).is_ok() );
    assert!( renderer.begin_frame( &context ).is_ok() );
    assert!( renderer.render_scene( &scene ).is_ok() );
    assert!( renderer.end_frame().is_ok() );
    
    let output = renderer.output().unwrap();
    assert!( output.contains( &format!( "font-family=\"{expected_family}\"" ) ) );
  }
}

/// Tests text anchor positioning.
/// Test Focus: TextAnchor to SVG text-anchor and dominant-baseline conversion
#[ test ]
fn test_text_anchor_positioning()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  
  let anchor_tests = vec![
    ( TextAnchor::TopLeft, "start", "hanging" ),
    ( TextAnchor::TopCenter, "middle", "hanging" ),
    ( TextAnchor::TopRight, "end", "hanging" ),
    ( TextAnchor::CenterLeft, "start", "central" ),
    ( TextAnchor::Center, "middle", "central" ),
    ( TextAnchor::CenterRight, "end", "central" ),
    ( TextAnchor::BottomLeft, "start", "baseline" ),
    ( TextAnchor::BottomCenter, "middle", "baseline" ),
    ( TextAnchor::BottomRight, "end", "baseline" ),
  ];
  
  for ( anchor, expected_text_anchor, expected_baseline ) in anchor_tests
  {
    let mut scene = Scene::new();
    scene.add( RenderCommand::Text( TextCommand::new(
      "Test",
      Point2D::default(),
      FontStyle::default(),
      anchor
    ) ) );
    
    renderer.cleanup().unwrap();
    assert!( renderer.initialize( &context ).is_ok() );
    assert!( renderer.begin_frame( &context ).is_ok() );
    assert!( renderer.render_scene( &scene ).is_ok() );
    assert!( renderer.end_frame().is_ok() );
    
    let output = renderer.output().unwrap();
    assert!( output.contains( &format!( "text-anchor=\"{expected_text_anchor}\"" ) ) );
    assert!( output.contains( &format!( "dominant-baseline=\"{expected_baseline}\"" ) ) );
  }
}

/// Tests unsupported command handling.
/// Test Focus: Error handling for unsupported commands
#[ test ]
fn test_unsupported_commands()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  // Test tilemap command (unsupported)
  scene.add( RenderCommand::Tilemap( TilemapCommand::new(
    Point2D::default(),
    32.0, 32.0, 2, 2, 0,
    &[ 1, 2, 3, 4 ]
  ) ) );
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  let result = renderer.render_scene( &scene );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::UnsupportedCommand( _ ) ) );
}

/// Tests SVG document structure.
/// Test Focus: Valid XML document generation
#[ test ]
fn test_svg_document_structure()
{
  let mut renderer = SvgRenderer::new();
  let mut context = RenderContext::default();
  context.width = 400;
  context.height = 300;
  context.background_color = [ 0.9, 0.9, 0.9, 1.0 ];
  context.clear_background = true;
  
  let scene = Scene::new(); // Empty scene
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify document structure
  assert!( output.starts_with( "<?xml version=\"1.0\" encoding=\"UTF-8\"?>" ) );
  assert!( output.contains( "<svg width=\"400\" height=\"300\" viewBox=\"0 0 400 300\"" ) );
  assert!( output.contains( "xmlns=\"http://www.w3.org/2000/svg\">" ) );
  assert!( output.contains( "<rect width=\"100%\" height=\"100%\" fill=\"rgb(229,229,229)\"/>" ) );
  assert!( output.ends_with( "</svg>\n" ) );
}

/// Tests error conditions during rendering.
/// Test Focus: State validation and error handling
#[ test ]
fn test_rendering_error_conditions()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let scene = Scene::new();
  
  // Test operations without initialization
  let result = renderer.begin_frame( &context );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::InvalidContext( _ ) ) );
  
  // Test double initialization
  assert!( renderer.initialize( &context ).is_ok() );
  let result = renderer.initialize( &context );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::InitializationFailed( _ ) ) );
  
  // Test output before frame completion
  assert!( renderer.begin_frame( &context ).is_ok() );
  let result = renderer.output();
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::OutputError( _ ) ) );
  
  // Test rendering without active frame
  assert!( renderer.end_frame().is_ok() );
  let result = renderer.render_scene( &scene );
  assert!( result.is_err() );
  assert!( matches!( result.unwrap_err(), RenderError::RenderFailed( _ ) ) );
}

/// Tests complex scene rendering.
/// Test Focus: Multiple command types in single scene
#[ test ]
fn test_complex_scene_rendering()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let mut scene = Scene::new();
  
  // Add multiple command types
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 100.0, y: 100.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 100.0, y: 0.0 },
    control1: Point2D { x: 125.0, y: 25.0 },
    control2: Point2D { x: 175.0, y: 75.0 },
    end: Point2D { x: 200.0, y: 100.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Text( TextCommand::new(
    "Complex Scene",
    Point2D { x: 150.0, y: 50.0 },
    FontStyle::default(),
    TextAnchor::Center
  ) ) );
  
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let output = renderer.output().unwrap();
  
  // Verify all elements are present
  assert!( output.contains( "<line" ) );
  assert!( output.contains( "<path" ) );
  assert!( output.contains( "<text" ) );
  assert!( output.contains( ">Complex Scene</text>" ) );
}

/// Tests renderer cleanup functionality.
/// Test Focus: Resource cleanup and state reset
#[ test ]
fn test_renderer_cleanup()
{
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  let scene = Scene::new();
  
  // Complete a rendering cycle
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let first_output = renderer.output().unwrap();
  assert!( !first_output.is_empty() );
  
  // Cleanup and verify state reset
  assert!( renderer.cleanup().is_ok() );
  
  // Should be able to initialize again
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  let second_output = renderer.output().unwrap();
  assert_eq!( first_output, second_output );
}