//! ## Test Matrix for Coordinate Operations
//!
//! | ID   | System | Operation | Input Type | Expected Result |
//! |------|--------|-----------|------------|-----------------|
//! | T1.1 | Axial  | Create    | Valid      | Success         |
//! | T1.2 | Axial  | Distance  | Valid      | Correct value   |
//! | T1.3 | Axial  | Neighbors | Valid      | 6 neighbors     |
//! | T2.1 | Offset | Convert   | From Axial | Correct coords  |
//! | T2.2 | Offset | Convert   | To Axial   | Correct coords  |

#![cfg(feature = "integration")]

use super::*;
use tiles_tools::coordinates::hexagonal::{Axial, Coordinate, Pointy};
use tiles_tools::coordinates::{Distance, Neighbors};

/// Tests axial coordinate creation with valid input
/// Test Combination: T1.1
#[test]
fn test_coordinate_creation_axial() {
    let coord = Coordinate::<Axial, Pointy>::new(0, 0);
    assert_eq!(coord.q, 0);
    assert_eq!(coord.r, 0);
}

/// Tests distance calculation between axial coordinates
/// Test Combination: T1.2
#[test]
fn test_coordinate_distance_axial() {
    let coord1 = Coordinate::<Axial, Pointy>::new(0, 0);
    let coord2 = Coordinate::<Axial, Pointy>::new(1, 1);
    let distance = coord1.distance(coord2);
    assert_eq!(distance, 2);
}

/// Tests neighbor generation for axial coordinates
/// Test Combination: T1.3
#[test]
fn test_coordinate_neighbors_axial() {
    let coord = Coordinate::<Axial, Pointy>::new(0, 0);
    let neighbors = coord.neighbors();
    assert_eq!(neighbors.len(), 6);

    // Verify all neighbors are unique
    let mut unique_neighbors = neighbors.clone();
    unique_neighbors.sort_by_key(|c| (c.q, c.r));
    unique_neighbors.dedup();
    assert_eq!(unique_neighbors.len(), 6);
}
