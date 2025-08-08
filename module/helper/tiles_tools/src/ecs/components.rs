//! Core components for tile-based game entities.
//!
//! This module provides fundamental components that entities can have in a
//! tile-based game world. Components are pure data structures that describe
//! entity properties and capabilities.
//!
//! # Component Categories
//!
//! - **Spatial**: Position, Movement capabilities, Size/Shape
//! - **Gameplay**: Health, Stats, Inventory, Teams
//! - **Visual**: Sprites, Animations, Visibility
//! - **Behavioral**: AI, Player control, Triggers
//!
//! # Grid Awareness
//!
//! Many components are designed to work with any coordinate system through
//! generic type parameters, allowing seamless migration between grid types
//! or mixing entities from different coordinate systems in the same world.

use crate::coordinates::{ Distance, Neighbors };
use serde::{ Deserialize, Serialize };
// Note: PhantomData not needed for current components

// =============================================================================
// Spatial Components
// =============================================================================

/// Position component storing an entity's location in any coordinate system.
///
/// This is the fundamental spatial component that anchors entities to specific
/// grid locations. It supports all coordinate systems through generics.
///
/// # Examples
///
/// ```rust
/// use tiles_tools::ecs::Position;
/// use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, FourConnected };
/// use tiles_tools::coordinates::hexagonal::{ Coordinate as HexCoord, Axial, Pointy };
/// 
/// // Square grid position
/// let square_pos = Position::new( SquareCoord::< FourConnected >::new( 3, 7 ) );
/// 
/// // Hexagonal grid position
/// let hex_pos = Position::new( HexCoord::< Axial, Pointy >::new( 2, -1 ) );
/// ```
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub struct Position< C >
{
  /// The coordinate location of this entity
  pub coord : C,
}

impl< C > Position< C >
{
  /// Creates a new position component at the specified coordinate.
  pub fn new( coord : C ) -> Self
  {
    Self { coord }
  }

  /// Updates the position to a new coordinate.
  pub fn set( &mut self, coord : C )
  {
    self.coord = coord;
  }

  /// Gets the current coordinate.
  pub fn get( &self ) -> &C
  {
    &self.coord
  }
}

impl< C > Position< C >
where
  C : Distance,
{
  /// Calculates distance to another position.
  pub fn distance_to( &self, other : &Position< C > ) -> u32
  {
    self.coord.distance( &other.coord )
  }
}

impl< C > Position< C >
where
  C : Neighbors,
{
  /// Gets all neighboring positions from this location.
  pub fn neighbors( &self ) -> Vec< Position< C > >
  {
    self.coord.neighbors().into_iter().map( Position::new ).collect()
  }

  /// Checks if another position is adjacent to this one.
  pub fn is_adjacent_to( &self, other : &Position< C > ) -> bool
  where
    C : PartialEq,
  {
    self.coord.neighbors().contains( &other.coord )
  }
}

/// Movement capability component defining how an entity can move.
///
/// This component describes an entity's movement characteristics including
/// speed, movement type, and any constraints or special abilities.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub struct Movable
{
  /// Maximum movement range per action/turn
  pub range : u32,
  /// Whether the entity can move diagonally (for applicable grid types)
  pub diagonal_movement : bool,
  /// Whether the entity can move through other entities
  pub can_pass_through_entities : bool,
  /// Whether the entity can move through obstacles
  pub can_pass_through_obstacles : bool,
}

impl Movable
{
  /// Creates a new movable component with basic movement.
  pub fn new( range : u32 ) -> Self
  {
    Self
    {
      range,
      diagonal_movement : false,
      can_pass_through_entities : false,
      can_pass_through_obstacles : false,
    }
  }

  /// Creates a movable component with diagonal movement capability.
  pub fn with_diagonal( mut self ) -> Self
  {
    self.diagonal_movement = true;
    self
  }

  /// Creates a movable component that can pass through entities.
  pub fn with_entity_passthrough( mut self ) -> Self
  {
    self.can_pass_through_entities = true;
    self
  }

  /// Creates a movable component that can pass through obstacles.
  pub fn with_obstacle_passthrough( mut self ) -> Self
  {
    self.can_pass_through_obstacles = true;
    self
  }
}

/// Size component defining how much space an entity occupies.
///
/// This component describes the entity's spatial footprint, which can affect
/// collision detection, movement constraints, and rendering.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub struct Size
{
  /// Width in grid units
  pub width : u32,
  /// Height in grid units  
  pub height : u32,
}

impl Size
{
  /// Creates a new size component.
  pub fn new( width : u32, height : u32 ) -> Self
  {
    Self { width, height }
  }

  /// Creates a square size (1x1).
  pub fn single() -> Self
  {
    Self::new( 1, 1 )
  }

  /// Creates a square size with the specified dimension.
  pub fn square( size : u32 ) -> Self
  {
    Self::new( size, size )
  }

  /// Calculates the total area occupied.
  pub fn area( &self ) -> u32
  {
    self.width * self.height
  }
}

// =============================================================================
// Gameplay Components
// =============================================================================

/// Health component for entities that can take damage.
///
/// This component manages hit points, damage, and healing for game entities.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub struct Health
{
  /// Current health points
  pub current : u32,
  /// Maximum health points
  pub maximum : u32,
}

impl Health
{
  /// Creates a new health component with the specified maximum health.
  pub fn new( maximum : u32 ) -> Self
  {
    Self
    {
      current : maximum,
      maximum,
    }
  }

  /// Deals damage to this entity, capped at 0.
  pub fn damage( &mut self, amount : u32 )
  {
    self.current = self.current.saturating_sub( amount );
  }

  /// Heals this entity, capped at maximum health.
  pub fn heal( &mut self, amount : u32 )
  {
    self.current = ( self.current + amount ).min( self.maximum );
  }

  /// Sets health to maximum.
  pub fn full_heal( &mut self )
  {
    self.current = self.maximum;
  }

  /// Returns whether this entity is alive (health > 0).
  pub fn is_alive( &self ) -> bool
  {
    self.current > 0
  }

  /// Returns whether this entity is at full health.
  pub fn is_full_health( &self ) -> bool
  {
    self.current == self.maximum
  }

  /// Returns health as a percentage (0.0 to 1.0).
  pub fn health_percentage( &self ) -> f32
  {
    if self.maximum == 0
    {
      0.0
    }
    else
    {
      self.current as f32 / self.maximum as f32
    }
  }
}

/// Stats component for general entity attributes.
///
/// This component provides common RPG-style statistics that can affect
/// various game mechanics.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub struct Stats
{
  /// Attack power for damage calculations
  pub attack : u32,
  /// Defense value for damage reduction
  pub defense : u32,
  /// Speed for turn order and movement
  pub speed : u32,
  /// Level for scaling and requirements
  pub level : u32,
}

impl Stats
{
  /// Creates new stats with specified values.
  pub fn new( attack : u32, defense : u32, speed : u32, level : u32 ) -> Self
  {
    Self { attack, defense, speed, level }
  }

  /// Creates basic level 1 stats.
  pub fn basic() -> Self
  {
    Self::new( 10, 10, 10, 1 )
  }

  /// Calculates damage dealt to a target with specified defense.
  pub fn calculate_damage( &self, target_defense : u32 ) -> u32
  {
    self.attack.saturating_sub( target_defense / 2 ).max( 1 )
  }
}

/// Team component for entity allegiances and relationships.
///
/// This component determines which entities are allies, enemies, or neutral
/// to each other, affecting AI behavior and combat mechanics.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub struct Team
{
  /// Team identifier
  pub id : u32,
  /// Whether this team is hostile to others by default
  pub default_hostile : bool,
}

impl Team
{
  /// Creates a new team component.
  pub fn new( id : u32 ) -> Self
  {
    Self
    {
      id,
      default_hostile : false,
    }
  }

  /// Creates a hostile team component.
  pub fn hostile( id : u32 ) -> Self
  {
    Self
    {
      id,
      default_hostile : true,
    }
  }

  /// Checks if this team is allied with another team.
  pub fn is_allied_with( &self, other : &Team ) -> bool
  {
    self.id == other.id
  }

  /// Checks if this team is hostile to another team.
  pub fn is_hostile_to( &self, other : &Team ) -> bool
  {
    if self.id == other.id
    {
      false // Same team is never hostile
    }
    else
    {
      self.default_hostile || other.default_hostile
    }
  }
}

// =============================================================================
// Visual Components
// =============================================================================

/// Sprite component for visual representation.
///
/// This component defines how an entity appears when rendered, including
/// texture information and visual properties.
#[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
pub struct Sprite
{
  /// Sprite texture identifier or path
  pub texture_id : String,
  /// Tint color (RGBA)
  pub tint : [ f32; 4 ],
  /// Scale factor for rendering
  pub scale : f32,
  /// Rotation in degrees
  pub rotation : f32,
  /// Whether the sprite is currently visible
  pub visible : bool,
}

impl Sprite
{
  /// Creates a new sprite component.
  pub fn new( texture_id : impl Into< String > ) -> Self
  {
    Self
    {
      texture_id : texture_id.into(),
      tint : [ 1.0, 1.0, 1.0, 1.0 ], // White, fully opaque
      scale : 1.0,
      rotation : 0.0,
      visible : true,
    }
  }

  /// Sets the tint color.
  pub fn with_tint( mut self, r : f32, g : f32, b : f32, a : f32 ) -> Self
  {
    self.tint = [ r, g, b, a ];
    self
  }

  /// Sets the scale.
  pub fn with_scale( mut self, scale : f32 ) -> Self
  {
    self.scale = scale;
    self
  }

  /// Sets the rotation.
  pub fn with_rotation( mut self, rotation : f32 ) -> Self
  {
    self.rotation = rotation;
    self
  }

  /// Hides the sprite.
  pub fn hide( &mut self )
  {
    self.visible = false;
  }

  /// Shows the sprite.
  pub fn show( &mut self )
  {
    self.visible = true;
  }
}

/// Animation component for animated sprites.
///
/// This component manages sprite animations including frame progression,
/// timing, and playback control.
#[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
pub struct Animation
{
  /// Current frame index
  pub current_frame : u32,
  /// Total number of frames
  pub frame_count : u32,
  /// Time per frame in seconds
  pub frame_duration : f32,
  /// Time accumulated for current frame
  pub frame_timer : f32,
  /// Whether the animation should loop
  pub looping : bool,
  /// Whether the animation is currently playing
  pub playing : bool,
}

impl Animation
{
  /// Creates a new animation component.
  pub fn new( frame_count : u32, frame_duration : f32 ) -> Self
  {
    Self
    {
      current_frame : 0,
      frame_count,
      frame_duration,
      frame_timer : 0.0,
      looping : true,
      playing : true,
    }
  }

  /// Updates the animation by the specified time delta.
  pub fn update( &mut self, dt : f32 )
  {
    if !self.playing
    {
      return;
    }

    self.frame_timer += dt;

    if self.frame_timer >= self.frame_duration
    {
      self.frame_timer = 0.0;
      self.current_frame += 1;

      if self.current_frame >= self.frame_count
      {
        if self.looping
        {
          self.current_frame = 0;
        }
        else
        {
          self.current_frame = self.frame_count - 1;
          self.playing = false;
        }
      }
    }
  }

  /// Starts or resumes the animation.
  pub fn play( &mut self )
  {
    self.playing = true;
  }

  /// Pauses the animation.
  pub fn pause( &mut self )
  {
    self.playing = false;
  }

  /// Resets the animation to the first frame.
  pub fn reset( &mut self )
  {
    self.current_frame = 0;
    self.frame_timer = 0.0;
  }
}

// =============================================================================
// Behavioral Components
// =============================================================================

/// Player control component marking entities controlled by players.
///
/// This component identifies entities that should respond to player input
/// and receive special treatment in game systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerControlled {
  /// Player identifier
  pub player_id: u32,
}

impl PlayerControlled {
  /// Creates a new player control component.
  pub fn new(player_id: u32) -> Self {
    Self { player_id }
  }
}

/// AI component for computer-controlled entities.
///
/// This component defines the artificial intelligence behavior and state
/// for entities that act autonomously.
#[derive(Debug, Clone, PartialEq)]
pub struct AI {
  /// Current AI state
  pub state: AIState,
  /// Target entity (if any)
  pub target: Option<hecs::Entity>,
  /// AI decision timer
  pub decision_timer: f32,
  /// Time between AI decisions
  pub decision_interval: f32,
}

/// AI behavioral states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIState {
  /// Entity is idle and looking for something to do
  Idle,
  /// Entity is patrolling an area
  Patrolling,
  /// Entity is pursuing a target
  Pursuing,
  /// Entity is attacking a target
  Attacking,
  /// Entity is fleeing from danger
  Fleeing,
  /// Entity is guarding a specific location
  Guarding,
}

impl AI {
  /// Creates a new AI component.
  pub fn new(decision_interval: f32) -> Self {
    Self {
      state: AIState::Idle,
      target: None,
      decision_timer: 0.0,
      decision_interval,
    }
  }

  /// Updates the AI decision timer.
  pub fn update(&mut self, dt: f32) {
    self.decision_timer += dt;
  }

  /// Returns whether it's time for a new AI decision.
  pub fn should_make_decision(&self) -> bool {
    self.decision_timer >= self.decision_interval
  }

  /// Resets the decision timer.
  pub fn reset_decision_timer(&mut self) {
    self.decision_timer = 0.0;
  }

  /// Sets a new target.
  pub fn set_target(&mut self, target: Option<hecs::Entity>) {
    self.target = target;
  }

  /// Changes the AI state.
  pub fn set_state(&mut self, state: AIState) {
    self.state = state;
  }
}

/// Trigger component for entities that can activate when certain conditions are met.
///
/// This component enables entities to respond to proximity, interaction, or
/// other game events with custom behaviors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
  /// The type of trigger condition
  pub trigger_type: TriggerType,
  /// Whether the trigger can be activated multiple times
  pub repeatable: bool,
  /// Whether the trigger has been activated
  pub activated: bool,
  /// Cooldown time between activations (if repeatable)
  pub cooldown: f32,
  /// Current cooldown timer
  pub cooldown_timer: f32,
}

/// Types of trigger conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
  /// Triggers when an entity enters the same tile
  OnEnter,
  /// Triggers when an entity leaves the tile
  OnExit,
  /// Triggers when an entity is adjacent
  OnProximity,
  /// Triggers when directly interacted with
  OnInteract,
  /// Triggers after a time delay
  OnTimer(u32), // time in game ticks
}

impl Trigger {
  /// Creates a new trigger component.
  pub fn new(trigger_type: TriggerType) -> Self {
    Self {
      trigger_type,
      repeatable: false,
      activated: false,
      cooldown: 0.0,
      cooldown_timer: 0.0,
    }
  }

  /// Makes the trigger repeatable with a cooldown.
  pub fn repeatable(mut self, cooldown: f32) -> Self {
    self.repeatable = true;
    self.cooldown = cooldown;
    self
  }

  /// Updates the trigger cooldown timer.
  pub fn update(&mut self, dt: f32) {
    if self.cooldown_timer > 0.0 {
      self.cooldown_timer -= dt;
    }
  }

  /// Returns whether the trigger can be activated.
  pub fn can_activate(&self) -> bool {
    if self.activated && !self.repeatable {
      false
    } else {
      self.cooldown_timer <= 0.0
    }
  }

  /// Activates the trigger.
  pub fn activate(&mut self) {
    self.activated = true;
    if self.repeatable {
      self.cooldown_timer = self.cooldown;
    }
  }
}