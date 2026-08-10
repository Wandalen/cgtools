//! Tests for the `debug` module — pathfinding debugger rendering, ECS inspector
//! reports, performance profiler stats, formatting utilities, and coordinate
//! conversion, driven purely through the public surface.
//!
//! Relocated from `src/debug.rs` by task 072. Two `GridRenderer` state tests remain
//! inline in `src/debug.rs` as a documented exception (they pin private builder
//! fields with no public accessor).

#![ cfg( feature = "enabled" ) ]

#![allow(clippy::needless_return)]
#![allow(clippy::implicit_return)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::similar_names)]
#![allow(clippy::duplicated_attributes)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_else)]

use tiles_tools::debug::*;
use std::time::Duration;

#[test]
fn test_pathfinding_debugger() {
  let mut debugger = PathfindingDebugger::new(10, 10);

  debugger.set_start((0, 0));
  debugger.set_goal((9, 9));
  debugger.add_obstacle((5, 5));
  debugger.add_path(vec![(0, 0), (1, 1), (2, 2), (3, 3)], "Test Path");

  let output = debugger.render_ascii();
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

  inspector.record_entity(entity);
  inspector.record_system_timing("MovementSystem".to_string(), Duration::from_millis(5));

  let report = inspector.generate_report();
  assert!(report.contains("Entity 42"));
  assert!(report.contains("Position"));
  assert!(report.contains("MovementSystem"));
}

#[test]
fn test_performance_profiler() {
  let mut profiler = PerformanceProfiler::new();

  profiler.record_frame_time(Duration::from_millis(16));
  profiler.record_frame_time(Duration::from_millis(18));
  profiler.record_system_time("RenderSystem".to_string(), Duration::from_millis(8));
  profiler.record_memory_sample(1024 * 1024, 100); // 1MB, 100 entities

  let stats = profiler.get_stats();
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

  let output = utils::render_bool_grid(&grid, '#', '.');
  assert!(output.contains('#'));
  assert!(output.contains('.'));

  let duration = Duration::from_micros(1500);
  let formatted = utils::format_duration(duration);
  assert!(formatted.contains("1.5ms"));

  let memory = utils::format_memory(1536 * 1024); // 1.5 MB
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
