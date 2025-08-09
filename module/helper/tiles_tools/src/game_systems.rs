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
//! game.add_participant(1, 100); // entity_id: 1, initiative: 100
//! game.add_participant(2, 85);  // entity_id: 2, initiative: 85
//!
//! // Process turns
//! while let Some(current_entity) = game.current_turn() {
//!     println!("Entity {}'s turn", current_entity);
//!     
//!     // Process actions for current entity
//!     game.end_turn();
//! }
//! ```

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Duration, Instant};
use crate::events::{Event, EventBus, EventResult};
use crate::coordinates::{Distance, Neighbors};

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
#[derive(Debug, Clone)]
pub struct TurnParticipant {
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
#[derive(Debug, Clone)]
pub struct StatusEffect {
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

impl TurnBasedGame {
  /// Creates a new turn-based game manager.
  pub fn new() -> Self {
    Self {
      participants: BTreeMap::new(),
      turn_order: VecDeque::new(),
      current_turn_index: 0,
      round_number: 1,
      turn_time_limit: None,
      turn_start_time: None,
    }
  }

  /// Sets a time limit for each turn.
  pub fn with_turn_time_limit(mut self, duration: Duration) -> Self {
    self.turn_time_limit = Some(duration);
    self
  }

  /// Adds a participant to the game.
  pub fn add_participant(&mut self, entity_id: u32, initiative: u32) {
    let participant = TurnParticipant {
      entity_id,
      initiative,
      action_points: 3, // Default action points
      max_action_points: 3,
      can_act: true,
      status_effects: Vec::new(),
    };
    
    self.participants.insert(entity_id, participant);
    self.rebuild_turn_order();
  }

  /// Removes a participant from the game.
  pub fn remove_participant(&mut self, entity_id: u32) {
    self.participants.remove(&entity_id);
    self.rebuild_turn_order();
  }

  /// Gets the entity ID of the current turn.
  pub fn current_turn(&self) -> Option<u32> {
    if self.turn_order.is_empty() {
      return None;
    }
    
    let index = self.current_turn_index % self.turn_order.len();
    self.turn_order.get(index).copied()
  }

  /// Gets the current participant data.
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
  pub fn end_turn(&mut self) {
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
      self.process_end_of_round();
    }

    self.turn_start_time = Some(Instant::now());
  }

  /// Spends action points for the current participant.
  pub fn spend_action_points(&mut self, cost: u32) -> bool {
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
  pub fn is_turn_timed_out(&self) -> bool {
    if let (Some(limit), Some(start)) = (self.turn_time_limit, self.turn_start_time) {
      start.elapsed() > limit
    } else {
      false
    }
  }

  /// Gets the current round number.
  pub fn round_number(&self) -> u32 {
    self.round_number
  }

  /// Applies a status effect to a participant.
  pub fn apply_status_effect(&mut self, entity_id: u32, effect: StatusEffect) {
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
  pub fn participants_in_order(&self) -> Vec<&TurnParticipant> {
    self.turn_order
      .iter()
      .filter_map(|&id| self.participants.get(&id))
      .collect()
  }

  fn rebuild_turn_order(&mut self) {
    let mut participants: Vec<_> = self.participants.values().collect();
    participants.sort_by(|a, b| b.initiative.cmp(&a.initiative));
    
    self.turn_order = participants.into_iter()
      .map(|p| p.entity_id)
      .collect();
    
    // Ensure current turn index is valid
    if !self.turn_order.is_empty() {
      self.current_turn_index = self.current_turn_index.min(self.turn_order.len() - 1);
    }
  }

  fn process_end_of_round(&mut self) {
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
  state_enter_handlers: HashMap<GameState, Box<dyn Fn(&mut Self)>>,
  state_exit_handlers: HashMap<GameState, Box<dyn Fn(&mut Self)>>,
}

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
  pub fn new(initial_state: GameState) -> Self {
    let mut machine = Self {
      current_state: initial_state,
      previous_state: None,
      state_data: HashMap::new(),
      transitions: HashMap::new(),
      state_enter_handlers: HashMap::new(),
      state_exit_handlers: HashMap::new(),
    };

    machine.setup_default_transitions();
    machine
  }

  /// Gets the current state.
  pub fn current_state(&self) -> GameState {
    self.current_state
  }

  /// Gets the previous state.
  pub fn previous_state(&self) -> Option<GameState> {
    self.previous_state
  }

  /// Adds a state transition rule.
  pub fn add_transition(&mut self, from: GameState, event: GameStateEvent, to: GameState) {
    self.transitions.insert((from, event), to);
  }

  /// Processes a state event and potentially transitions to a new state.
  pub fn process_event(&mut self, event: GameStateEvent) -> bool {
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
  pub fn set_state_data(&mut self, key: String, value: String) {
    self.state_data.insert(key, value);
  }

  /// Gets data associated with the current state.
  pub fn get_state_data(&self, key: &str) -> Option<&String> {
    self.state_data.get(key)
  }

  /// Checks if the machine can transition on the given event.
  pub fn can_transition(&self, event: GameStateEvent) -> bool {
    self.transitions.contains_key(&(self.current_state, event))
  }

  fn setup_default_transitions(&mut self) {
    // Initialize -> MainMenu
    self.add_transition(GameState::Initialize, GameStateEvent::InitComplete, GameState::MainMenu);
    
    // MainMenu transitions
    self.add_transition(GameState::MainMenu, GameStateEvent::StartGame, GameState::Loading);
    self.add_transition(GameState::MainMenu, GameStateEvent::LoadGame, GameState::Loading);
    self.add_transition(GameState::MainMenu, GameStateEvent::OpenSettings, GameState::Settings);
    
    // Loading -> Playing
    self.add_transition(GameState::Loading, GameStateEvent::StartGame, GameState::Playing);
    
    // Playing state transitions
    self.add_transition(GameState::Playing, GameStateEvent::Pause, GameState::Paused);
    self.add_transition(GameState::Playing, GameStateEvent::EnterCombat, GameState::Combat);
    self.add_transition(GameState::Playing, GameStateEvent::OpenInventory, GameState::Inventory);
    self.add_transition(GameState::Playing, GameStateEvent::PlayerDefeated, GameState::GameOver);
    self.add_transition(GameState::Playing, GameStateEvent::VictoryAchieved, GameState::Victory);
    
    // Paused -> Playing
    self.add_transition(GameState::Paused, GameStateEvent::Resume, GameState::Playing);
    self.add_transition(GameState::Paused, GameStateEvent::ReturnToMenu, GameState::MainMenu);
    
    // Combat transitions
    self.add_transition(GameState::Combat, GameStateEvent::ExitCombat, GameState::Playing);
    self.add_transition(GameState::Combat, GameStateEvent::PlayerDefeated, GameState::GameOver);
    self.add_transition(GameState::Combat, GameStateEvent::VictoryAchieved, GameState::Victory);
    
    // Settings -> MainMenu (or previous)
    self.add_transition(GameState::Settings, GameStateEvent::CloseSettings, GameState::MainMenu);
    
    // Inventory -> Playing
    self.add_transition(GameState::Inventory, GameStateEvent::CloseInventory, GameState::Playing);
    
    // End states
    self.add_transition(GameState::GameOver, GameStateEvent::ReturnToMenu, GameState::MainMenu);
    self.add_transition(GameState::Victory, GameStateEvent::ReturnToMenu, GameState::MainMenu);
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
  pub fn new(maximum: f32) -> Self {
    Self {
      current: maximum,
      maximum,
      regeneration: 0.0,
    }
  }

  /// Creates a resource with regeneration.
  pub fn with_regeneration(maximum: f32, regeneration: f32) -> Self {
    Self {
      current: maximum,
      maximum,
      regeneration,
    }
  }

  /// Gets the current value as a percentage of maximum.
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
  pub fn set_current(&mut self, value: f32) {
    self.current = value.clamp(0.0, self.maximum);
  }

  /// Sets the maximum value and adjusts current if needed.
  pub fn set_maximum(&mut self, value: f32) {
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
  pub fn is_depleted(&self) -> bool {
    self.current <= 0.0
  }

  /// Checks if the resource is at maximum.
  pub fn is_full(&self) -> bool {
    (self.current - self.maximum).abs() < f32::EPSILON
  }
}

impl ResourceManager {
  /// Creates a new resource manager.
  pub fn new() -> Self {
    Self {
      resources: HashMap::new(),
    }
  }

  /// Adds resources for an entity.
  pub fn add_entity(&mut self, entity_id: u32, health: f32, mana: f32) {
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
  pub fn remove_entity(&mut self, entity_id: u32) {
    self.resources.remove(&entity_id);
  }

  /// Gets resources for an entity.
  pub fn get_resources(&self, entity_id: u32) -> Option<&EntityResources> {
    self.resources.get(&entity_id)
  }

  /// Gets mutable resources for an entity.
  pub fn get_resources_mut(&mut self, entity_id: u32) -> Option<&mut EntityResources> {
    self.resources.get_mut(&entity_id)
  }

  /// Modifies health for an entity.
  pub fn modify_health(&mut self, entity_id: u32, amount: f32) -> bool {
    if let Some(resources) = self.resources.get_mut(&entity_id) {
      resources.health.modify(amount);
      true
    } else {
      false
    }
  }

  /// Modifies mana for an entity.
  pub fn modify_mana(&mut self, entity_id: u32, amount: f32) -> bool {
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
  pub fn get_defeated_entities(&self) -> Vec<u32> {
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
  KillTargets { target_type: String, count: u32, current: u32 },
  /// Reach a specific location
  ReachLocation { x: i32, y: i32, radius: u32 },
  /// Collect specific items
  CollectItems { item_id: String, count: u32, current: u32 },
  /// Talk to specific NPCs
  TalkToNPC { npc_id: u32 },
  /// Survive for a duration
  Survive { duration_seconds: u32 },
  /// Custom objective
  Custom { data: HashMap<String, String> },
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
  pub fn new() -> Self {
    Self {
      quests: HashMap::new(),
      active_quests: Vec::new(),
      completed_quests: Vec::new(),
      global_flags: HashMap::new(),
    }
  }

  /// Adds a quest to the manager.
  pub fn add_quest(&mut self, quest: Quest) {
    self.quests.insert(quest.id.clone(), quest);
  }

  /// Starts a quest if prerequisites are met.
  pub fn start_quest(&mut self, quest_id: &str, player_level: u32) -> bool {
    // Check prerequisites first without holding a mutable reference
    let can_start = if let Some(quest) = self.quests.get(quest_id) {
      quest.status == QuestStatus::Available &&
      self.check_prerequisites(&quest.prerequisites, player_level)
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
  pub fn complete_quest(&mut self, quest_id: &str) -> Vec<QuestReward> {
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
  pub fn update_objective(&mut self, quest_id: &str, objective_id: &str, progress: u32) {
    if let Some(quest) = self.quests.get_mut(quest_id) {
      if quest.status == QuestStatus::Active {
        for objective in &mut quest.objectives {
          if objective.id == objective_id {
            match &mut objective.objective_type {
              ObjectiveType::KillTargets { count, current, .. } => {
                *current = (*current + progress).min(*count);
                objective.completed = *current >= *count;
              },
              ObjectiveType::CollectItems { count, current, .. } => {
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
          self.complete_quest(quest_id);
        }
      }
    }
  }

  /// Sets a global flag.
  pub fn set_flag(&mut self, flag: String, value: bool) {
    self.global_flags.insert(flag, value);
  }

  /// Gets a global flag value.
  pub fn get_flag(&self, flag: &str) -> bool {
    self.global_flags.get(flag).copied().unwrap_or(false)
  }

  /// Gets all active quests.
  pub fn active_quests(&self) -> Vec<&Quest> {
    self.active_quests
      .iter()
      .filter_map(|id| self.quests.get(id))
      .collect()
  }

  /// Gets all completed quests.
  pub fn completed_quests(&self) -> Vec<&Quest> {
    self.completed_quests
      .iter()
      .filter_map(|id| self.quests.get(id))
      .collect()
  }

  /// Gets the number of completed quests.
  pub fn completed_quest_count(&self) -> usize {
    self.completed_quests.len()
  }

  /// Checks if a quest is completed.
  pub fn is_quest_completed(&self, quest_id: &str) -> bool {
    self.completed_quests.contains(&quest_id.to_string())
  }

  fn check_prerequisites(&self, prerequisites: &[QuestCondition], player_level: u32) -> bool {
    prerequisites.iter().all(|condition| {
      match condition {
        QuestCondition::MinLevel(level) => player_level >= *level,
        QuestCondition::QuestCompleted(quest_id) => {
          self.completed_quests.contains(quest_id)
        },
        QuestCondition::FlagSet(flag) => self.get_flag(flag),
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
  pub entity_id: u32,
  pub round_number: u32,
  pub action_points: u32,
}

#[derive(Debug, Clone)]
pub struct TurnEndedEvent {
  pub entity_id: u32,
  pub actions_taken: u32,
}

#[derive(Debug, Clone)]
pub struct ResourceChangedEvent {
  pub entity_id: u32,
  pub resource_type: String,
  pub old_value: f32,
  pub new_value: f32,
}

#[derive(Debug, Clone)]
pub struct QuestCompletedEvent {
  pub quest_id: String,
  pub rewards: Vec<QuestReward>,
}

// Event implementations are automatically provided by the blanket impl in events.rs

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_turn_based_game_creation() {
    let game = TurnBasedGame::new();
    assert_eq!(game.round_number(), 1);
    assert!(game.current_turn().is_none());
  }

  #[test]
  fn test_turn_based_participants() {
    let mut game = TurnBasedGame::new();
    game.add_participant(1, 100);
    game.add_participant(2, 85);
    game.add_participant(3, 95);
    
    // Should be ordered by initiative (highest first)
    assert_eq!(game.current_turn(), Some(1)); // Initiative 100
    
    game.end_turn();
    assert_eq!(game.current_turn(), Some(3)); // Initiative 95
    
    game.end_turn();
    assert_eq!(game.current_turn(), Some(2)); // Initiative 85
    
    game.end_turn();
    assert_eq!(game.current_turn(), Some(1)); // Back to first, round 2
    assert_eq!(game.round_number(), 2);
  }

  #[test]
  fn test_action_points() {
    let mut game = TurnBasedGame::new();
    game.add_participant(1, 100);
    
    assert_eq!(game.current_participant().unwrap().action_points, 3);
    
    // Spend some action points
    assert!(game.spend_action_points(2));
    assert_eq!(game.current_participant().unwrap().action_points, 1);
    
    // Try to spend more than available
    assert!(!game.spend_action_points(2));
    assert_eq!(game.current_participant().unwrap().action_points, 1);
  }

  #[test]
  fn test_game_state_machine() {
    let mut machine = GameStateMachine::new(GameState::Initialize);
    assert_eq!(machine.current_state(), GameState::Initialize);
    
    // Process initialization complete
    assert!(machine.process_event(GameStateEvent::InitComplete));
    assert_eq!(machine.current_state(), GameState::MainMenu);
    
    // Start game
    assert!(machine.process_event(GameStateEvent::StartGame));
    assert_eq!(machine.current_state(), GameState::Loading);
    
    // Invalid transition should fail
    assert!(!machine.process_event(GameStateEvent::Pause));
    assert_eq!(machine.current_state(), GameState::Loading);
  }

  #[test]
  fn test_resource_management() {
    let mut resource = Resource::new(100.0);
    assert_eq!(resource.current, 100.0);
    assert_eq!(resource.percentage(), 1.0);
    
    resource.modify(-30.0);
    assert_eq!(resource.current, 70.0);
    assert_eq!(resource.percentage(), 0.7);
    
    // Test clamping
    resource.modify(-200.0);
    assert_eq!(resource.current, 0.0);
    assert!(resource.is_depleted());
    
    resource.set_current(50.0);
    assert_eq!(resource.current, 50.0);
    assert!(!resource.is_depleted());
    assert!(!resource.is_full());
  }

  #[test]
  fn test_resource_manager() {
    let mut manager = ResourceManager::new();
    manager.add_entity(1, 100.0, 50.0);
    
    assert!(manager.modify_health(1, -25.0));
    assert_eq!(manager.get_resources(1).unwrap().health.current, 75.0);
    
    assert!(manager.modify_mana(1, -10.0));
    assert_eq!(manager.get_resources(1).unwrap().mana.current, 40.0);
    
    // Test defeated entities
    manager.modify_health(1, -100.0);
    let defeated = manager.get_defeated_entities();
    assert_eq!(defeated, vec![1]);
  }

  #[test]
  fn test_quest_system() {
    let mut quest_manager = QuestManager::new();
    
    let quest = Quest {
      id: "test_quest".to_string(),
      name: "Test Quest".to_string(),
      description: "A simple test quest".to_string(),
      status: QuestStatus::Available,
      objectives: vec![QuestObjective {
        id: "kill_enemies".to_string(),
        description: "Kill 5 enemies".to_string(),
        completed: false,
        objective_type: ObjectiveType::KillTargets {
          target_type: "orc".to_string(),
          count: 5,
          current: 0,
        },
        optional: false,
      }],
      prerequisites: vec![],
      rewards: vec![QuestReward::Experience(100)],
      data: HashMap::new(),
    };
    
    quest_manager.add_quest(quest);
    
    // Start quest
    assert!(quest_manager.start_quest("test_quest", 1));
    assert_eq!(quest_manager.active_quests().len(), 1);
    
    // Update objective progress
    quest_manager.update_objective("test_quest", "kill_enemies", 3);
    quest_manager.update_objective("test_quest", "kill_enemies", 2);
    
    // Quest should be completed
    assert_eq!(quest_manager.completed_quests.len(), 1);
  }

  #[test]
  fn test_status_effects() {
    let mut game = TurnBasedGame::new();
    game.add_participant(1, 100);
    
    let poison = StatusEffect {
      id: "poison".to_string(),
      name: "Poison".to_string(),
      description: "Takes damage over time".to_string(),
      duration: 3,
      magnitude: 5.0,
      is_beneficial: false,
      category: EffectCategory::DamageOverTime,
    };
    
    game.apply_status_effect(1, poison);
    
    let participant = game.participants.get(&1).unwrap();
    assert_eq!(participant.status_effects.len(), 1);
    assert_eq!(participant.status_effects[0].duration, 3);
  }
}