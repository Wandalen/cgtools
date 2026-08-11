//! Integration tests for flow field pathfinding functionality.
//!
//! These tests verify the flowfield API's cross-module contracts — coordinate-bound
//! calculation calls, batch queries, group flow, multi-goal fields, and ECS
//! interplay — using hexagonal coordinates, the one coordinate system `Grid2D`
//! (the flowfield backing store) can index by.
//!
//! # Why hexagonal-only
//!
//! `Grid2D`'s `Index`/`IndexMut` impls require `C : Into< hexagonal::Coordinate >`,
//! and no other coordinate system converts into hexagonal — so the square-coordinate
//! flowfield tests in the pre-revival version of this file were structurally
//! unsatisfiable and were retired rather than repaired. The revival census, per-test
//! dispositions, and the `Ord` impl on `hexagonal::Coordinate` that made
//! `calculate_flow` / `add_goal` callable at all are recorded in task 078.
//!
//! # Test Matrix for Flow Field Integration
//!
//! | Test ID | System    | Operation   | Expected           |
//! |---------|-----------|-------------|--------------------|
//! | FF2.2   | Hex       | Flow Calc   | Completes          |
//! | FF3.1   | Hex       | Batch Query | Length preserved   |
//! | FF3.2   | Hex       | Group Flow  | Length preserved   |
//! | FF3.3   | Hex       | Multi-Goal  | One field per goal |
//! | FF5.1   | Hex + ECS | ECS Batch   | All units answered |


use tiles_tools::flowfield::{FlowField, MultiGoalFlowField};
use tiles_tools::coordinates::hexagonal::{Coordinate as HexCoord, Axial, Pointy};

// =============================================================================
// Hexagonal Grid Flow Field Tests
// =============================================================================

#[ test ]
fn test_hex_grid_with_water_obstacles()
{
  let mut flow_field = FlowField::<Axial, Pointy>::new(10, 10);

  let goal = HexCoord::<Axial, Pointy>::new(4, -2);

  // Define water hexes as impassable
  let water_hexes = [HexCoord::<Axial, Pointy>::new(1, 0),
    HexCoord::<Axial, Pointy>::new(2, -1),
    HexCoord::<Axial, Pointy>::new(2, 0)];

  flow_field.calculate_flow(&goal,
    |coord| !water_hexes.contains(coord),
    |_| 1
  );

  // Units should path around water
}

// =============================================================================
// Batch Processing Tests
// =============================================================================

#[ test ]
fn test_batch_flow_direction_queries()
{
  let flow_field = FlowField::<Axial, Pointy>::new(12, 12);

  let test_coordinates = vec![
    HexCoord::<Axial, Pointy>::new(1, 1),
    HexCoord::<Axial, Pointy>::new(3, 5),
    HexCoord::<Axial, Pointy>::new(7, 2),
    HexCoord::<Axial, Pointy>::new(9, 8),
  ];

  let directions = flow_field.get_flow_directions_batch(&test_coordinates);
  assert_eq!(directions.len(), test_coordinates.len());
}

#[ test ]
fn test_group_movement_flow_application()
{
  let flow_field = FlowField::<Axial, Pointy>::new(15, 15);

  let unit_positions = vec![
    HexCoord::<Axial, Pointy>::new(2, 3),
    HexCoord::<Axial, Pointy>::new(3, 3),
    HexCoord::<Axial, Pointy>::new(4, 4),
    HexCoord::<Axial, Pointy>::new(2, 5),
  ];

  let group_flow = flow_field.calculate_group_flow(&unit_positions);
  assert_eq!(group_flow.len(), unit_positions.len());
}

// =============================================================================
// Multi-Goal Flow Field Tests
// =============================================================================

#[ test ]
fn test_multi_goal_capture_points()
{
  let mut multi_field = MultiGoalFlowField::<Axial, Pointy>::new(30, 30);

  // Add capture points for RTS scenario
  let capture_points = vec![
    HexCoord::<Axial, Pointy>::new(-3, 2),
    HexCoord::<Axial, Pointy>::new(4, -1),
    HexCoord::<Axial, Pointy>::new(1, 3),
  ];

  for point in capture_points {
    multi_field.add_goal(&point, |_| true, |_| 1);
  }

  // One field is stored per registered goal (assertion absorbed from the
  // retired square-coordinate resource-gathering variant of this test).
  assert_eq!(multi_field.goal_fields.len(), 3);

  // Units should move toward nearest capturable point
  let unit_pos = HexCoord::<Axial, Pointy>::new(0, 0);
  let _direction = multi_field.get_optimal_direction(&unit_pos);
}

// =============================================================================
// Integration with ECS Tests
// =============================================================================

#[ test ]
fn test_flow_field_ecs_integration()
{
  use tiles_tools::ecs::{World, Position, Movable};

  let mut world = World::new();
  let flow_field = FlowField::<Axial, Pointy>::new(20, 20);

  // Spawn some units
  let _unit1 = world.spawn((
    Position::new(HexCoord::<Axial, Pointy>::new(3, 3)),
    Movable::new(2),
  ));

  let _unit2 = world.spawn((
    Position::new(HexCoord::<Axial, Pointy>::new(7, 5)),
    Movable::new(3),
  ));

  // Collect unit positions for batch flow processing
  let mut unit_positions = Vec::new();
  let mut query = world.query::<&Position<HexCoord<Axial, Pointy>>>();
  for (_entity, pos) in &mut query {
    unit_positions.push(pos.coord);
  }

  // Get flow directions for all units
  let flow_directions = flow_field.get_flow_directions_batch(&unit_positions);
  assert_eq!(flow_directions.len(), 2);
}
