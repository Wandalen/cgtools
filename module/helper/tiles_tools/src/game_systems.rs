//! Advanced game mechanics and systems integration for tile-based games.
//!
//! This module provides comprehensive game systems that integrate all the core
//! components of tiles_tools into cohesive game mechanics. It includes turn-based
//! utilities, game state management, multi-system coordination, and advanced
//! gameplay patterns commonly used in tile-based games.
//!
//! # Game Systems Features
//!
//! - **Turn-Based Management**: Initiative systems, action points, turn queues
//! - **Game State Machine**: State transitions, game phases, conditional logic
//! - **Resource Management**: Health, mana, inventory, economics
//! - **Combat Systems**: Damage calculation, status effects, tactical mechanics
//! - **Quest Management**: Objectives, triggers, branching narratives
//! - **World Simulation**: Day/night cycles, weather, environmental effects
//!
//! # System Integration
//!
//! This module coordinates between:
//! - ECS for entity management
//! - Events for decoupled communication
//! - Pathfinding for movement planning
//! - Animation for visual feedback
//! - Serialization for persistence
//! - Debug tools for development
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::game_systems::*;
//! use tiles_tools::coordinates::square::{Coordinate, FourConnected};
//!
//! // Create a turn-based game manager
//! let mut game = TurnBasedGame::new();
//!
//! // Add players to the turn order
//! game.participant_add(1, 100); // entity_id: 1, initiative: 100
//! game.participant_add(2, 85);  // entity_id: 2, initiative: 85
//!
//! // Process a few turns
//! for _ in 0..3 {
//!     if let Some(current_entity) = game.current_turn() {
//!         println!("Entity {}'s turn", current_entity);
//!
//!         // Process actions for current entity
//!         game.turn_end();
//!     }
//! }
//! ```

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Duration, Instant};

/// Turn-based game manager for handling initiative, action points, and turn order.
pub struct TurnBasedGame {
  participants: BTreeMap<u32, TurnParticipant>,
  turn_order: VecDeque<u32>,
  current_turn_index: usize,
  round_number: u32,
  turn_time_limit: Option<Duration>,
  turn_start_time: Option<Instant>,
}

/// Participant in a turn-based game.
#[ derive( Debug, Clone ) ]
pub struct TurnParticipant
{
  /// Entity ID
  pub entity_id: u32,
  /// Initiative score (higher goes first)
  pub initiative: u32,
  /// Current action points
  pub action_points: u32,
  /// Maximum action points per turn
  pub max_action_points: u32,
  /// Whether this participant can act this turn
  pub can_act: bool,
  /// Status effects affecting this participant
  pub status_effects: Vec<StatusEffect>,
}

/// Status effect that can be applied to entities.
#[ derive( Debug, Clone ) ]
pub struct StatusEffect
{
  /// Unique identifier for the effect
  pub id: String,
  /// Human-readable name
  pub name: String,
  /// Effect description
  pub description: String,
  /// Remaining duration in turns
  pub duration: u32,
  /// Effect magnitude (context-dependent)
  pub magnitude: f32,
  /// Whether this is a beneficial effect
  pub is_beneficial: bool,
  /// Effect category for stacking rules
  pub category: EffectCategory,
}

/// Categories of status effects for stacking and interaction rules.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectCategory {
  /// Damage over time effects
  DamageOverTime,
  /// Healing over time effects
  HealingOverTime,
  /// Movement speed modifiers
  MovementSpeed,
  /// Attack power modifiers
  AttackPower,
  /// Defense modifiers
  Defense,
  /// Crowd control effects
  CrowdControl,
  /// Vision/detection modifiers
  Vision,
  /// Resource regeneration
  Regeneration,
  /// Custom effect category
  Custom(String),
}

impl TurnBasedGame
{
  /// Creates a new turn-based game manager.
  #[must_use]
  pub fn new() -> Self
  {
    Self
    {
      participants: BTreeMap::new(),
      turn_order: VecDeque::new(),
      current_turn_index: 0,
      round_number: 1,
      turn_time_limit: None,
      turn_start_time: None,
    }
  }

  /// Sets a time limit for each turn.
  #[must_use]
  pub fn with_turn_time_limit(mut self, duration: Duration) -> Self {
    self.turn_time_limit = Some(duration);
    self
  }

  /// Adds a participant to the game.
  pub fn participant_add(&mut self, entity_id: u32, initiative: u32) {
    let participant = TurnParticipant {
      entity_id,
      initiative,
      action_points: 3, // Default action points
      max_action_points: 3,
      can_act: true,
      status_effects: Vec::new(),
    };
    
    self.participants.insert(entity_id, participant);
    self.turn_order_rebuild();
  }

  /// Removes a participant from the game.
  pub fn participant_remove(&mut self, entity_id: u32) {
    self.participants.remove(&entity_id);
    self.turn_order_rebuild();
  }

  /// Gets the entity ID of the current turn.
  #[must_use]
  pub fn current_turn(&self) -> Option<u32> {
    if self.turn_order.is_empty() {
      return None;
    }
    
    let index = self.current_turn_index % self.turn_order.len();
    self.turn_order.get(index).copied()
  }

  /// Gets the current participant data.
  #[must_use]
  pub fn current_participant(&self) -> Option<&TurnParticipant> {
    self.current_turn().and_then(|id| self.participants.get(&id))
  }

  /// Gets mutable reference to current participant.
  pub fn current_participant_mut(&mut self) -> Option<&mut TurnParticipant> {
    if let Some(id) = self.current_turn() {
      self.participants.get_mut(&id)
    } else {
      None
    }
  }

  /// Ends the current turn and advances to the next participant.
  pub fn turn_end(&mut self) {
    if self.turn_order.is_empty() {
      return;
    }

    // Reset current participant's action points for next turn
    if let Some(participant) = self.current_participant_mut() {
      participant.action_points = participant.max_action_points;
    }

    self.current_turn_index += 1;
    
    // Check if we've completed a full round
    if self.current_turn_index >= self.turn_order.len() {
      self.round_number += 1;
      self.current_turn_index = 0;
      self.end_of_round_process();
    }

    self.turn_start_time = Some(Instant::now());
  }

  /// Spends action points for the current participant.
  pub fn action_points_spend(&mut self, cost: u32) -> bool {
    if let Some(participant) = self.current_participant_mut() {
      if participant.action_points >= cost {
        participant.action_points -= cost;
        true
      } else {
        false
      }
    } else {
      false
    }
  }

  /// Checks if the current turn has timed out.
  #[must_use]
  pub fn is_turn_timed_out(&self) -> bool {
    if let (Some(limit), Some(start)) = (self.turn_time_limit, self.turn_start_time) {
      start.elapsed() > limit
    } else {
      false
    }
  }

  /// Gets the current round number.
  #[must_use]
  pub fn round_number(&self) -> u32 {
    self.round_number
  }

  /// Applies a status effect to a participant.
  pub fn status_effect_apply(&mut self, entity_id: u32, effect: StatusEffect) {
    if let Some(participant) = self.participants.get_mut(&entity_id) {
      // Check for existing effects of the same category
      if let Some(existing_index) = participant.status_effects
        .iter()
        .position(|e| e.category == effect.category && e.id == effect.id) {
        // Replace or stack the effect
        participant.status_effects[existing_index] = effect;
      } else {
        participant.status_effects.push(effect);
      }
    }
  }

  /// Gets all participants in turn order.
  #[must_use]
  pub fn participants_in_order(&self) -> Vec<&TurnParticipant> {
    self.turn_order
      .iter()
      .filter_map(|&id| self.participants.get(&id))
      .collect()
  }

  fn turn_order_rebuild(&mut self) {
    let current_entity = self.turn_order.get(self.current_turn_index).copied();

    let mut participants: Vec<_> = self.participants.values().collect();
    participants.sort_by_key(|b| std::cmp::Reverse(b.initiative));

    self.turn_order = participants.into_iter()
      .map(|p| p.entity_id)
      .collect();

    // Fix(BUG-133)
    // Root cause: only clamped current_turn_index numerically against the
    // new turn_order's length, never remapped it to the same entity_id --
    // any participant_add/participant_remove call mid-round silently
    // reassigned "whose turn it is" to whichever entity happened to land on
    // that numeric slot after re-sorting, with no turn_end() call in between.
    // Pitfall: when the previously-current entity was itself removed, there
    // is no identity to preserve -- fall back to the same numeric clamp the
    // original code used unconditionally, so removing the acting entity
    // still advances play to whoever now occupies that slot instead of
    // panicking or stalling.
    if !self.turn_order.is_empty() {
      self.current_turn_index = current_entity
        .and_then(|id| self.turn_order.iter().position(|&e| e == id))
        .unwrap_or_else(|| self.current_turn_index.min(self.turn_order.len() - 1));
    }
  }

  fn end_of_round_process(&mut self) {
    // Process status effects for all participants
    for participant in self.participants.values_mut() {
      participant.status_effects.retain_mut(|effect| {
        effect.duration = effect.duration.saturating_sub(1);
        effect.duration > 0
      });
    }
  }
}

impl Default for TurnBasedGame {
  fn default() -> Self {
    Self::new()
  }
}

/// Game state machine for managing different phases of gameplay.
pub struct GameStateMachine {
  current_state: GameState,
  previous_state: Option<GameState>,
  state_data: HashMap<String, String>,
  transitions: HashMap<(GameState, GameStateEvent), GameState>,
  state_enter_handlers: HashMap<GameState, StateHandler>,
  state_exit_handlers: HashMap<GameState, StateHandler>,
}

/// Boxed handler invoked when the state machine enters or exits a state.
type StateHandler = Box<dyn Fn(&mut GameStateMachine)>;

/// Possible game states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
  /// Game is starting up
  Initialize,
  /// Main menu
  MainMenu,
  /// Loading a game or level
  Loading,
  /// Active gameplay
  Playing,
  /// Game is paused
  Paused,
  /// Player's turn in turn-based game
  PlayerTurn,
  /// AI's turn in turn-based game
  AITurn,
  /// Combat is occurring
  Combat,
  /// Showing cutscene or dialogue
  Cutscene,
  /// Game over state
  GameOver,
  /// Victory state
  Victory,
  /// Settings menu
  Settings,
  /// Inventory management
  Inventory,
}

/// Events that can trigger state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStateEvent {
  /// Game initialization complete
  InitComplete,
  /// Start new game
  StartGame,
  /// Load existing game
  LoadGame,
  /// Pause game
  Pause,
  /// Resume game
  Resume,
  /// Player action complete
  PlayerActionComplete,
  /// AI action complete
  AIActionComplete,
  /// Enter combat
  EnterCombat,
  /// Exit combat
  ExitCombat,
  /// Show cutscene
  ShowCutscene,
  /// Cutscene complete
  CutsceneComplete,
  /// Player defeated
  PlayerDefeated,
  /// Victory achieved
  VictoryAchieved,
  /// Open settings
  OpenSettings,
  /// Close settings
  CloseSettings,
  /// Open inventory
  OpenInventory,
  /// Close inventory
  CloseInventory,
  /// Return to menu
  ReturnToMenu,
  /// Quit game
  QuitGame,
}

impl GameStateMachine {
  /// Creates a new game state machine.
  #[must_use]
  pub fn new(initial_state: GameState) -> Self {
    let mut machine = Self {
      current_state: initial_state,
      previous_state: None,
      state_data: HashMap::new(),
      transitions: HashMap::new(),
      state_enter_handlers: HashMap::new(),
      state_exit_handlers: HashMap::new(),
    };

    machine.default_transitions_setup();
    machine
  }

  /// Gets the current state.
  #[must_use]
  pub fn current_state(&self) -> GameState {
    self.current_state
  }

  /// Gets the previous state.
  #[must_use]
  pub fn previous_state(&self) -> Option<GameState> {
    self.previous_state
  }

  /// Adds a state transition rule.
  pub fn transition_add(&mut self, from: GameState, event: GameStateEvent, to: GameState) {
    self.transitions.insert((from, event), to);
  }

  /// Processes a state event and potentially transitions to a new state.
  pub fn event_process(&mut self, event: GameStateEvent) -> bool {
    if let Some(&new_state) = self.transitions.get(&(self.current_state, event)) {
      self.transition_to(new_state);
      true
    } else {
      false
    }
  }

  /// Forces a transition to a specific state.
  pub fn transition_to(&mut self, new_state: GameState) {
    let old_state = self.current_state;
    
    // Call exit handler for current state
    if let Some(_handler) = self.state_exit_handlers.get(&old_state) {
      // Note: Can't call handler directly due to borrowing issues
      // In a real implementation, would use a different pattern
    }

    self.previous_state = Some(old_state);
    self.current_state = new_state;

    // Call enter handler for new state
    if let Some(_handler) = self.state_enter_handlers.get(&new_state) {
      // Note: Can't call handler directly due to borrowing issues
      // In a real implementation, would use a different pattern
    }
  }

  /// Sets data associated with the current state.
  pub fn state_data_set(&mut self, key: String, value: String) {
    self.state_data.insert(key, value);
  }

  /// Gets data associated with the current state.
  #[must_use]
  pub fn state_data_get(&self, key: &str) -> Option<&String> {
    self.state_data.get(key)
  }

  /// Checks if the machine can transition on the given event.
  #[must_use]
  pub fn can_transition(&self, event: GameStateEvent) -> bool {
    self.transitions.contains_key(&(self.current_state, event))
  }

  fn default_transitions_setup(&mut self) {
    // Initialize -> MainMenu
    self.transition_add(GameState::Initialize, GameStateEvent::InitComplete, GameState::MainMenu);
    
    // MainMenu transitions
    self.transition_add(GameState::MainMenu, GameStateEvent::StartGame, GameState::Loading);
    self.transition_add(GameState::MainMenu, GameStateEvent::LoadGame, GameState::Loading);
    self.transition_add(GameState::MainMenu, GameStateEvent::OpenSettings, GameState::Settings);
    
    // Loading -> Playing
    self.transition_add(GameState::Loading, GameStateEvent::StartGame, GameState::Playing);
    
    // Playing state transitions
    self.transition_add(GameState::Playing, GameStateEvent::Pause, GameState::Paused);
    self.transition_add(GameState::Playing, GameStateEvent::EnterCombat, GameState::Combat);
    self.transition_add(GameState::Playing, GameStateEvent::OpenInventory, GameState::Inventory);
    self.transition_add(GameState::Playing, GameStateEvent::PlayerDefeated, GameState::GameOver);
    self.transition_add(GameState::Playing, GameStateEvent::VictoryAchieved, GameState::Victory);
    
    // Paused -> Playing
    self.transition_add(GameState::Paused, GameStateEvent::Resume, GameState::Playing);
    self.transition_add(GameState::Paused, GameStateEvent::ReturnToMenu, GameState::MainMenu);
    
    // Combat transitions
    self.transition_add(GameState::Combat, GameStateEvent::ExitCombat, GameState::Playing);
    self.transition_add(GameState::Combat, GameStateEvent::PlayerDefeated, GameState::GameOver);
    self.transition_add(GameState::Combat, GameStateEvent::VictoryAchieved, GameState::Victory);
    
    // Settings -> MainMenu (or previous)
    self.transition_add(GameState::Settings, GameStateEvent::CloseSettings, GameState::MainMenu);
    
    // Inventory -> Playing
    self.transition_add(GameState::Inventory, GameStateEvent::CloseInventory, GameState::Playing);
    
    // End states
    self.transition_add(GameState::GameOver, GameStateEvent::ReturnToMenu, GameState::MainMenu);
    self.transition_add(GameState::Victory, GameStateEvent::ReturnToMenu, GameState::MainMenu);
  }
}

/// Resource management system for tracking health, mana, items, etc.
pub struct ResourceManager {
  resources: HashMap<u32, EntityResources>,
}

/// Resources associated with a single entity.
#[derive(Debug, Clone)]
pub struct EntityResources {
  /// Entity ID
  pub entity_id: u32,
  /// Health points
  pub health: Resource,
  /// Mana/energy points
  pub mana: Resource,
  /// Experience points
  pub experience: u64,
  /// Level
  pub level: u32,
  /// Currency/gold
  pub currency: u64,
  /// Custom resources
  pub custom: HashMap<String, f32>,
}

/// A resource with current and maximum values.
#[derive(Debug, Clone)]
pub struct Resource {
  /// Current value
  pub current: f32,
  /// Maximum value
  pub maximum: f32,
  /// Regeneration rate per second
  pub regeneration: f32,
}

impl Resource {
  /// Creates a new resource with the given maximum value.
  #[must_use]
  pub fn new(maximum: f32) -> Self {
    // Fix(BUG-349): clamp maximum to a non-negative value, matching the
    // invariant maximum_set already enforces (`self.maximum = value.max(0.0)`).
    // Root cause: modify/current_set both call `.clamp(0.0, self.maximum)`,
    // and f32::clamp asserts `min <= max` unconditionally -- a negative
    // maximum stored here made every later modify/current_set call panic.
    // Pitfall: a sibling setter (maximum_set) enforcing an invariant
    // correctly is not evidence every value-producing path (new,
    // with_regeneration) enforces the same invariant -- check each one.
    let maximum = maximum.max(0.0);
    Self {
      current: maximum,
      maximum,
      regeneration: 0.0,
    }
  }

  /// Creates a resource with regeneration.
  #[must_use]
  pub fn with_regeneration(maximum: f32, regeneration: f32) -> Self {
    // Fix(BUG-349): see `Resource::new` -- same clamp, same root cause.
    let maximum = maximum.max(0.0);
    Self {
      current: maximum,
      maximum,
      regeneration,
    }
  }

  /// Gets the current value as a percentage of maximum.
  #[must_use]
  pub fn percentage(&self) -> f32 {
    if self.maximum > 0.0 {
      (self.current / self.maximum).clamp(0.0, 1.0)
    } else {
      0.0
    }
  }

  /// Modifies the current value by the given amount.
  pub fn modify(&mut self, amount: f32) {
    self.current = (self.current + amount).clamp(0.0, self.maximum);
  }

  /// Sets the current value directly.
  pub fn current_set(&mut self, value: f32) {
    self.current = value.clamp(0.0, self.maximum);
  }

  /// Sets the maximum value and adjusts current if needed.
  pub fn maximum_set(&mut self, value: f32) {
    self.maximum = value.max(0.0);
    self.current = self.current.min(self.maximum);
  }

  /// Updates the resource with regeneration over time.
  pub fn update(&mut self, delta_time: f32) {
    if self.regeneration != 0.0 {
      self.modify(self.regeneration * delta_time);
    }
  }

  /// Checks if the resource is depleted.
  #[must_use]
  pub fn is_depleted(&self) -> bool {
    self.current <= 0.0
  }

  /// Checks if the resource is at maximum.
  #[must_use]
  pub fn is_full(&self) -> bool {
    (self.current - self.maximum).abs() < f32::EPSILON
  }
}

impl ResourceManager {
  /// Creates a new resource manager.
  #[must_use]
  pub fn new() -> Self {
    Self {
      resources: HashMap::new(),
    }
  }

  /// Adds resources for an entity.
  pub fn entity_add(&mut self, entity_id: u32, health: f32, mana: f32) {
    let resources = EntityResources {
      entity_id,
      health: Resource::new(health),
      mana: Resource::new(mana),
      experience: 0,
      level: 1,
      currency: 0,
      custom: HashMap::new(),
    };
    self.resources.insert(entity_id, resources);
  }

  /// Removes resources for an entity.
  pub fn entity_remove(&mut self, entity_id: u32) {
    self.resources.remove(&entity_id);
  }

  /// Gets resources for an entity.
  #[must_use]
  pub fn resources_get(&self, entity_id: u32) -> Option<&EntityResources> {
    self.resources.get(&entity_id)
  }

  /// Gets mutable resources for an entity.
  pub fn resources_get_mut(&mut self, entity_id: u32) -> Option<&mut EntityResources> {
    self.resources.get_mut(&entity_id)
  }

  /// Modifies health for an entity.
  pub fn health_modify(&mut self, entity_id: u32, amount: f32) -> bool {
    if let Some(resources) = self.resources.get_mut(&entity_id) {
      resources.health.modify(amount);
      true
    } else {
      false
    }
  }

  /// Modifies mana for an entity.
  pub fn mana_modify(&mut self, entity_id: u32, amount: f32) -> bool {
    if let Some(resources) = self.resources.get_mut(&entity_id) {
      resources.mana.modify(amount);
      true
    } else {
      false
    }
  }

  /// Updates all resources with regeneration.
  pub fn update_all(&mut self, delta_time: f32) {
    for resources in self.resources.values_mut() {
      resources.health.update(delta_time);
      resources.mana.update(delta_time);
    }
  }

  /// Gets all entities with depleted health.
  #[must_use]
  pub fn defeated_entities_get(&self) -> Vec<u32> {
    self.resources
      .iter()
      .filter(|(_, r)| r.health.is_depleted())
      .map(|(&id, _)| id)
      .collect()
  }
}

impl Default for ResourceManager {
  fn default() -> Self {
    Self::new()
  }
}

/// Quest and objective management system.
pub struct QuestManager {
  quests: HashMap<String, Quest>,
  active_quests: Vec<String>,
  completed_quests: Vec<String>,
  global_flags: HashMap<String, bool>,
}

/// A quest with objectives and branching logic.
#[derive(Debug, Clone)]
pub struct Quest {
  /// Unique quest identifier
  pub id: String,
  /// Display name
  pub name: String,
  /// Quest description
  pub description: String,
  /// Current quest status
  pub status: QuestStatus,
  /// Quest objectives
  pub objectives: Vec<QuestObjective>,
  /// Prerequisites to start this quest
  pub prerequisites: Vec<QuestCondition>,
  /// Rewards for completing the quest
  pub rewards: Vec<QuestReward>,
  /// Custom quest data
  pub data: HashMap<String, String>,
}

/// Quest completion status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuestStatus {
  /// Quest is not yet available
  Locked,
  /// Quest is available but not started
  Available,
  /// Quest is in progress
  Active,
  /// Quest is completed
  Completed,
  /// Quest failed
  Failed,
}

/// Individual quest objective.
#[derive(Debug, Clone)]
pub struct QuestObjective {
  /// Objective identifier
  pub id: String,
  /// Description of what to do
  pub description: String,
  /// Whether this objective is completed
  pub completed: bool,
  /// Objective type and parameters
  pub objective_type: ObjectiveType,
  /// Whether this objective is optional
  pub optional: bool,
}

/// Types of quest objectives.
#[derive(Debug, Clone)]
pub enum ObjectiveType {
  /// Kill specific entities
  KillTargets {
    /// Entity type that must be killed.
    target_type: String,
    /// Total kills required.
    count: u32,
    /// Kills achieved so far.
    current: u32,
  },
  /// Reach a specific location
  ReachLocation {
    /// Target x coordinate.
    x: i32,
    /// Target y coordinate.
    y: i32,
    /// Acceptance radius around the target.
    radius: u32,
  },
  /// Collect specific items
  CollectItems {
    /// Identifier of the item to collect.
    item_id: String,
    /// Total items required.
    count: u32,
    /// Items collected so far.
    current: u32,
  },
  /// Talk to specific NPCs
  TalkToNPC {
    /// Identifier of the NPC to talk to.
    npc_id: u32,
  },
  /// Survive for a duration
  Survive {
    /// How long to survive, in seconds.
    duration_seconds: u32,
  },
  /// Custom objective
  Custom {
    /// Free-form objective parameters.
    data: HashMap<String, String>,
  },
}

/// Conditions for quest availability.
#[derive(Debug, Clone)]
pub enum QuestCondition {
  /// Player must be at least this level
  MinLevel(u32),
  /// Another quest must be completed
  QuestCompleted(String),
  /// A global flag must be set
  FlagSet(String),
  /// Player must have specific items
  HasItems(String, u32),
}

/// Quest completion rewards.
#[derive(Debug, Clone)]
pub enum QuestReward {
  /// Experience points
  Experience(u64),
  /// Currency/gold
  Currency(u64),
  /// Specific items
  Items(String, u32),
  /// Unlock new quest
  UnlockQuest(String),
  /// Set global flag
  SetFlag(String),
}

impl QuestManager {
  /// Creates a new quest manager.
  #[must_use]
  pub fn new() -> Self {
    Self {
      quests: HashMap::new(),
      active_quests: Vec::new(),
      completed_quests: Vec::new(),
      global_flags: HashMap::new(),
    }
  }

  /// Adds a quest to the manager.
  pub fn quest_add(&mut self, quest: Quest) {
    self.quests.insert(quest.id.clone(), quest);
  }

  /// Starts a quest if prerequisites are met.
  pub fn quest_start(&mut self, quest_id: &str, player_level: u32) -> bool {
    // Check prerequisites first without holding a mutable reference
    let can_start = if let Some(quest) = self.quests.get(quest_id) {
      quest.status == QuestStatus::Available &&
      self.prerequisites_check(&quest.prerequisites, player_level)
    } else {
      false
    };
    
    if can_start {
      if let Some(quest) = self.quests.get_mut(quest_id) {
        quest.status = QuestStatus::Active;
        self.active_quests.push(quest_id.to_string());
        return true;
      }
    }
    false
  }

  /// Completes a quest and awards rewards.
  pub fn quest_complete(&mut self, quest_id: &str) -> Vec<QuestReward> {
    if let Some(quest) = self.quests.get_mut(quest_id) {
      if quest.status == QuestStatus::Active {
        quest.status = QuestStatus::Completed;
        
        // Remove from active and add to completed
        self.active_quests.retain(|id| id != quest_id);
        self.completed_quests.push(quest_id.to_string());
        
        return quest.rewards.clone();
      }
    }
    Vec::new()
  }

  /// Updates quest objectives based on game events.
  pub fn objective_update(&mut self, quest_id: &str, objective_id: &str, progress: u32) {
    if let Some(quest) = self.quests.get_mut(quest_id) {
      if quest.status == QuestStatus::Active {
        for objective in &mut quest.objectives {
          if objective.id == objective_id {
            match &mut objective.objective_type {
              ObjectiveType::KillTargets { count, current, .. }
              | ObjectiveType::CollectItems { count, current, .. } => {
                *current = (*current + progress).min(*count);
                objective.completed = *current >= *count;
              },
              _ => {}
            }
          }
        }
        
        // Check if all required objectives are complete
        let all_required_complete = quest.objectives
          .iter()
          .filter(|obj| !obj.optional)
          .all(|obj| obj.completed);
        
        if all_required_complete {
          self.quest_complete(quest_id);
        }
      }
    }
  }

  /// Sets a global flag.
  pub fn flag_set(&mut self, flag: String, value: bool) {
    self.global_flags.insert(flag, value);
  }

  /// Gets a global flag value.
  #[must_use]
  pub fn flag_get(&self, flag: &str) -> bool {
    self.global_flags.get(flag).copied().unwrap_or(false)
  }

  /// Gets all active quests.
  #[must_use]
  pub fn active_quests(&self) -> Vec<&Quest> {
    self.active_quests
      .iter()
      .filter_map(|id| self.quests.get(id))
      .collect()
  }

  /// Gets all completed quests.
  #[must_use]
  pub fn completed_quests(&self) -> Vec<&Quest> {
    self.completed_quests
      .iter()
      .filter_map(|id| self.quests.get(id))
      .collect()
  }

  /// Gets the number of completed quests.
  #[must_use]
  pub fn completed_quest_count(&self) -> usize {
    self.completed_quests.len()
  }

  /// Checks if a quest is completed.
  #[must_use]
  pub fn is_quest_completed(&self, quest_id: &str) -> bool {
    self.completed_quests.contains(&quest_id.to_string())
  }

  fn prerequisites_check(&self, prerequisites: &[QuestCondition], player_level: u32) -> bool {
    prerequisites.iter().all(|condition| {
      match condition {
        QuestCondition::MinLevel(level) => player_level >= *level,
        QuestCondition::QuestCompleted(quest_id) => {
          self.completed_quests.contains(quest_id)
        },
        QuestCondition::FlagSet(flag) => self.flag_get(flag),
        QuestCondition::HasItems(_, _) => true, // Simplified for this example
      }
    })
  }
}

impl Default for QuestManager {
  fn default() -> Self {
    Self::new()
  }
}

/// Game events for system integration.
#[derive(Debug, Clone)]
pub struct TurnStartedEvent {
  /// Entity whose turn started.
  pub entity_id: u32,
  /// Current round number.
  pub round_number: u32,
  /// Action points available this turn.
  pub action_points: u32,
}

/// Event fired when an entity's turn ends.
#[derive(Debug, Clone)]
pub struct TurnEndedEvent {
  /// Entity whose turn ended.
  pub entity_id: u32,
  /// Actions the entity took during the turn.
  pub actions_taken: u32,
}

/// Event fired when an entity's resource amount changes.
#[derive(Debug, Clone)]
pub struct ResourceChangedEvent {
  /// Entity whose resource changed.
  pub entity_id: u32,
  /// Which resource changed.
  pub resource_type: String,
  /// Amount before the change.
  pub old_value: f32,
  /// Amount after the change.
  pub new_value: f32,
}

/// Event fired when a quest is completed.
#[derive(Debug, Clone)]
pub struct QuestCompletedEvent {
  /// Identifier of the completed quest.
  pub quest_id: String,
  /// Rewards granted on completion.
  pub rewards: Vec<QuestReward>,
}

// Event implementations are automatically provided by the blanket impl in events.rs
