//! WebGPU adapter comprehensive test suite.
//!
//! This test suite validates the WebGPU backend adapter implementation
//! following the Test Matrix approach from the design rulebook.

use tilemap_renderer::
{
  adapters::WebGPURenderer,
  ports::{ Renderer, PrimitiveRenderer, RenderContext },
  commands::{ LineCommand, CurveCommand, TextCommand, Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, TilemapCommand, ParticleEmitterCommand, RenderCommand },
  scene::Scene,
};

/// Test Matrix: WebGPU Renderer Creation and Configuration
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | new() | Default constructor | Default settings |
/// | with_dimensions() | Custom size | Specified dimensions |
/// | set_mouse_picking_enabled() | Mouse picking toggle | Interactive functionality |
/// | set_particle_simulation_enabled() | Particle simulation | Advanced GPU features |
/// | set_tessellation_enabled() | Tessellation toggle | High-quality curves |
/// | set_max_workgroups() | Compute workgroup limits | GPU utilization |
#[ test ]
fn test_webgpu_renderer_creation()
{
  // Default constructor
  let renderer = WebGPURenderer::new();
  assert!( renderer.get_stats().is_none() ); // Not initialized yet
  
  // Custom dimensions
  let renderer = WebGPURenderer::with_dimensions( 1920, 1080 );
  assert!( renderer.get_stats().is_none() ); // Context not initialized
}

#[ test ]
fn test_webgpu_renderer_configuration()
{
  let mut renderer = WebGPURenderer::new();
  
  // Test mouse picking configuration
  renderer.set_mouse_picking_enabled( true );
  
  // Test particle simulation configuration
  renderer.set_particle_simulation_enabled( true );
  
  // Test tessellation configuration
  renderer.set_tessellation_enabled( true );
  
  // Test workgroup configuration
  renderer.set_max_workgroups( 2048 );
  
  // Test mouse event handling (should return None when not initialized)
  let picked = renderer.handle_mouse_event( 100.0, 200.0 );
  assert!( picked.is_none() );
}

/// Test Matrix: WebGPU Renderer Lifecycle
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | capabilities() | Backend info query | WebGPU capabilities |
/// | initialize() | Context setup | Ready for rendering |
/// | begin_frame() | Frame start | Active frame state |
/// | end_frame() | Frame completion | Output ready |
/// | cleanup() | Resource release | Clean state |
#[ test ]
fn test_webgpu_renderer_lifecycle()
{
  let mut renderer = WebGPURenderer::new();
  let context = RenderContext::default();
  
  // Test capabilities before initialization
  let caps = renderer.capabilities();
  assert_eq!( caps.backend_name, "WebGPU" );
  assert_eq!( caps.backend_version, "1.0" );
  assert_eq!( caps.max_texture_size, 8192 );
  assert!( caps.supports_transparency );
  assert!( caps.supports_antialiasing );
  assert!( caps.supports_custom_fonts );
  assert!( caps.supports_particles );
  assert!( caps.supports_realtime );
  assert_eq!( caps.max_scene_complexity, 1_000_000 );
  
  // Test initialization
  let init_result = renderer.initialize( &context );
  assert!( init_result.is_ok() );
  
  // Test frame lifecycle
  let begin_result = renderer.begin_frame( &context );
  assert!( begin_result.is_ok() );
  
  let end_result = renderer.end_frame();
  assert!( end_result.is_ok() );
  
  // Test output generation
  let output_result = renderer.output();
  assert!( output_result.is_ok() );
  let output = output_result.unwrap();
  assert!( output.contains( "WebGPU" ) );
  
  // Test cleanup
  let cleanup_result = renderer.cleanup();
  assert!( cleanup_result.is_ok() );
}

/// Test Matrix: WebGPU Primitive Rendering
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | render_line() | Line primitive | GPU-accelerated line |
/// | render_curve() | Bezier curve | Tessellated curve |
/// | render_text() | Text rendering | GPU font atlas |
/// | render_tilemap() | Tilemap rendering | Instanced rendering |
/// | render_particle_emitter() | Particle system | Compute shader simulation |
#[ test ]
fn test_webgpu_primitive_rendering()
{
  let mut renderer = WebGPURenderer::new();
  let context = RenderContext::default();
  
  // Initialize renderer
  renderer.initialize( &context ).unwrap();
  renderer.begin_frame( &context ).unwrap();
  
  // Test line rendering
  let line = LineCommand
  {
    start: Point2D { x: 10.0, y: 10.0 },
    end: Point2D { x: 100.0, y: 100.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ],
      width: 2.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Miter,
    },
  };
  let line_result = renderer.render_line( &line );
  assert!( line_result.is_ok() );
  
  // Test curve rendering
  let curve = CurveCommand
  {
    start: Point2D { x: 10.0, y: 10.0 },
    control1: Point2D { x: 30.0, y: 80.0 },
    control2: Point2D { x: 70.0, y: 80.0 },
    end: Point2D { x: 90.0, y: 10.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 1.0, 0.0, 1.0 ],
      width: 3.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  let curve_result = renderer.render_curve( &curve );
  assert!( curve_result.is_ok() );
  
  // Test text rendering
  let text = TextCommand
  {
    text: *b"WebGPU Test Text\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    text_len: 16,
    position: Point2D { x: 50.0, y: 50.0 },
    font_style: FontStyle
    {
      family_id: 0,
      size: 16.0,
      weight: 400,
      italic: false,
      color: [ 0.0, 0.0, 1.0, 1.0 ],
    },
    anchor: TextAnchor::TopLeft,
  };
  let text_result = renderer.render_text( &text );
  assert!( text_result.is_ok() );
  
  // Test tilemap rendering
  let tilemap = TilemapCommand
  {
    position: Point2D { x: 0.0, y: 0.0 },
    tile_width: 32.0,
    tile_height: 32.0,
    map_width: 4,
    map_height: 4,
    tile_count: 16,
    tile_data: [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
    tileset_id: 1,
  };
  let tilemap_result = renderer.render_tilemap( &tilemap );
  assert!( tilemap_result.is_ok() );
  
  // Test particle emitter rendering
  let particles = ParticleEmitterCommand
  {
    position: Point2D { x: 200.0, y: 200.0 },
    emission_rate: 50.0,
    particle_lifetime: 2.0,
    initial_velocity: Point2D { x: 10.0, y: -20.0 },
    particle_size: 4.0,
    particle_color: [ 1.0, 1.0, 0.0, 0.8 ],
    color_variance: [ 0.1, 0.1, 0.1, 0.0 ],
    size_variance: 1.0,
    velocity_variance: Point2D { x: 2.0, y: 2.0 },
  };
  let particles_result = renderer.render_particle_emitter( &particles );
  assert!( particles_result.is_ok() );
  
  renderer.end_frame().unwrap();
}

/// Test Matrix: WebGPU Scene Rendering
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | render_scene() | Complete scene | All primitives rendered |
/// | complex_scene() | High complexity | Performance within limits |
#[ test ]
fn test_webgpu_scene_rendering()
{
  let mut renderer = WebGPURenderer::new();
  let context = RenderContext::default();
  
  // Initialize renderer
  renderer.initialize( &context ).unwrap();
  renderer.begin_frame( &context ).unwrap();
  
  // Create test scene with multiple primitives
  let mut scene = Scene::new();
  
  // Add line command wrapped in RenderCommand
  let line_cmd = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 50.0, y: 50.0 },
    style: StrokeStyle::default(),
  } );
  scene.add( line_cmd );
  
  // Add curve command wrapped in RenderCommand  
  let curve_cmd = RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 50.0, y: 0.0 },
    control1: Point2D { x: 75.0, y: 25.0 },
    control2: Point2D { x: 75.0, y: 25.0 },
    end: Point2D { x: 100.0, y: 50.0 },
    style: StrokeStyle::default(),
  } );
  scene.add( curve_cmd );
  
  // Add text command wrapped in RenderCommand
  let text_cmd = RenderCommand::Text( TextCommand
  {
    text: *b"WebGPU\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    text_len: 6,
    position: Point2D { x: 25.0, y: 25.0 },
    font_style: FontStyle::default(),
    anchor: TextAnchor::TopLeft,
  } );
  scene.add( text_cmd );
  
  // Test scene rendering
  let scene_result = renderer.render_scene( &scene );
  assert!( scene_result.is_ok() );
  
  renderer.end_frame().unwrap();
  
  // Verify output contains expected content
  let output = renderer.output().unwrap();
  assert!( output.contains( "WebGPU" ) );
  assert!( output.contains( "vertices_computed" ) );
  assert!( output.contains( "render_passes" ) );
  assert!( output.contains( "compute_passes" ) );
}

/// Test Matrix: WebGPU Statistics and Performance
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | get_stats() | Statistics retrieval | Valid performance data |
/// | reset_stats() | Statistics reset | Cleared counters |
#[ test ]
fn test_webgpu_statistics()
{
  let mut renderer = WebGPURenderer::new();
  let context = RenderContext::default();
  
  // Initialize and render something
  renderer.initialize( &context ).unwrap();
  renderer.begin_frame( &context ).unwrap();
  
  let line = LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 100.0, y: 100.0 },
    style: StrokeStyle::default(),
  };
  renderer.render_line( &line ).unwrap();
  
  renderer.end_frame().unwrap();
  
  // Check statistics
  let stats = renderer.get_stats();
  assert!( stats.is_some() );
  let stats = stats.unwrap();
  assert!( stats.render_passes > 0 );
  
  // Reset statistics
  renderer.reset_stats();
  let reset_stats = renderer.get_stats();
  assert!( reset_stats.is_some() );
  let reset_stats = reset_stats.unwrap();
  assert_eq!( reset_stats.vertices_computed, 0 );
  assert_eq!( reset_stats.render_passes, 0 );
  assert_eq!( reset_stats.compute_passes, 0 );
}

/// Test Matrix: WebGPU Error Handling
/// 
/// | Test Case | Description | Expected Outcome |
/// |-----------|-------------|------------------|
/// | double_initialization() | Duplicate init | Proper error |
/// | frame_without_init() | No initialization | Error handling |
/// | render_without_frame() | No active frame | Error handling |
/// | end_without_begin() | No frame started | Error handling |
#[ test ]
fn test_webgpu_error_handling()
{
  let mut renderer = WebGPURenderer::new();
  let context = RenderContext::default();
  
  // Test rendering without initialization
  let begin_result = renderer.begin_frame( &context );
  assert!( begin_result.is_err() );
  
  // Initialize properly
  renderer.initialize( &context ).unwrap();
  
  // Test double initialization
  let double_init = renderer.initialize( &context );
  assert!( double_init.is_err() );
  
  // Test ending frame without beginning
  let end_without_begin = renderer.end_frame();
  assert!( end_without_begin.is_err() );
  
  // Test rendering without active frame
  let line = LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 100.0, y: 100.0 },
    style: StrokeStyle::default(),
  };
  let render_without_frame = renderer.render_line( &line );
  assert!( render_without_frame.is_err() );
  
  // Test double frame begin
  renderer.begin_frame( &context ).unwrap();
  let double_begin = renderer.begin_frame( &context );
  assert!( double_begin.is_err() );
  
  renderer.end_frame().unwrap();
}