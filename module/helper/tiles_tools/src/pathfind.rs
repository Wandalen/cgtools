//! This module provides a generic implementation of the A* pathfinding algorithm,
//! adaptable to any grid-like structure that defines coordinates, accessibility, and costs.

use crate::coordinates::{ Distance, Neighbors };
use std::hash::Hash;

/// Finds the shortest path between a start and goal coordinate using the A* algorithm.
///
/// This function is a generic wrapper around the `pathfinding::prelude::astar` function,
/// tailored for coordinate systems that implement `Distance` and `Neighbors`.
///
/// # Type Parameters
/// * `C`: The type of the coordinate, which must support distance calculation, neighbor finding,
///   equality checks, cloning, and hashing.
/// * `Fa`: A closure that takes a coordinate and returns `true` if it is traversable.
/// * `Fc`: A closure that takes a coordinate and returns the cost `u32` of moving onto it.
///
/// # Arguments
/// * `start`: The starting coordinate for the path.
/// * `goal`: The target coordinate for the path.
/// * `is_accessible`: A function that determines if a given coordinate can be part of the path.
/// * `cost`: A function that provides the cost for moving to a specific coordinate.
///
/// # Returns
/// An `Option` containing a tuple with the path as a `Vec<C>` and the total cost as a `u32`.
/// Returns `None` if no path from `start` to `goal` can be found.
pub fn astar< C, Fa, Fc >
(
  start : &C,
  goal : &C,
  mut is_accessible : Fa,
  mut cost : Fc,
)
-> Option< ( Vec< C >, u32 ) >
where
  C : Distance + Neighbors + Eq + Clone + Hash,
  Fa : FnMut( &C ) -> bool,
  Fc : FnMut( &C ) -> u32,
{
  pathfinding::prelude::astar
  (
    start,
        // origin coord
    | coord |
    {
            coord
                .neighbors()
                .iter()
                .filter(|coord| is_accessible(coord))
                .map(|coord| (coord.clone(), cost(coord))) // TODO: pass origin coord and destination coord
                .collect::<Vec<_>>()
        },
        |coord| goal.distance(coord),
        |p| *p == *goal,
    )
}
