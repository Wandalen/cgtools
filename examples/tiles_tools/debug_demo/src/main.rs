//! Debug system demonstration showing visual debugging and profiling tools.

//!
//! This example demonstrates the comprehensive debug system including:
//! - Grid visualization with multiple styles and coordinate systems
//! - Pathfinding debug overlays with cost visualization
//! - ECS component inspection and entity tracking
//! - Performance profiling with frame timing and bottleneck detection
//! - ASCII art rendering of grid, pathfinding, and profiling state
//! - Memory usage monitoring and system performance metrics

use tiles_tools::debug::{GridRenderer, GridStyle, DebugColor, HighlightStyle, PathfindingDebugger, ECSInspector, EntityDebugInfo, PerformanceProfiler};
use tiles_tools::debug::utils;
use std::time::{ Duration, Instant };
use std::collections::HashMap;

fn main()
{
  println!("🔍 Debug System Demonstration");
  println!("==============================");

  // === GRID RENDERER DEMONSTRATION ===
  println!("\n📊 Grid Renderer");
  println!("----------------");

  // Create different grid styles
  grid_styles_demonstrate();
  
  // === PATHFINDING DEBUG DEMONSTRATION ===
  println!("\n🗺️ Pathfinding Debug Visualization");
  println!("----------------------------------");
  
  pathfinding_debug_demonstrate();

  // === ECS INSPECTOR DEMONSTRATION ===
  println!("\n🔍 ECS Component Inspector");
  println!("-------------------------");
  
  ecs_inspector_demonstrate();

  // === PERFORMANCE PROFILER DEMONSTRATION ===
  println!("\n⚡ Performance Profiler");
  println!("----------------------");
  
  performance_profiler_demonstrate();

  // === UTILITY FUNCTIONS DEMONSTRATION ===
  println!("\n🛠️ Debug Utilities");
  println!("------------------");
  
  debug_utilities_demonstrate();

  // === INTEGRATION DEMONSTRATION ===
  println!("\n🎮 Integrated Game Debug Session");
  println!("-------------------------------");
  
  integrated_debugging_demonstrate();

  println!("\n✨ Debug Demo Complete!");
  println!("\nKey features demonstrated:");
  println!("• Grid visualization with multiple coordinate systems");
  println!("• Pathfinding debug overlays and cost visualization");
  println!("• ECS entity and component inspection");
  println!("• Performance profiling and bottleneck detection");
  println!("• ASCII art rendering for console debugging");
  println!("• SVG export for documentation and analysis");
  println!("• Memory usage monitoring and system metrics");
  println!("• Integrated debugging workflows");
}

fn grid_styles_demonstrate()
{
  println!("Testing different grid styles...");

  // Square grid with markers
  let mut square_grid = GridRenderer::new()
    .with_size(8, 6)
    .with_style(GridStyle::Square4);

  square_grid.colored_marker_add((1, 1), "S", "Start", DebugColor::Green, 10);
  square_grid.colored_marker_add((6, 4), "G", "Goal", DebugColor::Blue, 10);
  square_grid.colored_marker_add((3, 2), "X", "Obstacle", DebugColor::Red, 5);
  square_grid.path_add(vec![(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (6, 2), (6, 3), (6, 4)], "Path", DebugColor::Yellow);

  println!("\n□ Square Grid (4-connected):");
  println!("{}", square_grid.ascii_render());

  // Export square grid as SVG (commented out file operations for demo)
  // square_grid.svg_export("-debug_square_grid.svg").expect("Failed to export SVG");
  // println!("✅ Square grid exported to -debug_square_grid.svg");

  // Hexagonal grid
  let mut hex_grid = GridRenderer::new()
    .with_size(10, 7)
    .with_style(GridStyle::Hexagonal);

  hex_grid.colored_marker_add((2, 2), "H", "Hero", DebugColor::Green, 10);
  hex_grid.colored_marker_add((7, 5), "T", "Treasure", DebugColor::Yellow, 10);
  hex_grid.area_add(vec![(4, 3), (5, 3), (4, 4), (5, 4)], "Water", DebugColor::Blue, HighlightStyle::Fill);

  println!("⬢ Hexagonal Grid:");
  println!("{}", hex_grid.ascii_render());

  // Triangular grid
  let mut tri_grid = GridRenderer::new()
    .with_size(8, 5)
    .with_style(GridStyle::Triangular);

  tri_grid.colored_marker_add((3, 2), "△", "Peak", DebugColor::Purple, 10);
  tri_grid.area_add(vec![(1, 4), (2, 4), (3, 4)], "Valley", DebugColor::Green, HighlightStyle::Outline);

  println!("▲ Triangular Grid:");
  println!("{}", tri_grid.ascii_render());
}

fn pathfinding_debug_demonstrate()
{
  let mut pathfinder = PathfindingDebugger::new(12, 8);

  // Set up a pathfinding scenario
  pathfinder.start_set((1, 1));
  pathfinder.goal_set((10, 6));

  // Add obstacles
  let obstacles = vec![
    (4, 2), (4, 3), (4, 4), (4, 5),
    (7, 1), (7, 2), (7, 3),
    (2, 6), (3, 6), (4, 6),
  ];

  for obstacle in obstacles {
    pathfinder.obstacle_add(obstacle);
  }

  // Add found path
  let path = vec![
    (1, 1), (2, 1), (3, 1), (5, 1), (6, 1),
    (8, 1), (9, 1), (10, 1), (10, 2), (10, 3),
    (10, 4), (10, 5), (10, 6),
  ];
  pathfinder.path_add(path, "Optimal Path");

  // Add algorithm state
  let visited = vec![
    (1, 1), (2, 1), (3, 1), (1, 2), (2, 2),
    (5, 1), (6, 1), (5, 2), (6, 2), (7, 4),
    (8, 1), (9, 1), (8, 4), (9, 4), (10, 1),
    (10, 2), (10, 3),
  ];
  pathfinder.visited_nodes_add(visited);

  let open = vec![(3, 0), (4, 1), (5, 0), (10, 4), (10, 5), (9, 6)];
  pathfinder.open_nodes_add(open);

  // Add cost information
  let mut costs = HashMap::new();
  costs.insert((2, 2), 3);  // Rough terrain
  costs.insert((5, 2), 2);  // Hill
  costs.insert((8, 4), 4);  // Swamp
  costs.insert((9, 4), 4);  // Swamp
  costs.insert((9, 5), 2);  // Hill
  pathfinder.costs_set(costs);

  println!("Pathfinding Debug Visualization:");
  println!("{}", pathfinder.ascii_render());

  // Export pathfinding debug
  // pathfinder.svg_export("-debug_pathfinding.svg").expect("Failed to export pathfinding SVG");
  // println!("✅ Pathfinding debug exported to -debug_pathfinding.svg");
}

fn ecs_inspector_demonstrate()
{
  let mut inspector = ECSInspector::new();

  // Simulate entity data from a game session
  let entities = vec![
    EntityDebugInfo {
      id: 1,
      components: vec!["Position".to_string(), "Health".to_string(), "Player".to_string()],
      position: Some((5, 10)),
      data: vec![
        ("health".to_string(), "100".to_string()),
        ("level".to_string(), "5".to_string()),
        ("class".to_string(), "Warrior".to_string()),
      ].into_iter().collect(),
    },
    EntityDebugInfo {
      id: 2,
      components: vec!["Position".to_string(), "AI".to_string(), "Health".to_string()],
      position: Some((15, 8)),
      data: vec![
        ("health".to_string(), "75".to_string()),
        ("ai_state".to_string(), "Patrolling".to_string()),
        ("enemy_type".to_string(), "Orc".to_string()),
      ].into_iter().collect(),
    },
    EntityDebugInfo {
      id: 3,
      components: vec!["Position".to_string(), "Velocity".to_string(), "Projectile".to_string()],
      position: Some((12, 12)),
      data: vec![
        ("damage".to_string(), "25".to_string()),
        ("speed".to_string(), "10.0".to_string()),
        ("owner".to_string(), "1".to_string()),
      ].into_iter().collect(),
    },
    EntityDebugInfo {
      id: 4,
      components: vec!["Position".to_string(), "Health".to_string(), "AI".to_string()],
      position: Some((3, 15)),
      data: vec![
        ("health".to_string(), "50".to_string()),
        ("ai_state".to_string(), "Fleeing".to_string()),
        ("enemy_type".to_string(), "Goblin".to_string()),
      ].into_iter().collect(),
    },
    EntityDebugInfo {
      id: 5,
      components: vec!["Position".to_string(), "Item".to_string()],
      position: Some((20, 5)),
      data: vec![
        ("item_type".to_string(), "HealthPotion".to_string()),
        ("value".to_string(), "50".to_string()),
      ].into_iter().collect(),
    },
  ];

  // Record entity data
  for entity in entities {
    inspector.entity_record(entity);
  }

  // Record system timings
  inspector.system_timing_record("MovementSystem".to_string(), Duration::from_micros(1500));
  inspector.system_timing_record("RenderSystem".to_string(), Duration::from_micros(8200));
  inspector.system_timing_record("AISystem".to_string(), Duration::from_micros(3100));
  inspector.system_timing_record("PhysicsSystem".to_string(), Duration::from_micros(4700));
  inspector.system_timing_record("CollisionSystem".to_string(), Duration::from_micros(2800));

  println!("ECS Inspector Report:");
  println!("{}", inspector.report_generate());

  println!("\nECS Data as JSON:");
  println!("{}", inspector.json_export());
}

fn performance_profiler_demonstrate()
{
  let mut profiler = PerformanceProfiler::new();

  // Simulate frame data over time
  println!("Simulating game performance over 120 frames...");

  let base_frame_time = Duration::from_micros(16667); // ~60 FPS
  
  for frame in 0..120 {
    // Simulate varying frame times
    let variance = if frame % 20 == 0 {
      // Occasional spike
      Duration::from_millis(8)
    } else if frame % 7 == 0 {
      // Regular minor spike
      Duration::from_millis(2)
    } else {
      Duration::from_micros(u64::from((frame * 37) % 1000)) // Random variance
    };

    let frame_time = base_frame_time + variance;
    profiler.frame_time_record(frame_time);

    // Record system times for this frame
    profiler.system_time_record("MovementSystem".to_string(), Duration::from_micros(1000 + u64::from(frame % 500)));
    profiler.system_time_record("RenderSystem".to_string(), Duration::from_micros(8000 + u64::from(frame % 2000)));
    profiler.system_time_record("AISystem".to_string(), Duration::from_micros(2000 + u64::from(frame % 800)));
    profiler.system_time_record("PhysicsSystem".to_string(), Duration::from_micros(3000 + u64::from(frame % 1200)));

    // Record memory samples every 10 frames
    if frame % 10 == 0 {
      let base_memory = 50 * 1024 * 1024; // 50MB base
      let memory_growth = u64::from(frame) * 1024 * 10; // 10KB per frame
      let entity_count = 100 + (frame / 10) * 5; // Growing entity count
      
      profiler.memory_sample_record(base_memory + memory_growth, entity_count);
    }
  }

  println!("Performance Profile Report:");
  println!("{}", profiler.report_generate());

  let stats = profiler.stats_get();
  println!("\nQuick Performance Summary:");
  println!("• Average FPS: {:.1}", stats.fps);
  println!("• Frame Time: {:.2}ms avg, {:.2}ms max", 
    stats.avg_frame_time.as_secs_f64() * 1000.0,
    stats.max_frame_time.as_secs_f64() * 1000.0);
  println!("• Memory: {}", utils::memory_format(stats.current_memory));
  println!("• Entities: {}", stats.current_entities);
  println!("• Uptime: {}", utils::duration_format(stats.uptime));

  // Export performance data (commented out for demo)
  // profiler.csv_export("-performance_data.csv").expect("Failed to export CSV");
  // println!("✅ Performance data exported to performance_data.csv");
}

fn debug_utilities_demonstrate()
{
  println!("Testing debug utility functions...");

  // Boolean grid visualization
  let visibility_map = vec![
    vec![true, true, false, false, true],
    vec![true, false, false, true, true],
    vec![false, false, true, true, true],
    vec![true, false, true, false, false],
    vec![true, true, true, true, false],
  ];

  println!("\nVisibility Map (# = visible, . = hidden):");
  println!("{}", utils::bool_grid_render(&visibility_map, '#', '.'));

  // Duration formatting
  let durations = vec![
    Duration::from_nanos(500),
    Duration::from_micros(150),
    Duration::from_millis(25),
    Duration::from_secs(2),
  ];

  println!("Duration Formatting:");
  for duration in durations {
    println!("• {}", utils::duration_format(duration));
  }

  // Memory formatting
  let memory_sizes = vec![512, 1536, 2048 * 1024, 1536 * 1024 * 1024];
  
  println!("\nMemory Formatting:");
  for size in memory_sizes {
    println!("• {}", utils::memory_format(size));
  }
}

fn integrated_debugging_demonstrate()
{
  println!("Simulating integrated debugging session...");

  // Create a game world debug scenario
  let mut main_renderer = GridRenderer::new()
    .with_size(15, 10)
    .with_style(GridStyle::Square8);

  // Set up a tactical game scenario
  main_renderer.colored_marker_add((2, 2), "P1", "Player 1", DebugColor::Green, 20);
  main_renderer.colored_marker_add((12, 8), "P2", "Player 2", DebugColor::Blue, 20);
  
  // Add enemies
  main_renderer.colored_marker_add((7, 3), "E1", "Enemy Archer", DebugColor::Red, 15);
  main_renderer.colored_marker_add((5, 7), "E2", "Enemy Knight", DebugColor::Red, 15);
  
  // Add environmental elements
  main_renderer.colored_marker_add((6, 4), "T", "Tree", DebugColor::Green, 5);
  main_renderer.colored_marker_add((8, 6), "R", "Rock", DebugColor::Gray, 5);
  
  // Add area effects
  main_renderer.area_add(
    vec![(10, 4), (11, 4), (10, 5), (11, 5)], 
    "Fire Area", 
    DebugColor::Orange, 
    HighlightStyle::Fill
  );

  // Add movement ranges
  main_renderer.area_add(
    vec![(1, 1), (2, 1), (3, 1), (1, 2), (3, 2), (1, 3), (2, 3), (3, 3)],
    "P1 Movement Range",
    DebugColor::Green,
    HighlightStyle::Dotted
  );

  // Add annotations
  main_renderer.annotation_add((7, 1), "Archer Range", DebugColor::Red);
  main_renderer.annotation_add((10, 3), "Danger Zone", DebugColor::Yellow);

  println!("\nTactical Game State:");
  println!("{}", main_renderer.ascii_render());

  // Performance snapshot for this frame
  let mut frame_profiler = PerformanceProfiler::new();
  let start = Instant::now();
  
  // Simulate some game logic timing
  std::thread::sleep(Duration::from_micros(100)); // Simulate work
  frame_profiler.system_time_record("GameLogic".to_string(), start.elapsed());

  let render_start = Instant::now();
  std::thread::sleep(Duration::from_micros(200)); // Simulate rendering
  frame_profiler.system_time_record("Rendering".to_string(), render_start.elapsed());

  frame_profiler.frame_time_record(start.elapsed());
  frame_profiler.memory_sample_record(45 * 1024 * 1024, 8); // 45MB, 8 entities

  println!("\nFrame Performance:");
  let stats = frame_profiler.stats_get();
  println!("• Frame time: {}", utils::duration_format(stats.avg_frame_time));
  println!("• Memory usage: {}", utils::memory_format(stats.current_memory));
  println!("• Active entities: {}", stats.current_entities);

  // ECS inspector snapshot
  let mut ecs = ECSInspector::new();
  
  // Add current entities
  let current_entities = vec![
    EntityDebugInfo {
      id: 1,
      components: vec!["Position".to_string(), "Health".to_string(), "Player".to_string()],
      position: Some((2, 2)),
      data: vec![("health".to_string(), "85".to_string())].into_iter().collect(),
    },
    EntityDebugInfo {
      id: 2,
      components: vec!["Position".to_string(), "Health".to_string(), "Player".to_string()],
      position: Some((12, 8)),
      data: vec![("health".to_string(), "92".to_string())].into_iter().collect(),
    },
    EntityDebugInfo {
      id: 3,
      components: vec!["Position".to_string(), "AI".to_string(), "Weapon".to_string()],
      position: Some((7, 3)),
      data: vec![("weapon".to_string(), "Bow".to_string()), ("ai_state".to_string(), "Aiming".to_string())].into_iter().collect(),
    },
  ];

  for entity in current_entities {
    ecs.entity_record(entity);
  }

  println!("\nEntity Summary:");
  println!("Total entities: {}", ecs.entity_count());
  let entity_ids = ecs.entity_ids();
  for id in entity_ids {
    if let Some(entity) = ecs.entity_get(id) {
      if let Some(pos) = entity.position {
        println!("• Entity {}: {} at ({}, {})", id, entity.components.join("+"), pos.0, pos.1);
      }
    }
  }

  println!("\n🎯 Integrated debugging session complete!");
  println!("This demonstrates how all debug tools work together to provide");
  println!("comprehensive visibility into game state, performance, and entities.");
}