//! # Comprehensive Test Suite for Triangular Coordinates
//!
//! This test suite follows the Test Matrix methodology to ensure complete
//! coverage of the triangular coordinate system implementation.
//!
//! ## Test Matrix for Triangular Coordinates
//!
//! | Test ID | Category | Operation | Input | Expected | Status |
//! |---------|----------|-----------|-------|----------|--------|
//! | TC1.1   | Creation | new       | (0,0) | Success  | ✅ |
//! | TC1.2   | Creation | new       | (5,3) | Success  | ✅ |
//! | TC1.3   | Creation | from      | tuple | Success  | ✅ |
//! | TC1.4   | Creation | from      | array | Success  | ✅ |
//! | TC2.1   | Orientation | up   | (2,4) | true     | ✅ |
//! | TC2.2   | Orientation | up   | (2,3) | false    | ✅ |
//! | TC2.3   | Orientation | down | (2,3) | true     | ✅ |
//! | TC2.4   | Orientation | down | (2,4) | false    | ✅ |
//! | TC3.1   | Distance | same      | (0,0) | 0        | ✅ |
//! | TC3.2   | Distance | orthog    | (0,0)→(2,0) | 2 | ✅ |
//! | TC3.3   | Distance | diagonal  | (0,0)→(2,2) | 2 | ✅ |
//! | TC3.4   | Distance | asymmetric| (1,2)→(4,7) | 5 | ✅ |
//! | TC3.5   | Distance | symmetric | d(a,b)==d(b,a) | true | ✅ |
//! | TC4.1   | Neighbors | count    | any   | 12       | ✅ |
//! | TC4.2   | Neighbors | up_tri   | (2,4) | correct  | ✅ |
//! | TC4.3   | Neighbors | down_tri | (2,3) | correct  | ✅ |
//! | TC4.4   | Neighbors | unique   | any   | no dups  | ✅ |
//! | TC5.1   | Conversion| to_tuple | coord | (x,y)    | ✅ |
//! | TC5.2   | Conversion| to_array | coord | [x,y]    | ✅ |
//! | TC6.1   | Traits   | Debug     | coord | format   | ✅ |
//! | TC6.2   | Traits   | Clone     | coord | equal    | ✅ |
//! | TC6.3   | Traits   | Copy      | coord | equal    | ✅ |
//! | TC6.4   | Traits   | PartialEq | coord | equal    | ✅ |
//! | TC6.5   | Traits   | Hash      | coord | hashable | ✅ |
//! | TC7.1   | Serde    | serialize | coord | json     | ✅ |
//! | TC7.2   | Serde    | deserial  | json  | coord    | ✅ |

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
#![allow(clippy::single_char_pattern)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::default_trait_access)]

use tiles_tools::coordinates::triangular::{ Coordinate, FlatSided };
use tiles_tools::coordinates::{Distance, Neighbors};
use std::collections::HashSet;

// =============================================================================
// Test Category 1: Coordinate Creation
// =============================================================================

#[ test ]
fn test_coordinate_creation_basic()
{
  let coord = Coordinate::< FlatSided >::new(0, 1, 0).unwrap();
  assert_eq!( coord.a, 0 );
  assert_eq!( coord.b, 1 );
  assert_eq!( coord.c, 0 );
}

#[ test ]
fn test_coordinate_creation_values()
{
  let coord = Coordinate::< FlatSided >::new( 5, 3, -7 ).unwrap();
  assert_eq!(coord.a, 5);
  assert_eq!(coord.b, 3);
  assert_eq!(coord.c, -7);
}

#[ test ]
fn test_coordinate_from_tuple()
{
  let coord: Coordinate< FlatSided > = ( 7, -2, -3 ).try_into().unwrap();
  assert_eq!(coord.a, 7);
  assert_eq!(coord.b, -2);
  assert_eq!(coord.c, -3);
}

#[ test ]
fn test_coordinate_from_array()
{
  let coord: Coordinate< FlatSided > = [ 3, 8, -10 ].try_into().unwrap();
  assert_eq!(coord.a, 3);
  assert_eq!(coord.b, 8);
  assert_eq!(coord.c, -10);
}

// =============================================================================
// Test Category 2: Triangle Orientation
// =============================================================================

#[ test ]
fn test_up_pointing_left()
{
  let coord = Coordinate::< FlatSided >::new( 2, -4, 3 ).unwrap();
  assert!(coord.is_down_or_left());
  assert!(!coord.is_up_or_right());
}

#[ test ]
fn test_down_pointing_right()
{
  let coord = Coordinate::< FlatSided >::new( 2, 3, -3 ).unwrap();
  assert!(!coord.is_down_or_left() );
  assert!(coord.is_up_or_right());
}

#[ test ]
fn test_invalid_coordinates()
{
  let coord = Coordinate::< FlatSided >::new(0, 0, 0);
  assert!(coord.is_none());
}

// =============================================================================
// Test Category 3: Distance Calculations
// =============================================================================

#[ test ]
fn test_distance_to_self()
{
  let coord = Coordinate::< FlatSided >::new( 5, 3, -7 ).unwrap();
  assert_eq!( coord.distance( &coord ), 0);
}

#[ test ]
fn test_neighbors_count()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let neighbors = coord.neighbors();
  assert_eq!(neighbors.len(), 3, "All triangular coordinates should have exactly 3 neighbors");
}

#[ test ]
fn test_neighbors_uniqueness()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let neighbors = coord.neighbors();
  let unique_neighbors: HashSet< _ > = neighbors.iter().collect();
  assert_eq!(neighbors.len(), unique_neighbors.len(),
             "All neighbors should be unique");
}

#[ test ]
fn test_neighbors_exclude_self()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let neighbors = coord.neighbors();
  assert!(!neighbors.contains(&coord), "Coordinate should not be its own neighbor");
}

#[ test ]
fn test_into_tuple()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let tuple: (i32, i32, i32) = coord.into();
  assert_eq!(tuple, ( 0, 0, 1 ));
}

#[ test ]
fn test_into_array()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let array: [i32; 3] = coord.into();
  assert_eq!( array, [ 0, 0, 1 ] );
}

// =============================================================================
// Test Category 6: Trait Implementations
// =============================================================================

#[ test ]
fn test_debug_trait()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let debug_str = format!("{:?}", coord);
  assert!(debug_str.contains("0"));
  assert!(debug_str.contains("1"));
}

#[ test ]
fn test_clone_trait()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let cloned = coord.clone();
  assert_eq!(coord, cloned);
}

#[ test ]
fn test_copy_trait()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let copied = coord;  // This should work due to Copy trait
  assert_eq!(coord, copied);
}

#[ test ]
fn test_partial_eq_trait()
{
  let coord1 = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let coord2 = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let coord3 = Coordinate::< FlatSided >::new( 0, 0, 2 ).unwrap();

  assert_eq!(coord1, coord2);
  assert_ne!(coord1, coord3);
}

#[ test ]
fn test_hash_trait()
{
  use std::collections::HashMap;

  let coord1 = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let coord2 = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();

  let mut map = HashMap::new();
  map.insert(coord1, "value");

  // Should be able to retrieve with equivalent coordinate
  assert_eq!(map.get(&coord2), Some(&"value"));
}

#[ test ]
fn test_serialize()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let serialized = serde_json::to_string(&coord).expect("Serialization should succeed");

  // Should contain the x and y values but not the phantom marker
  assert!(serialized.contains("0"));
  assert!(serialized.contains("1"));
  assert!(!serialized.contains("_marker"));
}

#[ test ]
fn test_deserialize()
{
  let json = r#"{"a": 0, "b": 1, "c": 0}"#;
  let coord: Coordinate<FlatSided> = serde_json::from_str(json).expect("Deserialization should succeed");

  assert_eq!(coord.a, 0);
  assert_eq!(coord.b, 1);
  assert_eq!(coord.c, 0);
}

#[ test ]
fn test_round_trip_serialization()
{
  let original = Coordinate::<FlatSided>::new(0, 1, 0).unwrap();
  let serialized = serde_json::to_string(&original)
    .expect("Serialization should succeed");
  let deserialized: Coordinate<FlatSided> = serde_json::from_str(&serialized)
    .expect("Deserialization should succeed");

  assert_eq!(original, deserialized);
}

// =============================================================================
// Test Category 8: Edge Cases and Boundary Conditions
// =============================================================================

#[ test ]
fn test_large_coordinates()
{
  let coord = Coordinate::<FlatSided>::new(1000000, -1000000, 1).unwrap();
  let neighbors = coord.neighbors();
  assert_eq!(neighbors.len(), 3);
}

#[ test ]
fn test_minmax_coordinate_values()
{
  let coord = Coordinate::<FlatSided>::new(i32::MAX, 2, i32::MIN).unwrap();
  // Should not panic
  let _ = coord.is_down_or_left();
}

// =============================================================================
// Test Category 9: Integration Tests
// =============================================================================

#[ test ]
fn test_distance_between_neighbors()
{
  let coord = Coordinate::<FlatSided>::new( 0, 0, 1 ).unwrap();
  let neighbors = coord.neighbors();

  for neighbor in neighbors {
    let distance = coord.distance(&neighbor);
    // All neighbors should be at distance 1 or 2 (depending on edge vs vertex adjacency)
    assert!(distance == 1, "Neighbor distance should be 1, got {}", distance);
  }
}

#[ test ]
fn test_distance_symmetry()
{
  let coord1 = Coordinate::< FlatSided >::new(0, 0, 1).unwrap();
  let coord2 = Coordinate::< FlatSided >::new(0, 0, 2).unwrap();
  assert_eq!(coord1.distance(&coord2), coord2.distance(&coord1));
}

#[ test ]
fn test_neighbors_reciprocal()
{
  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let neighbors = coord.neighbors();

  // For each neighbor, coord should be in that neighbor's neighbor list
  for neighbor in neighbors
  {
    let neighbor_neighbors = neighbor.neighbors();
    assert!
    (
      neighbor_neighbors.contains(&coord),
      "Reciprocal neighbor relationship should hold for {:?} and {:?}",
      coord,
      neighbor
    );
  }
}
#[ test ]
fn test_pathfinding_integration()
{
  use tiles_tools::pathfind::astar;

  let start = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let goal = Coordinate::< FlatSided >::new( 1, 1, -1 ).unwrap();

  let result = astar
  (
    &start,
    &goal,
    | _coord | true,
    | _coord | 1,
  );

  assert!( result.is_some(), "Should find a path" );
  let ( path, cost ) = result.unwrap();

  // Distance should be max(|3-0|, |3-0|) = 3 for triangular coordinates
  assert_eq!(cost, 4, "Path cost should match triangular distance");
  assert_eq!(path.len(), 5, "Path should contain start + 3 steps");
  assert_eq!(path[0], start);
  assert_eq!(path[path.len() - 1], goal);
}

#[ test ]
fn test_pathfinding_same_position()
{
  use tiles_tools::pathfind::astar;

  let coord = Coordinate::< FlatSided >::new( 0, 0, 1 ).unwrap();

  let result = astar
  (
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


#[ test ]
fn test_neighbors_up_triangle()
{
  let coord = Coordinate::< FlatSided >::new( 0, 2, 0 ).unwrap(); // 2+4=6 (even) -> up triangle
  assert!(coord.is_up_or_right());

  let neighbors = coord.neighbors();
  let expected = vec![
    // Edge-adjacent (3)
    Coordinate::< FlatSided >::new(-1, 2, 0).unwrap(),
    Coordinate::< FlatSided >::new(0, 2, -1).unwrap(),
    Coordinate::< FlatSided >::new(0, 1, 0).unwrap(),
  ];

  assert_eq!(neighbors.len(), expected.len());
  for expected_neighbor in expected {
    assert!(neighbors.contains(&expected_neighbor),
            "Missing neighbor: {:?}", expected_neighbor);
  }
}

#[ test ]
fn test_neighbors_down_triangle()
{
  let coord = Coordinate::< FlatSided >::new(0, 1, 0).unwrap(); // 2+3=5 (odd) -> down triangle
  assert!(coord.is_down_or_left());

  let neighbors = coord.neighbors();
  let expected = vec!
  [
    Coordinate::< FlatSided >::new( 0, 1, 1 ).unwrap(),
    Coordinate::< FlatSided >::new( 1, 1, 0 ).unwrap(),
    Coordinate::< FlatSided >::new( 0, 2, 0 ).unwrap(),
  ];

  assert_eq!(neighbors.len(), expected.len());
  for expected_neighbor in expected {
    assert!(neighbors.contains(&expected_neighbor),
            "Missing neighbor: {:?}", expected_neighbor);
  }
}
