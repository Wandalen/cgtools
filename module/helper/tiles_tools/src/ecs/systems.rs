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

use crate::ecs::components::*;
use crate::coordinates::{Distance, Neighbors};
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

impl MovementSystem {
  /// Processes movement for all movable entities.
  ///
  /// This method validates movement requests, performs pathfinding when needed,
  /// and updates entity positions based on their movement capabilities.
  pub fn process_movement<C>(
    world: &mut hecs::World,
    movement_requests: &HashMap<hecs::Entity, C>,
  ) -> Vec<MovementResult<C>>
  where
    C: Distance + Neighbors + Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
  {
    let mut results = Vec::new();

    for (entity, target) in movement_requests {
      if let Ok((pos, movable)) = world.query_one_mut::<(&mut Position<C>, &Movable)>(*entity) {
        let movement_result = Self::calculate_movement(&pos.coord, target, movable);
        
        match movement_result
        {
          MovementResult::Success { path, new_position } =>
          {
            pos.coord = new_position.clone();
            results.push(MovementResult::Success { path, new_position });
          }
          other => results.push(other),
        }
      }
    }

    results
  }

  /// Calculates movement path and validates movement request.
  fn calculate_movement<C>(
    current: &C,
    target: &C,
    movable: &Movable,
  ) -> MovementResult<C>
  where
    C: Distance + Neighbors + Clone + PartialEq + Eq + std::hash::Hash,
  {
    // Check if target is within movement range
    let distance = current.distance(target);
    if distance > movable.range {
      return MovementResult::OutOfRange {
        requested_distance: distance,
        maximum_range: movable.range,
      };
    }

    // Use pathfinding to find valid path
    let path_result = astar(
      current,
      target,
      |_coord| true, // TODO: Add obstacle checking
      |_coord| 1,    // TODO: Add terrain cost calculation
    );

    match path_result
    {
      Some((path, cost)) =>
      {
        if cost <= movable.range {
          MovementResult::Success {
            path: path.clone(),
            new_position: target.clone(),
          }
        } else {
          MovementResult::PathTooLong {
            path_length: cost,
            maximum_range: movable.range,
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
  pub fn process_combat(world: &mut hecs::World) -> Vec<CombatEvent> {
    let mut combat_events = Vec::new();
    
    // Simplified combat processing - in a real game this would handle
    // position-based combat with specific coordinate systems
    // For now, we just check for defeated entities
    
    for (entity, health) in world.query::<&Health>().iter() {
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
  pub fn update_ai(world: &mut hecs::World, dt: f32) {
    for (_entity, ai) in world.query_mut::<&mut AI>() {
      ai.update(dt);
      
      if ai.should_make_decision() {
        // Simplified AI decision making
        ai.reset_decision_timer();
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
  pub fn update_animations(world: &mut hecs::World, dt: f32) {
    for (_entity, animation) in world.query_mut::<&mut Animation>() {
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
  pub fn cleanup_defeated_entities(world: &mut hecs::World) -> Vec<hecs::Entity> {
    let mut entities_to_remove = Vec::new();

    // Find entities with 0 health
    for (entity, health) in world.query::<&Health>().iter() {
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
pub fn find_entities_in_range<C>(
  world: &hecs::World,
  center: &Position<C>,
  range: u32,
) -> Vec<(hecs::Entity, Position<C>)>
where
  C: Distance + Clone + Send + Sync + 'static,
{
  let mut entities = Vec::new();

  for (entity, pos) in world.query::<&Position<C>>().iter() {
    if center.distance_to(pos) <= range {
      entities.push((entity, pos.clone()));
    }
  }

  entities
}

/// Finds the nearest entity to a given position.
pub fn find_nearest_entity<C>(
  world: &hecs::World,
  center: &Position<C>,
) -> Option<(hecs::Entity, Position<C>, u32)>
where
  C: Distance + Clone + Send + Sync + 'static,
{
  let mut nearest = None;
  let mut nearest_distance = u32::MAX;

  for (entity, pos) in world.query::<&Position<C>>().iter() {
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
  pub fn detect_collisions<C>(
    world: &hecs::World,
  ) -> Vec<CollisionEvent<C>>
  where
    C: Distance + Clone + PartialEq + Send + Sync + 'static,
  {
    let mut collisions = Vec::new();
    let mut query = world.query::<(&Position<C>, &Collision)>();
    let entities_with_collision: Vec<_> = query.iter().collect();

    // Check all pairs of entities for collisions
    for i in 0..entities_with_collision.len() {
      for j in (i + 1)..entities_with_collision.len() {
        let (entity1, (pos1, collision1)) = entities_with_collision[i];
        let (entity2, (pos2, collision2)) = entities_with_collision[j];

        if Self::check_collision(pos1, collision1, pos2, collision2) {
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
  fn check_collision<C>(
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
  pub fn resolve_collisions<C>(
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
  pub fn new(radius: u32) -> Self {
    Self {
      radius,
      solid: true,
      layer: 0,
    }
  }

  /// Sets the collision as non-solid (can overlap).
  pub fn non_solid(mut self) -> Self {
    self.solid = false;
    self
  }

  /// Sets the collision layer.
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
  pub fn query_circle<C>(
    world: &hecs::World,
    center: &Position<C>,
    radius: u32,
  ) -> Vec<(hecs::Entity, Position<C>)>
  where
    C: Distance + Clone + Send + Sync + 'static,
  {
    find_entities_in_range(world, center, radius)
  }

  /// Finds all entities along a line between two points.
  pub fn query_line<C>(
    world: &hecs::World,
    start: &Position<C>,
    end: &Position<C>,
  ) -> Vec<(hecs::Entity, Position<C>)>
  where
    C: Distance + Neighbors + Clone + PartialEq + std::hash::Hash + Send + Sync + 'static,
  {
    let mut entities = Vec::new();
    
    // Get line positions using simplified line tracing
    let line_positions = Self::trace_line(&start.coord, &end.coord);
    
    // Find entities at each position along the line
    for line_pos in line_positions {
      for (entity, pos) in world.query::<&Position<C>>().iter() {
        if pos.coord == line_pos {
          entities.push((entity, pos.clone()));
        }
      }
    }

    entities
  }

  /// Finds all entities within a rectangular area.
  pub fn query_rectangle<C>(
    world: &hecs::World,
    center: &Position<C>,
    width: u32,
    height: u32,
  ) -> Vec<(hecs::Entity, Position<C>)>
  where
    C: Distance + Clone + Send + Sync + 'static,
  {
    let mut entities = Vec::new();
    let max_distance = ((width * width + height * height) as f32).sqrt() as u32;

    for (entity, pos) in world.query::<&Position<C>>().iter() {
      let distance = center.distance_to(pos);
      if distance <= max_distance {
        // Additional filtering could be added here for precise rectangular bounds
        entities.push((entity, pos.clone()));
      }
    }

    entities
  }

  /// Finds entities by team affiliation within a range.
  pub fn query_by_team<C>(
    world: &hecs::World,
    center: &Position<C>,
    radius: u32,
    team_filter: impl Fn(&Team) -> bool,
  ) -> Vec<(hecs::Entity, Position<C>, Team)>
  where
    C: Distance + Clone + Send + Sync + 'static,
  {
    let mut entities = Vec::new();

    for (entity, (pos, team)) in world.query::<(&Position<C>, &Team)>().iter() {
      if center.distance_to(pos) <= radius && team_filter(team) {
        entities.push((entity, pos.clone(), team.clone()));
      }
    }

    entities
  }

  /// Simplified line tracing for spatial queries.
  fn trace_line<C>(start: &C, end: &C) -> Vec<C>
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