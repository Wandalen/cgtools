//! Tests for the `game_systems` module — turn management, state machine, resources,
//! quests, and status effects driven purely through the public surface.
//!
//! Relocated from `src/game_systems.rs` by task 072. Two tests were losslessly
//! rewritten from private-field reads onto existing public accessors:
//! `completed_quests` field → `completed_quests()` method, and
//! `participants.get( &1 )` → `current_participant()` (participant 1 is the sole,
//! current participant at that point).

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
  game.participant_add(1, 100);
  game.participant_add(2, 85);
  game.participant_add(3, 95);

  // Should be ordered by initiative (highest first)
  assert_eq!(game.current_turn(), Some(1)); // Initiative 100

  game.turn_end();
  assert_eq!(game.current_turn(), Some(3)); // Initiative 95

  game.turn_end();
  assert_eq!(game.current_turn(), Some(2)); // Initiative 85

  game.turn_end();
  assert_eq!(game.current_turn(), Some(1)); // Back to first, round 2
  assert_eq!(game.round_number(), 2);
}

#[test]
fn test_action_points() {
  let mut game = TurnBasedGame::new();
  game.participant_add(1, 100);

  assert_eq!(game.current_participant().unwrap().action_points, 3);

  // Spend some action points
  assert!(game.action_points_spend(2));
  assert_eq!(game.current_participant().unwrap().action_points, 1);

  // Try to spend more than available
  assert!(!game.action_points_spend(2));
  assert_eq!(game.current_participant().unwrap().action_points, 1);
}

#[test]
fn test_game_state_machine() {
  let mut machine = GameStateMachine::new(GameState::Initialize);
  assert_eq!(machine.current_state(), GameState::Initialize);

  // Process initialization complete
  assert!(machine.event_process(GameStateEvent::InitComplete));
  assert_eq!(machine.current_state(), GameState::MainMenu);

  // Start game
  assert!(machine.event_process(GameStateEvent::StartGame));
  assert_eq!(machine.current_state(), GameState::Loading);

  // Invalid transition should fail
  assert!(!machine.event_process(GameStateEvent::Pause));
  assert_eq!(machine.current_state(), GameState::Loading);
}

#[ expect( clippy::float_cmp, reason = "whole-number resource arithmetic is exact; the asserts pin the exact stored values" ) ]
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

  resource.current_set(50.0);
  assert_eq!(resource.current, 50.0);
  assert!(!resource.is_depleted());
  assert!(!resource.is_full());
}

#[ expect( clippy::float_cmp, reason = "whole-number resource arithmetic is exact; the asserts pin the exact stored values" ) ]
#[test]
fn test_resource_manager() {
  let mut manager = ResourceManager::new();
  manager.entity_add(1, 100.0, 50.0);

  assert!(manager.health_modify(1, -25.0));
  assert_eq!(manager.resources_get(1).unwrap().health.current, 75.0);

  assert!(manager.mana_modify(1, -10.0));
  assert_eq!(manager.resources_get(1).unwrap().mana.current, 40.0);

  // Test defeated entities
  manager.health_modify(1, -100.0);
  let defeated = manager.defeated_entities_get();
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

  quest_manager.quest_add(quest);

  // Start quest
  assert!(quest_manager.quest_start("test_quest", 1));
  assert_eq!(quest_manager.active_quests().len(), 1);

  // Update objective progress
  quest_manager.objective_update("test_quest", "kill_enemies", 3);
  quest_manager.objective_update("test_quest", "kill_enemies", 2);

  // Quest should be completed
  assert_eq!(quest_manager.completed_quests().len(), 1);
}

// test_kind: bug_reproducer(BUG-133)
/// ## Root Cause
/// `turn_order_rebuild` only clamped `current_turn_index` numerically against
/// the new `turn_order`'s length -- it never remapped the index to the same
/// entity_id. Any `participant_add`/`participant_remove` call mid-round
/// silently reassigned "whose turn it is" to whichever entity happened to
/// land on that numeric slot after re-sorting, with no `turn_end()` call in
/// between.
/// ## Why Not Caught
/// The existing `test_turn_based_participants` only ever calls
/// `participant_add` before any `turn_end()`, and never calls
/// `participant_remove` at all -- it never exercises a rebuild that happens
/// while `current_turn_index` already points partway through the order.
/// ## Fix Applied
/// `turn_order_rebuild` now captures the current entity_id before rebuilding,
/// then looks up that entity's new position in the freshly-sorted order --
/// falling back to the original numeric clamp only when that entity no
/// longer exists (i.e. it was itself the one removed).
/// ## Prevention
/// n/a -- covered by this test.
/// ## Pitfall
/// Invisible whenever every `participant_add`/`participant_remove` call
/// happens before the first `turn_end()`, or whenever removed/added entities
/// never precede the current turn holder in initiative order -- both leave
/// the numeric index and the identity-correct index coincidentally equal.
#[test]
fn test_turn_order_rebuild_preserves_current_entity_across_removal()
{
  let mut game = TurnBasedGame::new();
  game.participant_add(1, 100);
  game.participant_add(2, 90);
  game.participant_add(3, 80);
  assert_eq!(game.current_turn(), Some(1)); // Initiative 100

  game.turn_end();
  assert_eq!(game.current_turn(), Some(2)); // Initiative 90

  // Removing an unrelated, earlier-ordered participant mid-round must not
  // change whose turn it currently is.
  game.participant_remove(1);
  assert_eq!(
    game.current_turn(), Some(2),
    "removing participant 1 shifted the current turn away from participant 2, \
     who was never removed and never had turn_end() called"
  );
}

#[test]
fn test_status_effects() {
  let mut game = TurnBasedGame::new();
  game.participant_add(1, 100);

  let poison = StatusEffect {
    id: "poison".to_string(),
    name: "Poison".to_string(),
    description: "Takes damage over time".to_string(),
    duration: 3,
    magnitude: 5.0,
    is_beneficial: false,
    category: EffectCategory::DamageOverTime,
  };

  game.status_effect_apply(1, poison);

  let participant = game.current_participant().unwrap();
  assert_eq!(participant.status_effects.len(), 1);
  assert_eq!(participant.status_effects[0].duration, 3);
}
