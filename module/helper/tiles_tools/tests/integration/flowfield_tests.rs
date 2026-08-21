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
//! `flow_calculate` / `goal_add` callable at all are recorded in task 078.
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
//!
//! # BUG-474 (stub disclosure)
//!
//! Every operation this file exercises (`flow_calculate`, `flow_directions_batch_get`,
//! `group_flow_calculate`, `goal_add`, `optimal_direction_get`) is a documented stub
//! that never computes a real answer -- see `src/flowfield.rs`'s module doc. The
//! "Expected" column above was, and remains, satisfied by shape/length checks alone;
//! none of it asserts on the actual returned *values*, which is why the module's
//! complete non-functionality went undetected. Each test below now also asserts the
//! stub's current all-`None` output explicitly, so this file stops silently passing
//! against stub behavior and instead documents it -- any future real implementation
//! must update these assertions, at which point they become genuine regression
//! coverage instead of accidental silence.

#![ expect( deprecated, reason = "exercises the flowfield stub API pending a real implementation -- see BUG-474" ) ]

use tiles_tools::flowfield::{FlowField, MultiGoalFlowField};
use tiles_tools::coordinates::hexagonal::{Coordinate as HexCoord, Axial, Pointy};

// =============================================================================
// Hexagonal Grid Flow Field Tests
// =============================================================================

// test_kind: bug_reproducer(BUG-474)
/// ## Root Cause
/// `FlowField::flow_calculate`'s two phases (`integration_field_calculate`,
/// `flow_directions_generate`) are stub bodies containing only comments --
/// neither reads `is_passable`/`get_cost` nor writes any field state. Every
/// downstream query (`flow_direction_get`, `flow_apply`,
/// `flow_directions_batch_get`, `group_flow_calculate`) is itself a stub
/// that always returns `None`/an all-`None` collection, regardless of the
/// goal, obstacles, or grid passed in.
/// ## Why Not Caught
/// This test (and the four others in this file) called the stub API and
/// checked only that the call completed and that output length/count
/// matched input length/count -- never that the *values* were sane. This
/// specific test's closing comment, `// Units should path around water`,
/// was aspirational, not asserted -- exactly the gap that let a fully
/// non-functional pathfinding module ship undetected.
/// ## Fix Applied
/// Documentation-only fix (see BUG-474's report) -- the stub behavior
/// itself is unchanged; every public item it touches is now marked
/// `#[deprecated]`. This test gained an explicit assertion pinning the
/// current (broken) output instead of silently discarding it.
/// ## Prevention
/// A test that only checks "didn't panic" or "length matches" against a
/// function with a richer contract (a computed direction, not just a
/// slot count) provides zero coverage of whether the computation itself
/// is real. Assert on actual returned values, not just their shape.
/// ## Pitfall
/// An aspirational comment (`// Units should path around water`) sitting
/// next to a call that computes nothing reads as documentation of intent,
/// not of verified behavior -- it is easy to skim past during review and
/// mistake for a real assertion.
#[ test ]
fn test_hex_grid_with_water_obstacles()
{
  let mut flow_field = FlowField::<Axial, Pointy>::new(10, 10);

  let goal = HexCoord::<Axial, Pointy>::new(4, -2);

  // Define water hexes as impassable
  let water_hexes = [HexCoord::<Axial, Pointy>::new(1, 0),
    HexCoord::<Axial, Pointy>::new(2, -1),
    HexCoord::<Axial, Pointy>::new(2, 0)];

  flow_field.flow_calculate(&goal,
    |coord| !water_hexes.contains(coord),
    |_| 1
  );

  // Units should path around water -- BUG-474: they cannot, because
  // `flow_calculate` never computed any direction in the first place.
  assert_eq!(
    flow_field.flow_direction_get(&goal), None,
    "flow_calculate is a stub (BUG-474); no direction has ever been computed, even at the goal itself"
  );
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

  let directions = flow_field.flow_directions_batch_get(&test_coordinates);
  assert_eq!(directions.len(), test_coordinates.len());
  // BUG-474: flow_directions_batch_get is stub-cascading; pin the current
  // all-None output so a future real implementation must consciously update this.
  assert!(directions.iter().all(Option::is_none), "flow_directions_batch_get is a stub (BUG-474); no direction has ever been computed");
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

  let group_flow = flow_field.group_flow_calculate(&unit_positions);
  assert_eq!(group_flow.len(), unit_positions.len());
  // BUG-474: group_flow_calculate is stub-cascading; pin the current
  // all-None output so a future real implementation must consciously update this.
  assert!(group_flow.iter().all(Option::is_none), "group_flow_calculate is a stub (BUG-474); no unit has ever received a real move target");
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
    multi_field.goal_add(&point, |_| true, |_| 1);
  }

  // One field is stored per registered goal (assertion absorbed from the
  // retired square-coordinate resource-gathering variant of this test).
  assert_eq!(multi_field.goal_fields.len(), 3);

  // Units should move toward nearest capturable point -- BUG-474: they
  // cannot, optimal_direction_get is a stub that always returns None.
  let unit_pos = HexCoord::<Axial, Pointy>::new(0, 0);
  let direction = multi_field.optimal_direction_get(&unit_pos);
  assert_eq!(direction, None, "optimal_direction_get is a stub (BUG-474); no direction has ever been computed");
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
  for pos in &mut query {
    unit_positions.push(pos.coord);
  }

  // Get flow directions for all units
  let flow_directions = flow_field.flow_directions_batch_get(&unit_positions);
  assert_eq!(flow_directions.len(), 2);
  // BUG-474: flow_directions_batch_get is stub-cascading; pin the current
  // all-None output so a future real implementation must consciously update this.
  assert!(flow_directions.iter().all(Option::is_none), "flow_directions_batch_get is a stub (BUG-474); no unit has ever received a real direction");
}
