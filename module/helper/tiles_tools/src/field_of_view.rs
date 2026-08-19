//! Field-of-view (FOV) calculations for tactical games and roguelikes.
//!
//! This module provides line-of-sight and area visibility calculations
//! that work across all coordinate systems. Essential for:
//!
//! - Tactical RPGs with vision-based gameplay
//! - Roguelike exploration and stealth mechanics
//! - RTS fog-of-war systems
//! - Turn-based strategy games
//!
//! # Algorithms
//!
//! - **Bresenham Line**: Fast line-of-sight between two points
//! - **Shadowcasting**: Efficient FOV calculation using recursive shadows
//! - **Ray Casting**: Precise visibility with adjustable precision
//! - **Flood Fill FOV**: Simple area-based visibility spreading
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::field_of_view::{FieldOfView, VisibilityMap};
//! use tiles_tools::coordinates::square::{Coordinate as SquareCoord, EightConnected};
//!
//! // Create FOV calculator
//! let mut fov = FieldOfView::new();
//!
//! // Calculate visibility from a position
//! let viewer = SquareCoord::<EightConnected>::new(2, 2);
//! let visibility = fov.fov_calculate(&viewer, 3, |coord| {
//!     // Return true if position blocks line of sight
//!     false // Open terrain
//! });
//!
//! // Check if target is visible
//! let target = SquareCoord::<EightConnected>::new(3, 3);
//! if visibility.is_visible(&target) {
//!     println!("Target is visible!");
//! }
//! ```

use crate::coordinates::{ Distance, Neighbors };
use std::collections::HashSet;

/// Field-of-view calculation algorithms.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum FOVAlgorithm
{
  /// Recursive shadowcasting (balanced speed/quality)
  Shadowcasting,
  /// Simple ray casting (slower but precise)
  RayCasting,
  /// Flood-fill based visibility (fast for small ranges)
  FloodFill,
  /// Bresenham line algorithm (fast but basic)
  Bresenham,
}

/// Visibility state for a coordinate position.
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub struct VisibilityState
{
  /// Whether the position is visible
  pub visible: bool,
  /// Distance from the viewer (0 = viewer position)
  pub distance: u32,
  /// Light level (0.0 = dark, 1.0 = full visibility)
  pub light_level: f32,
  /// Whether line of sight is blocked beyond this point
  pub blocks_sight: bool,
}

impl VisibilityState
{
  /// Creates a new visibility state
  #[ must_use ]
  pub fn new( visible : bool, distance : u32, light_level : f32 ) -> Self
  {
    Self
    {
      visible,
      distance,
      light_level,
      blocks_sight : false,
    }
  }

  /// Creates a visibility state for a blocking position
  #[ must_use ]
  pub fn blocking( distance : u32, light_level : f32 ) -> Self
  {
    Self
    {
      visible : true, // Blocking positions are visible themselves
      distance,
      light_level,
      blocks_sight : true,
    }
  }

  /// Creates an invisible state
  #[ must_use ]
  pub fn invisible() -> Self
  {
    Self
    {
      visible : false,
      distance : u32::MAX,
      light_level : 0.0,
      blocks_sight : false,
    }
  }
}

/// Map storing visibility information for coordinate positions.
pub struct VisibilityMap< C >
{
  /// Visibility states by coordinate
  visibility : std::collections::HashMap< C, VisibilityState >,
}

impl< C > VisibilityMap< C >
where
  C : Clone + std::hash::Hash + Eq,
{
  /// Creates a new empty visibility map.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      visibility : std::collections::HashMap::new(),
    }
  }

  /// Sets visibility state for a coordinate.
  pub fn visibility_set( &mut self, coord : &C, state : VisibilityState )
  {
    self.visibility.insert( coord.clone(), state );
  }

  /// Gets visibility state for a coordinate.
  pub fn visibility_get( &self, coord : &C ) -> Option< &VisibilityState >
  {
    self.visibility.get( coord )
  }

  /// Checks if a coordinate is visible.
  pub fn is_visible( &self, coord : &C ) -> bool
  {
    self.visibility.get( coord )
      .is_some_and( | state | state.visible )
  }

  /// Gets the distance to a coordinate.
  pub fn distance_to( &self, coord : &C ) -> Option< u32 >
  {
    self.visibility.get( coord ).map( | state | state.distance )
  }

  /// Gets light level at a coordinate.
  pub fn light_level_at( &self, coord : &C ) -> f32
  {
    self.visibility.get( coord )
      .map_or( 0.0, | state | state.light_level )
  }

  /// Returns all visible coordinates.
  #[ must_use ]
  pub fn visible_coordinates( &self ) -> Vec< C >
  {
    self.visibility.iter()
      .filter( | ( _, state ) | state.visible )
      .map( | ( coord, _ ) | coord.clone() )
      .collect()
  }

  /// Returns coordinates within a specific distance range.
  #[ must_use ]
  pub fn coordinates_in_range( &self, min_dist : u32, max_dist : u32 ) -> Vec< C >
  {
    self.visibility.iter()
      .filter( | ( _, state ) |
      {
        state.visible && state.distance >= min_dist && state.distance <= max_dist
      })
      .map( | ( coord, _ ) | coord.clone() )
      .collect()
  }

  /// Returns iterator over all visible positions.
  ///
  /// This method provides an efficient way to iterate over positions
  /// that are currently visible without allocating a new vector.
  pub fn visible_positions( &self ) -> impl Iterator< Item = C > + '_
  {
    self.visibility.iter()
      .filter_map( | ( coord, state ) |
      {
        if state.visible
        {
          Some( coord.clone() )
        }
        else
        {
          None
        }
      })
  }
}

impl< C > Default for VisibilityMap< C >
where
  C : Clone + std::hash::Hash + Eq,
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Main field-of-view calculator supporting multiple algorithms.
pub struct FieldOfView
{
  /// Algorithm to use for FOV calculations
  algorithm : FOVAlgorithm,
  /// Whether to include the viewer position in results
  include_viewer : bool,
}

impl FieldOfView
{
  /// Creates a new FOV calculator with shadowcasting algorithm.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      algorithm : FOVAlgorithm::Shadowcasting,
      include_viewer : true,
    }
  }

  /// Creates a FOV calculator with a specific algorithm.
  #[ must_use ]
  pub fn with_algorithm( algorithm : FOVAlgorithm ) -> Self
  {
    Self
    {
      algorithm,
      include_viewer : true,
    }
  }

  /// Sets whether to include the viewer position in visibility results.
  #[ must_use ]
  pub fn viewer_include( mut self, include : bool ) -> Self
  {
    self.include_viewer = include;
    self
  }

  /// Returns the algorithm this calculator is configured to use.
  #[ must_use ]
  pub fn algorithm( &self ) -> FOVAlgorithm
  {
    self.algorithm
  }

  /// Returns `true` when the viewer position is included in visibility results.
  #[ must_use ]
  pub fn includes_viewer( &self ) -> bool
  {
    self.include_viewer
  }

  /// Calculates field of view from a position.
  ///
  /// # Arguments
  /// - `viewer`: The position calculating FOV from
  /// - `max_range`: Maximum distance to calculate visibility
  /// - `blocks_sight`: Function returning true if a position blocks line of sight
  ///
  /// # Returns
  /// A visibility map containing all visible positions and their states.
  pub fn fov_calculate< C, F >
  (
    &self,
    viewer : &C,
    max_range : u32,
    blocks_sight : F
  ) -> VisibilityMap< C >
  where
    C : Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F : Fn( &C ) -> bool,
  {
    let mut visibility_map = VisibilityMap::new();

    match self.algorithm
    {
      FOVAlgorithm::Shadowcasting =>
      {
        Self::shadowcasting_fov_calculate( viewer, max_range, &blocks_sight, &mut visibility_map );
      }
      FOVAlgorithm::RayCasting =>
      {
        Self::ray_casting_fov_calculate( viewer, max_range, &blocks_sight, &mut visibility_map );
      }
      FOVAlgorithm::FloodFill =>
      {
        Self::flood_fill_fov_calculate( viewer, max_range, &blocks_sight, &mut visibility_map );
      }
      FOVAlgorithm::Bresenham =>
      {
        Self::bresenham_fov_calculate( viewer, max_range, &blocks_sight, &mut visibility_map );
      }
    }

    // Add viewer position if requested
    if self.include_viewer
    {
      visibility_map.visibility_set( viewer, VisibilityState::new( true, 0, 1.0 ) );
    }

    visibility_map
  }

  /// Calculates line of sight between two specific points.
  pub fn line_of_sight< C, F >( &self, from : &C, to : &C, blocks_sight : F ) -> bool
  where
    C : Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F : Fn( &C ) -> bool,
  {
    let distance = from.distance( to );
    let visibility = self.fov_calculate( from, distance + 1, blocks_sight );
    visibility.is_visible( to )
  }

  /// Shadowcasting FOV algorithm implementation.
  ///
  /// This implements recursive shadowcasting that processes octants systematically
  /// to create accurate field-of-view calculations with proper shadow casting.
  fn shadowcasting_fov_calculate< C, F >
  (
    viewer : &C,
    max_range : u32,
    blocks_sight : &F,
    visibility_map : &mut VisibilityMap< C >
  )
  where
    C : Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F : Fn( &C ) -> bool,
  {
    // Cast shadows in 8 octants around the viewer
    let neighbors = viewer.neighbors();
    let neighbor_count = neighbors.len();

    // For each direction from the viewer, cast rays outward
    for i in 0..neighbor_count
    {
      Self::octant_shadows_cast( viewer, max_range, blocks_sight, visibility_map, i, neighbor_count );
    }
  }

  /// Casts shadows in a specific octant direction.
  fn octant_shadows_cast< C, F >
  (
    viewer : &C,
    max_range : u32,
    blocks_sight : &F,
    visibility_map : &mut VisibilityMap< C >,
    octant : usize,
    total_directions : usize,
  )
  where
    C : Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F : Fn( &C ) -> bool,
  {
    // Simple octant-based shadowcasting implementation
    // Start from viewer and expand outward in the specified direction
    let mut current_positions = vec![ viewer.clone() ];
    let mut visited_positions = rustc_hash::FxHashSet::default();

    for _distance in 1..=max_range
    {
      let mut next_positions = rustc_hash::FxHashSet::default();
      let mut blocked_positions = rustc_hash::FxHashSet::default();

      for pos in &current_positions
      {
        let neighbors = pos.neighbors();

        // Select neighbors in the octant direction
        //
        // Fix(BUG-135)
        // Root cause: filtering already-visited neighbors *before* enumerating
        // desynced the loop index `i` from the fixed direction slot each
        // neighbor actually occupies in `pos.neighbors()`, so the
        // octant-membership check below tested the wrong slot once any
        // neighbor of `pos` had already been visited (true for every ring
        // beyond the first).
        // Pitfall: `enumerate()` must run on the unfiltered iterator -- filtering
        // afterwards keeps `i` tied to each neighbor's real position in the
        // fixed-order array `pos.neighbors()` returns, which is what the
        // octant math below assumes.
        for ( i, neighbor ) in neighbors.iter().enumerate().filter( | ( _, n ) | !visited_positions.contains( *n ) )
        {
          if ( i + total_directions - octant ) % total_directions < 3 ||
             ( i + total_directions - octant ) % total_directions > total_directions - 3
          {
            let actual_distance = viewer.distance( neighbor );
            if actual_distance <= max_range
            {
              let light_level = ( 1.0f32 - ( actual_distance as f32 / max_range as f32 ) ).max( 0.0f32 );

              let is_blocked = blocks_sight( neighbor );
              let visibility_state = if is_blocked
              {
                blocked_positions.insert( neighbor.clone() );
                VisibilityState::blocking( actual_distance, light_level )
              }
              else
              {
                VisibilityState::new( true, actual_distance, light_level )
              };

              visibility_map.visibility_set( neighbor, visibility_state );

              if !is_blocked
              {
                next_positions.insert( neighbor.clone() );
              }
            }
          }
        }
      }

      visited_positions.extend( current_positions.iter().cloned() );

      // Remove blocked positions from expansion
      current_positions = next_positions.into_iter()
        .filter( | pos | !blocked_positions.contains( pos ) )
        .collect();

      if current_positions.is_empty()
      {
        break;
      }
    }
  }

  /// Ray casting FOV algorithm implementation.
  ///
  /// This casts rays in all directions from the viewer to determine visibility.
  /// More precise than shadowcasting but computationally more expensive.
  fn ray_casting_fov_calculate<C, F>(
    viewer: &C,
    max_range: u32,
    blocks_sight: &F,
    visibility_map: &mut VisibilityMap<C>
  )
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F: Fn(&C) -> bool,
  {
    // Use all neighbors as ray directions
    let neighbors = viewer.neighbors();

    // Cast rays in each neighbor direction
    for start_neighbor in neighbors
    {
      Self::directional_ray_cast(viewer, &start_neighbor, max_range, blocks_sight, visibility_map);
    }

    // Also cast rays to diagonal directions by combining neighbor directions
    let neighbor_list = viewer.neighbors();
    for i in 0..neighbor_list.len()
    {
      for j in (i + 1)..neighbor_list.len()
      {
        // Try to find positions that represent diagonal rays
        if let Some(diagonal_target) = Self::diagonal_target_find(viewer, &neighbor_list[i], &neighbor_list[j], max_range)
        {
          Self::directional_ray_cast(viewer, &diagonal_target, max_range, blocks_sight, visibility_map);
        }
      }
    }
  }

  /// Casts a single ray in a specific direction.
  fn directional_ray_cast<C, F>(
    viewer: &C,
    direction_target: &C,
    max_range: u32,
    blocks_sight: &F,
    visibility_map: &mut VisibilityMap<C>
  )
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F: Fn(&C) -> bool,
  {
    // Trace along the direction using neighbor-based stepping
    let mut current = viewer.clone();
    let mut distance = 0u32;

    while distance < max_range
    {
      let neighbors = current.neighbors();
      let mut best_next = None;
      let mut best_alignment = f32::MIN;

      // Find the neighbor that best aligns with our target direction
      for neighbor in neighbors
      {
        let alignment = Self::direction_alignment_calculate(viewer, direction_target, &current, &neighbor);
        if alignment > best_alignment
        {
          best_alignment = alignment;
          best_next = Some(neighbor);
        }
      }

      if let Some(next) = best_next
      {
        current = next;
        distance = viewer.distance(&current);

        if distance > max_range
        {
          break;
        }

        let light_level = (1.0f32 - (distance as f32 / max_range as f32)).max(0.0f32);
        let is_blocked = blocks_sight(&current);

        let visibility_state = if is_blocked
        {
          VisibilityState::blocking(distance, light_level)
        }
        else
        {
          VisibilityState::new(true, distance, light_level)
        };

        visibility_map.visibility_set(&current, visibility_state);

        if is_blocked
        {
          break; // Ray is blocked, stop casting
        }
      }
      else
      {
        break; // No valid next position
      }
    }
  }

  /// Calculates how well a move from current to next aligns with the target direction.
  fn direction_alignment_calculate<C>(
    viewer: &C,
    direction_target: &C,
    current: &C,
    next: &C,
  ) -> f32
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
  {
    // Simple alignment calculation based on distance ratios
    let target_distance = viewer.distance(direction_target) as f32;
    let current_distance = viewer.distance(current) as f32;
    let next_distance = viewer.distance(next) as f32;
    let target_to_next = direction_target.distance(next) as f32;

    // Fix(BUG-267): `current_distance == 0.0` is true on every ray's first
    // hop (directional_ray_cast starts with `current = viewer.clone()`), so
    // this guard fired for every candidate neighbor on the first step of
    // every ray regardless of `direction_target`, making all of them tie at
    // alignment 0.0. The strict `>` comparison in directional_ray_cast's
    // caller then always kept the first-iterated neighbor, so every ray --
    // whatever direction it was aimed at -- took its first hop toward the
    // same fixed neighbor.
    // Root cause: `current_distance` is never used as a divisor anywhere in
    // this function (only `target_distance` is), so guarding on it being
    // zero protects nothing; it was likely copy-pasted alongside the
    // legitimate `target_distance == 0.0` guard without checking whether it
    // applied.
    // Pitfall: a zero-value guard must protect an actual division by that
    // value -- a guard that merely mentions a variable used elsewhere in the
    // function does not verify that the variable is a divisor there.
    if target_distance == 0.0
    {
      return 0.0;
    }

    // Prefer moves that keep us on track toward the direction
    let progress = (next_distance - current_distance) / target_distance;
    let deviation_penalty = target_to_next / (target_distance + 1.0);

    progress - deviation_penalty
  }

  /// Finds a diagonal target position for ray casting.
  fn diagonal_target_find<C>(
    viewer: &C,
    neighbor1: &C,
    neighbor2: &C,
    max_range: u32,
  ) -> Option<C>
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
  {
    // Try to find a position that represents a diagonal direction
    // This is a simplified approach - we look for common neighbors
    let first_neighbors = neighbor1.neighbors();
    let second_neighbors = neighbor2.neighbors();

    // Find positions that are neighbors to both directions
    for n1 in &first_neighbors
    {
      for n2 in &second_neighbors
      {
        if n1 == n2 && viewer.distance(n1) <= max_range
        {
          return Some(n1.clone());
        }
      }
    }

    None
  }

  /// Flood fill FOV algorithm implementation.
  fn flood_fill_fov_calculate<C, F>(
    viewer: &C,
    max_range: u32,
    blocks_sight: &F,
    visibility_map: &mut VisibilityMap<C>
  )
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F: Fn(&C) -> bool,
  {
    // Simple flood-fill visibility
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    queue.push_back((viewer.clone(), 0));

    while let Some((current_pos, distance)) = queue.pop_front() {
      if visited.contains(&current_pos) || distance > max_range {
        continue;
      }

      visited.insert(current_pos.clone());

      let light_level = (1.0f32 - (distance as f32 / max_range as f32)).max(0.0f32);
      let is_blocked = blocks_sight(&current_pos);

      let visibility_state = if is_blocked {
        VisibilityState::blocking(distance, light_level)
      } else {
        VisibilityState::new(true, distance, light_level)
      };

      visibility_map.visibility_set(&current_pos, visibility_state);

      // Continue spreading if not blocked
      if !is_blocked && distance < max_range {
        for neighbor_coord in current_pos.neighbors() {
          if !visited.contains(&neighbor_coord) {
            queue.push_back((neighbor_coord, distance + 1));
          }
        }
      }
    }
  }

  /// Bresenham line FOV algorithm implementation.
  fn bresenham_fov_calculate<C, F>(
    viewer: &C,
    max_range: u32,
    blocks_sight: &F,
    visibility_map: &mut VisibilityMap<C>
  )
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F: Fn(&C) -> bool,
  {
    // Use Bresenham lines to all positions within range
    let mut all_positions = HashSet::new();

    // Collect all positions within max_range using BFS
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((viewer.clone(), 0));

    while let Some((current_pos, distance)) = queue.pop_front() {
      if visited.contains(&current_pos) || distance > max_range {
        continue;
      }

      visited.insert(current_pos.clone());
      all_positions.insert(current_pos.clone());

      for neighbor_coord in current_pos.neighbors() {
        if !visited.contains(&neighbor_coord) {
          queue.push_back((neighbor_coord, distance + 1));
        }
      }
    }

    // Check line of sight to each position
    for target in all_positions {
      let distance = viewer.distance(&target);
      let has_line_of_sight = Self::bresenham_line_check(viewer, &target, blocks_sight);

      if has_line_of_sight {
        let light_level = (1.0f32 - (distance as f32 / max_range as f32)).max(0.0f32);
        let is_blocked = blocks_sight(&target);

        let visibility_state = if is_blocked {
          VisibilityState::blocking(distance, light_level)
        } else {
          VisibilityState::new(true, distance, light_level)
        };

        visibility_map.visibility_set(&target, visibility_state);
      }
    }
  }

  /// Checks line of sight using Bresenham line algorithm.
  fn bresenham_line_check<C, F>(from: &C, to: &C, blocks_sight: &F) -> bool
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
    F: Fn(&C) -> bool,
  {
    // Use neighbor-based line tracing for generic coordinate systems
    let line_positions = Self::bresenham_line_trace(from, to);

    // Check if any position along the line (except endpoints) blocks sight
    for pos in line_positions.iter().skip(1) // Skip starting position
    {
      if pos == to
      {
        break; // Target position reached
      }

      if blocks_sight(pos)
      {
        return false; // Line of sight blocked
      }
    }

    true // Clear line of sight
  }

  /// Traces a line between two coordinates using neighbor-based approximation.
  ///
  /// This provides a Bresenham-like line tracing that works with any coordinate
  /// system by using neighbor relationships rather than integer arithmetic.
  fn bresenham_line_trace<C>(from: &C, to: &C) -> Vec<C>
  where
    C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
  {
    // BUG-346 task/bug/346_bresenham_line_of_sight_asymmetric.md -- greedy walk
    // made line_of_sight asymmetric between call directions; fix below.
    // Fix(BUG-346): canonicalize the walk direction (always walk from the
    // hash-smaller endpoint toward the hash-larger one, then reverse the
    // result if the caller's `from`/`to` were the other way round) so the
    // set of intermediate cells visited no longer depends on which endpoint
    // the caller labeled `from` vs `to`.
    // Root cause: the walk below is a greedy "step to whichever neighbor is
    // closest to the fixed target" search, which is not path-reversible --
    // tracing A->B and B->A could visit different intermediate cells, so one
    // direction could route around a wall the other ran straight through.
    // Pitfall: canonicalizing via a coordinate-specific ordering would need
    // an `Ord` bound that ripples out to every coordinate system usable with
    // `FieldOfView` (all 4 algorithms share this function's generic bounds);
    // comparing `Hash` output instead needs no new bound and is still
    // deterministic across both call directions, since the two hash values
    // being compared are identical regardless of which endpoint is passed
    // as `from` vs `to`.
    use std::hash::Hasher;
    let hash_of = | c : &C |
    {
      let mut hasher = std::collections::hash_map::DefaultHasher::new();
      c.hash( &mut hasher );
      hasher.finish()
    };

    if from == to
    {
      return vec![ from.clone() ];
    }

    let swapped = hash_of( from ) > hash_of( to );
    let ( start, end ) = if swapped { ( to, from ) } else { ( from, to ) };

    let mut line_positions = Vec::new();
    let mut current = start.clone();
    line_positions.push(current.clone());

    // Simple neighbor-based line tracing
    while current != *end
    {
      let neighbors = current.neighbors();
      let mut best_neighbor = None;
      let mut best_distance = u32::MAX;

      // Find neighbor that gets us closest to the target
      for neighbor in neighbors
      {
        let distance_to_target = neighbor.distance(end);
        if distance_to_target < best_distance
        {
          best_distance = distance_to_target;
          best_neighbor = Some(neighbor);
        }
      }

      if let Some(next) = best_neighbor
      {
        if next == current
        {
          break; // Prevent infinite loop
        }
        current = next;
        line_positions.push(current.clone());

        // Prevent infinite loops in complex coordinate systems
        if line_positions.len() > 1000
        {
          break;
        }
      }
      else
      {
        break; // No valid path found
      }
    }

    if swapped
    {
      line_positions.reverse();
    }

    line_positions
  }
}

impl Default for FieldOfView
{
  fn default() -> Self
  {
    Self::new()
  }
}

// =============================================================================
// Advanced FOV Features
// =============================================================================

/// Light source for dynamic lighting calculations.
#[ derive( Debug, Clone ) ]
pub struct LightSource< C >
{
  /// Position of the light source
  pub position : C,
  /// Maximum light radius
  pub radius : u32,
  /// Light intensity (0.0 to 1.0)
  pub intensity : f32,
  /// Light color (RGB values 0.0 to 1.0)
  pub color : ( f32, f32, f32 ),
  /// Whether light passes through blocking terrain
  pub penetrates_walls : bool,
}

impl< C > LightSource< C >
{
  /// Creates a new light source.
  pub fn new( position : C, radius : u32, intensity : f32 ) -> Self
  {
    Self
    {
      position,
      radius,
      intensity,
      color : ( 1.0, 1.0, 1.0 ), // White light
      penetrates_walls : false,
    }
  }

  /// Sets the light color.
  #[ must_use ]
  pub fn with_color( mut self, r : f32, g : f32, b : f32 ) -> Self
  {
    self.color = ( r, g, b );
    self
  }

  /// Sets whether light penetrates walls.
  #[ must_use ]
  pub fn penetrating( mut self, penetrates : bool ) -> Self
  {
    self.penetrates_walls = penetrates;
    self
  }
}

/// Multi-source lighting calculator.
pub struct LightingCalculator< C >
{
  /// Light sources in the scene
  light_sources : Vec< LightSource< C > >,
  /// FOV calculator for line-of-sight checks
  fov_calculator : FieldOfView,
}

impl< C > LightingCalculator< C >
where
  C : Distance + Neighbors + Clone + std::hash::Hash + Eq,
{
  /// Creates a new lighting calculator.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      light_sources : Vec::new(),
      fov_calculator : FieldOfView::new(),
    }
  }

  /// Adds a light source.
  pub fn light_source_add( &mut self, light : LightSource< C > )
  {
    self.light_sources.push( light );
  }

  /// Calculates combined lighting from all sources.
  pub fn lighting_calculate< F >( &self, blocks_sight : F ) -> std::collections::HashMap< C, f32 >
  where
    F : Fn( &C ) -> bool,
  {
    let mut lighting_map = std::collections::HashMap::new();

    // Calculate lighting contribution from each source
    for light_source in &self.light_sources
    {
      let visibility_map = if light_source.penetrates_walls
      {
        // For penetrating light, create visibility without sight blocking
        self.fov_calculator.fov_calculate( &light_source.position, light_source.radius, | _ | false )
      }
      else
      {
        // Normal line-of-sight based lighting
        self.fov_calculator.fov_calculate( &light_source.position, light_source.radius, &blocks_sight )
      };

      // Add light contribution to each visible position
      for coord in visibility_map.visible_coordinates()
      {
        let distance = light_source.position.distance( &coord ) as f32;
        let light_falloff = ( 1.0f32 - ( distance / light_source.radius as f32 ) ).max( 0.0f32 );
        let light_contribution = light_source.intensity * light_falloff;

        let current_light = lighting_map.get( &coord ).unwrap_or( &0.0 );
        lighting_map.insert( coord, ( current_light + light_contribution ).min( 1.0 ) );
      }
    }

    lighting_map
  }
}

impl< C > Default for LightingCalculator< C >
where
  C : Distance + Neighbors + Clone + std::hash::Hash + Eq,
{
  fn default() -> Self
  {
    Self::new()
  }
}
