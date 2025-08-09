//! Generic A* pathfinding implementation for tile-based grids and coordinate systems.
//!
//! This module provides a flexible A* pathfinding algorithm that works with any
//! coordinate system implementing the required traits. The implementation is optimized
//! for tile-based games including strategy games, RPGs, puzzle games, and roguelikes.
//!
//! # Algorithm
//!
//! The A* (A-star) algorithm finds the shortest path between two points by:
//! 1. Maintaining an open set of nodes to explore
//! 2. Using a heuristic to estimate remaining distance to goal
//! 3. Always exploring the most promising node next
//! 4. Guaranteeing an optimal path if one exists
//!
//! # Performance Characteristics
//!
//! - **Time Complexity**: O(b^d) where b is branching factor and d is solution depth
//! - **Space Complexity**: O(b^d) for storing the search frontier
//! - **Optimality**: Guaranteed optimal path with admissible heuristic
//! - **Completeness**: Always finds a path if one exists
//!
//! # Coordinate System Support
//!
//! Works with all coordinate systems in this crate:
//! - **Square grids**: 4-connected or 8-connected movement
//! - **Hexagonal grids**: 6-neighbor movement patterns
//! - **Triangular grids**: 12-neighbor connectivity
//! - **Isometric grids**: Diamond-projected square grids
//!
//! # Examples
//!
//! ## Basic Pathfinding on Square Grid
//!
//! ```rust
//! use tiles_tools::pathfind::astar;
//! use tiles_tools::coordinates::square::{Coordinate as SquareCoord, FourConnected};
//! use std::collections::HashSet;
//! 
//! // Create start and goal positions
//! let start = SquareCoord::<FourConnected>::new(0, 0);
//! let goal = SquareCoord::<FourConnected>::new(5, 5);
//! 
//! // Define obstacles
//! let obstacles: HashSet<_> = [
//!     SquareCoord::<FourConnected>::new(2, 2),
//!     SquareCoord::<FourConnected>::new(2, 3),
//!     SquareCoord::<FourConnected>::new(2, 4),
//! ].into_iter().collect();
//! 
//! // Find path avoiding obstacles
//! let result = astar(
//!     &start,
//!     &goal,
//!     |coord| !obstacles.contains(coord), // Accessibility function
//!     |_coord| 1, // Uniform cost function
//! );
//! 
//! if let Some((path, total_cost)) = result {
//!     println!("Found path with {} steps, total cost: {}", path.len(), total_cost);
//!     for (i, pos) in path.iter().enumerate() {
//!         println!("Step {}: ({}, {})", i, pos.x, pos.y);
//!     }
//! } else {
//!     println!("No path found!");
//! }
//! ```
//!
//! ## Hexagonal Grid with Variable Terrain Costs
//!
//! ```rust
//! use tiles_tools::pathfind::astar;
//! use tiles_tools::coordinates::hexagonal::{Coordinate as HexCoord, Axial, Pointy};
//! use std::collections::HashMap;
//! 
//! let start = HexCoord::<Axial, Pointy>::new(-2, 2);
//! let goal = HexCoord::<Axial, Pointy>::new(3, -1);
//! 
//! // Define terrain costs (higher cost = harder to traverse)
//! let mut terrain_costs = HashMap::new();
//! terrain_costs.insert(HexCoord::<Axial, Pointy>::new(0, 0), 5);    // Difficult terrain
//! terrain_costs.insert(HexCoord::<Axial, Pointy>::new(1, -1), 10); // Very difficult
//! 
//! let result = astar(
//!     &start,
//!     &goal,
//!     |_coord| true, // All positions accessible
//!     |coord| terrain_costs.get(coord).copied().unwrap_or(1), // Variable costs
//! );
//! 
//! if let Some((path, cost)) = result {
//!     println!("Found hexagonal path with total cost: {}", cost);
//! }
//! ```
//!
//! ## Pathfinding with Dynamic Obstacles
//!
//! ```rust
//! use tiles_tools::pathfind::astar;
//! use tiles_tools::coordinates::square::{Coordinate as SquareCoord, EightConnected};
//! use tiles_tools::coordinates::Distance;
//! 
//! // Dynamic obstacle system
//! struct GameState {
//!     player_positions: Vec<SquareCoord<EightConnected>>,
//!     walls: Vec<SquareCoord<EightConnected>>,
//! }
//! 
//! let game_state = GameState {
//!     player_positions: vec![
//!         SquareCoord::<EightConnected>::new(3, 3),
//!         SquareCoord::<EightConnected>::new(4, 4),
//!     ],
//!     walls: vec![
//!         SquareCoord::<EightConnected>::new(5, 0),
//!         SquareCoord::<EightConnected>::new(5, 1),
//!     ],
//! };
//! 
//! let start = SquareCoord::<EightConnected>::new(0, 0);
//! let goal = SquareCoord::<EightConnected>::new(7, 7);
//! 
//! let result = astar(
//!     &start,
//!     &goal,
//!     |coord| {
//!         // Position is accessible if not blocked by walls or other players
//!         !game_state.walls.contains(coord) && 
//!         !game_state.player_positions.contains(coord)
//!     },
//!     |coord| {
//!         // Higher cost near other players (for AI avoidance behavior)
//!         let base_cost = 1;
//!         let player_penalty = game_state.player_positions.iter()
//!             .map(|player| {
//!                 let distance = coord.distance(player);
//!                 if distance <= 2 { 2 } else { 0 }
//!             })
//!             .sum::<u32>();
//!         base_cost + player_penalty
//!     },
//! );
//! ```
//!
//! # Integration with Game Systems
//!
//! The pathfinding system integrates well with other modules:
//! - Use with **Field of View** for line-of-sight pathfinding
//! - Combine with **Flow Fields** for multi-unit movement
//! - Works with **ECS systems** for entity movement
//! - Compatible with all **coordinate conversion** utilities

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
