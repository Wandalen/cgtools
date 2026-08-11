//! World management for tile-based ECS games.
//!
//! This module provides a high-level interface for managing game worlds,
//! combining HECS entity management with tile-specific functionality.
//!
//! # World Features
//!
//! - Entity spawning and management
//! - Component queries and iteration
//! - System execution scheduling
//! - Grid-aware spatial queries
//! - Event handling and propagation
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::ecs::{ World, Position, Health, Movable, Stats, Team };
//! use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, FourConnected };
//! 
//! // Create a world
//! let mut world = World::new();
//! 
//! // Spawn a player entity
//! let player = world.spawn((
//!     Position::new( SquareCoord::< FourConnected >::new( 0, 0 ) ),
//!     Health::new( 100 ),
//!     Movable::new( 3 ),
//!     Stats::new( 20, 15, 12, 1 ),
//!     Team::new( 0 ),
//! ));
//! 
//! // Query entities
//! for ( entity, ( pos, health ) ) in world.query::< ( &Position< SquareCoord< FourConnected > >, &Health ) >().iter()
//! {
//!     println!( "Entity at ({}, {}) has {} health", pos.coord.x, pos.coord.y, health.current );
//! }
//! 
//! // Update game systems
//! world.update( 0.016 ); // 60 FPS
//!
//! // Request a movement; the next update applies it to the Position component
//! world.request_movement( player, SquareCoord::< FourConnected >::new( 1, 0 ) );
//! world.update( 0.016 );
//! let position = world.get::< Position< SquareCoord< FourConnected > > >( player ).unwrap();
//! assert_eq!( ( position.coord.x, position.coord.y ), ( 1, 0 ) );
//! ```

use crate::ecs::{ components::{Position, Stats, Team, Health, Size, Movable, PlayerControlled, AI, TriggerType, Trigger, Sprite}, systems::{AnimationSystem, AISystem, CombatSystem, CleanupSystem, find_entities_in_range, find_nearest_entity, AIAction, CombatEvent} };
use crate::coordinates::Distance;
use std::collections::HashMap;

/// Boxed apply closure for one queued movement request ( see `World::movement_requests` ).
type MovementRequestApply = Box< dyn FnOnce( &mut hecs::World ) -> bool + Send + Sync >;

/// Game world containing entities, components, and systems.
///
/// The World is the central container for all game state and logic in the ECS
/// architecture. It manages entity lifecycles, component storage, and system
/// execution.
pub struct World
{
  /// The underlying HECS world for entity-component storage
  pub hecs_world : hecs::World,
  /// Pending movement requests from various sources (AI, player input, etc.),
  /// latest request per entity wins. Each request is stored as a boxed apply
  /// closure capturing its typed target coordinate — closure capture is what makes
  /// the queue type-safe across coordinate systems without `World` itself being
  /// generic over any single coordinate type.
  movement_requests : HashMap< hecs::Entity, MovementRequestApply >,
  /// Game events generated this frame
  events : Vec< GameEvent >,
  /// Total elapsed time
  elapsed_time : f32,
}

impl World
{
  /// Creates a new empty world.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      hecs_world : hecs::World::new(),
      movement_requests : HashMap::new(),
      events : Vec::new(),
      elapsed_time : 0.0,
    }
  }

  /// Spawns a new entity with the given components.
  pub fn spawn( &mut self, components : impl hecs::DynamicBundle ) -> hecs::Entity
  {
    self.hecs_world.spawn( components )
  }

  /// Despawns an entity, removing it and all its components.
  ///
  /// # Errors
  /// Returns an error when the entity does not exist.
  pub fn despawn( &mut self, entity : hecs::Entity ) -> Result< (), hecs::NoSuchEntity >
  {
    self.hecs_world.despawn( entity )
  }

  /// Returns a query over entities with the specified components.
  pub fn query< Q : hecs::Query >( &self ) -> hecs::QueryBorrow< '_, Q >
  {
    self.hecs_world.query::< Q >()
  }

  /// Returns a mutable query over entities with the specified components.
  pub fn query_mut< Q : hecs::Query >( &mut self ) -> hecs::QueryMut< '_, Q >
  {
    self.hecs_world.query_mut::< Q >()
  }


  /// Gets a component from a specific entity.
  ///
  /// # Errors
  /// Returns an error when the entity does not exist or lacks the component.
  pub fn get< T : hecs::Component >( &self, entity : hecs::Entity ) -> Result< hecs::Ref< '_, T >, hecs::ComponentError >
  {
    self.hecs_world.get::< &T >( entity )
  }

  /// Gets a mutable component from a specific entity.
  ///
  /// # Errors
  /// Returns an error when the entity does not exist or lacks the component.
  pub fn get_mut< T : hecs::Component >( &mut self, entity : hecs::Entity ) -> Result< hecs::RefMut< '_, T >, hecs::ComponentError >
  {
    self.hecs_world.get::< &mut T >( entity )
  }

  /// Updates all game systems with the specified time delta.
  pub fn update( &mut self, dt : f32 )
  {
    self.elapsed_time += dt;
    self.events.clear();

    // Update animations
    AnimationSystem::update_animations( &mut self.hecs_world, dt );

    // Update AI systems
    AISystem::update_ai( &mut self.hecs_world, dt );

    // Process movement requests
    self.process_movement_requests();

    // Process combat
    let combat_events = CombatSystem::process_combat( &mut self.hecs_world );
    self.process_combat_events( combat_events );

    // Clean up defeated entities
    let defeated_entities = CleanupSystem::cleanup_defeated_entities( &mut self.hecs_world );
    for entity in defeated_entities
    {
      self.events.push( GameEvent::EntityDestroyed { entity } );
    }
  }

  /// Requests movement for an entity to a specific coordinate.
  ///
  /// The request is queued and applied by the next `update` call ( latest request
  /// per entity wins ). On application, the entity's `Position< C >` component is
  /// set to `target` and a `GameEvent::EntityMoved` event is emitted. A request
  /// whose entity no longer exists, or whose `C` does not match the entity's
  /// actual `Position< C >` component type, is discarded without effect.
  pub fn request_movement< C >( &mut self, entity : hecs::Entity, target : C )
  where
    C : 'static + Clone + Send + Sync,
  {
    self.movement_requests.insert
    (
      entity,
      Box::new( move | hecs_world : &mut hecs::World | -> bool
      {
        match hecs_world.get::< &mut Position< C > >( entity )
        {
          Ok( mut position ) =>
          {
            position.coord = target;
            true
          }
          Err( _ ) => false,
        }
      }),
    );
  }

  /// Gets all events generated this frame.
  pub fn events( &self ) -> &[ GameEvent ]
  {
    &self.events
  }

  /// Clears all pending events.
  pub fn clear_events( &mut self )
  {
    self.events.clear();
  }

  /// Gets the current elapsed time.
  pub fn elapsed_time( &self ) -> f32
  {
    self.elapsed_time
  }

  /// Finds all entities within range of a position.
  pub fn find_entities_in_range< C >
  (
    &self,
    center : &Position< C >,
    range : u32,
  ) -> Vec< ( hecs::Entity, Position< C > ) >
  where
    C : Distance + Clone + Send + Sync + 'static,
  {
    find_entities_in_range( &self.hecs_world, center, range )
  }

  /// Finds the nearest entity to a position.
  pub fn find_nearest_entity< C >
  (
    &self,
    center : &Position< C >,
  ) -> Option< ( hecs::Entity, Position< C >, u32 ) >
  where
    C : Distance + Clone + Send + Sync + 'static,
  {
    find_nearest_entity( &self.hecs_world, center )
  }

  /// Processes AI actions generated by the AI system.
  #[ allow( dead_code, reason = "not yet wired into the update loop; kept for the AI integration pass" ) ]
  fn process_ai_actions< C >( &mut self, actions : Vec< AIAction< C > > )
  where
    C : 'static + Clone + Send + Sync,
  {
    for action in actions
    {
      match action
      {
        AIAction::MoveToward { entity, target_position } =>
        {
          self.request_movement( entity, target_position );
        }
        AIAction::Attack { entity, target } =>
        {
          self.events.push( GameEvent::AttackAttempt { attacker : entity, target } );
        }
        AIAction::StartPursuit { entity, target, .. } =>
        {
          self.events.push( GameEvent::PursuitStarted { pursuer : entity, target } );
        }
        AIAction::StartPatrol { entity } =>
        {
          self.events.push( GameEvent::PatrolStarted { entity } );
        }
      }
    }
  }

  /// Applies pending movement requests to their entities' `Position` components,
  /// emitting `GameEvent::EntityMoved` for each request that actually moved an
  /// entity. Requests whose entity is gone or whose coordinate type does not match
  /// the entity's `Position` component are discarded.
  fn process_movement_requests( &mut self )
  {
    for ( entity, apply ) in self.movement_requests.drain()
    {
      if apply( &mut self.hecs_world )
      {
        self.events.push( GameEvent::EntityMoved { entity } );
      }
    }
  }

  /// Processes combat events.
  fn process_combat_events( &mut self, combat_events : Vec< CombatEvent > )
  {
    for event in combat_events
    {
      match event
      {
        CombatEvent::Damage { attacker, target, damage } =>
        {
          self.events.push( GameEvent::Damage { attacker, target, damage } );
        }
        CombatEvent::Defeated { entity } =>
        {
          self.events.push( GameEvent::EntityDefeated { entity } );
        }
      }
    }
  }
}

impl Default for World
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Game events that can occur during world updates.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum GameEvent
{
  /// An entity was spawned
  EntitySpawned
  {
    /// The newly spawned entity
    entity : hecs::Entity,
  },
  /// An entity was destroyed
  EntityDestroyed
  {
    /// The destroyed entity
    entity : hecs::Entity,
  },
  /// An entity was defeated (health reached 0)
  EntityDefeated
  {
    /// The defeated entity
    entity : hecs::Entity,
  },
  /// Damage was dealt to an entity
  Damage
  {
    /// Entity that dealt the damage
    attacker : hecs::Entity,
    /// Entity that received the damage
    target : hecs::Entity,
    /// Amount of damage dealt
    damage : u32,
  },
  /// An entity started pursuing another
  PursuitStarted
  {
    /// Entity doing the pursuing
    pursuer : hecs::Entity,
    /// Entity being pursued
    target : hecs::Entity,
  },
  /// An entity started patrolling
  PatrolStarted
  {
    /// Entity that started patrolling
    entity : hecs::Entity,
  },
  /// An attack was attempted
  AttackAttempt
  {
    /// Entity attempting the attack
    attacker : hecs::Entity,
    /// Entity being attacked
    target : hecs::Entity,
  },
  /// An entity moved to a new position
  EntityMoved
  {
    /// Entity that moved
    entity : hecs::Entity,
  },
  /// A trigger was activated
  TriggerActivated
  {
    /// The trigger entity that was activated
    trigger_entity : hecs::Entity,
    /// The entity that activated the trigger
    activated_by : hecs::Entity,
  },
}

// =============================================================================
// Utility Functions for World Management
// =============================================================================

/// Helper for spawning common entity archetypes.
pub struct EntityBuilder;

impl EntityBuilder
{
  /// Creates a basic unit entity with position, health, and stats.
  pub fn unit< C >( position : C, health : u32, stats : Stats, team : Team ) -> impl hecs::DynamicBundle 
  where
    C : 'static + Send + Sync,
  {
    (
      Position::new( position ),
      Health::new( health ),
      stats,
      team,
      Size::single(),
    )
  }

  /// Creates a player-controlled unit.
  pub fn player< C >
  (
    position : C, 
    health : u32, 
    stats : Stats, 
    player_id : u32
  ) -> impl hecs::DynamicBundle
  where
    C : 'static + Send + Sync,
  {
    (
      Position::new( position ),
      Health::new( health ),
      Movable::new( 3 ).with_diagonal(),
      stats,
      Team::new( 0 ), // Player team
      PlayerControlled::new( player_id ),
      Size::single(),
    )
  }

  /// Creates an AI-controlled enemy.
  pub fn enemy< C >
  (
    position : C,
    health : u32,
    stats : Stats,
    team : Team,
  ) -> impl hecs::DynamicBundle
  where
    C : 'static + Send + Sync,
  {
    (
      Position::new( position ),
      Health::new( health ),
      Movable::new( 2 ),
      stats,
      team,
      AI::new( 1.0 ), // 1 second decision interval
      Size::single(),
    )
  }

  /// Creates a static object (wall, obstacle, etc.).
  pub fn obstacle< C >( position : C ) -> impl hecs::DynamicBundle
  where
    C : 'static + Send + Sync,
  {
    (
      Position::new( position ),
      Size::single(),
    )
  }

  /// Creates a trigger entity.
  pub fn trigger< C >( position : C, trigger_type : TriggerType ) -> impl hecs::DynamicBundle
  where
    C : 'static + Send + Sync,
  {
    (
      Position::new( position ),
      Trigger::new( trigger_type ),
      Size::single(),
    )
  }

  /// Creates a visual-only entity (decoration, effects, etc.).
  pub fn decoration< C >( position : C, sprite : Sprite ) -> impl hecs::DynamicBundle
  where
    C : 'static + Send + Sync,
  {
    (
      Position::new( position ),
      sprite,
      Size::single(),
    )
  }
}