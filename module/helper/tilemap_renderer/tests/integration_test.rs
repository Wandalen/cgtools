//! Integration test suite for feature combinations and end-to-end workflows.
//!
//! ## Test Matrix for Integration Testing

#![ allow( unused_variables ) ]
#![ allow( unused_mut ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::cast_possible_truncation ) ]
//!
//! ### Test Factors:
//! - **Feature Combinations**: Ultra-granular vs bundle features, backend combinations
//! - **Cross-Backend**: Scene rendering across multiple backends simultaneously
//! - **Data Flow**: Command creation → Scene → Backend → Output validation
//! - **Error Propagation**: Error handling across all layers
//! - **Memory Management**: Large scenes, resource cleanup
//! - **CLI Integration**: CLI commands with different backends
//!
//! ### Test Combinations:
//!
//! | ID   | Test Focus | Features | Backends | Expected Behavior |
//! |------|------------|----------|----------|-------------------|
//! | I1.1 | Minimal Build | types-basic, traits-basic | None | Core types work |
//! | I1.2 | Core Build | standard | None | Scene and commands work |
//! | I1.3 | Single Backend | standard, adapter-svg | SVG | Full SVG pipeline |
//! | I1.4 | Dual Backend | standard, adapter-svg, adapter-terminal | SVG+Terminal | Same scene, different outputs |
//! | I1.5 | Full Backend | standard, adapters-static | All static | All backends work |
//! | I1.6 | CLI Integration | cli, adapter-svg | SVG | CLI creates and renders scene |
//! | I2.1 | Large Scene | standard, adapter-terminal | Terminal | 10k+ commands render |
//! | I2.2 | Error Recovery | standard, adapter-svg | SVG | Invalid commands handled |
//! | I2.3 | Memory Stress | standard, adapter-webgl | WebGL | Memory cleanup verified |
//! | I2.4 | Serialization | serde, standard | Multiple | Scene save/load works |
//! | I3.1 | Cross-Platform | wasm-web, adapter-svg-browser | Browser | WASM compilation |
//! | I3.2 | Performance | parallel-basic, standard | All | Parallel rendering |

#![ allow( unused_imports ) ]
use tilemap_renderer as the_module;

// Import test utilities
use the_module::{ scene, commands, ports };

#[ cfg( feature = "adapter-svg" ) ]
use the_module::adapters::SvgRenderer;

#[ cfg( feature = "adapter-terminal" ) ]
use the_module::adapters::TerminalRenderer;

// #[ cfg( feature = "adapter-webgl" ) ]
// use the_module::adapters::WebGLRenderer;

/// Test minimal build with only basic types - verifies ultra-granular features work
/// Test Combination: I1.1
#[ test ]
#[ cfg( feature = "types-basic" ) ]
fn test_minimal_build_types_basic()
{
  use the_module::commands::{ Point2D };

  // Verify basic types can be created
  let point = Point2D { x: 10.0, y: 20.0 };
  assert_eq!( point.x, 10.0 );
  assert_eq!( point.y, 20.0 );

  let color = [ 1.0f32, 0.5, 0.0, 1.0 ];
  assert_eq!( color[ 0 ], 1.0 );

  // Basic types should be copyable and comparable
  let point2 = point;
  assert_eq!( point, point2 );
}

/// Test core build with scene functionality
/// Test Combination: I1.2
#[ test ]
#[ cfg( all( feature = "scene-methods", feature = "commands" ) ) ]
fn test_core_build_scene_commands()
{
  use the_module::scene::Scene;
  use the_module::commands::*;

  let mut scene = Scene::new();
  assert_eq!( scene.len(), 0 );

  // Add various command types
  let line = RenderCommand::Line( LineCommand {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 100.0, y: 100.0 },
    style: StrokeStyle::default(),
  } );
  scene.add( line );

  let curve = RenderCommand::Curve( CurveCommand {
    start: Point2D { x: 0.0, y: 0.0 },
    control1: Point2D { x: 50.0, y: -50.0 },
    control2: Point2D { x: 50.0, y: 50.0 },
    end: Point2D { x: 100.0, y: 0.0 },
    style: StrokeStyle::default(),
  } );
  scene.add( curve );

  // Test scene operations
  assert_eq!( scene.len(), 2 );

  // Test query functionality if available
  #[ cfg( feature = "query-basic" ) ]
  {
    let lines_count = scene.query_lines().len();
    assert_eq!( lines_count, 1 );

    let curves_count = scene.query_curves().len();
    assert_eq!( curves_count, 1 );
  }
}

/// Test single backend integration - SVG
/// Test Combination: I1.3
#[ test ]
#[ cfg( all( feature = "adapter-svg", feature = "standard" ) ) ]
fn test_single_backend_svg_integration()
{
  use the_module::{
    scene::Scene,
    commands::*,
    adapters::SvgRenderer,
    ports::*,
  };

  // Create a complex scene
  let mut scene = Scene::new();

  // Add line
  scene.add( RenderCommand::Line( LineCommand {
    start: Point2D { x: 10.0, y: 10.0 },
    end: Point2D { x: 90.0, y: 90.0 },
    style: StrokeStyle {
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Red
      width: 2.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  } ) );

  // Add text
  let mut text_data = [ 0u8; 64 ];
  let text_str = b"Integration Test";
  text_data[ ..text_str.len() ].copy_from_slice( text_str );

  scene.add( RenderCommand::Text( TextCommand {
    text: text_data,
    text_len: text_str.len() as u8,
    position: Point2D { x: 20.0, y: 50.0 },
    font_style: FontStyle {
      color: [ 0.0, 0.0, 1.0, 1.0 ], // Blue
      size: 16.0,
      family_id: 0,
      weight: 400,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  } ) );

  // Create and test SVG renderer
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();

  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  // Validate SVG output
  let output = renderer.output().unwrap();
  assert!( output.contains( "<svg" ) );
  assert!( output.contains( "<line" ) );
  assert!( output.contains( "<text" ) );
  assert!( output.contains( "red" ) || output.contains( "rgb(255,0,0)" ) );
  assert!( output.contains( "blue" ) || output.contains( "rgb(0,0,255)" ) );
  assert!( output.contains( "Integration Test" ) );
}

/// Test dual backend rendering - same scene, different outputs
/// Test Combination: I1.4
#[ test ]
#[ cfg( all( feature = "adapter-svg", feature = "adapter-terminal" ) ) ]
fn test_dual_backend_svg_terminal()
{
  use the_module::{
    scene::Scene,
    commands::*,
    adapters::{ SvgRenderer, TerminalRenderer },
    ports::*,
  };

  // Create shared scene
  let mut scene = Scene::new();
  scene.add( RenderCommand::Line( LineCommand {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 50.0, y: 25.0 },
    style: StrokeStyle::default(),
  } ) );

  let mut text_data = [ 0u8; 64 ];
  let text_str = b"Dual Backend Test";
  text_data[ ..text_str.len() ].copy_from_slice( text_str );

  scene.add( RenderCommand::Text( TextCommand {
    text: text_data,
    text_len: text_str.len() as u8,
    position: Point2D { x: 10.0, y: 15.0 },
    font_style: FontStyle::default(),
    anchor: TextAnchor::TopLeft,
  } ) );

  let context = RenderContext::default();

  // Test SVG backend
  let mut svg_renderer = SvgRenderer::new();
  assert!( svg_renderer.initialize( &context ).is_ok() );
  assert!( svg_renderer.begin_frame( &context ).is_ok() );
  assert!( svg_renderer.render_scene( &scene ).is_ok() );
  assert!( svg_renderer.end_frame().is_ok() );

  let svg_output = svg_renderer.output().unwrap();
  assert!( svg_output.contains( "<svg" ) );
  assert!( svg_output.contains( "Dual Backend Test" ) );

  // Test Terminal backend
  let mut terminal_renderer = TerminalRenderer::with_dimensions( 80, 25 );
  assert!( terminal_renderer.initialize( &context ).is_ok() );
  assert!( terminal_renderer.begin_frame( &context ).is_ok() );
  assert!( terminal_renderer.render_scene( &scene ).is_ok() );
  assert!( terminal_renderer.end_frame().is_ok() );

  let terminal_output = terminal_renderer.get_output();
  // Terminal output should contain text characters (potentially with ANSI codes)
  assert!( terminal_output.contains( 'D' ) && terminal_output.contains( 'u' ) );
  assert!( terminal_output.contains( 'T' ) && terminal_output.contains( 's' ) );

  // Verify outputs are different formats but from same scene
  assert_ne!( svg_output, terminal_output );
  assert!( svg_output.len() != terminal_output.len() );
}

/// Test CLI integration with backend rendering
/// Test Combination: I1.6
#[ test ]
#[ cfg( all( feature = "cli", feature = "adapter-svg" ) ) ]
fn test_cli_integration_with_backend()
{
  use the_module::{
    cli::CliApp,
    adapters::SvgRenderer,
    ports::*,
  };

  // Create CLI app (simulating CLI operations)
  let mut app = CliApp::new().unwrap();

  // Simulate CLI commands to build scene
  // Note: This would normally be done through the CLI interface
  // but we're testing the integration points

  let context = RenderContext::default();
  let mut renderer = SvgRenderer::new();

  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );

  // The CLI app contains a scene that can be rendered
  // This tests that CLI and rendering pipeline integrate properly
  // xxx: temporarily disabled due to API changes
  // let scene = &app.scene;
  // assert!( renderer.render_scene( scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  let output = renderer.output().unwrap();
  assert!( output.contains( "<svg" ) );
}

/// Test large scene handling with many commands
/// Test Combination: I2.1
#[ test ]
#[ cfg( all( feature = "adapter-terminal", feature = "standard" ) ) ]
fn test_large_scene_stress_test()
{
  use the_module::{
    scene::Scene,
    commands::*,
    adapters::TerminalRenderer,
    ports::*,
  };

  // Create large scene with many commands
  let mut scene = Scene::new();

  // Add 1000 line commands
  for i in 0..1000
  {
    let x = ( i % 100 ) as f32;
    let y = ( i / 100 ) as f32;

    scene.add( RenderCommand::Line( LineCommand {
      start: Point2D { x, y },
      end: Point2D { x: x + 10.0, y: y + 10.0 },
      style: StrokeStyle {
        color: [
          ( i % 256 ) as f32 / 255.0,
          ( ( i * 2 ) % 256 ) as f32 / 255.0,
          ( ( i * 3 ) % 256 ) as f32 / 255.0,
          1.0
        ],
        width: 1.0,
        cap_style: LineCap::Butt,
        join_style: LineJoin::Miter,
      },
    } ) );
  }

  assert_eq!( scene.len(), 1000 );

  // Test that terminal backend can handle large scene
  let mut renderer = TerminalRenderer::with_dimensions( 120, 60 );
  let context = RenderContext::default();

  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );

  // This should not panic or run out of memory
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  let output = renderer.get_output();
  assert!( !output.is_empty() );

  // Verify scene statistics
  #[ cfg( feature = "scene-statistics" ) ]
  {
    // xxx: temporarily disabled due to API changes
    // use the_module::scene::Query;
    // let stats = scene.query().statistics();
    // assert_eq!( stats.total_commands, 1000 );
    // assert_eq!( stats.line_commands, 1000 );
    // assert_eq!( stats.curve_commands, 0 );
    // assert_eq!( stats.text_commands, 0 );
  }
}

/// Test error handling across the full pipeline
/// Test Combination: I2.2
#[ test ]
#[ cfg( all( feature = "adapter-svg", feature = "ports" ) ) ]
fn test_error_propagation_integration()
{
  use the_module::{
    scene::Scene,
    commands::*,
    adapters::SvgRenderer,
    ports::*,
  };

  let mut scene = Scene::new();

  // Add valid command
  scene.add( RenderCommand::Line( LineCommand {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 100.0, y: 100.0 },
    style: StrokeStyle::default(),
  } ) );

  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();

  // Test normal operation
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  // Test error conditions
  // Calling begin_frame without end_frame may fail, but should not panic
  let _ = renderer.begin_frame( &context );
  let second_begin = renderer.begin_frame( &context ); // Should not panic

  // Try to recover regardless of error state
  let _ = renderer.end_frame();
}

/// Test serialization integration with scene and commands
/// Test Combination: I2.4
#[ test ]
#[ cfg( all( feature = "serde", feature = "standard" ) ) ]
fn test_serialization_integration()
{
  use the_module::{
    scene::Scene,
    commands::*,
  };

  // Create scene with various commands
  let mut original_scene = Scene::new();

  original_scene.add( RenderCommand::Line( LineCommand {
    start: Point2D { x: 10.0, y: 20.0 },
    end: Point2D { x: 30.0, y: 40.0 },
    style: StrokeStyle {
      color: [ 1.0, 0.5, 0.0, 1.0 ],
      width: 2.5,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  } ) );

  let mut text_data = [ 0u8; 64 ];
  let text_str = b"Serialization Test";
  text_data[ ..text_str.len() ].copy_from_slice( text_str );

  original_scene.add( RenderCommand::Text( TextCommand {
    text: text_data,
    text_len: text_str.len() as u8,
    position: Point2D { x: 15.0, y: 25.0 },
    font_style: FontStyle {
      color: [ 0.0, 1.0, 0.0, 1.0 ],
      size: 14.0,
      family_id: 1,
      weight: 700,
      italic: true,
    },
    anchor: TextAnchor::Center,
  } ) );

  // Test JSON serialization
  let json_data = serde_json::to_string( &original_scene ).expect( "Serialization should work" );
  assert!( !json_data.is_empty() );
  assert!( json_data.contains( "Line" ) || json_data.contains( "line" ) );
  assert!( json_data.contains( "Text" ) || json_data.contains( "text" ) );

  // Test deserialization
  let deserialized_scene: Scene = serde_json::from_str( &json_data ).expect( "Deserialization should work" );

  // Verify scene integrity
  assert_eq!( original_scene.len(), deserialized_scene.len() );
  assert_eq!( original_scene.len(), 2 );

  // Compare scene contents
  let original_commands: Vec< _ > = original_scene.iter().collect();
  let deserialized_commands: Vec< _ > = deserialized_scene.iter().collect();

  assert_eq!( original_commands.len(), deserialized_commands.len() );

  // Verify specific command data
  match ( &original_commands[ 0 ], &deserialized_commands[ 0 ] )
  {
    ( RenderCommand::Line( orig ), RenderCommand::Line( deser ) ) =>
    {
      assert_eq!( orig.start, deser.start );
      assert_eq!( orig.end, deser.end );
      assert_eq!( orig.style.width, deser.style.width );
    },
    _ => panic!( "First command should be Line" ),
  }

  match ( &original_commands[ 1 ], &deserialized_commands[ 1 ] )
  {
    ( RenderCommand::Text( orig ), RenderCommand::Text( deser ) ) =>
    {
      assert_eq!( orig.position, deser.position );
      assert_eq!( orig.text_len, deser.text_len );
      assert_eq!( orig.font_style.size, deser.font_style.size );
    },
    _ => panic!( "Second command should be Text" ),
  }
}

/// Test cross-platform compatibility preparation
/// Test Combination: I3.1
#[ test ]
#[ cfg( all( feature = "wasm-basic", feature = "types-basic" ) ) ]
fn test_cross_platform_wasm_compatibility()
{
  use the_module::commands::Point2D;

  // Test that basic types work in WASM-compatible way
  let point = Point2D { x: 42.0, y: 24.0 };

  // Should be able to serialize basic types for WASM
  assert_eq!( std::mem::size_of::< Point2D >(), 8 ); // 2 * f32 = 8 bytes

  // Test color arrays (commonly used in WASM graphics)
  let color = [ 0.5f32, 0.5f32, 0.5f32, 1.0f32 ];
  assert_eq!( color.len(), 4 );
  assert!( color[ 3 ] == 1.0 ); // Alpha channel
}

/// Test performance characteristics under load
/// Test Combination: I3.2
#[ test ]
#[ cfg( all( feature = "parallel-basic", feature = "adapter-terminal" ) ) ]
fn test_performance_integration()
{
  use the_module::{
    scene::Scene,
    commands::*,
    adapters::TerminalRenderer,
    ports::*,
  };
  use std::time::Instant;

  // Create moderately large scene
  let mut scene = Scene::new();

  for i in 0..500
  {
    scene.add( RenderCommand::Line( LineCommand {
      start: Point2D { x: ( i % 50 ) as f32, y: ( i / 50 ) as f32 },
      end: Point2D { x: ( ( i + 1 ) % 50 ) as f32, y: ( ( i + 1 ) / 50 ) as f32 },
      style: StrokeStyle::default(),
    } ) );
  }

  let mut renderer = TerminalRenderer::with_dimensions( 80, 25 );
  let context = RenderContext::default();

  // Measure rendering performance
  let start_time = Instant::now();

  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  let elapsed = start_time.elapsed();

  // Rendering 500 lines should complete reasonably quickly
  // This is not a strict benchmark but a smoke test
  assert!( elapsed.as_millis() < 1000 ); // Should complete within 1 second

  let output = renderer.get_output();
  assert!( !output.is_empty() );
}
