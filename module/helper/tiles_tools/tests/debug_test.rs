//! Tests for the `debug` module — pathfinding debugger rendering, ECS inspector
//! reports, performance profiler stats, formatting utilities, and coordinate
//! conversion, driven purely through the public surface.
//!
//! Relocated from `src/debug.rs` by task 072. The two formerly-inline
//! `GridRenderer` state tests moved here once the `width()`/`height()`/`style()`/
//! `marker_count()`/`has_marker()` getters made that state publicly observable,
//! per the all-tests-in-tests/ convention.

#![ cfg( feature = "enabled" ) ]


use tiles_tools::debug::*;
use std::time::Duration;

#[test]
fn test_pathfinding_debugger() {
  let mut debugger = PathfindingDebugger::new(10, 10);

  debugger.start_set((0, 0));
  debugger.goal_set((9, 9));
  debugger.obstacle_add((5, 5));
  debugger.path_add(vec![(0, 0), (1, 1), (2, 2), (3, 3)], "Test Path");

  let output = debugger.ascii_render();
  assert!(output.contains("Start"));
  assert!(output.contains("Goal"));
  assert!(output.contains("Obstacle"));
}

#[test]
fn test_ecs_inspector() {
  let mut inspector = ECSInspector::new();

  let entity = EntityDebugInfo {
    id: 42,
    components: vec!["Position".to_string(), "Health".to_string()],
    position: Some((10, 20)),
    data: vec![("level".to_string(), "5".to_string())].into_iter().collect(),
  };

  inspector.entity_record(entity);
  inspector.system_timing_record("MovementSystem".to_string(), Duration::from_millis(5));

  let report = inspector.report_generate();
  assert!(report.contains("Entity 42"));
  assert!(report.contains("Position"));
  assert!(report.contains("MovementSystem"));
}

#[test]
fn test_performance_profiler() {
  let mut profiler = PerformanceProfiler::new();

  profiler.frame_time_record(Duration::from_millis(16));
  profiler.frame_time_record(Duration::from_millis(18));
  profiler.system_time_record("RenderSystem".to_string(), Duration::from_millis(8));
  profiler.memory_sample_record(1024 * 1024, 100); // 1MB, 100 entities

  let stats = profiler.stats_get();
  assert_eq!(stats.frame_count, 2);
  assert!(stats.fps > 0.0);
  assert_eq!(stats.current_memory, 1024 * 1024);
  assert_eq!(stats.current_entities, 100);
}

#[test]
fn test_debug_utilities() {
  let grid = vec![
    vec![true, false, true],
    vec![false, true, false],
    vec![true, true, false],
  ];

  let output = utils::bool_grid_render(&grid, '#', '.');
  assert!(output.contains('#'));
  assert!(output.contains('.'));

  let duration = Duration::from_micros(1500);
  let formatted = utils::duration_format(duration);
  assert!(formatted.contains("1.5ms"));

  let memory = utils::memory_format(1536 * 1024); // 1.5 MB
  assert!(memory.contains("1.5") && memory.contains("MB"));
}

#[test]
fn test_coordinate_conversion() {
  let int_coord = (5, 10);
  let float_coord = (5.7, 10.3);
  let usize_coord = (5usize, 10usize);

  assert_eq!(int_coord.into_debug_coord(), (5, 10));
  assert_eq!(float_coord.into_debug_coord(), (5, 10));
  assert_eq!(usize_coord.into_debug_coord(), (5, 10));
}

/// Pins `GridRenderer`'s builder-state accumulation ( size and style ) through the
/// `width()`/`height()`/`style()` getters.
#[ test ]
fn test_grid_renderer_creation()
{
  let renderer = GridRenderer::new()
  .with_size( 10, 8 )
  .with_style( GridStyle::Hexagonal );

  assert_eq!( renderer.width(), 10 );
  assert_eq!( renderer.height(), 8 );
  assert!( matches!( renderer.style(), GridStyle::Hexagonal ) );
}

/// Pins marker storage through the `marker_count()`/`has_marker()` queries — rendering
/// output is deliberately not used here, since a marker can be stored yet not rendered.
#[ test ]
fn test_grid_renderer_markers()
{
  let mut renderer = GridRenderer::new();
  renderer.marker_add( ( 5, 3 ), "S", "Start position" );
  renderer.colored_marker_add( ( 8, 6 ), "G", "Goal", DebugColor::Blue, 10 );

  assert_eq!( renderer.marker_count(), 2 );
  assert!( renderer.has_marker( ( 5, 3 ) ) );
  assert!( renderer.has_marker( ( 8, 6 ) ) );
}
