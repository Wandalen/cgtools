//! ## Test Matrix for Coordinate Operations

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
#![allow(clippy::min_ident_chars)]
//!
//! | ID   | System | Operation | Input Type | Expected Result |
//! |------|--------|-----------|------------|-----------------|
//! | T1.1 | Axial  | Create    | Valid      | Success         |
//! | T1.2 | Axial  | Distance  | Valid      | Correct value   |
//! | T1.3 | Axial  | Neighbors | Valid      | 6 neighbors     |
//! | T2.1 | Offset | Convert   | From Axial | Correct coords  |
//! | T2.2 | Offset | Convert   | To Axial   | Correct coords  |

#![ cfg( feature = "integration" ) ]

use super::*;
use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate, Pointy };
use tiles_tools::coordinates::{ Distance, Neighbors };

/// Tests axial coordinate creation with valid input
/// Test Combination: T1.1
#[ test ]
fn test_coordinate_creation_axial()
{
  let coord = Coordinate::< Axial, Pointy >::new( 0, 0 );
  assert_eq!( coord.q, 0 );
  assert_eq!( coord.r, 0 );
}

/// Tests distance calculation between axial coordinates
/// Test Combination: T1.2
#[ test ]
fn test_coordinate_distance_axial() {
    let coord1 = Coordinate::<Axial, Pointy>::new(0, 0);
    let coord2 = Coordinate::<Axial, Pointy>::new(1, 1);
    let distance = coord1.distance(coord2);
    assert_eq!(distance, 2);
}

/// Tests neighbor generation for axial coordinates
/// Test Combination: T1.3
#[ test ]
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
