//! # Comprehensive Test Suite for Isometric Coordinates
//!
//! This test suite follows the Test Matrix methodology to ensure complete
//! coverage of the isometric coordinate system implementation.
//!
//! ## Test Matrix for Isometric Coordinates
//!
//! | Test ID | Category | Operation | Input | Expected | Status |
//! |---------|----------|-----------|-------|----------|--------|
//! | IC1.1   | Creation | new       | (0,0) | Success  | ✅ |
//! | IC1.2   | Creation | new       | (5,3) | Success  | ✅ |
//! | IC1.3   | Creation | from      | tuple | Success  | ✅ |
//! | IC1.4   | Creation | from      | array | Success  | ✅ |
//! | IC2.1   | Validation | is_valid | any   | true     | ✅ |
//! | IC3.1   | Distance | same      | (0,0) | 0        | ✅ |
//! | IC3.2   | Distance | orthog    | (0,0)→(3,0) | 3 | ✅ |
//! | IC3.3   | Distance | diagonal  | (0,0)→(2,2) | 4 | ✅ |
//! | IC3.4   | Distance | asymmetric| (1,2)→(4,7) | 8 | ✅ |
//! | IC3.5   | Distance | symmetric | d(a,b)==d(b,a) | true | ✅ |
//! | IC4.1   | Neighbors | count    | any   | 4        | ✅ |
//! | IC4.2   | Neighbors | positions| (2,3) | correct  | ✅ |
//! | IC4.3   | Neighbors | unique   | any   | no dups  | ✅ |
//! | IC5.1   | Transform | to_screen | (0,0) | (0,0)    | ✅ |
//! | IC5.2   | Transform | to_screen | (2,1) | correct  | ✅ |
//! | IC5.3   | Transform | from_screen| pixel | correct | ✅ |
//! | IC5.4   | Transform | roundtrip | coord | equal   | ✅ |
//! | IC6.1   | Rendering | tile_corners| coord | 4 points| ✅ |
//! | IC7.1   | Conversion| to_tuple | coord | (x,y)    | ✅ |
//! | IC7.2   | Conversion| to_array | coord | [x,y]    | ✅ |
//! | IC8.1   | Traits   | Debug     | coord | format   | ✅ |
//! | IC8.2   | Traits   | Clone     | coord | equal    | ✅ |
//! | IC8.3   | Traits   | Copy      | coord | equal    | ✅ |
//! | IC8.4   | Traits   | PartialEq | coord | equal    | ✅ |
//! | IC8.5   | Traits   | Hash      | coord | hashable | ✅ |
//! | IC9.1   | Serde    | serialize | coord | json     | ✅ |
//! | IC9.2   | Serde    | deserial  | json  | coord    | ✅ |

use tiles_tools::coordinates::isometric::{Coordinate, Diamond, IsometricCoord};
use tiles_tools::coordinates::{Distance, Neighbors, pixel::Pixel};
use std::collections::HashSet;

// =============================================================================
// Test Category 1: Coordinate Creation
// =============================================================================

#[test]
fn test_coordinate_creation_basic() {
  let coord = Coordinate::<Diamond>::new(0, 0);
  assert_eq!(coord.x, 0);
  assert_eq!(coord.y, 0);
}

#[test]
fn test_coordinate_creation_values() {
  let coord = Coordinate::<Diamond>::new(5, 3);
  assert_eq!(coord.x, 5);
  assert_eq!(coord.y, 3);
}

#[test]
fn test_coordinate_creation_negative() {
  let coord = Coordinate::<Diamond>::new(-5, -3);
  assert_eq!(coord.x, -5);
  assert_eq!(coord.y, -3);
}

#[test]
fn test_coordinate_from_tuple() {
  let coord: Coordinate<Diamond> = (7, -2).into();
  assert_eq!(coord.x, 7);
  assert_eq!(coord.y, -2);
}

#[test]
fn test_coordinate_from_array() {
  let coord: Coordinate<Diamond> = [3, 8].into();
  assert_eq!(coord.x, 3);
  assert_eq!(coord.y, 8);
}

#[test]
fn test_isometric_coord_alias() {
  let coord = IsometricCoord::new(4, 6);
  assert_eq!(coord.x, 4);
  assert_eq!(coord.y, 6);
}

// =============================================================================
// Test Category 2: Coordinate Validation
// =============================================================================

#[test]
fn test_coordinate_is_valid_positive() {
  let coord = Coordinate::<Diamond>::new(10, 20);
  assert!(coord.is_valid());
}

#[test]
fn test_coordinate_is_valid_negative() {
  let coord = Coordinate::<Diamond>::new(-10, -20);
  assert!(coord.is_valid());
}

#[test]
fn test_coordinate_is_valid_mixed() {
  let coord = Coordinate::<Diamond>::new(-5, 15);
  assert!(coord.is_valid());
}

#[test]
fn test_coordinate_is_valid_zero() {
  let coord = Coordinate::<Diamond>::new(0, 0);
  assert!(coord.is_valid());
}

// =============================================================================
// Test Category 3: Distance Calculations
// =============================================================================

#[test]
fn test_distance_to_self() {
  let coord = Coordinate::<Diamond>::new(5, 3);
  assert_eq!(coord.distance(&coord), 0);
}

#[test]
fn test_distance_horizontal() {
  let coord1 = Coordinate::<Diamond>::new(0, 0);
  let coord2 = Coordinate::<Diamond>::new(3, 0);
  assert_eq!(coord1.distance(&coord2), 3);
}

#[test]
fn test_distance_vertical() {
  let coord1 = Coordinate::<Diamond>::new(0, 0);
  let coord2 = Coordinate::<Diamond>::new(0, 4);
  assert_eq!(coord1.distance(&coord2), 4);
}

#[test]
fn test_distance_diagonal() {
  let coord1 = Coordinate::<Diamond>::new(0, 0);
  let coord2 = Coordinate::<Diamond>::new(2, 2);
  // Manhattan distance: |2-0| + |2-0| = 4
  assert_eq!(coord1.distance(&coord2), 4);
}

#[test]
fn test_distance_asymmetric() {
  let coord1 = Coordinate::<Diamond>::new(1, 2);
  let coord2 = Coordinate::<Diamond>::new(4, 7);
  // |4-1| + |7-2| = 3 + 5 = 8
  assert_eq!(coord1.distance(&coord2), 8);
}

#[test]
fn test_distance_negative_coordinates() {
  let coord1 = Coordinate::<Diamond>::new(-3, -2);
  let coord2 = Coordinate::<Diamond>::new(1, 4);
  // |1-(-3)| + |4-(-2)| = 4 + 6 = 10
  assert_eq!(coord1.distance(&coord2), 10);
}

#[test]
fn test_distance_symmetry() {
  let coord1 = Coordinate::<Diamond>::new(2, 5);
  let coord2 = Coordinate::<Diamond>::new(7, 1);
  assert_eq!(coord1.distance(&coord2), coord2.distance(&coord1));
}

// =============================================================================
// Test Category 4: Neighbor Finding
// =============================================================================

#[test]
fn test_neighbors_count() {
  let coord = Coordinate::<Diamond>::new(5, 3);
  let neighbors = coord.neighbors();
  assert_eq!(neighbors.len(), 4, "Isometric coordinates should have exactly 4 neighbors");
}

#[test]
fn test_neighbors_positions() {
  let coord = Coordinate::<Diamond>::new(2, 3);
  let neighbors = coord.neighbors();
  let expected = vec![
    Coordinate::<Diamond>::new(3, 3), // Right
    Coordinate::<Diamond>::new(1, 3), // Left
    Coordinate::<Diamond>::new(2, 4), // Up
    Coordinate::<Diamond>::new(2, 2), // Down
  ];
  
  assert_eq!(neighbors.len(), expected.len());
  for expected_neighbor in expected {
    assert!(neighbors.contains(&expected_neighbor),
            "Missing neighbor: {:?}", expected_neighbor);
  }
}

#[test]
fn test_neighbors_uniqueness() {
  let coord = Coordinate::<Diamond>::new(5, 7);
  let neighbors = coord.neighbors();
  let unique_neighbors: HashSet<_> = neighbors.iter().collect();
  assert_eq!(neighbors.len(), unique_neighbors.len(),
             "All neighbors should be unique");
}

#[test]
fn test_neighbors_exclude_self() {
  let coord = Coordinate::<Diamond>::new(3, 8);
  let neighbors = coord.neighbors();
  assert!(!neighbors.contains(&coord),
          "Coordinate should not be its own neighbor");
}

// =============================================================================
// Test Category 5: Screen Coordinate Transformations
// =============================================================================

#[test]
fn test_to_screen_origin() {
  let coord = Coordinate::<Diamond>::new(0, 0);
  let screen_pos = coord.to_screen(32.0);
  
  // Origin should map to (0, 0) in screen coordinates
  assert!((screen_pos.x() - 0.0).abs() < 1e-6);
  assert!((screen_pos.y() - 0.0).abs() < 1e-6);
}

#[test]
fn test_to_screen_positive() {
  let coord = Coordinate::<Diamond>::new(2, 1);
  let screen_pos = coord.to_screen(32.0);
  
  // Isometric transformation: x_screen = (x - y) * tile_size/2
  //                          y_screen = (x + y) * tile_size/4
  let expected_x = (2 - 1) as f32 * 16.0; // (2-1) * 32/2 = 16
  let expected_y = (2 + 1) as f32 * 8.0;  // (2+1) * 32/4 = 24
  
  assert!((screen_pos.x() - expected_x).abs() < 1e-6);
  assert!((screen_pos.y() - expected_y).abs() < 1e-6);
}

#[test]
fn test_to_screen_negative() {
  let coord = Coordinate::<Diamond>::new(-1, 2);
  let screen_pos = coord.to_screen(32.0);
  
  let expected_x = (-1 - 2) as f32 * 16.0; // (-1-2) * 32/2 = -48
  let expected_y = (-1 + 2) as f32 * 8.0;  // (-1+2) * 32/4 = 8
  
  assert!((screen_pos.x() - expected_x).abs() < 1e-6);
  assert!((screen_pos.y() - expected_y).abs() < 1e-6);
}

#[test]
fn test_from_screen_origin() {
  let screen_pos = Pixel::new(0.0, 0.0);
  let coord = Coordinate::<Diamond>::from_screen(screen_pos, 32.0);
  
  assert_eq!(coord.x, 0);
  assert_eq!(coord.y, 0);
}

#[test]
fn test_from_screen_positive() {
  let screen_pos = Pixel::new(16.0, 24.0);
  let coord = Coordinate::<Diamond>::from_screen(screen_pos, 32.0);
  
  // Should convert back to (2, 1)
  assert_eq!(coord.x, 2);
  assert_eq!(coord.y, 1);
}

#[test]
fn test_screen_coordinate_roundtrip() {
  let original = Coordinate::<Diamond>::new(3, -2);
  let screen_pos = original.to_screen(32.0);
  let converted_back = Coordinate::<Diamond>::from_screen(screen_pos, 32.0);
  
  assert_eq!(original, converted_back);
}

#[test]
fn test_screen_coordinate_roundtrip_various_sizes() {
  let coords = vec![
    Coordinate::<Diamond>::new(0, 0),
    Coordinate::<Diamond>::new(5, 3),
    Coordinate::<Diamond>::new(-2, 4),
    Coordinate::<Diamond>::new(10, -5),
  ];
  
  let tile_sizes = vec![16.0, 32.0, 48.0, 64.0];
  
  for coord in coords {
    for tile_size in &tile_sizes {
      let screen_pos = coord.to_screen(*tile_size);
      let converted_back = Coordinate::<Diamond>::from_screen(screen_pos, *tile_size);
      assert_eq!(coord, converted_back,
                 "Roundtrip failed for coord {:?} with tile size {}", coord, tile_size);
    }
  }
}

// =============================================================================
// Test Category 6: Tile Rendering Support
// =============================================================================

#[test]
fn test_tile_corners() {
  let coord = Coordinate::<Diamond>::new(1, 1);
  let corners = coord.tile_corners(32.0);
  
  assert_eq!(corners.len(), 4);
  
  // Get the center position for verification
  let center = coord.to_screen(32.0);
  
  // Verify the corners form a diamond shape around the center
  let half_width = 16.0;  // 32.0 / 2.0
  let half_height = 8.0;  // 32.0 / 4.0
  
  let expected_corners = [
    Pixel::new(center.x(), center.y() - half_height), // Top
    Pixel::new(center.x() + half_width, center.y()),  // Right
    Pixel::new(center.x(), center.y() + half_height), // Bottom
    Pixel::new(center.x() - half_width, center.y()),  // Left
  ];
  
  for (i, expected) in expected_corners.iter().enumerate() {
    assert!((corners[i].x() - expected.x()).abs() < 1e-6,
            "Corner {} x mismatch: got {}, expected {}", i, corners[i].x(), expected.x());
    assert!((corners[i].y() - expected.y()).abs() < 1e-6,
            "Corner {} y mismatch: got {}, expected {}", i, corners[i].y(), expected.y());
  }
}

#[test]
fn test_tile_corners_different_sizes() {
  let coord = Coordinate::<Diamond>::new(0, 0);
  
  let sizes_and_expected = vec![
    (16.0, (8.0, 4.0)),   // half_width, half_height
    (32.0, (16.0, 8.0)),
    (64.0, (32.0, 16.0)),
  ];
  
  for (tile_size, (expected_half_width, expected_half_height)) in sizes_and_expected {
    let corners = coord.tile_corners(tile_size);
    
    // For origin coordinate, center should be (0,0)
    let expected_corners = [
      Pixel::new(0.0, -expected_half_height), // Top
      Pixel::new(expected_half_width, 0.0),   // Right
      Pixel::new(0.0, expected_half_height),  // Bottom
      Pixel::new(-expected_half_width, 0.0),  // Left
    ];
    
    for (i, expected) in expected_corners.iter().enumerate() {
      assert!((corners[i].x() - expected.x()).abs() < 1e-6);
      assert!((corners[i].y() - expected.y()).abs() < 1e-6);
    }
  }
}

// =============================================================================
// Test Category 7: Conversions
// =============================================================================

#[test]
fn test_into_tuple() {
  let coord = Coordinate::<Diamond>::new(7, -3);
  let tuple: (i32, i32) = coord.into();
  assert_eq!(tuple, (7, -3));
}

#[test]
fn test_into_array() {
  let coord = Coordinate::<Diamond>::new(-2, 9);
  let array: [i32; 2] = coord.into();
  assert_eq!(array, [-2, 9]);
}

// =============================================================================
// Test Category 8: Trait Implementations
// =============================================================================

#[test]
fn test_debug_trait() {
  let coord = Coordinate::<Diamond>::new(5, -1);
  let debug_str = format!("{:?}", coord);
  assert!(debug_str.contains("5"));
  assert!(debug_str.contains("-1"));
}

#[test]
fn test_clone_trait() {
  let coord = Coordinate::<Diamond>::new(4, 2);
  let cloned = coord.clone();
  assert_eq!(coord, cloned);
}

#[test]
fn test_copy_trait() {
  let coord = Coordinate::<Diamond>::new(1, 6);
  let copied = coord;  // This should work due to Copy trait
  assert_eq!(coord, copied);
}

#[test]
fn test_partial_eq_trait() {
  let coord1 = Coordinate::<Diamond>::new(3, 4);
  let coord2 = Coordinate::<Diamond>::new(3, 4);
  let coord3 = Coordinate::<Diamond>::new(3, 5);
  
  assert_eq!(coord1, coord2);
  assert_ne!(coord1, coord3);
}

#[test]
fn test_hash_trait() {
  use std::collections::HashMap;
  
  let coord1 = Coordinate::<Diamond>::new(2, 3);
  let coord2 = Coordinate::<Diamond>::new(2, 3);
  
  let mut map = HashMap::new();
  map.insert(coord1, "value");
  
  // Should be able to retrieve with equivalent coordinate
  assert_eq!(map.get(&coord2), Some(&"value"));
}

#[test]
fn test_default_trait() {
  let coord: Coordinate<Diamond> = Default::default();
  assert_eq!(coord.x, 0);
  assert_eq!(coord.y, 0);
}

// =============================================================================
// Test Category 9: Serialization/Deserialization
// =============================================================================

#[test]
fn test_serialize() {
  let coord = Coordinate::<Diamond>::new(5, -2);
  let serialized = serde_json::to_string(&coord).expect("Serialization should succeed");
  
  // Should contain the x and y values but not the phantom marker
  assert!(serialized.contains("5"));
  assert!(serialized.contains("-2"));
  assert!(!serialized.contains("_marker"));
}

#[test]
fn test_deserialize() {
  let json = r#"{"x": 7, "y": 3}"#;
  let coord: Coordinate<Diamond> = serde_json::from_str(json)
    .expect("Deserialization should succeed");
  
  assert_eq!(coord.x, 7);
  assert_eq!(coord.y, 3);
}

#[test]
fn test_round_trip_serialization() {
  let original = Coordinate::<Diamond>::new(-4, 8);
  let serialized = serde_json::to_string(&original)
    .expect("Serialization should succeed");
  let deserialized: Coordinate<Diamond> = serde_json::from_str(&serialized)
    .expect("Deserialization should succeed");
  
  assert_eq!(original, deserialized);
}

// =============================================================================
// Test Category 10: Edge Cases and Boundary Conditions
// =============================================================================

#[test]
fn test_large_coordinates() {
  let coord = Coordinate::<Diamond>::new(1000000, -1000000);
  let neighbors = coord.neighbors();
  assert_eq!(neighbors.len(), 4);
  assert!(coord.is_valid());
}

#[test]
fn test_screen_transform_large_coordinates() {
  let coord = Coordinate::<Diamond>::new(1000, -500);
  let screen_pos = coord.to_screen(32.0);
  let converted_back = Coordinate::<Diamond>::from_screen(screen_pos, 32.0);
  assert_eq!(coord, converted_back);
}

#[test]
fn test_tile_corners_extreme_positions() {
  let coord = Coordinate::<Diamond>::new(-100, 200);
  let corners = coord.tile_corners(32.0);
  assert_eq!(corners.len(), 4);
  
  // Corners should still form a valid diamond pattern
  let center = coord.to_screen(32.0);
  for corner in corners {
    // Each corner should be at a reasonable distance from center
    let dx = corner.x() - center.x();
    let dy = corner.y() - center.y();
    let distance_sq = dx * dx + dy * dy;
    assert!(distance_sq > 0.0, "Corner should not be at center");
  }
}

// =============================================================================
// Test Category 11: Integration Tests
// =============================================================================

#[test]
fn test_distance_between_neighbors() {
  let coord = Coordinate::<Diamond>::new(5, 3);
  let neighbors = coord.neighbors();
  
  for neighbor in neighbors {
    let distance = coord.distance(&neighbor);
    // All orthogonal neighbors should be at Manhattan distance 1
    assert_eq!(distance, 1, "Neighbor should be at distance 1");
  }
}

#[test]
fn test_neighbors_reciprocal() {
  let coord = Coordinate::<Diamond>::new(3, 7);
  let neighbors = coord.neighbors();
  
  // For each neighbor, coord should be in that neighbor's neighbor list
  for neighbor in neighbors {
    let neighbor_neighbors = neighbor.neighbors();
    assert!(neighbor_neighbors.contains(&coord),
            "Reciprocal neighbor relationship should hold for {:?} and {:?}",
            coord, neighbor);
  }
}

#[test]
fn test_isometric_visual_properties() {
  // Test that the isometric transformation creates the expected diamond pattern
  let coords = vec![
    Coordinate::<Diamond>::new(0, 0),  // Center
    Coordinate::<Diamond>::new(1, 0),  // Right in world -> NE in screen
    Coordinate::<Diamond>::new(0, 1),  // Up in world -> NW in screen
    Coordinate::<Diamond>::new(-1, 0), // Left in world -> SW in screen
    Coordinate::<Diamond>::new(0, -1), // Down in world -> SE in screen
  ];
  
  let tile_size = 32.0;
  let screen_positions: Vec<_> = coords.iter().map(|c| c.to_screen(tile_size)).collect();
  
  // Center should be at origin
  assert!((screen_positions[0].x() - 0.0).abs() < 1e-6);
  assert!((screen_positions[0].y() - 0.0).abs() < 1e-6);
  
  // Right (1,0) -> (16, 8) in screen coordinates
  assert!((screen_positions[1].x() - 16.0).abs() < 1e-6);
  assert!((screen_positions[1].y() - 8.0).abs() < 1e-6);
  
  // Up (0,1) -> (-16, 8) in screen coordinates
  assert!((screen_positions[2].x() - (-16.0)).abs() < 1e-6);
  assert!((screen_positions[2].y() - 8.0).abs() < 1e-6);
  
  // Left (-1,0) -> (-16, -8) in screen coordinates
  assert!((screen_positions[3].x() - (-16.0)).abs() < 1e-6);
  assert!((screen_positions[3].y() - (-8.0)).abs() < 1e-6);
  
  // Down (0,-1) -> (16, -8) in screen coordinates
  assert!((screen_positions[4].x() - 16.0).abs() < 1e-6);
  assert!((screen_positions[4].y() - (-8.0)).abs() < 1e-6);
}

// =============================================================================
// Test Category 12: Pathfinding Integration
// =============================================================================

#[test]
fn test_pathfinding_integration() {
  use tiles_tools::pathfind::astar;
  
  let start = IsometricCoord::new(0, 0);
  let goal = IsometricCoord::new(3, 4);
  
  let result = astar(
    &start,
    &goal,
    |_coord| true,  // All tiles accessible
    |_coord| 1,     // Unit cost
  );
  
  assert!(result.is_some(), "Should find a path");
  let (path, cost) = result.unwrap();
  
  // Distance should be Manhattan: |3-0| + |4-0| = 7
  assert_eq!(cost, 7, "Path cost should match Manhattan distance");
  assert_eq!(path.len(), 8, "Path should contain start + 7 steps");
  assert_eq!(path[0], start);
  assert_eq!(path[path.len() - 1], goal);
}

#[test]
fn test_pathfinding_blocked_path() {
  use tiles_tools::pathfind::astar;
  
  let start = IsometricCoord::new(0, 0);
  let goal = IsometricCoord::new(2, 0);
  
  // Block only a specific tile, not the entire column
  let result = astar(
    &start,
    &goal,
    |coord| !(coord.x == 1 && coord.y == 0), // Block only (1,0)
    |_coord| 1,
  );
  
  assert!(result.is_some(), "Should find alternative path");
  let (path, _cost) = result.unwrap();
  
  // Should find path around the blocked area
  assert_eq!(path[0], start);
  assert_eq!(path[path.len() - 1], goal);
  
  // Verify path doesn't go through the specific blocked tile
  assert!(!path.contains(&IsometricCoord::new(1, 0)), 
          "Path should not go through blocked tile (1,0)");
}

#[test]
fn test_pathfinding_same_position() {
  use tiles_tools::pathfind::astar;
  
  let coord = IsometricCoord::new(5, 3);
  
  let result = astar(
    &coord,
    &coord,
    |_coord| true,
    |_coord| 1,
  );
  
  assert!(result.is_some(), "Should handle same start/goal");
  let (path, cost) = result.unwrap();
  assert_eq!(cost, 0, "Cost to same position should be 0");
  assert_eq!(path.len(), 1, "Path should contain only the position itself");
  assert_eq!(path[0], coord);
}

#[test]
fn test_pathfinding_impossible() {
  use tiles_tools::pathfind::astar;
  
  let start = IsometricCoord::new(0, 0);
  let goal = IsometricCoord::new(2, 0);
  
  let result = astar(
    &start,
    &goal,
    |_coord| false, // No tiles accessible
    |_coord| 1,
  );
  
  assert!(result.is_none(), "Should return None when no path exists");
}