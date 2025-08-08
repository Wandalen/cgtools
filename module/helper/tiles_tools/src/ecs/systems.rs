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
        
        match movement_result {
          MovementResult::Success { path, new_position } => {
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

    match path_result {
      Some((path, cost)) => {
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
    path: Vec<C>,
    new_position: C,
  },
  /// Target is out of movement range
  OutOfRange {
    requested_distance: u32,
    maximum_range: u32,
  },
  /// Path exists but is too long
  PathTooLong {
    path_length: u32,
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
    attacker: hecs::Entity,
    target: hecs::Entity,
    damage: u32,
  },
  /// Entity was defeated
  Defeated {
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
    entity: hecs::Entity,
    target: hecs::Entity,
    target_position: C,
  },
  /// Start patrolling
  StartPatrol {
    entity: hecs::Entity,
  },
  /// Move toward a position
  MoveToward {
    entity: hecs::Entity,
    target_position: C,
  },
  /// Attack a target
  Attack {
    entity: hecs::Entity,
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