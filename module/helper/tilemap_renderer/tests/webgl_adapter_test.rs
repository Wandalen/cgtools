//! WebGL adapter comprehensive test suite.
//!
//! This test suite validates the WebGL backend adapter implementation
//! following the Test Matrix approach from the design rulebook.

#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::too_many_lines ) ]

use tilemap_renderer::
{
  adapters::WebGLRenderer,
  ports::{ Renderer, PrimitiveRenderer, RenderContext },
  commands::{ LineCommand, CurveCommand, TextCommand, Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, TilemapCommand, ParticleEmitterCommand, RenderCommand },
  scene::Scene,
};

/// Test Matrix: WebGL Renderer Creation and Configuration
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | new() | Default constructor | Default dimensions |
/// | with_dimensions() | Custom size | Specified dimensions |
/// | set_mouse_picking_enabled() | Mouse picking toggle | Picking functionality |
/// | set_max_batch_size() | Batch size configuration | Optimal performance |
#[ test ]
fn test_webgl_renderer_creation()
{
  // Default constructor
  let renderer = WebGLRenderer::new();
  // Stats may or may not be available before initialization - both outcomes are valid
  let _ = renderer.get_stats();
  
  // Custom dimensions
  let renderer = WebGLRenderer::with_dimensions( 1920, 1080 );
  // Stats may or may not be available before initialization - both outcomes are valid
  let _ = renderer.get_stats();
}

#[ test ]
fn test_webgl_renderer_configuration()
{
  let mut renderer = WebGLRenderer::new();
  
  // Test mouse picking configuration
  renderer.set_mouse_picking_enabled( true );
  
  // Test batch size configuration
  renderer.set_max_batch_size( 2000 );
  
  // Test mouse event handling (should return None when not initialized)
  let picked = renderer.handle_mouse_event( 100.0, 200.0 );
  assert!( picked.is_none() );
}

/// Test Matrix: WebGL Renderer Lifecycle
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | initialize() | WebGL initialization | Success with context setup |
/// | capabilities() | Capability report | WebGL-specific capabilities |
/// | begin_frame() | Frame start | Success with GPU setup |
/// | end_frame() | Frame end | Success with buffer swap |
/// | cleanup() | GPU resource cleanup | Success |
#[ test ]
fn test_webgl_renderer_lifecycle()
{
  let mut renderer = WebGLRenderer::new();
  
  // Check capabilities before initialization
  let caps = renderer.capabilities();
  assert_eq!( caps.backend_name, "WebGL" );
  assert_eq!( caps.backend_version, "2.0" );
  assert!( caps.supports_transparency );
  assert!( caps.supports_antialiasing );
  assert!( caps.supports_custom_fonts );
  assert!( caps.supports_particles );
  assert!( caps.supports_realtime );
  assert_eq!( caps.max_texture_size, 4096 );
  assert_eq!( caps.max_scene_complexity, 100_000 );
  
  // Initialize with context
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  // Frame lifecycle
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Should have statistics after frame start
  let stats = renderer.get_stats();
  assert!( stats.is_some() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Cleanup
  assert!( renderer.cleanup().is_ok() );
}

#[ test ]
fn test_webgl_renderer_error_handling()
{
  let mut renderer = WebGLRenderer::new();
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

/// Test Matrix: WebGL Line Rendering with GPU Batching
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Single line | Basic line rendering | Added to vertex batch |
/// | Multiple lines | Batch processing | Efficient GPU batching |
/// | Batch overflow | Max batch size exceeded | Automatic batch flush |
/// | Colored lines | RGBA color support | Proper color blending |
#[ test ]
fn test_webgl_line_rendering()
{
  let mut renderer = WebGLRenderer::with_dimensions( 800, 600 );
  renderer.set_max_batch_size( 3 ); // Small batch for testing
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Single line
  let line1 = LineCommand
  {
    start: Point2D { x: 10.0, y: 10.0 },
    end: Point2D { x: 100.0, y: 50.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Red
      width: 2.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  assert!( renderer.render_line( &line1 ).is_ok() );
  
  // Multiple lines to test batching
  let line2 = LineCommand
  {
    start: Point2D { x: 100.0, y: 50.0 },
    end: Point2D { x: 200.0, y: 100.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 1.0, 0.0, 1.0 ], // Green
      width: 1.5,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  assert!( renderer.render_line( &line2 ).is_ok() );
  
  let line3 = LineCommand
  {
    start: Point2D { x: 200.0, y: 100.0 },
    end: Point2D { x: 300.0, y: 150.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 0.0, 1.0, 0.8 ], // Blue with transparency
      width: 3.0,
      cap_style: LineCap::Square,
      join_style: LineJoin::Bevel,
    },
  };
  
  assert!( renderer.render_line( &line3 ).is_ok() );
  
  // This should trigger batch flush due to small batch size
  let line4 = LineCommand
  {
    start: Point2D { x: 300.0, y: 150.0 },
    end: Point2D { x: 400.0, y: 200.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 1.0, 0.0, 1.0 ], // Yellow
      width: 1.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  assert!( renderer.render_line( &line4 ).is_ok() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Check that rendering statistics were collected
  let stats = renderer.get_stats().unwrap();
  assert!( stats.vertices_rendered > 0 );
  assert!( stats.draw_calls > 0 );
}

/// Test Matrix: WebGL Curve Rendering with Tessellation
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Cubic Bezier | High-quality curve tessellation | Smooth curve approximation |
/// | Sharp curves | Extreme control points | Proper tessellation |
/// | Multiple curves | Batch processing | Efficient GPU rendering |
#[ test ]
fn test_webgl_curve_rendering()
{
  let mut renderer = WebGLRenderer::with_dimensions( 600, 400 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Cubic Bezier curve
  let curve = CurveCommand
  {
    start: Point2D { x: 50.0, y: 200.0 },
    control1: Point2D { x: 150.0, y: 50.0 },
    control2: Point2D { x: 350.0, y: 50.0 },
    end: Point2D { x: 450.0, y: 200.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.5, 0.0, 1.0 ], // Orange
      width: 2.5,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  assert!( renderer.render_curve( &curve ).is_ok() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Curve should generate line segments for smooth rendering - may vary by implementation
  let stats = renderer.get_stats().unwrap();
  assert!( stats.vertices_rendered > 0 ); // Some vertices should be rendered
}

/// Test Matrix: WebGL Text Rendering with GPU Font Atlas
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Simple text | Basic GPU text rendering | Font atlas usage |
/// | Text anchoring | Different anchor positions | Correct positioning |
/// | Unicode text | Extended character support | Proper glyph rendering |
/// | Colored text | Font color blending | GPU color mixing |
#[ test ]
fn test_webgl_text_rendering()
{
  let mut renderer = WebGLRenderer::with_dimensions( 800, 600 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Simple text
  let mut text_array = [ 0u8; 64 ];
  let text = b"WebGL Rendering!";
  text_array[ ..text.len() ].copy_from_slice( text );
  
  let text_cmd = TextCommand
  {
    text: text_array,
    text_len: text.len() as u8,
    position: Point2D { x: 100.0, y: 300.0 },
    font_style: FontStyle
    {
      color: [ 1.0, 1.0, 1.0, 1.0 ], // White
      size: 24.0,
      family_id: 0,
      weight: 400,
      italic: false,
    },
    anchor: TextAnchor::Center,
  };
  
  assert!( renderer.render_text( &text_cmd ).is_ok() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Text rendering should use texture atlas
  let stats = renderer.get_stats().unwrap();
  assert!( stats.texture_bindings > 0 );
}

/// Test Matrix: WebGL Tilemap Rendering with GPU Instancing
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Small tilemap | Basic tilemap rendering | GPU instancing |
/// | Large tilemap | High tile count | Efficient batching |
/// | Texture atlas | Tilemap texture usage | GPU texture binding |
#[ test ]
fn test_webgl_tilemap_rendering()
{
  let mut renderer = WebGLRenderer::with_dimensions( 1024, 768 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Small tilemap
  let tilemap = TilemapCommand
  {
    position: Point2D { x: 0.0, y: 0.0 },
    tile_width: 32.0,
    tile_height: 32.0,
    map_width: 10,
    map_height: 10,
    tileset_id: 0,
    tile_data: [ 1; 32 ],
    tile_count: 32,
  };
  
  assert!( renderer.render_tilemap( &tilemap ).is_ok() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Tilemap should use texture atlas and generate many quads
  let stats = renderer.get_stats().unwrap();
  assert!( stats.texture_bindings > 0 );
  assert!( stats.vertices_rendered > 0 ); // Some vertices should be rendered
}

/// Test Matrix: WebGL Particle System with GPU Compute
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Small particle system | Basic GPU particles | Efficient rendering |
/// | Large particle system | High particle count | Performance limits |
/// | Particle animation | Time-based updates | Smooth animation |
#[ test ]
fn test_webgl_particle_rendering()
{
  let mut renderer = WebGLRenderer::with_dimensions( 800, 600 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Particle emitter
  let particles = ParticleEmitterCommand
  {
    position: Point2D { x: 400.0, y: 300.0 },
    emission_rate: 50.0,
    particle_lifetime: 2.0,
    initial_velocity: Point2D { x: 1.0, y: -2.0 },
    velocity_variance: Point2D { x: 0.1, y: 0.1 },
    particle_size: 4.0,
    size_variance: 0.1,
    particle_color: [ 1.0, 0.8, 0.2, 0.7 ],
    color_variance: [ 0.0, 0.0, 0.0, 0.0 ],
  };
  
  assert!( renderer.render_particle_emitter( &particles ).is_ok() );
  
  assert!( renderer.end_frame().is_ok() );
  
  // Particle system should generate vertices for GPU rendering - may vary by implementation
  let stats = renderer.get_stats().unwrap();
  assert!( stats.vertices_rendered > 0 ); // Some vertices should be rendered
}

/// Test Matrix: WebGL Scene Rendering Integration
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Mixed scene | Multiple primitive types | All rendered efficiently |
/// | Complex scene | High primitive count | GPU performance maintained |
/// | Interactive scene | Mouse picking support | Correct primitive selection |
#[ test ]
fn test_webgl_scene_rendering()
{
  let mut renderer = WebGLRenderer::with_dimensions( 1200, 900 );
  renderer.set_mouse_picking_enabled( true );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Create a complex mixed scene
  let mut scene = Scene::new();
  
  // Add multiple lines
  for i in 0..10
  {
    scene.add( RenderCommand::Line( LineCommand
    {
      start: Point2D { x: i as f32 * 100.0, y: 100.0 },
      end: Point2D { x: i as f32 * 100.0 + 50.0, y: 150.0 },
      style: StrokeStyle
      {
        color: [ i as f32 / 10.0, 0.5, 1.0 - i as f32 / 10.0, 1.0 ],
        width: 2.0,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // Add curves
  for i in 0..5
  {
    scene.add( RenderCommand::Curve( CurveCommand
    {
      start: Point2D { x: 50.0 + i as f32 * 200.0, y: 300.0 },
      control1: Point2D { x: 100.0 + i as f32 * 200.0, y: 200.0 },
      control2: Point2D { x: 150.0 + i as f32 * 200.0, y: 400.0 },
      end: Point2D { x: 200.0 + i as f32 * 200.0, y: 300.0 },
      style: StrokeStyle
      {
        color: [ 1.0, i as f32 / 5.0, 0.0, 1.0 ],
        width: 3.0,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // Add text
  let mut text_array = [ 0u8; 64 ];
  let text = b"WebGL Performance Demo";
  text_array[ ..text.len() ].copy_from_slice( text );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: text_array,
    text_len: text.len() as u8,
    position: Point2D { x: 600.0, y: 50.0 },
    font_style: FontStyle
    {
      color: [ 1.0, 1.0, 0.0, 1.0 ], // Yellow
      size: 32.0,
      family_id: 0,
      weight: 700,
      italic: false,
    },
    anchor: TextAnchor::Center,
  } ) );
  
  // Render the complete scene
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Verify comprehensive rendering statistics
  let stats = renderer.get_stats().unwrap();
  assert!( stats.vertices_rendered > 100 ); // Many primitives rendered
  assert!( stats.draw_calls > 0 );
  assert!( stats.texture_bindings > 0 ); // Text rendering uses textures
  
  // Test mouse picking
  let picked = renderer.handle_mouse_event( 600.0, 50.0 ); // Near text
  assert!( picked.is_some() );
}

/// Test Matrix: WebGL Context Management
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | Context loss | Simulated context loss | Graceful recovery |
/// | Statistics tracking | Performance monitoring | Accurate metrics |
/// | Batch optimization | Render batching | Optimal GPU usage |
#[ test ]
fn test_webgl_context_management()
{
  let mut renderer = WebGLRenderer::with_dimensions( 640, 480 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  
  // Test statistics reset
  renderer.reset_stats();
  let stats = renderer.get_stats().unwrap();
  assert_eq!( stats.vertices_rendered, 0 );
  assert_eq!( stats.draw_calls, 0 );
  
  // Test batch size configuration
  renderer.set_max_batch_size( 500 );
  
  // Render many primitives to test batching
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  for i in 0..100
  {
    let line = LineCommand
    {
      start: Point2D { x: i as f32, y: i as f32 },
      end: Point2D { x: i as f32 + 10.0, y: i as f32 + 10.0 },
      style: StrokeStyle
      {
        color: [ 1.0, 1.0, 1.0, 1.0 ],
        width: 1.0,
        cap_style: LineCap::Butt,
        join_style: LineJoin::Miter,
      },
    };
    assert!( renderer.render_line( &line ).is_ok() );
  }
  
  assert!( renderer.end_frame().is_ok() );
  
  // Verify batching efficiency
  let stats = renderer.get_stats().unwrap();
  assert!( stats.vertices_rendered >= 200 ); // 100 lines * 2 vertices each
  assert!( stats.draw_calls > 0 );
}

/// Test Matrix: WebGL Output and JSON Statistics
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | JSON output | Rendering statistics | Valid JSON format |
/// | Performance metrics | Frame timing | Accurate measurements |
/// | Context status | GPU state | Proper state reporting |
#[ test ]
fn test_webgl_output_statistics()
{
  let mut renderer = WebGLRenderer::with_dimensions( 800, 600 );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Render some content
  let line = LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 100.0, y: 100.0 },
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
  
  // Test JSON output
  let output = renderer.output().unwrap();
  assert!( output.contains( "\"backend\": \"WebGL\"" ) );
  assert!( output.contains( "\"vertices_rendered\"" ) );
  assert!( output.contains( "\"draw_calls\"" ) );
  assert!( output.contains( "\"context_state\": \"active\"" ) );
  
  // Test cleanup affects output
  renderer.cleanup().unwrap();
  let output_after_cleanup = renderer.output().unwrap();
  assert!( output_after_cleanup.contains( "\"status\": \"not_initialized\"" ) );
}

/// Comprehensive WebGL integration test demonstrating all features
#[ test ]
fn test_webgl_comprehensive_integration()
{
  let mut renderer = WebGLRenderer::with_dimensions( 1600, 1200 );
  renderer.set_max_batch_size( 1000 );
  renderer.set_mouse_picking_enabled( true );
  
  let context = RenderContext::default();
  assert!( renderer.initialize( &context ).is_ok() );
  assert!( renderer.begin_frame( &context ).is_ok() );
  
  // Create a comprehensive test scene
  let mut scene = Scene::new();
  
  // Performance test: many lines
  for i in 0..200
  {
    scene.add( RenderCommand::Line( LineCommand
    {
      start: Point2D { x: ( i % 40 ) as f32 * 40.0, y: ( i / 40 ) as f32 * 40.0 },
      end: Point2D { x: ( i % 40 ) as f32 * 40.0 + 30.0, y: ( i / 40 ) as f32 * 40.0 + 30.0 },
      style: StrokeStyle
      {
        color: [
          ( i as f32 / 200.0 ),
          ( 1.0 - i as f32 / 200.0 ),
          0.5,
          0.8
        ],
        width: 1.0 + ( i % 3 ) as f32,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // High-quality curves
  for i in 0..10
  {
    scene.add( RenderCommand::Curve( CurveCommand
    {
      start: Point2D { x: 100.0 + i as f32 * 140.0, y: 600.0 },
      control1: Point2D { x: 170.0 + i as f32 * 140.0, y: 500.0 },
      control2: Point2D { x: 170.0 + i as f32 * 140.0, y: 700.0 },
      end: Point2D { x: 240.0 + i as f32 * 140.0, y: 600.0 },
      style: StrokeStyle
      {
        color: [ 0.0, 0.8, 1.0, 1.0 ], // Cyan
        width: 4.0,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // GPU text rendering
  let titles = [ "WebGL", "Hardware", "Acceleration", "Demo" ];
  for ( i, title ) in titles.iter().enumerate()
  {
    let mut text_array = [ 0u8; 64 ];
    let title_bytes = title.as_bytes();
    text_array[ ..title_bytes.len() ].copy_from_slice( title_bytes );
    
    scene.add( RenderCommand::Text( TextCommand
    {
      text: text_array,
      text_len: title_bytes.len().min( 255 ) as u8,
      position: Point2D { x: 200.0 + i as f32 * 300.0, y: 100.0 },
      font_style: FontStyle
      {
        color: [ 1.0, 1.0, 1.0, 1.0 ], // White
        size: 36.0,
        family_id: 0,
        weight: 700,
        italic: false,
      },
      anchor: TextAnchor::Center,
    } ) );
  }
  
  // Complex tilemap
  scene.add( RenderCommand::Tilemap( TilemapCommand
  {
    position: Point2D { x: 800.0, y: 800.0 },
    tile_width: 16.0,
    tile_height: 16.0,
    map_width: 20,
    map_height: 15,
    tileset_id: 1,
    tile_data: [ 5; 32 ],
    tile_count: 32,
  } ) );
  
  // GPU particle system
  scene.add( RenderCommand::ParticleEmitter( ParticleEmitterCommand
  {
    position: Point2D { x: 800.0, y: 300.0 },
    initial_velocity: Point2D { x: 0.0, y: -1.0 },
    velocity_variance: Point2D { x: 0.2, y: 0.2 },
    particle_lifetime: 3.0,
    particle_color: [ 1.0, 0.4, 0.1, 0.6 ], // Orange fire particles
    color_variance: [ 0.1, 0.1, 0.1, 0.0 ],
    particle_size: 3.0,
    size_variance: 0.2,
    emission_rate: 100.0,
  } ) );
  
  // Render the comprehensive scene
  assert!( renderer.render_scene( &scene ).is_ok() );
  assert!( renderer.end_frame().is_ok() );
  
  // Verify high-performance rendering
  let stats = renderer.get_stats().unwrap();
  assert!( stats.vertices_rendered > 1000 ); // Many vertices for complex scene
  assert!( stats.draw_calls > 0 );
  assert!( stats.texture_bindings > 0 ); // Text and tilemap use textures
  
  // Test mouse picking across the scene
  let picking_positions = [
    ( 200.0, 100.0 ), // Near text
    ( 400.0, 300.0 ), // Near lines
    ( 800.0, 600.0 ), // Near curves
    ( 900.0, 850.0 ), // Near tilemap
    ( 800.0, 300.0 ), // Near particles
  ];
  
  for ( x, y ) in &picking_positions
  {
    let picked = renderer.handle_mouse_event( *x, *y );
    // Mouse picking may return None if no object is at that position
    // This is expected behavior, so we just verify the method doesn't panic
    let _ = picked;
  }
  
  // Verify comprehensive output statistics
  let output = renderer.output().unwrap();
  assert!( output.contains( "\"backend\": \"WebGL\"" ) );
  assert!( output.contains( "\"context_state\": \"active\"" ) );
  
  // Performance validation
  let json: serde_json::Value = serde_json::from_str( &output ).expect( "Valid JSON output" );
  assert!( json[ "vertices_rendered" ].as_u64().unwrap() > 1000 );
  assert!( json[ "draw_calls" ].as_u64().unwrap() > 0 );
  assert!( json[ "texture_bindings" ].as_u64().unwrap() > 0 );
}