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
use std::collections::{ HashMap, HashSet };

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
                .map(|coord| (coord.clone(), cost(coord)))
                .collect::<Vec<_>>()
        },
        |coord| goal.distance(coord),
        |p| *p == *goal,
    )
}

/// Enhanced A* pathfinding with edge costs (movement from one coordinate to another).
///
/// This version allows specifying different costs for movement between specific coordinates,
/// enabling more sophisticated pathfinding scenarios.
///
/// # Arguments
/// * `start`: The starting coordinate for the path.
/// * `goal`: The target coordinate for the path.
/// * `is_accessible`: A function that determines if a given coordinate can be part of the path.
/// * `edge_cost`: A function that provides the cost for moving from one coordinate to another.
///
/// # Examples
/// ```rust
/// use tiles_tools::pathfind::astar_with_edge_costs;
/// use tiles_tools::coordinates::square::{Coordinate as SquareCoord, EightConnected};
/// 
/// let start = SquareCoord::<EightConnected>::new(0, 0);
/// let goal = SquareCoord::<EightConnected>::new(3, 3);
/// 
/// let result = astar_with_edge_costs(
///     &start,
///     &goal,
///     |_coord| true,
///     |from, to| {
///         // Diagonal movement costs more
///         let dx = (to.x - from.x).abs();
///         let dy = (to.y - from.y).abs();
///         if dx == 1 && dy == 1 { 14 } else { 10 } // ~1.4x cost for diagonal
///     },
/// );
/// ```
pub fn astar_with_edge_costs< C, Fa, Fc >
(
  start : &C,
  goal : &C,
  mut is_accessible : Fa,
  mut edge_cost : Fc,
)
-> Option< ( Vec< C >, u32 ) >
where
  C : Distance + Neighbors + Eq + Clone + Hash,
  Fa : FnMut( &C ) -> bool,
  Fc : FnMut( &C, &C ) -> u32,
{
  pathfinding::prelude::astar
  (
    start,
    | coord |
    {
      coord
        .neighbors()
        .iter()
        .filter(|neighbor| is_accessible(neighbor))
        .map(|neighbor| (neighbor.clone(), edge_cost(coord, neighbor)))
        .collect::<Vec<_>>()
    },
    |coord| goal.distance(coord),
    |p| *p == *goal,
  )
}

/// Pathfinding configuration for complex scenarios.
///
/// This struct provides a builder pattern for configuring advanced pathfinding
/// with multiple constraint types and optimization options.
#[ derive( Debug, Clone ) ]
pub struct PathfindingConfig< C >
where
  C : Clone + Hash + Eq,
{
  /// Maximum search distance (prevents infinite searches)
  pub max_distance : Option< u32 >,
  /// Set of impassable coordinates
  pub obstacles : HashSet< C >,
  /// Terrain cost modifiers for specific coordinates
  pub terrain_costs : HashMap< C, u32 >,
  /// Entities that block movement
  pub blocking_entities : HashMap< C, u32 >, // position -> entity_id
  /// Base movement cost
  pub base_cost : u32,
  /// Whether diagonal movement is allowed (for applicable grids)
  pub allow_diagonal : bool,
}

impl< C > Default for PathfindingConfig< C >
where
  C : Clone + Hash + Eq,
{
  fn default() -> Self
  {
    Self
    {
      max_distance : None,
      obstacles : HashSet::new(),
      terrain_costs : HashMap::new(),
      blocking_entities : HashMap::new(),
      base_cost : 1,
      allow_diagonal : true,
    }
  }
}

impl< C > PathfindingConfig< C >
where
  C : Clone + Hash + Eq,
{
  /// Creates a new pathfinding configuration.
  pub fn new() -> Self
  {
    Self::default()
  }

  /// Sets the maximum search distance.
  pub fn with_max_distance( mut self, max_distance : u32 ) -> Self
  {
    self.max_distance = Some( max_distance );
    self
  }

  /// Adds an obstacle at the specified coordinate.
  pub fn with_obstacle( mut self, coord : C ) -> Self
  {
    self.obstacles.insert( coord );
    self
  }

  /// Adds multiple obstacles.
  pub fn with_obstacles< I >( mut self, obstacles : I ) -> Self
  where
    I : IntoIterator< Item = C >,
  {
    self.obstacles.extend( obstacles );
    self
  }

  /// Sets terrain cost for a specific coordinate.
  pub fn with_terrain_cost( mut self, coord : C, cost : u32 ) -> Self
  {
    self.terrain_costs.insert( coord, cost );
    self
  }

  /// Adds a blocking entity at the specified position.
  pub fn with_blocking_entity( mut self, coord : C, entity_id : u32 ) -> Self
  {
    self.blocking_entities.insert( coord, entity_id );
    self
  }

  /// Sets the base movement cost.
  pub fn with_base_cost( mut self, cost : u32 ) -> Self
  {
    self.base_cost = cost;
    self
  }

  /// Disables diagonal movement.
  pub fn without_diagonal( mut self ) -> Self
  {
    self.allow_diagonal = false;
    self
  }
}

/// Advanced pathfinding with comprehensive configuration support.
///
/// This function provides sophisticated pathfinding capabilities including:
/// - Obstacle avoidance
/// - Variable terrain costs
/// - Entity blocking
/// - Movement constraints
/// - Search distance limits
///
/// # Examples
/// ```rust
/// use tiles_tools::pathfind::{ astar_advanced, PathfindingConfig };
/// use tiles_tools::coordinates::square::{Coordinate as SquareCoord, FourConnected};
/// use std::collections::{ HashMap, HashSet };
/// 
/// let start = SquareCoord::<FourConnected>::new(0, 0);
/// let goal = SquareCoord::<FourConnected>::new(10, 10);
/// 
/// let config = PathfindingConfig::new()
///     .with_max_distance(20)
///     .with_obstacles([
///         SquareCoord::<FourConnected>::new(5, 5),
///         SquareCoord::<FourConnected>::new(5, 6),
///     ])
///     .with_terrain_cost(SquareCoord::<FourConnected>::new(3, 3), 5)
///     .with_base_cost(1);
/// 
/// if let Some((path, cost)) = astar_advanced(&start, &goal, &config) {
///     println!("Found advanced path with cost: {}", cost);
/// }
/// ```
pub fn astar_advanced< C >
(
  start : &C,
  goal : &C,
  config : &PathfindingConfig< C >,
)
-> Option< ( Vec< C >, u32 ) >
where
  C : Distance + Neighbors + Eq + Clone + Hash,
{
  // Check if goal is reachable within max distance
  if let Some( max_dist ) = config.max_distance
  {
    if start.distance( goal ) > max_dist
    {
      return None;
    }
  }

  pathfinding::prelude::astar
  (
    start,
    | coord |
    {
      coord
        .neighbors()
        .iter()
        .filter( | neighbor |
          {
            // Check obstacles
            if config.obstacles.contains( neighbor )
            {
              return false;
            }

            // Check blocking entities
            if config.blocking_entities.contains_key( neighbor )
            {
              return false;
            }

            // Check max distance constraint
            if let Some( max_dist ) = config.max_distance
            {
              if start.distance( neighbor ) > max_dist
              {
                return false;
              }
            }

            true
          } )
        .map( | neighbor |
          {
            let mut cost = config.base_cost;

            // Add terrain cost if specified
            if let Some( terrain_cost ) = config.terrain_costs.get( neighbor )
            {
              cost += terrain_cost;
            }

            ( neighbor.clone(), cost )
          } )
        .collect::< Vec< _ > >()
    },
    | coord | goal.distance( coord ),
    | p | *p == *goal,
  )
}

/// Finds multiple paths to different goals, returning the best path.
///
/// This function is useful for AI that has multiple possible targets and wants
/// to choose the closest or most efficient one to reach.
///
/// # Arguments
/// * `start`: The starting coordinate.
/// * `goals`: A slice of potential goal coordinates.
/// * `is_accessible`: Function to determine if a coordinate is accessible.
/// * `cost`: Function to determine movement cost.
///
/// # Returns
/// The best path found among all goals, along with the target goal that was reached.
///
/// # Examples
/// ```rust
/// use tiles_tools::pathfind::astar_multi_goal;
/// use tiles_tools::coordinates::hexagonal::{Coordinate as HexCoord, Axial, Pointy};
/// 
/// let start = HexCoord::<Axial, Pointy>::new(0, 0);
/// let goals = [
///     HexCoord::<Axial, Pointy>::new(5, 2),
///     HexCoord::<Axial, Pointy>::new(-3, 4),
///     HexCoord::<Axial, Pointy>::new(2, -5),
/// ];
/// 
/// if let Some((path, cost, goal)) = astar_multi_goal(&start, &goals, |_| true, |_| 1) {
///     println!("Best path leads to {:?} with cost {}", goal, cost);
/// }
/// ```
pub fn astar_multi_goal< C, Fa, Fc >
(
  start : &C,
  goals : &[ C ],
  mut is_accessible : Fa,
  mut cost : Fc,
) 
-> Option< ( Vec< C >, u32, C ) >
where
  C : Distance + Neighbors + Eq + Clone + Hash,
  Fa : FnMut( &C ) -> bool,
  Fc : FnMut( &C ) -> u32,
{
  let mut best_result = None;
  let mut best_cost = u32::MAX;

  for goal in goals
  {
    if let Some( ( path, total_cost ) ) = astar( start, goal, &mut is_accessible, &mut cost )
    {
      if total_cost < best_cost
      {
        best_cost = total_cost;
        best_result = Some( ( path, total_cost, goal.clone() ) );
      }
    }
  }

  best_result
}
