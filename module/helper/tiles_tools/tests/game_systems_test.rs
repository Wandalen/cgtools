//! Tests for the `game_systems` module — turn management, state machine, resources,
//! quests, and status effects driven purely through the public surface.
//!
//! Relocated from `src/game_systems.rs` by task 072. Two tests were losslessly
//! rewritten from private-field reads onto existing public accessors:
//! `completed_quests` field → `completed_quests()` method, and
//! `participants.get( &1 )` → `current_participant()` (participant 1 is the sole,
//! current participant at that point).

#![allow(clippy::float_cmp)] // Tests assert exact stored/configured values; no arithmetic precedes the comparisons.

#![ cfg( feature = "enabled" ) ]


use tiles_tools::game_systems::*;
use std::collections::HashMap;

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
  assert_eq!(quest_manager.completed_quests().len(), 1);
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

  let participant = game.current_participant().unwrap();
  assert_eq!(participant.status_effects.len(), 1);
  assert_eq!(participant.status_effects[0].duration, 3);
}
