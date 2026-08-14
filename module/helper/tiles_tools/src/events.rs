//! Event system for decoupled game logic and inter-system communication.
//!
//! This module provides a comprehensive event system that enables loose coupling
//! between different game systems. Events allow systems to communicate without
//! direct dependencies, making code more modular, testable, and maintainable.
//!
//! # Event System Architecture
//!
//! The event system is built around several core concepts:
//!
//! - **Events**: Data structures representing game occurrences
//! - **Event Bus**: Central hub for publishing and subscribing to events
//! - **Listeners**: Functions or closures that respond to specific events
//! - **Channels**: Typed event channels for organizing different event types
//! - **Priorities**: System for controlling event processing order
//!
//! ## Event Flow
//!
//! 1. **Publish**: Systems publish events to the event bus
//! 2. **Route**: Event bus routes events to appropriate listeners
//! 3. **Process**: Listeners process events and optionally produce new events
//! 4. **Consume**: Events are consumed or propagated based on listener response
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::events::*;
//! use tiles_tools::coordinates::square::{Coordinate, FourConnected};
//!
//! // Create an event bus
//! let mut bus = EventBus::new();
//!
//! // Define a custom event
//! #[derive(Debug, Clone)]
//! struct PlayerMoved {
//!     player_id: u32,
//!     from: Coordinate<FourConnected>,
//!     to: Coordinate<FourConnected>,
//! }
//!
//! // Subscribe to events
//! bus.subscribe(|event: &PlayerMoved| {
//!     println!("Player {} moved from {:?} to {:?}", 
//!              event.player_id, event.from, event.to);
//!     EventResult::Continue
//! });
//!
//! // Publish an event
//! bus.publish(PlayerMoved {
//!     player_id: 1,
//!     from: Coordinate::new(5, 5),
//!     to: Coordinate::new(6, 5),
//! });
//!
//! // Process all pending events
//! bus.events_process();
//! ```

use std::collections::{HashMap, VecDeque};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

/// Result of event processing by a listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult
{
  /// Continue processing this event with other listeners
  Continue,
  /// Stop processing this event (consume it)
  Consume,
  /// Stop processing and remove this listener
  Unsubscribe,
}

/// Priority level for event listeners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum EventPriority
{
  /// Lowest priority - processed last
  Low = 0,
  /// Normal priority - default processing order
  #[default]
  Normal = 1,
  /// High priority - processed before normal
  High = 2,
  /// Critical priority - processed first
  Critical = 3,
}


/// Unique identifier for event listeners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

impl ListenerId {
  fn new() -> Self {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    ListenerId(COUNTER.fetch_add(1, Ordering::Relaxed))
  }
}

/// Trait for events that can be published through the event system.
pub trait Event: Any + Send + Sync + std::fmt::Debug + Clone {}

/// Automatic implementation of Event trait for types that meet requirements.
impl<T> Event for T where T: Any + Send + Sync + std::fmt::Debug + Clone {}

/// Function type for event listeners.
pub type EventListener<T> = Box<dyn Fn(&T) -> EventResult + Send + Sync>;

/// Container for a prioritized event listener.
#[derive(Clone)]
struct PrioritizedListener<T> {
  id: ListenerId,
  priority: EventPriority,
  listener: Arc<dyn Fn(&T) -> EventResult + Send + Sync>,
}

impl<T> std::fmt::Debug for PrioritizedListener<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PrioritizedListener")
      .field("id", &self.id)
      .field("priority", &self.priority)
      .finish_non_exhaustive()
  }
}

/// Channel for managing listeners of a specific event type.
struct EventChannel<T> {
  listeners: Vec<PrioritizedListener<T>>,
  pending_events: VecDeque<T>,
}

impl<T> EventChannel<T> {
  fn new() -> Self {
    Self {
      listeners: Vec::new(),
      pending_events: VecDeque::new(),
    }
  }

  fn listener_add(&mut self, listener: EventListener<T>, priority: EventPriority) -> ListenerId {
    let id = ListenerId::new();
    let prioritized = PrioritizedListener {
      id,
      priority,
      listener: Arc::from(listener),
    };
    
    // Insert in priority order (highest first)
    let insert_pos = self.listeners
      .binary_search_by(|a| prioritized.priority.cmp(&a.priority).then(a.id.0.cmp(&prioritized.id.0)))
      .unwrap_or_else(|pos| pos);
    
    self.listeners.insert(insert_pos, prioritized);
    id
  }

  fn listener_remove(&mut self, id: ListenerId) -> bool {
    if let Some(pos) = self.listeners.iter().position(|l| l.id == id) {
      self.listeners.remove(pos);
      true
    } else {
      false
    }
  }

  fn publish(&mut self, event: T) {
    self.pending_events.push_back(event);
  }

  fn events_process(&mut self) {
    while let Some(event) = self.pending_events.pop_front() {
      let mut listeners_to_remove = Vec::new();

      for listener in &self.listeners {
        match (listener.listener)(&event) {
          EventResult::Continue => {}
          EventResult::Consume => break,
          EventResult::Unsubscribe => {
            listeners_to_remove.push(listener.id);
          }
        }
      }

      // Remove listeners that requested unsubscription
      for id in listeners_to_remove {
        self.listener_remove(id);
      }
    }
  }

  fn listener_count(&self) -> usize {
    self.listeners.len()
  }

  fn pending_count(&self) -> usize {
    self.pending_events.len()
  }
}

impl<T> Default for EventChannel<T> {
  fn default() -> Self {
    Self::new()
  }
}

/// Type-erased event channel for storage in the event bus.
trait AnyEventChannel: Send + Sync {
  fn events_process(&mut self);
  fn listener_count(&self) -> usize;
  fn pending_count(&self) -> usize;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Event> AnyEventChannel for EventChannel<T> {
  fn events_process(&mut self) {
    EventChannel::events_process(self);
  }

  fn listener_count(&self) -> usize {
    EventChannel::listener_count(self)
  }

  fn pending_count(&self) -> usize {
    EventChannel::pending_count(self)
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

/// Central event bus for managing all event channels and routing.
#[derive(Default)]
pub struct EventBus {
  channels: HashMap<TypeId, Box<dyn AnyEventChannel>>,
  statistics: EventStatistics,
}

impl EventBus {
  /// Creates a new event bus.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Subscribes to events of a specific type with default priority.
  pub fn subscribe<T, F>(&mut self, listener: F) -> ListenerId
  where
    T: Event,
    F: Fn(&T) -> EventResult + Send + Sync + 'static,
  {
    self.subscribe_with_priority(listener, EventPriority::default())
  }

  /// Subscribes to events of a specific type with specified priority.
  pub fn subscribe_with_priority<T, F>(&mut self, listener: F, priority: EventPriority) -> ListenerId
  where
    T: Event,
    F: Fn(&T) -> EventResult + Send + Sync + 'static,
  {
    let channel = self.channel_get_or_create::<T>();
    let id = channel.listener_add(Box::new(listener), priority);
    self.statistics.total_subscribers += 1;
    id
  }

  /// Unsubscribes a listener by its ID.
  pub fn unsubscribe<T: Event>(&mut self, id: ListenerId) -> bool {
    let type_id = TypeId::of::<T>();
    if let Some(channel) = self.channels.get_mut(&type_id) {
      if let Some(channel) = channel.as_any_mut().downcast_mut::<EventChannel<T>>() {
        if channel.listener_remove(id) {
          self.statistics.total_subscribers = self.statistics.total_subscribers.saturating_sub(1);
          return true;
        }
      }
    }
    false
  }

  /// Publishes an event to all subscribers.
  pub fn publish<T: Event>(&mut self, event: T) {
    let channel = self.channel_get_or_create::<T>();
    channel.publish(event);
    self.statistics.events_published += 1;
  }

  /// Publishes multiple events of the same type.
  pub fn batch_publish<T: Event>(&mut self, events: Vec<T>) {
    let channel = self.channel_get_or_create::<T>();
    let event_count = events.len() as u64;
    for event in events {
      channel.publish(event);
    }
    self.statistics.events_published += event_count;
  }

  /// Processes all pending events across all channels.
  pub fn events_process(&mut self) {
    let mut events_processed = 0;

    for channel in self.channels.values_mut() {
      let pending_before = channel.pending_count();
      channel.events_process();
      events_processed += pending_before;
    }

    self.statistics.events_processed += events_processed as u64;
    self.statistics.process_cycles += 1;
  }

  /// Processes events for a specific event type only.
  pub fn events_for_type_process<T: Event>(&mut self) {
    let type_id = TypeId::of::<T>();
    if let Some(channel) = self.channels.get_mut(&type_id) {
      let pending_before = channel.pending_count();
      channel.events_process();
      self.statistics.events_processed += pending_before as u64;
    }
  }

  /// Gets statistics about event bus usage.
  #[must_use]
  pub fn statistics(&self) -> &EventStatistics {
    &self.statistics
  }

  /// Resets statistics counters.
  pub fn statistics_reset(&mut self) {
    self.statistics = EventStatistics::default();
  }

  /// Gets the number of subscribers for a specific event type.
  #[must_use]
  pub fn subscriber_count<T: Event>(&self) -> usize {
    let type_id = TypeId::of::<T>();
    self.channels.get(&type_id)
      .map_or(0, |channel| channel.listener_count())
  }

  /// Gets the number of pending events for a specific type.
  #[must_use]
  pub fn pending_count<T: Event>(&self) -> usize {
    let type_id = TypeId::of::<T>();
    self.channels.get(&type_id)
      .map_or(0, |channel| channel.pending_count())
  }

  /// Gets the total number of pending events across all types.
  #[must_use]
  pub fn total_pending_count(&self) -> usize {
    self.channels.values()
      .map(|channel| channel.pending_count())
      .sum()
  }

  /// Checks if there are any pending events.
  #[must_use]
  pub fn has_pending_events(&self) -> bool {
    self.channels.values().any(|channel| channel.pending_count() > 0)
  }

  /// Clears all events and listeners.
  pub fn clear(&mut self) {
    self.channels.clear();
    self.statistics = EventStatistics::default();
  }

  /// Gets the number of different event types registered.
  #[must_use]
  pub fn channel_count(&self) -> usize {
    self.channels.len()
  }

  // Private helper methods

  fn channel_get_or_create<T: Event>(&mut self) -> &mut EventChannel<T> {
    let type_id = TypeId::of::<T>();
    self.channels.entry(type_id)
      .or_insert_with(|| Box::new(EventChannel::<T>::new()))
      .as_any_mut()
      .downcast_mut::<EventChannel<T>>()
      .expect("Type mismatch in event channel")
  }
}

/// Statistics about event bus performance and usage.
#[derive(Debug, Default, Clone)]
pub struct EventStatistics {
  /// Total number of events published
  pub events_published: u64,
  /// Total number of events processed
  pub events_processed: u64,
  /// Number of event processing cycles
  pub process_cycles: u64,
  /// Current number of active subscribers
  pub total_subscribers: u64,
}

impl EventStatistics {
  /// Gets the average events processed per cycle.
  #[must_use]
  pub fn average_events_per_cycle(&self) -> f64 {
    if self.process_cycles > 0 {
      self.events_processed as f64 / self.process_cycles as f64
    } else {
      0.0
    }
  }

  /// Gets the processing efficiency (processed / published).
  #[must_use]
  pub fn processing_efficiency(&self) -> f64 {
    if self.events_published > 0 {
      self.events_processed as f64 / self.events_published as f64
    } else {
      1.0
    }
  }
}

// === COMMON EVENT TYPES ===

/// Common game events for typical tile-based game scenarios.
pub mod common_events {
  

  /// Event fired when an entity moves from one position to another.
  #[derive(Debug, Clone)]
  pub struct EntityMoved<C> {
    /// Entity that moved
    pub entity_id: u32,
    /// Previous position
    pub from: C,
    /// New position
    pub to: C,
    /// Movement speed or duration
    pub movement_type: MovementType,
  }

  /// Type of movement that occurred.
  #[derive(Debug, Clone, PartialEq)]
  pub enum MovementType {
    /// Instant teleportation
    Teleport,
    /// Walking at normal speed
    Walk,
    /// Running at high speed  
    Run,
    /// Flying movement
    Fly,
    /// Custom movement with specific duration
    Custom {
      /// How long the effect lasts, in milliseconds.
      duration_ms: u32,
    },
  }

  /// Event fired when an entity's health changes.
  #[derive(Debug, Clone)]
  pub struct HealthChanged {
    /// Entity whose health changed
    pub entity_id: u32,
    /// Previous health value
    pub old_health: i32,
    /// New health value
    pub new_health: i32,
    /// Cause of health change
    pub cause: HealthChangeCause,
  }

  /// Cause of health change.
  #[derive(Debug, Clone, PartialEq)]
  pub enum HealthChangeCause {
    /// Damage from combat
    Damage {
      /// Entity that dealt the damage, if any.
      attacker_id: Option<u32>,
    },
    /// Healing from items or spells
    Healing {
      /// What produced the healing.
      source: HealingSource,
    },
    /// Natural regeneration over time
    Regeneration,
    /// Direct modification (cheats, admin commands)
    Direct,
  }

  /// Source of healing.
  #[derive(Debug, Clone, PartialEq)]
  pub enum HealingSource {
    /// Healing potion or item
    Item {
      /// Identifier of the healing item.
      item_id: u32,
    },
    /// Healing spell or ability
    Spell {
      /// Identifier of the healing spell.
      spell_id: u32,
      /// Entity that cast it, if any.
      caster_id: Option<u32>,
    },
    /// Environmental healing (shrine, fountain)
    Environmental,
  }

  /// Event fired when two entities collide.
  #[derive(Debug, Clone)]
  pub struct EntitiesCollided<C> {
    /// First entity in collision
    pub entity1: u32,
    /// Second entity in collision
    pub entity2: u32,
    /// Position where collision occurred
    pub position: C,
    /// Type of collision
    pub collision_type: CollisionType,
  }

  /// Type of collision that occurred.
  #[derive(Debug, Clone, PartialEq)]
  pub enum CollisionType {
    /// Entities overlap physically
    Physical,
    /// Entity entered another's trigger zone
    Trigger,
    /// Projectile hit target
    Projectile {
      /// Damage the projectile carries.
      damage: i32,
    },
  }

  /// Event fired when an item is collected.
  #[derive(Debug, Clone)]
  pub struct ItemCollected<C> {
    /// Entity that collected the item
    pub collector_id: u32,
    /// ID of the collected item
    pub item_id: u32,
    /// Type of item collected
    pub item_type: String,
    /// Position where collection occurred
    pub position: C,
  }

  /// Event fired when a spell or ability is cast.
  #[derive(Debug, Clone)]
  pub struct SpellCast<C> {
    /// Entity casting the spell
    pub caster_id: u32,
    /// ID of the spell being cast
    pub spell_id: u32,
    /// Target position (if applicable)
    pub target_position: Option<C>,
    /// Target entity (if applicable)
    pub target_entity: Option<u32>,
    /// Mana or resource cost
    pub cost: i32,
  }

  /// Event fired when a game objective is completed.
  #[derive(Debug, Clone)]
  pub struct ObjectiveCompleted {
    /// Player or team that completed objective
    pub player_id: u32,
    /// ID of the completed objective
    pub objective_id: String,
    /// Reward given for completion
    pub reward: ObjectiveReward,
  }

  /// Reward for completing an objective.
  #[derive(Debug, Clone, PartialEq)]
  pub enum ObjectiveReward {
    /// Experience points
    Experience(u32),
    /// Gold or currency
    Gold(u32),
    /// Specific item
    Item {
      /// Identifier of the collected item.
      item_id: u32,
      /// How many were collected.
      quantity: u32,
    },
    /// Multiple rewards
    Multiple(Vec<ObjectiveReward>),
    /// No reward
    None,
  }

  /// Event fired when game state changes significantly.
  #[derive(Debug, Clone)]
  pub struct GameStateChanged {
    /// Previous game state
    pub old_state: GameState,
    /// New game state
    pub new_state: GameState,
    /// Reason for state change
    pub reason: String,
  }

  /// Possible game states.
  #[derive(Debug, Clone, PartialEq)]
  pub enum GameState {
    /// Game is initializing
    Initializing,
    /// Main menu
    MainMenu,
    /// Game is actively running
    Playing,
    /// Game is paused
    Paused,
    /// Game over screen
    GameOver,
    /// Loading screen
    Loading,
  }
}

// === UTILITY FUNCTIONS ===

/// Creates a simple event listener that just logs the event.
pub fn debug_listener<T: Event>(name: &str) -> impl Fn(&T) -> EventResult {
  let name = name.to_string();
  move |event| {
    println!("[{name}] Event: {event:?}");
    EventResult::Continue
  }
}

/// Creates a counter listener that counts how many events it has seen.
pub fn counting_listener<T: Event>() -> (impl Fn(&T) -> EventResult, Arc<Mutex<u64>>) {
  let counter = Arc::new(Mutex::new(0u64));
  let counter_clone = counter.clone();
  
  let listener = move |_event: &T| {
    if let Ok(mut count) = counter_clone.lock() {
      *count += 1;
    }
    EventResult::Continue
  };
  
  (listener, counter)
}
