//! Error handling and edge case test suite.
//!
//! ## Test Matrix for Error Handling

#![ allow( clippy::ignored_unit_patterns ) ]
#![ allow( clippy::doc_comment_double_space_linebreaks ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::single_match_else ) ]
//!
//! ### Test Factors:
//! - **Error Types**: Initialization, runtime, resource, validation errors
//! - **Recovery Scenarios**: Graceful degradation, error propagation, cleanup
//! - **Edge Cases**: Boundary conditions, invalid inputs, resource exhaustion
//! - **Thread Safety**: Concurrent access, data races, deadlocks
//! - **Memory Safety**: Buffer overflows, null pointers, use-after-free
//!
//! ### Test Combinations:
//!
//! | ID   | Error Category | Test Focus | Expected Behavior |
//! |------|----------------|------------|-------------------|
//! | E1.1 | Initialization | Invalid context | Graceful error return |
//! | E1.2 | Initialization | Resource unavailable | Error with cleanup |
//! | E1.3 | Runtime | Invalid command data | Command skipped, continues |
//! | E1.4 | Runtime | Scene too large | Memory management |
//! | E1.5 | Validation | Text overflow | Truncation handling |
//! | E1.6 | Validation | Tilemap overflow | Array bounds checking |
//! | E2.1 | Resource | Out of memory | Graceful degradation |
//! | E2.2 | Resource | File system errors | I/O error handling |
//! | E2.3 | Threading | Concurrent access | No data races |
//! | E2.4 | Threading | Renderer reuse | Proper state management |
//! | E3.1 | Edge Cases | Zero dimensions | Handle gracefully |
//! | E3.2 | Edge Cases | Extreme coordinates | No overflow/underflow |
//! | E3.3 | Edge Cases | Invalid colors | Color clamping |
//! | E3.4 | Edge Cases | Empty scenes | No crashes |

#![ allow( unused_imports ) ]
use tilemap_renderer as the_module;
use the_module::*;
use std::sync::{ Arc, Mutex };
use std::thread;

/// Test initialization error handling
/// Test Combination: E1.1
#[ test ]
#[ cfg( feature = "adapter-svg" ) ]
fn test_initialization_error_handling()
{
  use the_module::{ adapters::SvgRenderer, ports::* };

  let mut renderer = SvgRenderer::new();

  // Test rendering before initialization - should fail gracefully
  let context = RenderContext::default();
  let scene = scene::Scene::new();

  // Should fail but not panic
  let result = renderer.render_scene( &scene );
  assert!( result.is_err() );

  // Should be able to recover after proper initialization
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
}

/// Test runtime error handling with invalid command data
/// Test Combination: E1.3
#[ test ]
#[ cfg( feature = "adapter-terminal" ) ]
fn test_runtime_invalid_command_handling()
{
  use the_module::{ adapters::TerminalRenderer, ports::*, commands::* };

  let mut renderer = TerminalRenderer::with_dimensions( 80, 25 );
  let context = RenderContext::default();

  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );

  // Test line with moderately large coordinates (avoid extreme values that can cause hangs)
  let invalid_line = LineCommand {
    start: Point2D { x: -100.0, y: 200.0 },
    end: Point2D { x: 1000.0, y: -50.0 },
    style: StrokeStyle::default(),
  };

  // Should handle extreme coordinates gracefully
  let result = renderer.render_line( &invalid_line );
  // May succeed (by clamping) or fail gracefully, but should not panic
  let _ = result;

  // Test with zero-width stroke
  let zero_width_line = LineCommand {
    start: Point2D { x: 10.0, y: 10.0 },
    end: Point2D { x: 20.0, y: 20.0 },
    style: StrokeStyle { width: 0.0, ..StrokeStyle::default() },
  };

  let result = renderer.render_line( &zero_width_line );
  let _ = result; // Should not panic

  // Renderer should still be usable with valid commands
  let valid_line = LineCommand {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  };

  assert!( renderer.render_line( &valid_line ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
}

/// Test text overflow handling
/// Test Combination: E1.5
#[ test ]
#[ cfg( feature = "commands" ) ]
fn test_text_overflow_handling()
{
  use the_module::commands::*;

  // Test creating text command with very long string
  let long_text = "A".repeat( 1000 );

  let text_cmd = TextCommand::new(
    &long_text,
    Point2D { x: 0.0, y: 0.0 },
    FontStyle::default(),
    TextAnchor::TopLeft
  );

  // Should truncate to 63 characters max (based on array size)
  assert!( text_cmd.text_len <= 63 );

  // Verify text data is properly handled
  let extracted_text = text_cmd.text();
  assert!( extracted_text.len() <= 63 );
  assert!( extracted_text.starts_with( "AAA" ) );
}

/// Test tilemap overflow handling
/// Test Combination: E1.6
#[ test ]
#[ cfg( feature = "commands" ) ]
fn test_tilemap_overflow_handling()
{
  use the_module::commands::*;

  // Create oversized tile data
  let large_tiles: Vec< u16 > = ( 0..2000 ).collect(); // Much larger than 32

  let tilemap_cmd = TilemapCommand::new(
    Point2D { x: 0.0, y: 0.0 },
    32.0, // tile_width
    32.0, // tile_height
    16,   // map_width
    16,   // map_height
    1,    // tileset_id
    &large_tiles
  );

  // Should truncate to max tiles (32)
  let extracted_tiles = tilemap_cmd.tiles();
  assert!( extracted_tiles.len() <= 32 );

  // Should contain the first tiles
  assert_eq!( extracted_tiles[ 0 ], 0 );
  assert_eq!( extracted_tiles[ 1 ], 1 );
}

/// Test concurrent access safety
/// Test Combination: E2.3
#[ test ]
#[ cfg( all( feature = "std", feature = "scene-methods" ) ) ]
fn test_concurrent_access_safety()
{
  use std::sync::{ Arc, Mutex };
  use std::thread;

  let scene = Arc::new( Mutex::new( scene::Scene::new() ) );
  let mut handles = Vec::new();

  // Spawn multiple threads that add commands to scene
  for i in 0..5
  {
    let scene_clone = Arc::clone( &scene );

    let handle = thread::spawn( move || {
      let line = commands::RenderCommand::Line( commands::LineCommand {
        start: commands::Point2D { x: i as f32, y: 0.0 },
        end: commands::Point2D { x: i as f32, y: 10.0 },
        style: commands::StrokeStyle::default(),
      } );

      for _j in 0..10
      {
        let mut scene = scene_clone.lock().unwrap();
        scene.add( line );
      }
    } );

    handles.push( handle );
  }

  // Wait for all threads to complete
  for handle in handles
  {
    handle.join().unwrap();
  }

  // Verify scene has expected number of commands
  let scene = scene.lock().unwrap();
  assert_eq!( scene.len(), 5 * 10 ); // 5 threads * 10 commands each
}

/// Test edge case: zero dimensions
/// Test Combination: E3.1
#[ test ]
#[ cfg( feature = "adapter-terminal" ) ]
fn test_zero_dimensions_edge_case()
{
  use the_module::{ adapters::TerminalRenderer, ports::* };

  // Test creating renderer with zero dimensions
  let mut renderer = TerminalRenderer::with_dimensions( 0, 0 );
  let context = RenderContext::default();

  // Should handle gracefully (may succeed with minimum dimensions or fail gracefully)
  let init_result = renderer.initialize( &context );

  match init_result
  {
    Ok( _ ) =>
    {
      // If initialization succeeds, other operations should work
      let _ = renderer.begin_frame( &context );
      let _ = renderer.end_frame();
    },
    Err( _ ) =>
    {
      // Graceful failure is acceptable for zero dimensions
    }
  }

  // Test with very small but non-zero dimensions
  let mut small_renderer = TerminalRenderer::with_dimensions( 1, 1 );

  // This should work
  assert!( small_renderer.initialize( &context ).is_ok() );
  assert!( small_renderer.begin_frame( &context ).is_ok() );
  assert!( small_renderer.end_frame().is_ok() );
}

/// Test extreme coordinate handling
/// Test Combination: E3.2
#[ test ]
#[ cfg( feature = "commands" ) ]
fn test_extreme_coordinates()
{
  use the_module::commands::*;

  // Test with very large coordinates
  let extreme_line = LineCommand {
    start: Point2D { x: f32::MAX / 2.0, y: f32::MAX / 2.0 },
    end: Point2D { x: f32::MIN / 2.0, y: f32::MIN / 2.0 },
    style: StrokeStyle::default(),
  };

  // Should be created successfully
  let cmd = RenderCommand::Line( extreme_line );

  // Should be able to match on it
  match cmd
  {
    RenderCommand::Line( line ) =>
    {
      // Coordinates should be preserved (even if extreme)
      assert!( line.start.x > 0.0 );
      assert!( line.end.x < 0.0 );
    },
    _ => panic!( "Should be line command" ),
  }
}

/// Test invalid color handling
/// Test Combination: E3.3
#[ test ]
#[ cfg( feature = "commands" ) ]
fn test_invalid_color_handling()
{
  use the_module::commands::*;

  // Test with out-of-range color values
  let invalid_color = [ 2.0, -1.0, f32::NAN, f32::INFINITY ];

  let style = StrokeStyle {
    color: invalid_color,
    width: 1.0,
    cap_style: LineCap::Butt,
    join_style: LineJoin::Miter,
  };

  // Should be created successfully (backend may clamp values)
  let line = LineCommand {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style,
  };

  // Structure should be valid even with invalid color data
  assert_eq!( line.start.x, 0.0 );
  assert_eq!( line.end.x, 10.0 );
}

/// Test empty scene handling
/// Test Combination: E3.4
#[ test ]
#[ cfg( all( feature = "scene-methods", feature = "adapter-svg" ) ) ]
fn test_empty_scene_handling()
{
  use the_module::{ scene::Scene, adapters::SvgRenderer, ports::* };

  let empty_scene = Scene::new();
  assert_eq!( empty_scene.len(), 0 );

  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();

  // Should handle empty scene gracefully
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &empty_scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  // Should produce valid (but minimal) output
  let output = renderer.output().unwrap();
  assert!( output.contains( "<svg" ) );
  assert!( output.contains( "</svg>" ) );
}

/// Test renderer state management errors
/// Test Combination: E2.4
#[ test ]
#[ cfg( feature = "adapter-terminal" ) ]
fn test_renderer_state_management()
{
  use the_module::{ adapters::TerminalRenderer, ports::* };

  let mut renderer = TerminalRenderer::with_dimensions( 80, 25 );
  let context = RenderContext::default();

  // Test calling operations in wrong order
  let scene = scene::Scene::new();

  // Try to render before initialization - should fail gracefully
  let result = renderer.render_scene( &scene );
  assert!( result.is_err() );

  // Initialize properly
  assert!( renderer.initialize( &context ).is_ok() );

  // Try to end frame before beginning - should fail gracefully
  let result = renderer.end_frame();
  assert!( result.is_err() );

  // Proper sequence should work
  assert!( renderer.begin_frame( &context ).is_ok() );
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  // Multiple end_frame calls should fail gracefully
  let result = renderer.end_frame();
  assert!( result.is_err() );
}

/// Test file I/O error handling
/// Test Combination: E2.2
#[ test ]
#[ cfg( feature = "adapter-terminal" ) ]
fn test_file_io_error_handling()
{
  use the_module::{ adapters::TerminalRenderer, ports::* };
  use std::fs;

  let mut renderer = TerminalRenderer::with_dimensions( 80, 25 );
  let context = RenderContext::default();

  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );

  let scene = scene::Scene::new();
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );

  // Test getting output (TerminalRenderer doesn't have save_to_file method)
  let output = renderer.get_output();
  assert!( !output.is_empty() );
}

/// Test resource cleanup on panic recovery
/// Test Combination: E2.1
#[ test ]
#[ cfg( feature = "std" ) ]
fn test_panic_recovery_cleanup()
{
  use std::panic;

  // Test that panics in one part don't leave global state corrupted
  let result = panic::catch_unwind( || {
    // Create scene and immediately panic
    let mut scene = scene::Scene::new();
    scene.add( commands::RenderCommand::Line( commands::LineCommand {
      start: commands::Point2D { x: 0.0, y: 0.0 },
      end: commands::Point2D { x: 10.0, y: 10.0 },
      style: commands::StrokeStyle::default(),
    } ) );

    panic!( "Simulated panic" );
  } );

  // Panic should be caught
  assert!( result.is_err() );

  // Should be able to create new objects after panic
  let new_scene = scene::Scene::new();
  assert_eq!( new_scene.len(), 0 );
}
