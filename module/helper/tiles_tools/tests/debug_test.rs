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

// test_kind: bug_reproducer(BUG-266)
/// ## Root Cause
/// `GridRenderer::svg_grid_render`'s fallback arm for styles without
/// dedicated SVG grid-line art called `self.svg_grid_render( writer,
/// cell_size )` -- itself -- instead of a square-grid helper. `self.style`
/// never changes between calls, so every recursive call matched the same
/// fallback arm again, recursing unconditionally with no base case until the
/// stack overflowed and the process aborted.
/// ## Why Not Caught
/// No existing test exercised `svg_export()` with `GridStyle::Triangular` or
/// `GridStyle::Isometric` -- the only style-specific SVG coverage exercised
/// `Square4`/`Hexagonal` indirectly through `PathfindingDebugger`, so the
/// fallback match arm was never reached end-to-end by any test.
/// ## Fix Applied
/// Extracted the square-grid line-drawing body into a new private
/// `square_svg_grid_render()` helper and made both the `Square4`/`Square8`
/// arm and the `_` fallback arm call that helper, instead of the fallback
/// arm calling `svg_grid_render` (itself) recursively.
/// ## Prevention
/// Exercise `svg_export()` for every `GridStyle` variant, not only the ones
/// with bespoke rendering art -- a fallback arm is exactly where an
/// accidental self-call is easiest to miss, since it reads like ordinary
/// delegation at a glance.
/// ## Pitfall
/// A `match` arm meant to "fall back to another case" must call a genuinely
/// different function or change the matched value -- calling the same
/// method on the same `self` from its own wildcard arm is unconditional
/// infinite recursion (a guaranteed stack-overflow abort), not a fallback.
#[ test ]
fn test_svg_export_triangular_and_isometric_styles_do_not_recurse_infinitely()
{
  for style in [ GridStyle::Triangular, GridStyle::Isometric ]
  {
    let renderer = GridRenderer::new()
    .with_size( 3, 3 )
    .with_style( style );

    let path = std::env::temp_dir().join( format!( "-tiles_tools_debug_svg_export_test_{style:?}.svg" ) );
    let result = renderer.svg_export( &path );

    assert!( result.is_ok(), "svg_export should succeed for {style:?} instead of recursing infinitely" );
    let contents = std::fs::read_to_string( &path ).expect( "SVG file should have been written" );
    assert!( contents.contains( "<line" ), "fallback rendering should draw grid lines for {style:?}" );

    std::fs::remove_file( &path ).ok();
  }
}

// BUG-347 task/bug/347_ecs_inspector_entity_record_inflates_component_counts.md
// -- reproducer for component_counts inflation on entity re-recording.
// test_kind: bug_reproducer(BUG-347)
/// ## Root Cause
/// `ECSInspector::entity_record` increments `component_counts` for each of an
/// entity's components on every call, but never decrements the *previous*
/// call's contribution before a re-record overwrites the same `entity_id` in
/// `entity_data`. There is no `entity_remove`/`unrecord` method anywhere in
/// `ECSInspector` either, so a component tally inflated by re-recording can
/// never be corrected.
/// ## Why Not Caught
/// `test_ecs_inspector` only ever calls `entity_record` once per entity ID --
/// no existing test re-records the same `entity_id` with a different
/// component set, which is the only way the stale increment becomes
/// observable.
/// ## Fix Applied
/// `entity_record` now looks up any existing entry for `entity.id` before
/// overwriting it, decrements `component_counts` for that old entry's
/// components, and only then applies the new insert and increments.
/// ## Prevention
/// n/a -- covered by this test.
/// ## Pitfall
/// A counter incremented inside a "record" method that can be called more
/// than once for the same identity needs a matching decrement path for the
/// value(s) it is replacing -- otherwise every re-record silently inflates
/// the counter forever, with no panic or error to surface it.
#[ test ]
fn test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts()
{
  let mut inspector = ECSInspector::new();

  inspector.entity_record( EntityDebugInfo
  {
    id : 1,
    components : vec![ "Position".to_string() ],
    position : None,
    data : vec![].into_iter().collect(),
  });

  inspector.entity_record( EntityDebugInfo
  {
    id : 1,
    components : vec![ "Position".to_string(), "Health".to_string() ],
    position : None,
    data : vec![].into_iter().collect(),
  });

  assert_eq!( inspector.entity_count(), 1 );

  let report = inspector.report_generate();
  assert!(
    report.contains( "Position: 1 entities" ),
    "re-recording entity 1 should leave Position's count at 1 (one currently-recorded entity), got:\n{report}"
  );
}
