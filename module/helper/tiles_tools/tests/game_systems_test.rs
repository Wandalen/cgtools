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

// test_kind: bug_reproducer(BUG-479)
/// ## Root Cause
/// `GameStateMachine` carried private `state_enter_handlers`/
/// `state_exit_handlers: HashMap<GameState, StateHandler>` fields with no
/// public registration method anywhere in the crate to populate them, and
/// `transition_to`'s only use of either map was a `.get()` lookup whose
/// `Some(_handler)` arm was immediately discarded behind a comment noting
/// the call was never wired up ("Can't call handler directly due to
/// borrowing issues"). The maps could never contain an entry and the lookup
/// could never do anything -- pure dead weight shaped like a real feature.
/// ## Why Not Caught
/// Both fields were private with no constructor parameter and no setter, so
/// no test anywhere in the crate could have populated them even if it tried
/// -- there was no reachable path to exercise the dead branches at all.
/// ## Fix Applied
/// Removed `state_enter_handlers`/`state_exit_handlers` and the
/// now-unused `StateHandler` type alias entirely (their initialization in
/// `new`, and the two dead `.get()` lookups in `transition_to`), rather than
/// building a registration API nothing in the crate asked for (YAGNI) --
/// this test instead pins that the field removal left `transition_to`'s
/// real behavior (current/previous state bookkeeping) unchanged.
/// ## Prevention
/// n/a -- covered by this test; `previous_state()` had no dedicated test
/// before this fix (`test_game_state_machine` only ever reads
/// `current_state()`).
/// ## Pitfall
/// A private field with no way for any caller to populate it, paired with a
/// lookup whose success arm is a no-op, is dead code wearing the shape of an
/// unfinished feature -- grep for a registration/setter path before
/// deciding whether to implement or delete.
#[test]
fn test_game_state_machine_transition_to_tracks_previous_state()
{
  let mut machine = GameStateMachine::new(GameState::Initialize);
  assert_eq!(machine.previous_state(), None);

  machine.transition_to(GameState::MainMenu);
  assert_eq!(machine.current_state(), GameState::MainMenu);
  assert_eq!(machine.previous_state(), Some(GameState::Initialize));

  machine.transition_to(GameState::Loading);
  assert_eq!(machine.current_state(), GameState::Loading);
  assert_eq!(machine.previous_state(), Some(GameState::MainMenu));
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

// test_kind: bug_reproducer(BUG-349)
/// ## Root Cause
/// `Resource::new`/`Resource::with_regeneration` store `maximum` unclamped,
/// but `Resource::modify`/`Resource::current_set` both call
/// `.clamp(0.0, self.maximum)` -- `f32::clamp` has an unconditional
/// `assert!(min <= max)`, so any negative `maximum` makes every subsequent
/// `modify`/`current_set` call panic. `Resource::maximum_set` already clamps
/// via `value.max(0.0)`, but `new`/`with_regeneration` never applied the same
/// invariant.
/// ## Why Not Caught
/// Every existing `Resource`/`ResourceManager` test constructs resources with
/// a positive maximum -- no test ever passed a negative value to `new` or
/// `with_regeneration`, so the missing clamp had no historical trigger.
/// ## Fix Applied
/// `new` and `with_regeneration` now clamp `maximum` to `.max(0.0)`, matching
/// the invariant `maximum_set` already enforced.
/// ## Prevention
/// n/a -- covered by this test.
/// ## Pitfall
/// A builder/constructor that stores a value one of the type's own methods
/// later assumes is non-negative (via an unconditional `f32::clamp` divisor
/// bound) must enforce that invariant itself at construction -- a sibling
/// setter clamping correctly (`maximum_set`) is not evidence every
/// value-producing path does the same.
#[test]
fn test_resource_new_with_negative_maximum_does_not_panic_on_modify()
{
  let mut resource = Resource::new(-5.0);
  resource.modify(1.0);
  assert!(resource.maximum >= 0.0, "maximum should be clamped to a non-negative value, got {}", resource.maximum);
  assert!(resource.current >= 0.0, "current should be clamped to a non-negative value, got {}", resource.current);
}

// test_kind: bug_reproducer(BUG-480)
/// ## Root Cause
/// `Resource::is_full` compared `(current - maximum).abs()` against the fixed
/// absolute `f32::EPSILON` (~1.19e-7). `f32::EPSILON` is only the gap between
/// 1.0 and the next representable f32 -- at a `maximum` far from 1.0, the
/// spacing between representable f32 values near `maximum` (its true ULP) is
/// itself much larger than `f32::EPSILON`, so a resource sitting at its real
/// maximum after ordinary float arithmetic can differ from `self.maximum` by
/// more than `f32::EPSILON` and wrongly report not-full.
/// ## Why Not Caught
/// `test_resource_management` only exercises `Resource::new(100.0)` and
/// compares `current`/`maximum` values produced by exact whole-number
/// arithmetic (which round-trips exactly in f32 at that small magnitude) --
/// no existing test used a `maximum` large enough for the fixed-epsilon
/// tolerance to fall below the magnitude's own float precision spacing.
/// ## Fix Applied
/// `is_full` now compares against `self.maximum.abs() * f32::EPSILON` -- a
/// tolerance that scales with `maximum`'s own magnitude, matching the
/// precision actually available to floats near that value.
/// ## Prevention
/// n/a -- covered by this test.
/// ## Pitfall
/// A fixed absolute floating-point tolerance (`f32::EPSILON`,
/// `1e-6`, etc.) is only valid for comparisons near the magnitude it was
/// implicitly chosen for -- comparisons against values of materially
/// different magnitude need a tolerance scaled to that magnitude.
#[ expect( clippy::float_cmp, reason = "direct field write to pin an exact float value for the reproducer, not a computed comparison" ) ]
#[test]
fn test_resource_is_full_uses_magnitude_scaled_tolerance()
{
  let maximum = 1_000_000.0_f32;
  let mut resource = Resource::new(maximum);

  // Simulate a `current` that float arithmetic left within one ULP of
  // `maximum` at this magnitude (~0.0625) -- well beyond the old fixed
  // `f32::EPSILON` (~1.19e-7) tolerance, but a resource genuinely at its max.
  resource.current = maximum - 0.06;
  assert_ne!(resource.current, maximum, "fixture must not round-trip to an exact match, or it would pass under either tolerance");

  assert!(
    resource.is_full(),
    "a resource within one float ULP of its maximum ({} vs {}) should report is_full() == true",
    resource.current, resource.maximum,
  );
}
