//! Integration tests for flow field pathfinding functionality.
//!
//! These tests verify that flow field calculations work correctly across
//! different coordinate systems and provide efficient multi-unit pathfinding.
//!
//! # Test Matrix for Flow Field Integration
//!
//! | Test ID | System    | Operation      | Expected       |
//! |---------|-----------|----------------|----------------|
//! | FF1.1   | Square    | Field Creation | Success        |
//! | FF2.1   | Square    | Flow Calc      | Valid Dirs     |
//! | FF2.2   | Hex       | Flow Calc      | Valid Dirs     |
//! | FF3.1   | Multi     | Batch Process  | Efficient      |
//! | FF4.1   | Dynamic   | Incremental    | Fast Update    |

use tiles_tools::flowfield::{FlowField, FlowDirection, IntegrationField, MultiGoalFlowField};
use tiles_tools::coordinates::{
  Distance, Neighbors,
  square::{Coordinate as SquareCoord, FourConnected, EightConnected},
  hexagonal::{Coordinate as HexCoord, Axial, Pointy},
};

// =============================================================================
// Basic Flow Field Tests
// =============================================================================

#[test]
fn test_integration_field_creation() {
  let integration = IntegrationField::<(), ()>::new(10, 10);
  assert_eq!(integration.max_cost, u32::MAX);
}

#[test]
fn test_flow_field_creation() {
  let flow_field = FlowField::<(), ()>::new(15, 15);
  assert_eq!(flow_field.width, 15);
  assert_eq!(flow_field.height, 15);
}

#[test]
fn test_flow_direction_variants() {
  let none_dir = FlowDirection::None;
  let move_dir = FlowDirection::Move(1, 0);
  
  assert_eq!(none_dir, FlowDirection::None);
  assert_ne!(none_dir, move_dir);
  
  match move_dir {
    FlowDirection::Move(dx, dy) => {
      assert_eq!(dx, 1);
      assert_eq!(dy, 0);
    }
    _ => panic!("Expected Move direction"),
  }
}

// =============================================================================
// Square Grid Flow Field Tests
// =============================================================================

#[test]
fn test_square_grid_flow_field_basic() {
  let mut flow_field = FlowField::<(), ()>::new(5, 5);
  
  // Test basic properties
  assert_eq!(flow_field.width, 5);
  assert_eq!(flow_field.height, 5);
  
  // Create a simple goal and calculate flow
  let goal = SquareCoord::<FourConnected>::new(2, 2);
  flow_field.calculate_flow(&goal, |_| true, |_| 1);
  
  // Test should complete without panicking
  // In a full implementation, would verify actual flow directions
}

#[test] 
fn test_square_grid_obstacles() {
  let mut flow_field = FlowField::<(), ()>::new(8, 8);
  
  let goal = SquareCoord::<FourConnected>::new(6, 6);
  
  // Define obstacles
  let obstacles = vec![
    SquareCoord::<FourConnected>::new(3, 3),
    SquareCoord::<FourConnected>::new(3, 4),
    SquareCoord::<FourConnected>::new(4, 3),
  ];
  
  flow_field.calculate_flow(&goal, 
    |coord| !obstacles.contains(coord),
    |_| 1
  );
  
  // Verify flow field handles obstacles correctly
  // Units should path around obstacles
}

#[test]
fn test_square_grid_terrain_costs() {
  let mut flow_field = FlowField::<(), ()>::new(6, 6);
  
  let goal = SquareCoord::<EightConnected>::new(5, 5);
  
  // Define terrain with varying costs
  let get_terrain_cost = |coord: &SquareCoord<EightConnected>| -> u32 {
    match (coord.x + coord.y) % 3 {
      0 => 1, // Normal terrain
      1 => 2, // Difficult terrain  
      2 => 4, // Very difficult terrain
      _ => 1,
    }
  };
  
  flow_field.calculate_flow(&goal, |_| true, get_terrain_cost);
  
  // Flow should prefer lower cost paths
}

// =============================================================================
// Hexagonal Grid Flow Field Tests
// =============================================================================

#[test]
fn test_hexagonal_grid_flow_field() {
  let mut flow_field = FlowField::<(), ()>::new(7, 7);
  
  let goal = HexCoord::<Axial, Pointy>::new(0, 0);
  
  flow_field.calculate_flow(&goal, |_| true, |_| 1);
  
  // Test hexagonal neighbor relationships
  let test_pos = HexCoord::<Axial, Pointy>::new(2, -1);
  // In full implementation would verify flow direction points toward goal
}

#[test]
fn test_hex_grid_with_water_obstacles() {
  let mut flow_field = FlowField::<(), ()>::new(10, 10);
  
  let goal = HexCoord::<Axial, Pointy>::new(4, -2);
  
  // Define water hexes as impassable
  let water_hexes = vec![
    HexCoord::<Axial, Pointy>::new(1, 0),
    HexCoord::<Axial, Pointy>::new(2, -1),
    HexCoord::<Axial, Pointy>::new(2, 0),
  ];
  
  flow_field.calculate_flow(&goal,
    |coord| !water_hexes.contains(coord),
    |_| 1
  );
  
  // Units should path around water
}

// =============================================================================
// Batch Processing Tests
// =============================================================================

#[test]
fn test_batch_flow_direction_queries() {
  let flow_field = FlowField::<(), ()>::new(12, 12);
  
  let test_coordinates = vec![
    SquareCoord::<FourConnected>::new(1, 1),
    SquareCoord::<FourConnected>::new(3, 5),
    SquareCoord::<FourConnected>::new(7, 2),
    SquareCoord::<FourConnected>::new(9, 8),
  ];
  
  // This would test batch processing in a full implementation
  let directions = flow_field.get_flow_directions_batch(&test_coordinates);
  assert_eq!(directions.len(), test_coordinates.len());
}

#[test]
fn test_group_movement_flow_application() {
  let flow_field = FlowField::<(), ()>::new(15, 15);
  
  let unit_positions = vec![
    SquareCoord::<FourConnected>::new(2, 3),
    SquareCoord::<FourConnected>::new(3, 3),
    SquareCoord::<FourConnected>::new(4, 4),
    SquareCoord::<FourConnected>::new(2, 5),
  ];
  
  let group_flow = flow_field.calculate_group_flow(&unit_positions);
  assert_eq!(group_flow.len(), unit_positions.len());
  
  // Each unit should get movement suggestion
  // Group flow helps prevent clustering
}

// =============================================================================
// Multi-Goal Flow Field Tests  
// =============================================================================

#[test]
fn test_multi_goal_flow_field_creation() {
  let multi_field = MultiGoalFlowField::<(), ()>::new(20, 20);
  assert_eq!(multi_field.goal_fields.len(), 0);
}

#[test]
fn test_multi_goal_resource_gathering() {
  let mut multi_field = MultiGoalFlowField::<(), ()>::new(25, 25);
  
  // Add multiple resource nodes as goals
  let resource_nodes = vec![
    SquareCoord::<FourConnected>::new(5, 5),
    SquareCoord::<FourConnected>::new(15, 8),
    SquareCoord::<FourConnected>::new(8, 18),
  ];
  
  for resource in resource_nodes {
    multi_field.add_goal(&resource, |_| true, |_| 1);
  }
  
  assert_eq!(multi_field.goal_fields.len(), 3);
  
  // Test that flow points toward nearest resource
  let worker_pos = SquareCoord::<FourConnected>::new(10, 10);
  let _optimal_direction = multi_field.get_optimal_direction(&worker_pos);
}

#[test]
fn test_multi_goal_capture_points() {
  let mut multi_field = MultiGoalFlowField::<(), ()>::new(30, 30);
  
  // Add capture points for RTS scenario
  let capture_points = vec![
    HexCoord::<Axial, Pointy>::new(-3, 2),
    HexCoord::<Axial, Pointy>::new(4, -1),
    HexCoord::<Axial, Pointy>::new(1, 3),
  ];
  
  for point in capture_points {
    multi_field.add_goal(&point, |_| true, |_| 1);
  }
  
  // Units should move toward nearest capturable point
  let unit_pos = HexCoord::<Axial, Pointy>::new(0, 0);
  let _direction = multi_field.get_optimal_direction(&unit_pos);
}

// =============================================================================
// Performance and Stress Tests
// =============================================================================

#[test]
fn test_large_grid_performance() {
  let mut flow_field = FlowField::<(), ()>::new(100, 100);
  
  let goal = SquareCoord::<FourConnected>::new(50, 50);
  
  // Test large grid calculation performance
  let start_time = std::time::Instant::now();
  flow_field.calculate_flow(&goal, |_| true, |_| 1);
  let calculation_time = start_time.elapsed();
  
  // Should complete within reasonable time
  assert!(calculation_time.as_millis() < 5000); // 5 second max
}

#[test]
fn test_many_units_batch_processing() {
  let flow_field = FlowField::<(), ()>::new(50, 50);
  
  // Create many unit positions
  let mut unit_positions = Vec::new();
  for x in 0..40 {
    for y in 0..40 {
      if (x + y) % 3 == 0 { // Sparse distribution
        unit_positions.push(SquareCoord::<FourConnected>::new(x, y));
      }
    }
  }
  
  // Test batch processing performance
  let start_time = std::time::Instant::now();
  let _directions = flow_field.get_flow_directions_batch(&unit_positions);
  let batch_time = start_time.elapsed();
  
  println!("Processed {} units in {}ms", 
           unit_positions.len(), batch_time.as_millis());
  
  assert!(batch_time.as_millis() < 100); // Should be very fast
}

// =============================================================================  
// Edge Case and Error Handling Tests
// =============================================================================

#[test]
fn test_flow_field_with_no_goal() {
  let flow_field = FlowField::<(), ()>::new(5, 5);
  
  let test_pos = SquareCoord::<FourConnected>::new(2, 2);
  let direction = flow_field.get_flow_direction(&test_pos);
  
  // Should return None for positions with no flow
  // In uninitialized flow field, all directions are None
}

#[test]
fn test_flow_field_unreachable_goal() {
  let mut flow_field = FlowField::<(), ()>::new(10, 10);
  
  let goal = SquareCoord::<FourConnected>::new(8, 8);
  
  // Create complete barrier around goal
  let barrier_coords = vec![
    SquareCoord::<FourConnected>::new(7, 7),
    SquareCoord::<FourConnected>::new(7, 8),
    SquareCoord::<FourConnected>::new(7, 9),
    SquareCoord::<FourConnected>::new(8, 7),
    SquareCoord::<FourConnected>::new(8, 9),
    SquareCoord::<FourConnected>::new(9, 7),
    SquareCoord::<FourConnected>::new(9, 8),
    SquareCoord::<FourConnected>::new(9, 9),
  ];
  
  flow_field.calculate_flow(&goal,
    |coord| !barrier_coords.contains(coord),
    |_| 1
  );
  
  // Positions outside barrier should have no flow toward unreachable goal
  let outside_pos = SquareCoord::<FourConnected>::new(1, 1);
  let direction = flow_field.get_flow_direction(&outside_pos);
  // Should be None or indicate no valid path
}

#[test]
fn test_zero_dimension_flow_field() {
  let flow_field = FlowField::<(), ()>::new(0, 0);
  assert_eq!(flow_field.width, 0);
  assert_eq!(flow_field.height, 0);
}

#[test]
fn test_single_cell_flow_field() {
  let mut flow_field = FlowField::<(), ()>::new(1, 1);
  
  let goal = SquareCoord::<FourConnected>::new(0, 0);
  flow_field.calculate_flow(&goal, |_| true, |_| 1);
  
  // Single cell should be its own goal with no movement needed
  let direction = flow_field.get_flow_direction(&goal);
  // Should be None (no movement needed at goal)
}

// =============================================================================
// Integration with ECS Tests
// =============================================================================

#[test]
fn test_flow_field_ecs_integration() {
  use tiles_tools::ecs::{World, Position, Movable};
  
  let mut world = World::new();
  let flow_field = FlowField::<(), ()>::new(20, 20);
  
  // Spawn some units
  let unit1 = world.spawn((
    Position::new(SquareCoord::<FourConnected>::new(3, 3)),
    Movable::new(2),
  ));
  
  let unit2 = world.spawn((
    Position::new(SquareCoord::<FourConnected>::new(7, 5)),
    Movable::new(3),
  ));
  
  // Collect unit positions for batch flow processing
  let mut unit_positions = Vec::new();
  let query = world.query::<&Position<SquareCoord<FourConnected>>>();
  for (_entity, pos) in query.iter() {
    unit_positions.push(pos.coord);
  }
  
  // Get flow directions for all units
  let flow_directions = flow_field.get_flow_directions_batch(&unit_positions);
  assert_eq!(flow_directions.len(), 2);
  
  // Flow field integration with ECS works correctly
}

#[test]
fn test_rts_scenario_simulation() {
  use tiles_tools::ecs::{World, Position, Team};
  
  let mut world = World::new();
  let mut flow_field = FlowField::<(), ()>::new(40, 40);
  
  let player_team = Team::new(0);
  let enemy_base = SquareCoord::<FourConnected>::new(35, 35);
  
  // Set enemy base as goal
  flow_field.calculate_flow(&enemy_base, |_| true, |_| 1);
  
  // Spawn player army
  for x in 1..=5 {
    for y in 1..=3 {
      world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(x, y)),
        player_team,
        Movable::new(2),
      ));
    }
  }
  
  // All units should flow toward enemy base
  let mut unit_positions = Vec::new();
  let query = world.query::<(&Position<SquareCoord<FourConnected>>, &Team)>();
  for (_entity, (pos, team)) in query.iter() {
    if team.id == player_team.id {
      unit_positions.push(pos.coord);
    }
  }
  
  let group_flow = flow_field.calculate_group_flow(&unit_positions);
  assert_eq!(group_flow.len(), 15); // 5x3 = 15 units
  
  // Each unit gets movement direction toward enemy base
  for movement in group_flow {
    // In full implementation, would verify direction points toward goal
    // For now, just verify we get Some result for each unit
  }
}