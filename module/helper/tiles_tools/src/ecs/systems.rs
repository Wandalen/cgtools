//! Game systems for processing entities and components.
//!
//! This module contains systems that implement game logic by operating on
//! entities with specific component combinations. Systems are the "behavior"
//! part of the ECS architecture.
//!
//! # System Categories
//!
//! - **Movement Systems**: Handle entity movement and pathfinding
//! - **Combat Systems**: Process damage, healing, and combat resolution
//! - **AI Systems**: Update computer-controlled entity behavior
//! - **Animation Systems**: Update visual animations and effects
//! - **Trigger Systems**: Process trigger activation and effects
//!
//! # Grid-Aware Systems
//!
//! Many systems are designed to work with the coordinate system abstractions,
//! allowing them to function correctly regardless of the underlying grid type
//! (hexagonal, square, triangular, or isometric).

use crate::ecs::components::{Position, Movable, Health, AI, Animation, Team};
use crate::coordinates::{Distance, Neighbors};
use crate::coordinates::square::Coordinate as SquareCoordinate;
use crate::pathfind::astar;
use std::collections::HashMap;

// =============================================================================
// Movement Systems
// =============================================================================

/// System for processing entity movement requests.
///
/// This system handles movement validation, pathfinding, and position updates
/// for entities with movement capabilities.
pub struct MovementSystem;

impl MovementSystem
{
  /// Processes movement for all movable entities.
  ///
  /// This method validates movement requests, performs pathfinding when needed,
  /// and updates entity positions based on their movement capabilities.
  ///
  /// `is_accessible` and `cost` are the caller's obstacle and terrain policies,
  /// forwarded verbatim to [ `astar` ]: the ECS deliberately defines no obstacle or
  /// terrain component to derive them from, so the caller owns both. Pass
  /// `| _ | true` and `| _ | 1` for an open field with uniform cost.
  pub fn movement_process< C, Fa, Fc >
  (
    world : &mut hecs::World,
    movement_requests : &HashMap< hecs::Entity, C >,
    mut is_accessible : Fa,
    mut cost : Fc,
  ) -> Vec< MovementResult< C > >
  where
    C : Distance + Neighbors + Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
    Fa : FnMut( &C ) -> bool,
    Fc : FnMut( &C ) -> u32,
  {
    let mut results = Vec::new();

    for ( entity, target ) in movement_requests
    {
      if let Ok( ( pos, movable ) ) = world.query_one_mut::< ( &mut Position< C >, &Movable ) >( *entity )
      {
        let movement_result = Self::movement_calculate( &pos.coord, target, *movable, &mut is_accessible, &mut cost );

        match movement_result
        {
          MovementResult::Success { path, new_position } =>
          {
            pos.coord = new_position.clone();
            results.push( MovementResult::Success { path, new_position } );
          }
          other => results.push( other ),
        }
      }
    }

    results
  }

  /// Calculates movement path and validates movement request, using the caller's
  /// `is_accessible`/`cost` policies for pathfinding.
  fn movement_calculate< C, Fa, Fc >
  (
    current : &C,
    target : &C,
    movable : Movable,
    is_accessible : Fa,
    cost : Fc,
  ) -> MovementResult< C >
  where
    C : Distance + Neighbors + Clone + PartialEq + Eq + std::hash::Hash,
    Fa : FnMut( &C ) -> bool,
    Fc : FnMut( &C ) -> u32,
  {
    // Fix(BUG-343): removed the raw-grid-distance pre-check that used to run
    // before pathfinding -- it rejected purely on `current.distance(target)`
    // exceeding `movable.range`, a completely different metric from the
    // weighted path `cost` this function actually gates reachability on
    // below (`cost <= movable.range`). A caller-supplied `cost` policy
    // cheaper than the raw-distance heuristic (e.g. free/low-cost terrain)
    // was rejected before pathfinding ever ran, even though the real
    // weighted cost was well within range.
    // Root cause: two different metrics -- raw grid distance vs. weighted
    // path cost -- were both used to gate the same `range` budget, and the
    // cheaper (raw-distance) one ran first and could reject a target the
    // more expensive, authoritative (weighted-cost) check would have
    // accepted.
    // Pitfall: `range` is a *cost* budget (compared against `astar`'s
    // returned path cost just below), not a *distance* bound -- do not
    // reintroduce a raw-distance short-circuit ahead of the pathfind unless
    // it is proven to never reject a target the cost-based check would
    // accept (it cannot be, in general, since `cost` is caller-defined and
    // may return values below 1 per step).
    // Use pathfinding to find valid path
    let path_result = astar( current, target, is_accessible, cost );

    match path_result
    {
      Some( ( path, cost ) ) =>
      {
        if cost <= movable.range
        {
          MovementResult::Success
          {
            path : path.clone(),
            new_position : target.clone(),
          }
        }
        else
        {
          MovementResult::PathTooLong
          {
            path_length : cost,
            maximum_range : movable.range,
          }
        }
      }
      None => MovementResult::NoPathFound,
    }
  }
}

/// Result of a movement attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum MovementResult<C> {
  /// Movement was successful
  Success {
    /// The computed path taken to reach the destination
    path: Vec<C>,
    /// The final position after movement
    new_position: C,
  },
  /// Target is out of movement range
  OutOfRange {
    /// The distance to the requested target
    requested_distance: u32,
    /// The maximum movement range for this entity
    maximum_range: u32,
  },
  /// Path exists but is too long
  PathTooLong {
    /// The length of the computed path
    path_length: u32,
    /// The maximum movement range for this entity
    maximum_range: u32,
  },
  /// No valid path to target
  NoPathFound,
}

// =============================================================================
// Combat Systems  
// =============================================================================

/// System for processing combat interactions between entities.
pub struct CombatSystem;

impl CombatSystem {
  /// Processes combat between all entities within attack range.
  /// Note: Simplified implementation for demonstration
  pub fn combat_process(world: &mut hecs::World) -> Vec<CombatEvent> {
    let mut combat_events = Vec::new();
    
    // Simplified combat processing - in a real game this would handle
    // position-based combat with specific coordinate systems
    // For now, we just check for defeated entities
    
    for (entity, health) in &mut world.query::<(hecs::Entity, &Health)>() {
      if !health.is_alive() {
        combat_events.push(CombatEvent::Defeated { entity });
      }
    }

    combat_events
  }

  // Combat range checking would be implemented here in a full system
}

/// Events generated by combat system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEvent {
  /// Damage was dealt
  Damage {
    /// Entity that initiated the attack
    attacker: hecs::Entity,
    /// Entity that received the damage
    target: hecs::Entity,
    /// Amount of damage dealt
    damage: u32,
  },
  /// Entity was defeated
  Defeated {
    /// Entity that was defeated and should be removed
    entity: hecs::Entity,
  },
}

// =============================================================================
// AI Systems
// =============================================================================

/// System for updating AI-controlled entities.
pub struct AISystem;

impl AISystem {
  /// Updates AI for all AI-controlled entities.
  /// Note: Simplified implementation for demonstration
  pub fn ai_update(world: &mut hecs::World, dt: f32) {
    for ai in world.query_mut::<&mut AI>() {
      ai.update(dt);

      if ai.should_make_decision() {
        // Simplified AI decision making
        ai.decision_timer_reset();
      }
    }
  }

  // AI decision making would be implemented here with specific coordinate types
}

/// Actions that AI can take.
#[derive(Debug, Clone, PartialEq)]
pub enum AIAction<C> {
  /// Start pursuing a target
  StartPursuit {
    /// The AI entity that will start pursuing
    entity: hecs::Entity,
    /// The entity to pursue
    target: hecs::Entity,
    /// Last known position of the target
    target_position: C,
  },
  /// Start patrolling
  StartPatrol {
    /// The AI entity that will start patrolling
    entity: hecs::Entity,
  },
  /// Move toward a position
  MoveToward {
    /// The AI entity that should move
    entity: hecs::Entity,
    /// The position to move toward
    target_position: C,
  },
  /// Attack a target
  Attack {
    /// The AI entity performing the attack
    entity: hecs::Entity,
    /// The target being attacked
    target: hecs::Entity,
  },
}

// =============================================================================
// Animation Systems
// =============================================================================

/// System for updating entity animations.
pub struct AnimationSystem;

impl AnimationSystem {
  /// Updates all animations by the specified time delta.
  pub fn animations_update(world: &mut hecs::World, dt: f32) {
    for animation in world.query_mut::<&mut Animation>() {
      animation.update(dt);
    }
  }
}

// =============================================================================
// Cleanup Systems
// =============================================================================

/// System for removing defeated entities and cleaning up resources.
pub struct CleanupSystem;

impl CleanupSystem {
  /// Removes entities that have died or should be cleaned up.
  pub fn defeated_entities_cleanup(world: &mut hecs::World) -> Vec<hecs::Entity> {
    let mut entities_to_remove = Vec::new();

    // Find entities with 0 health
    for (entity, health) in &mut world.query::<(hecs::Entity, &Health)>() {
      if !health.is_alive() {
        entities_to_remove.push(entity);
      }
    }

    // Remove the entities
    for entity in &entities_to_remove {
      if world.despawn(*entity).is_ok() {
        // Entity successfully removed
      }
    }

    entities_to_remove
  }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Finds all entities within a specified range of a position.
pub fn entities_in_range_find<C>(
  world: &hecs::World,
  center: &Position<C>,
  range: u32,
) -> Vec<(hecs::Entity, Position<C>)>
where
  C: Distance + Clone + Send + Sync + 'static,
{
  let mut entities = Vec::new();

  for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<C>)>() {
    if center.distance_to(pos) <= range {
      entities.push((entity, pos.clone()));
    }
  }

  entities
}

/// Finds the nearest entity to a given position.
pub fn nearest_entity_find<C>(
  world: &hecs::World,
  center: &Position<C>,
) -> Option<(hecs::Entity, Position<C>, u32)>
where
  C: Distance + Clone + Send + Sync + 'static,
{
  let mut nearest = None;
  let mut nearest_distance = u32::MAX;

  for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<C>)>() {
    let distance = center.distance_to(pos);
    if distance < nearest_distance {
      nearest_distance = distance;
      nearest = Some((entity, pos.clone(), distance));
    }
  }

  nearest
}

// =============================================================================
// Collision Detection Systems
// =============================================================================

/// System for handling collision detection between entities.
pub struct CollisionSystem;

impl CollisionSystem {
  /// Detects collisions between all entities with collision components.
  #[ expect( clippy::similar_names, reason = "pairwise collision loop; numbered pair bindings are the clearest naming" ) ]
  pub fn collisions_detect<C>(
    world: &hecs::World,
  ) -> Vec<CollisionEvent<C>>
  where
    C: Distance + Clone + PartialEq + Send + Sync + 'static,
  {
    let mut collisions = Vec::new();
    let mut query = world.query::<(hecs::Entity, (&Position<C>, &Collision))>();
    let entities_with_collision: Vec<_> = query.iter().collect();

    // Check all pairs of entities for collisions
    for i in 0..entities_with_collision.len() {
      for j in (i + 1)..entities_with_collision.len() {
        let (entity1, (pos1, collision1)) = entities_with_collision[i];
        let (entity2, (pos2, collision2)) = entities_with_collision[j];

        if Self::collision_check(pos1, collision1, pos2, collision2) {
          collisions.push(CollisionEvent {
            entity1,
            entity2,
            position1: pos1.clone(),
            position2: pos2.clone(),
          });
        }
      }
    }

    collisions
  }

  /// Checks if two entities are colliding based on their positions and collision properties.
  fn collision_check<C>(
    pos1: &Position<C>,
    collision1: &Collision,
    pos2: &Position<C>, 
    collision2: &Collision,
  ) -> bool
  where
    C: Distance,
  {
    let distance = pos1.distance_to(pos2);
    let collision_distance = collision1.radius + collision2.radius;
    distance <= collision_distance
  }

  /// Resolves collisions by separating overlapping entities.
  pub fn collisions_resolve<C>(
    world: &mut hecs::World,
    collisions: &[CollisionEvent<C>],
  )
  where
    C: Distance + Neighbors + Clone + Send + Sync + 'static,
  {
    for collision in collisions {
      // Handle each collision separately to avoid borrowing conflicts
      if let Ok(pos1) = world.query_one_mut::<&mut Position<C>>(collision.entity1) {
        let neighbors1 = pos1.coord.neighbors();
        if let Some(best_pos1) = neighbors1.iter()
          .max_by_key(|neighbor| collision.position2.coord.distance(neighbor))
        {
          pos1.coord = best_pos1.clone();
        }
      }
      
      if let Ok(pos2) = world.query_one_mut::<&mut Position<C>>(collision.entity2) {
        let neighbors2 = pos2.coord.neighbors();
        if let Some(best_pos2) = neighbors2.iter()
          .max_by_key(|neighbor| collision.position1.coord.distance(neighbor))
        {
          pos2.coord = best_pos2.clone();
        }
      }
    }
  }
}

/// Event representing a collision between two entities.
#[derive(Debug, Clone)]
pub struct CollisionEvent<C> {
  /// First entity in collision
  pub entity1: hecs::Entity,
  /// Second entity in collision  
  pub entity2: hecs::Entity,
  /// Position of first entity
  pub position1: Position<C>,
  /// Position of second entity
  pub position2: Position<C>,
}

/// Collision component for entities that can collide.
#[derive(Debug, Clone)]
pub struct Collision {
  /// Collision radius (distance at which collision occurs)
  pub radius: u32,
  /// Whether this entity can pass through other entities
  pub solid: bool,
  /// Collision layer for filtering collision detection
  pub layer: u32,
}

impl Collision {
  /// Creates a new collision component.
  #[ must_use ]
  pub fn new(radius: u32) -> Self {
    Self {
      radius,
      solid: true,
      layer: 0,
    }
  }

  /// Sets the collision as non-solid (can overlap).
  #[ must_use ]
  pub fn non_solid(mut self) -> Self {
    self.solid = false;
    self
  }

  /// Sets the collision layer.
  #[ must_use ]
  pub fn with_layer(mut self, layer: u32) -> Self {
    self.layer = layer;
    self
  }
}

// =============================================================================
// Spatial Query Systems
// =============================================================================

/// System for efficient spatial queries and neighbor finding.
pub struct SpatialQuerySystem;

impl SpatialQuerySystem {
  /// Finds all entities within a circular area.
  pub fn circle_query<C>(
    world: &hecs::World,
    center: &Position<C>,
    radius: u32,
  ) -> Vec<(hecs::Entity, Position<C>)>
  where
    C: Distance + Clone + Send + Sync + 'static,
  {
    entities_in_range_find(world, center, radius)
  }

  /// Finds all entities along a line between two points.
  pub fn line_query<C>(
    world: &hecs::World,
    start: &Position<C>,
    end: &Position<C>,
  ) -> Vec<(hecs::Entity, Position<C>)>
  where
    C: Distance + Neighbors + Clone + PartialEq + std::hash::Hash + Send + Sync + 'static,
  {
    let mut entities = Vec::new();

    // Get line positions using simplified line tracing
    let line_positions = Self::line_trace(&start.coord, &end.coord);
    
    // Find entities at each position along the line
    for line_pos in line_positions {
      for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<C>)>() {
        if pos.coord == line_pos {
          entities.push((entity, pos.clone()));
        }
      }
    }

    entities
  }

  /// Finds all entities within a rectangular area.
  ///
  /// The rectangle is axis-aligned and centered on `center`, spanning `width` total
  /// units along x and `height` total units along y.
  pub fn rectangle_query<Connectivity>(
    world: &hecs::World,
    center: &Position<SquareCoordinate<Connectivity>>,
    width: u32,
    height: u32,
  ) -> Vec<(hecs::Entity, Position<SquareCoordinate<Connectivity>>)>
  where
    Connectivity: Clone + Send + Sync + 'static,
  {
    let mut entities = Vec::new();
    let half_width = (width / 2) as i32;
    let half_height = (height / 2) as i32;

    // Fix(BUG-136)
    // Root cause: filtered by `distance_to <= sqrt(width^2 + height^2)` -- a
    // circular region of radius equal to the rectangle's FULL diagonal (not
    // even its own half-diagonal), always a strict superset of the true
    // axis-aligned rectangle. Copy-pasted from `circle_query`'s
    // distance-threshold shape without adapting it to a per-axis test.
    // Pitfall: a rectangle is a per-axis bounds check, not a distance-metric
    // threshold -- no single scalar "distance" can express it, so this needed
    // concrete x/y field access instead of the generic `Distance` bound this
    // file's sibling queries use, narrowing the function to square coordinates
    // specifically (the only coordinate system here with an unambiguous
    // Cartesian width/height rectangle concept).
    for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<SquareCoordinate<Connectivity>>)>() {
      let dx = (pos.coord.x - center.coord.x).abs();
      let dy = (pos.coord.y - center.coord.y).abs();
      if dx <= half_width && dy <= half_height {
        entities.push((entity, pos.clone()));
      }
    }

    entities
  }

  /// Finds entities by team affiliation within a range.
  pub fn by_team_query<C>(
    world: &hecs::World,
    center: &Position<C>,
    radius: u32,
    team_filter: impl Fn(&Team) -> bool,
  ) -> Vec<(hecs::Entity, Position<C>, Team)>
  where
    C: Distance + Clone + Send + Sync + 'static,
  {
    let mut entities = Vec::new();

    for (entity, (pos, team)) in &mut world.query::<(hecs::Entity, (&Position<C>, &Team))>() {
      if center.distance_to(pos) <= radius && team_filter(team) {
        entities.push((entity, pos.clone(), *team));
      }
    }

    entities
  }

  /// Simplified line tracing for spatial queries.
  fn line_trace<C>(start: &C, end: &C) -> Vec<C>
  where
    C: Distance + Neighbors + Clone + PartialEq,
  {
    let mut line_positions = Vec::new();
    let mut current = start.clone();
    line_positions.push(current.clone());

    while current != *end && line_positions.len() < 100 {
      let neighbors = current.neighbors();
      if let Some(next) = neighbors.iter()
        .min_by_key(|neighbor| neighbor.distance(end))
      {
        if next == &current {
          break; // Prevent infinite loop
        }
        current = next.clone();
        line_positions.push(current.clone());
      } else {
        break;
      }
    }

    line_positions
  }
}