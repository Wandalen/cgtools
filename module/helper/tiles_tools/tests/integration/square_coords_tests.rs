//! ## Test Matrix for Square Coordinates
//!
//! | Test ID | Operation | Connectivity | Input | Expected |
//! |---------|-----------|-------------|-------|----------|
//! | SC1.1   | Create    | 4-conn      | (0,0) | Success  |
//! | SC1.2   | Create    | 8-conn      | (5,10)| Success  |
//! | SC2.1   | Distance  | 4-conn      | (0,0)→(3,4) | 7 (Manhattan) |
//! | SC2.2   | Distance  | 8-conn      | (0,0)→(3,4) | 4 (Chebyshev) |
//! | SC2.3   | Distance  | 4-conn      | Same coords | 0 |
//! | SC2.4   | Distance  | Symmetric   | d(a,b)=d(b,a) | True |
//! | SC3.1   | Neighbors | 4-conn      | (2,3) | 4 coords |
//! | SC3.2   | Neighbors | 8-conn      | (2,3) | 8 coords |
//! | SC3.3   | Neighbors | 4-conn      | Orthogonal only | True |
//! | SC3.4   | Neighbors | 8-conn      | Include diagonal | True |
//! | SC4.1   | Math Ops  | Add         | (1,2)+(3,4) | (4,6) |
//! | SC4.2   | Math Ops  | Sub         | (5,7)-(2,3) | (3,4) |
//! | SC4.3   | Math Ops  | Mul         | (2,3)*2 | (4,6) |
//! | SC4.4   | Math Ops  | Div         | (4,6)/2 | (2,3) |
//! | SC5.1   | Conversion| From tuple  | (1,2) | Coord(1,2) |
//! | SC5.2   | Conversion| From array  | [3,4] | Coord(3,4) |
//! | SC5.3   | Conversion| To tuple    | Coord(5,6) | (5,6) |
//! | SC6.1   | Pathfinding| 4-conn A*  | (0,0)→(3,3) | Path found |
//! | SC6.2   | Pathfinding| 8-conn A*  | (0,0)→(3,3) | Shorter path |

#![cfg(feature = "enabled")]

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
#![allow(clippy::default_trait_access)]

use tiles_tools::coordinates::square::{Coordinate, FourConnected, EightConnected, SquareCoord4, SquareCoord8};
use tiles_tools::coordinates::{Distance, Neighbors};
use tiles_tools::pathfind::astar;

/// Test SC1.1: Create 4-connected coordinate
#[test]
fn test_square_coordinate_creation_4conn() {
    let coord = Coordinate::<FourConnected>::new(0, 0);
    assert_eq!(coord.x, 0);
    assert_eq!(coord.y, 0);
}

/// Test SC1.2: Create 8-connected coordinate  
#[test]
fn test_square_coordinate_creation_8conn() {
    let coord = Coordinate::<EightConnected>::new(5, 10);
    assert_eq!(coord.x, 5);
    assert_eq!(coord.y, 10);
}

/// Test SC2.1: Manhattan distance calculation
#[test]
fn test_manhattan_distance() {
    let coord1 = SquareCoord4::new(0, 0);
    let coord2 = SquareCoord4::new(3, 4);
    assert_eq!(coord1.distance(&coord2), 7); // |3-0| + |4-0| = 7
}

/// Test SC2.2: Chebyshev distance calculation
#[test]
fn test_chebyshev_distance() {
    let coord1 = SquareCoord8::new(0, 0);
    let coord2 = SquareCoord8::new(3, 4);
    assert_eq!(coord1.distance(&coord2), 4); // max(|3-0|, |4-0|) = max(3, 4) = 4
}

/// Test SC2.3: Distance to same coordinate is zero
#[test]
fn test_distance_to_self() {
    let coord = SquareCoord4::new(5, 7);
    assert_eq!(coord.distance(&coord), 0);
    
    let coord8 = SquareCoord8::new(5, 7);
    assert_eq!(coord8.distance(&coord8), 0);
}

/// Test SC2.4: Distance is symmetric
#[test]
fn test_distance_symmetry() {
    let coord1 = SquareCoord4::new(2, 3);
    let coord2 = SquareCoord4::new(8, 6);
    assert_eq!(coord1.distance(&coord2), coord2.distance(&coord1));
    
    let coord1_8 = SquareCoord8::new(2, 3);
    let coord2_8 = SquareCoord8::new(8, 6);
    assert_eq!(coord1_8.distance(&coord2_8), coord2_8.distance(&coord1_8));
}

/// Test SC3.1: 4-connected neighbors
#[test]
fn test_four_connected_neighbors() {
    let coord = SquareCoord4::new(2, 3);
    let neighbors = coord.neighbors();
    assert_eq!(neighbors.len(), 4);
    
    let expected = vec![
        SquareCoord4::new(3, 3), // Right
        SquareCoord4::new(1, 3), // Left
        SquareCoord4::new(2, 4), // Up
        SquareCoord4::new(2, 2), // Down
    ];
    
    for expected_neighbor in expected {
        assert!(neighbors.contains(&expected_neighbor), 
            "Missing expected neighbor {:?}", expected_neighbor);
    }
}

/// Test SC3.2: 8-connected neighbors
#[test]
fn test_eight_connected_neighbors() {
    let coord = SquareCoord8::new(2, 3);
    let neighbors = coord.neighbors();
    assert_eq!(neighbors.len(), 8);
    
    let expected = vec![
        // Orthogonal
        SquareCoord8::new(3, 3), // Right
        SquareCoord8::new(1, 3), // Left
        SquareCoord8::new(2, 4), // Up
        SquareCoord8::new(2, 2), // Down
        // Diagonal
        SquareCoord8::new(3, 4), // Up-Right
        SquareCoord8::new(1, 4), // Up-Left
        SquareCoord8::new(3, 2), // Down-Right
        SquareCoord8::new(1, 2), // Down-Left
    ];
    
    for expected_neighbor in expected {
        assert!(neighbors.contains(&expected_neighbor), 
            "Missing expected neighbor {:?}", expected_neighbor);
    }
}

/// Test SC3.3: 4-connected neighbors are only orthogonal
#[test]
fn test_four_connected_only_orthogonal() {
    let coord = SquareCoord4::new(5, 5);
    let neighbors = coord.neighbors();
    
    // Check that no diagonal neighbors are included
    let diagonal_positions = vec![
        SquareCoord4::new(4, 4), // Down-Left diagonal
        SquareCoord4::new(4, 6), // Up-Left diagonal  
        SquareCoord4::new(6, 4), // Down-Right diagonal
        SquareCoord4::new(6, 6), // Up-Right diagonal
    ];
    
    for diagonal_pos in diagonal_positions {
        assert!(!neighbors.contains(&diagonal_pos),
            "4-connected should not include diagonal {:?}", diagonal_pos);
    }
}

/// Test SC3.4: 8-connected neighbors include diagonals
#[test]
fn test_eight_connected_includes_diagonal() {
    let coord = SquareCoord8::new(5, 5);
    let neighbors = coord.neighbors();
    
    // Check that diagonal neighbors are included
    let diagonal_positions = vec![
        SquareCoord8::new(4, 4), // Down-Left diagonal
        SquareCoord8::new(4, 6), // Up-Left diagonal  
        SquareCoord8::new(6, 4), // Down-Right diagonal
        SquareCoord8::new(6, 6), // Up-Right diagonal
    ];
    
    for diagonal_pos in diagonal_positions {
        assert!(neighbors.contains(&diagonal_pos),
            "8-connected should include diagonal {:?}", diagonal_pos);
    }
}

/// Test SC4.1: Addition operation
#[test]
fn test_coordinate_addition() {
    let coord1 = SquareCoord4::new(1, 2);
    let coord2 = SquareCoord4::new(3, 4);
    let result = coord1 + coord2;
    assert_eq!(result.x, 4);
    assert_eq!(result.y, 6);
}

/// Test SC4.2: Subtraction operation
#[test]
fn test_coordinate_subtraction() {
    let coord1 = SquareCoord4::new(5, 7);
    let coord2 = SquareCoord4::new(2, 3);
    let result = coord1 - coord2;
    assert_eq!(result.x, 3);
    assert_eq!(result.y, 4);
}

/// Test SC4.3: Multiplication by scalar
#[test]
fn test_coordinate_multiplication() {
    let coord = SquareCoord4::new(2, 3);
    let result = coord * 2;
    assert_eq!(result.x, 4);
    assert_eq!(result.y, 6);
}

/// Test SC4.4: Division by scalar
#[test]
fn test_coordinate_division() {
    let coord = SquareCoord4::new(4, 6);
    let result = coord / 2;
    assert_eq!(result.x, 2);
    assert_eq!(result.y, 3);
}

/// Test SC5.1: Conversion from tuple
#[test]
fn test_from_tuple() {
    let coord: SquareCoord4 = (1, 2).into();
    assert_eq!(coord.x, 1);
    assert_eq!(coord.y, 2);
}

/// Test SC5.2: Conversion from array
#[test]
fn test_from_array() {
    let coord: SquareCoord4 = [3, 4].into();
    assert_eq!(coord.x, 3);
    assert_eq!(coord.y, 4);
}

/// Test SC5.3: Conversion to tuple
#[test]
fn test_to_tuple() {
    let coord = SquareCoord4::new(5, 6);
    let tuple: (i32, i32) = coord.into();
    assert_eq!(tuple, (5, 6));
}

/// Test SC6.1: A* pathfinding with 4-connected grid
#[test]
fn test_pathfinding_four_connected() {
    let start = SquareCoord4::new(0, 0);
    let goal = SquareCoord4::new(3, 3);
    
    let result = astar(
        &start,
        &goal,
        |_coord| true, // All tiles accessible
        |_coord| 1,    // Unit cost
    );
    
    assert!(result.is_some(), "Should find a path");
    let (path, cost) = result.unwrap();
    
    // Manhattan distance should be 6, path should have 7 positions (start + 6 steps)
    assert_eq!(cost, 6, "Path cost should equal Manhattan distance");
    assert_eq!(path.len(), 7, "Path should include start + 6 steps");
    assert_eq!(path[0], start, "Path should start at start position");
    assert_eq!(path[path.len() - 1], goal, "Path should end at goal position");
}

/// Test SC6.2: A* pathfinding with 8-connected grid should find shorter path
#[test]
fn test_pathfinding_eight_connected() {
    let start = SquareCoord8::new(0, 0);
    let goal = SquareCoord8::new(3, 3);
    
    let result = astar(
        &start,
        &goal,
        |_coord| true, // All tiles accessible
        |_coord| 1,    // Unit cost
    );
    
    assert!(result.is_some(), "Should find a path");
    let (path, cost) = result.unwrap();
    
    // Chebyshev distance should be 3, allowing diagonal movement
    assert_eq!(cost, 3, "Path cost should equal Chebyshev distance with diagonal moves");
    assert_eq!(path.len(), 4, "Path should include start + 3 steps");
    assert_eq!(path[0], start, "Path should start at start position");
    assert_eq!(path[path.len() - 1], goal, "Path should end at goal position");
}

/// Test pathfinding with obstacles
#[test]
fn test_pathfinding_with_obstacles() {
    let start = SquareCoord4::new(0, 0);
    let goal = SquareCoord4::new(2, 0);
    
    // Block the direct path
    let blocked_positions = vec![SquareCoord4::new(1, 0)];
    
    let result = astar(
        &start,
        &goal,
        |coord| !blocked_positions.contains(coord), // Blocked positions not accessible
        |_coord| 1, // Unit cost
    );
    
    assert!(result.is_some(), "Should find alternate path around obstacle");
    let (path, cost) = result.unwrap();
    
    // Should find a longer path around the obstacle
    assert!(cost > 2, "Path should be longer due to obstacle");
    
    // Ensure path doesn't contain blocked positions
    for pos in path {
        assert!(!blocked_positions.contains(&pos), "Path should not go through blocked position");
    }
}

/// Test edge case: pathfinding to same position
#[test]
fn test_pathfinding_same_position() {
    let coord = SquareCoord4::new(5, 5);
    
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

/// Test impossible pathfinding scenario
#[test]
fn test_pathfinding_impossible() {
    let start = SquareCoord4::new(0, 0);
    let goal = SquareCoord4::new(2, 0);
    
    let result = astar(
        &start,
        &goal,
        |_coord| false, // No tiles accessible
        |_coord| 1,
    );
    
    assert!(result.is_none(), "Should return None when no path exists");
}

/// Test serialization/deserialization
#[test]
fn test_serde_serialization() {
    let coord = SquareCoord4::new(10, 20);
    
    // Test JSON serialization
    let json = serde_json::to_string(&coord).expect("Should serialize to JSON");
    let deserialized: SquareCoord4 = serde_json::from_str(&json).expect("Should deserialize from JSON");
    
    assert_eq!(coord, deserialized, "Serialization round-trip should preserve value");
}

/// Test coordinate hashing for use in HashSet/HashMap
#[test]
fn test_coordinate_hashing() {
    use std::collections::HashSet;
    
    let mut set = HashSet::new();
    let coord1 = SquareCoord4::new(1, 2);
    let coord2 = SquareCoord4::new(1, 2); // Same values
    let coord3 = SquareCoord4::new(2, 1); // Different values
    
    set.insert(coord1);
    assert!(set.contains(&coord2), "Equal coordinates should hash equally");
    assert!(!set.contains(&coord3), "Different coordinates should hash differently");
    
    set.insert(coord3);
    assert_eq!(set.len(), 2, "Set should contain 2 unique coordinates");
}

/// Test default implementation
#[test]
fn test_default_coordinate() {
    let coord: SquareCoord4 = Default::default();
    assert_eq!(coord.x, 0);
    assert_eq!(coord.y, 0);
}

/// Benchmark-style test for performance characteristics
#[test]
fn test_performance_characteristics() {
    use std::time::Instant;
    
    let coord = SquareCoord8::new(0, 0);
    let other = SquareCoord8::new(1000, 1000);
    
    // Distance calculation should be O(1)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = coord.distance(&other);
    }
    let duration = start.elapsed();
    
    // Should complete very quickly (< 1ms for 10k operations)
    assert!(duration.as_millis() < 10, "Distance calculation should be very fast");
    
    // Neighbor calculation should be O(1)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = coord.neighbors();
    }
    let duration = start.elapsed();
    
    // Should complete very quickly (< 10ms for 10k operations)
    assert!(duration.as_millis() < 50, "Neighbor calculation should be very fast");
}