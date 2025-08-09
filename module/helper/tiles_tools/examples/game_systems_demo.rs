//! Game systems demonstration showing integrated turn-based gameplay.
//!
//! This example demonstrates the comprehensive game systems integration including:
//! - Turn-based game management with initiative and action points
//! - Game state machine with transitions and phases
//! - Resource management (health, mana, experience)
//! - Quest system with objectives and rewards
//! - Status effects and game mechanics
//! - System integration and event coordination

use tiles_tools::game_systems::*;
use tiles_tools::events::{EventBus, EventResult};
use tiles_tools::debug::{GridRenderer, GridStyle, DebugColor};
use std::collections::HashMap;

fn main() {
  println!("🎮 Game Systems Demonstration");
  println!("==============================");

  // === TURN-BASED GAME DEMONSTRATION ===
  println!("\n⚔️ Turn-Based Combat System");
  println!("---------------------------");

  demonstrate_turn_based_combat();

  // === GAME STATE MACHINE DEMONSTRATION ===
  println!("\n🎯 Game State Management");
  println!("------------------------");

  demonstrate_game_state_machine();

  // === RESOURCE MANAGEMENT DEMONSTRATION ===
  println!("\n💎 Resource Management");
  println!("----------------------");

  demonstrate_resource_management();

  // === QUEST SYSTEM DEMONSTRATION ===
  println!("\n📜 Quest System");
  println!("---------------");

  demonstrate_quest_system();

  // === INTEGRATED GAMEPLAY DEMONSTRATION ===
  println!("\n🌟 Integrated Tactical RPG Session");
  println!("----------------------------------");

  demonstrate_integrated_gameplay();

  println!("\n✨ Game Systems Demo Complete!");
  println!("\nKey features demonstrated:");
  println!("• Turn-based combat with initiative and action points");
  println!("• Game state machine with transition management");
  println!("• Resource management with regeneration and limits");
  println!("• Quest system with objectives and branching logic");
  println!("• Status effects with duration and stacking");
  println!("• Integrated gameplay with multiple systems");
  println!("• Event-driven architecture for system coordination");
}

fn demonstrate_turn_based_combat() {
  let mut game = TurnBasedGame::new();

  println!("Setting up combat participants...");

  // Add combat participants
  game.add_participant(1, 95);  // Player - high initiative
  game.add_participant(2, 80);  // Enemy Knight - medium initiative  
  game.add_participant(3, 110); // Enemy Archer - highest initiative
  game.add_participant(4, 65);  // Player Ally - low initiative

  println!("Turn Order (by initiative):");
  let participants = game.participants_in_order();
  for (i, participant) in participants.iter().enumerate() {
    let entity_type = match participant.entity_id {
      1 => "Player",
      2 => "Enemy Knight", 
      3 => "Enemy Archer",
      4 => "Player Ally",
      _ => "Unknown"
    };
    println!("  {}. {} (ID: {}, Initiative: {})", 
      i + 1, entity_type, participant.entity_id, participant.initiative);
  }

  println!("\nSimulating 2 rounds of combat:");

  for round in 1..=2 {
    println!("\n--- Round {} ---", round);
    
    let mut turns_in_round = 0;
    let start_round = game.round_number();

    while game.round_number() == start_round && turns_in_round < 10 {
      if let Some(current_entity) = game.current_turn() {
        let entity_name = match current_entity {
          1 => "Player",
          2 => "Enemy Knight",
          3 => "Enemy Archer", 
          4 => "Player Ally",
          _ => "Unknown"
        };

        if let Some(participant) = game.current_participant() {
          println!("  {}'s turn (AP: {})", entity_name, participant.action_points);
          
          // Simulate different actions based on entity type
          match current_entity {
            1 => { // Player
              println!("    Player attacks with sword (2 AP)");
              game.spend_action_points(2);
              if game.current_participant().unwrap().action_points > 0 {
                println!("    Player moves closer (1 AP)");
                game.spend_action_points(1);
              }
            },
            2 => { // Enemy Knight
              println!("    Knight charges (3 AP)");
              game.spend_action_points(3);
            },
            3 => { // Enemy Archer
              println!("    Archer shoots arrow (1 AP)");
              game.spend_action_points(1);
              println!("    Archer repositions (1 AP)");
              game.spend_action_points(1);
              println!("    Archer shoots again (1 AP)");
              game.spend_action_points(1);
            },
            4 => { // Player Ally
              println!("    Ally casts healing spell (2 AP)");
              game.spend_action_points(2);
              println!("    Ally moves to better position (1 AP)");
              game.spend_action_points(1);
            },
            _ => {}
          }
        }

        game.end_turn();
        turns_in_round += 1;
      }
    }
  }

  println!("\nCombat round completed! Final round: {}", game.round_number());

  // Demonstrate status effects
  println!("\nApplying status effects...");
  
  let poison = StatusEffect {
    id: "poison".to_string(),
    name: "Poison".to_string(), 
    description: "Takes 5 damage per turn".to_string(),
    duration: 3,
    magnitude: 5.0,
    is_beneficial: false,
    category: EffectCategory::DamageOverTime,
  };

  let blessing = StatusEffect {
    id: "blessing".to_string(),
    name: "Divine Blessing".to_string(),
    description: "+2 attack power".to_string(), 
    duration: 5,
    magnitude: 2.0,
    is_beneficial: true,
    category: EffectCategory::AttackPower,
  };

  game.apply_status_effect(1, blessing);
  game.apply_status_effect(2, poison);

  println!("Player has Divine Blessing (+2 attack, 5 turns)");
  println!("Enemy Knight is poisoned (5 damage/turn, 3 turns)");

  // Show participant status
  if let Some(player) = game.participants_in_order().iter().find(|p| p.entity_id == 1) {
    println!("\nPlayer status effects: {}", player.status_effects.len());
    for effect in &player.status_effects {
      println!("  • {} ({}): {} turns remaining", 
        effect.name, if effect.is_beneficial { "Buff" } else { "Debuff" }, effect.duration);
    }
  }
}

fn demonstrate_game_state_machine() {
  let mut state_machine = GameStateMachine::new(GameState::Initialize);
  
  println!("Starting game state machine...");
  println!("Current state: {:?}", state_machine.current_state());

  // Simulate game startup sequence
  let state_transitions = [
    (GameStateEvent::InitComplete, "Game initialized"),
    (GameStateEvent::StartGame, "Starting new game"),
    (GameStateEvent::StartGame, "Loading complete, entering gameplay"),
    (GameStateEvent::EnterCombat, "Combat encounter!"),
    (GameStateEvent::ExitCombat, "Combat resolved"),
    (GameStateEvent::Pause, "Game paused"),
    (GameStateEvent::Resume, "Game resumed"),
    (GameStateEvent::OpenInventory, "Checking inventory"),
    (GameStateEvent::CloseInventory, "Inventory closed"),
  ];

  for (event, description) in state_transitions {
    if state_machine.process_event(event) {
      println!("  {} → {:?} ({})", 
        format!("{:?}", event), 
        state_machine.current_state(), 
        description);
    } else {
      println!("  {} → Failed (invalid transition from {:?})", 
        format!("{:?}", event),
        state_machine.current_state());
    }
  }

  // Demonstrate state data
  state_machine.set_state_data("player_name".to_string(), "Hero".to_string());
  state_machine.set_state_data("level".to_string(), "forest_1".to_string());
  
  println!("\nState data:");
  if let Some(name) = state_machine.get_state_data("player_name") {
    println!("  Player name: {}", name);
  }
  if let Some(level) = state_machine.get_state_data("level") {
    println!("  Current level: {}", level);
  }

  println!("Final state: {:?}", state_machine.current_state());
}

fn demonstrate_resource_management() {
  let mut resource_manager = ResourceManager::new();

  println!("Setting up entities with resources...");

  // Create different entity types
  resource_manager.add_entity(1, 100.0, 50.0);  // Player: 100 HP, 50 MP
  resource_manager.add_entity(2, 80.0, 0.0);    // Warrior: 80 HP, 0 MP
  resource_manager.add_entity(3, 60.0, 100.0);  // Mage: 60 HP, 100 MP
  resource_manager.add_entity(4, 150.0, 25.0);  // Boss: 150 HP, 25 MP

  // Configure regeneration
  if let Some(resources) = resource_manager.get_resources_mut(1) {
    resources.health.regeneration = 2.0; // Player regenerates 2 HP/sec
    resources.mana.regeneration = 5.0;   // Player regenerates 5 MP/sec
  }

  if let Some(resources) = resource_manager.get_resources_mut(3) {
    resources.mana.regeneration = 3.0;   // Mage regenerates 3 MP/sec
  }

  println!("\nInitial resources:");
  for entity_id in [1, 2, 3, 4] {
    if let Some(resources) = resource_manager.get_resources(entity_id) {
      let entity_type = match entity_id {
        1 => "Player",
        2 => "Warrior", 
        3 => "Mage",
        4 => "Boss",
        _ => "Unknown"
      };
      println!("  {}: {:.0}/{:.0} HP, {:.0}/{:.0} MP", 
        entity_type,
        resources.health.current, resources.health.maximum,
        resources.mana.current, resources.mana.maximum);
    }
  }

  println!("\nSimulating combat damage...");

  // Player takes damage
  resource_manager.modify_health(1, -25.0);
  resource_manager.modify_mana(1, -15.0);
  println!("  Player takes 25 damage, uses 15 mana");

  // Warrior takes heavy damage
  resource_manager.modify_health(2, -60.0);
  println!("  Warrior takes 60 damage");

  // Mage uses lots of mana
  resource_manager.modify_mana(3, -80.0);
  println!("  Mage uses 80 mana for powerful spell");

  // Boss takes some damage
  resource_manager.modify_health(4, -45.0);
  println!("  Boss takes 45 damage from combined attacks");

  println!("\nAfter combat:");
  for entity_id in [1, 2, 3, 4] {
    if let Some(resources) = resource_manager.get_resources(entity_id) {
      let entity_type = match entity_id {
        1 => "Player",
        2 => "Warrior",
        3 => "Mage", 
        4 => "Boss",
        _ => "Unknown"
      };
      println!("  {}: {:.0}/{:.0} HP ({:.0}%), {:.0}/{:.0} MP ({:.0}%)",
        entity_type,
        resources.health.current, resources.health.maximum,
        resources.health.percentage() * 100.0,
        resources.mana.current, resources.mana.maximum,
        resources.mana.percentage() * 100.0);
    }
  }

  println!("\nSimulating 3 seconds of regeneration...");
  resource_manager.update_all(3.0);

  println!("After regeneration:");
  for entity_id in [1, 3] { // Only show entities with regen
    if let Some(resources) = resource_manager.get_resources(entity_id) {
      let entity_type = if entity_id == 1 { "Player" } else { "Mage" };
      println!("  {}: {:.0}/{:.0} HP, {:.0}/{:.0} MP",
        entity_type,
        resources.health.current, resources.health.maximum,
        resources.mana.current, resources.mana.maximum);
    }
  }

  // Check for defeated entities
  let defeated = resource_manager.get_defeated_entities();
  if !defeated.is_empty() {
    println!("\nDefeated entities: {:?}", defeated);
  } else {
    println!("\nNo entities defeated");
  }
}

fn demonstrate_quest_system() {
  let mut quest_manager = QuestManager::new();

  println!("Setting up quest system...");

  // Create a main quest
  let main_quest = Quest {
    id: "save_village".to_string(),
    name: "Save the Village".to_string(),
    description: "The village is under attack by orcs. Eliminate the threat!".to_string(),
    status: QuestStatus::Available,
    objectives: vec![
      QuestObjective {
        id: "kill_orcs".to_string(),
        description: "Eliminate 5 orc raiders".to_string(),
        completed: false,
        objective_type: ObjectiveType::KillTargets {
          target_type: "orc".to_string(),
          count: 5,
          current: 0,
        },
        optional: false,
      },
      QuestObjective {
        id: "find_chief".to_string(),
        description: "Find and defeat the orc chieftain".to_string(),
        completed: false,
        objective_type: ObjectiveType::KillTargets {
          target_type: "orc_chief".to_string(),
          count: 1,
          current: 0,
        },
        optional: false,
      },
      QuestObjective {
        id: "rescue_villagers".to_string(),
        description: "Rescue captured villagers (Optional)".to_string(),
        completed: false,
        objective_type: ObjectiveType::Custom {
          data: vec![("rescued".to_string(), "0".to_string())].into_iter().collect(),
        },
        optional: true,
      },
    ],
    prerequisites: vec![],
    rewards: vec![
      QuestReward::Experience(500),
      QuestReward::Currency(100),
      QuestReward::Items("healing_potion".to_string(), 3),
    ],
    data: HashMap::new(),
  };

  // Create a side quest
  let side_quest = Quest {
    id: "gather_herbs".to_string(),
    name: "Herb Gathering".to_string(),
    description: "Collect medicinal herbs for the village healer".to_string(),
    status: QuestStatus::Available,
    objectives: vec![
      QuestObjective {
        id: "collect_herbs".to_string(),
        description: "Collect 10 healing herbs".to_string(),
        completed: false,
        objective_type: ObjectiveType::CollectItems {
          item_id: "healing_herb".to_string(),
          count: 10,
          current: 0,
        },
        optional: false,
      },
    ],
    prerequisites: vec![],
    rewards: vec![
      QuestReward::Experience(100),
      QuestReward::Currency(25),
    ],
    data: HashMap::new(),
  };

  quest_manager.add_quest(main_quest);
  quest_manager.add_quest(side_quest);

  // Start both quests
  println!("\nStarting quests...");
  quest_manager.start_quest("save_village", 1);
  quest_manager.start_quest("gather_herbs", 1);

  println!("Active quests: {}", quest_manager.active_quests().len());
  for quest in quest_manager.active_quests() {
    println!("  • {} - {}", quest.name, quest.description);
    for objective in &quest.objectives {
      let status = if objective.completed { "✓" } else { "○" };
      let optional = if objective.optional { " (Optional)" } else { "" };
      println!("    {} {}{}", status, objective.description, optional);
    }
  }

  println!("\nSimulating quest progress...");

  // Progress herb collection
  quest_manager.update_objective("gather_herbs", "collect_herbs", 4);
  println!("  Collected 4 herbs...");
  
  quest_manager.update_objective("gather_herbs", "collect_herbs", 6);
  println!("  Collected 6 more herbs (10/10 complete)");

  // Progress orc elimination
  quest_manager.update_objective("save_village", "kill_orcs", 3);
  println!("  Defeated 3 orcs...");

  quest_manager.update_objective("save_village", "kill_orcs", 2);
  println!("  Defeated 2 more orcs (5/5 complete)");

  // Complete chieftain objective
  quest_manager.update_objective("save_village", "find_chief", 1);
  println!("  Defeated orc chieftain!");

  println!("\nQuest completion status:");
  println!("  Completed quests: {}", quest_manager.completed_quest_count());
  for quest in quest_manager.completed_quests() {
    println!("    ✓ {} - Completed!", quest.name);
    println!("      Rewards:");
    for reward in &quest.rewards {
      match reward {
        QuestReward::Experience(exp) => println!("        • {} Experience", exp),
        QuestReward::Currency(gold) => println!("        • {} Gold", gold),
        QuestReward::Items(item, count) => println!("        • {} x{}", item, count),
        QuestReward::UnlockQuest(quest_id) => println!("        • Unlock Quest: {}", quest_id),
        QuestReward::SetFlag(flag) => println!("        • Set Flag: {}", flag),
      }
    }
  }

  // Demonstrate flags
  quest_manager.set_flag("village_saved".to_string(), true);
  quest_manager.set_flag("hero_reputation".to_string(), true);
  
  println!("\nGlobal flags set:");
  println!("  village_saved: {}", quest_manager.get_flag("village_saved"));
  println!("  hero_reputation: {}", quest_manager.get_flag("hero_reputation"));
}

fn demonstrate_integrated_gameplay() {
  println!("Setting up integrated tactical RPG session...");

  // Initialize all systems
  let mut turn_game = TurnBasedGame::new();
  let mut resources = ResourceManager::new();
  let mut state_machine = GameStateMachine::new(GameState::Playing);
  let mut quest_manager = QuestManager::new();
  let mut event_bus = EventBus::new();

  // Set up event listeners for system integration
  event_bus.subscribe(|event: &TurnStartedEvent| {
    println!("🎯 Turn started for entity {} (Round {})", event.entity_id, event.round_number);
    EventResult::Continue
  });

  event_bus.subscribe(|event: &ResourceChangedEvent| {
    if event.resource_type == "health" && event.new_value <= 0.0 {
      println!("💀 Entity {} has fallen!", event.entity_id);
    }
    EventResult::Continue
  });

  // Create a tactical scenario
  println!("\n🗺️ Tactical Battle Setup");
  println!("Party vs Orc Raiders");

  // Add party members
  turn_game.add_participant(1, 85);   // Player Fighter
  turn_game.add_participant(2, 95);   // Player Mage
  turn_game.add_participant(3, 75);   // Player Cleric

  // Add enemies  
  turn_game.add_participant(11, 80);  // Orc Warrior 1
  turn_game.add_participant(12, 70);  // Orc Warrior 2
  turn_game.add_participant(13, 90);  // Orc Shaman

  // Initialize resources for all participants
  resources.add_entity(1, 120.0, 30.0);   // Fighter: High HP, Low MP
  resources.add_entity(2, 70.0, 100.0);   // Mage: Low HP, High MP
  resources.add_entity(3, 90.0, 80.0);    // Cleric: Medium HP, High MP

  resources.add_entity(11, 85.0, 10.0);   // Orc Warrior 1
  resources.add_entity(12, 85.0, 10.0);   // Orc Warrior 2  
  resources.add_entity(13, 65.0, 60.0);   // Orc Shaman

  // Create quest for this battle
  let battle_quest = Quest {
    id: "orc_encounter".to_string(),
    name: "Orc Encounter".to_string(),
    description: "Defeat the orc raiding party".to_string(),
    status: QuestStatus::Active,
    objectives: vec![
      QuestObjective {
        id: "defeat_orcs".to_string(),
        description: "Defeat all orc raiders".to_string(),
        completed: false,
        objective_type: ObjectiveType::KillTargets {
          target_type: "orc".to_string(),
          count: 3,
          current: 0,
        },
        optional: false,
      },
    ],
    prerequisites: vec![],
    rewards: vec![QuestReward::Experience(200)],
    data: HashMap::new(),
  };

  quest_manager.add_quest(battle_quest);

  // Create a visual representation
  let mut battle_grid = GridRenderer::new()
    .with_size(12, 8)
    .with_style(GridStyle::Square8);

  // Position party members
  battle_grid.add_colored_marker((2, 6), "F", "Fighter", DebugColor::Green, 10);
  battle_grid.add_colored_marker((1, 5), "M", "Mage", DebugColor::Blue, 10);
  battle_grid.add_colored_marker((3, 5), "C", "Cleric", DebugColor::Yellow, 10);

  // Position enemies
  battle_grid.add_colored_marker((9, 4), "O1", "Orc Warrior", DebugColor::Red, 10);
  battle_grid.add_colored_marker((10, 6), "O2", "Orc Warrior", DebugColor::Red, 10);
  battle_grid.add_colored_marker((8, 2), "S", "Orc Shaman", DebugColor::Purple, 10);

  // Add environmental elements
  battle_grid.add_colored_marker((5, 3), "T", "Tree", DebugColor::Green, 5);
  battle_grid.add_colored_marker((6, 5), "R", "Rock", DebugColor::Gray, 5);

  println!("\nBattlefield:");
  println!("{}", battle_grid.render_ascii());

  println!("Combat begins!");

  // Simulate one round of combat
  let mut actions_this_round = 0;
  let start_round = turn_game.round_number();

  while turn_game.round_number() == start_round && actions_this_round < 6 {
    if let Some(current_entity) = turn_game.current_turn() {
      let entity_name = match current_entity {
        1 => "Fighter",
        2 => "Mage", 
        3 => "Cleric",
        11 => "Orc Warrior 1",
        12 => "Orc Warrior 2",
        13 => "Orc Shaman",
        _ => "Unknown"
      };

      // Publish turn started event
      event_bus.publish(TurnStartedEvent {
        entity_id: current_entity,
        round_number: turn_game.round_number(),
        action_points: turn_game.current_participant().unwrap().action_points,
      });

      // Simulate actions based on entity type and AI
      match current_entity {
        1 => { // Fighter
          println!("  {} attacks Orc Warrior 1 with sword!", entity_name);
          resources.modify_health(11, -25.0);
          turn_game.spend_action_points(2);
          
          event_bus.publish(ResourceChangedEvent {
            entity_id: 11,
            resource_type: "health".to_string(),
            old_value: 85.0,
            new_value: resources.get_resources(11).unwrap().health.current,
          });
        },
        2 => { // Mage
          println!("  {} casts fireball at Orc Shaman!", entity_name);
          resources.modify_health(13, -35.0);
          resources.modify_mana(2, -20.0);
          turn_game.spend_action_points(3);
        },
        3 => { // Cleric  
          println!("  {} heals Fighter!", entity_name);
          resources.modify_health(1, 15.0);
          resources.modify_mana(3, -15.0);
          turn_game.spend_action_points(2);
        },
        11 => { // Orc Warrior 1
          if resources.get_resources(11).unwrap().health.current > 0.0 {
            println!("  {} attacks Fighter with axe!", entity_name);
            resources.modify_health(1, -20.0);
            turn_game.spend_action_points(2);
          }
        },
        12 => { // Orc Warrior 2
          println!("  {} charges at Mage!", entity_name);
          resources.modify_health(2, -18.0);
          turn_game.spend_action_points(3);
        },
        13 => { // Orc Shaman
          if resources.get_resources(13).unwrap().health.current > 0.0 {
            println!("  {} casts dark bolt at Cleric!", entity_name);
            resources.modify_health(3, -15.0);
            resources.modify_mana(13, -10.0);
            turn_game.spend_action_points(2);
          }
        },
        _ => {}
      }

      turn_game.end_turn();
      actions_this_round += 1;
    }

    event_bus.process_events();
  }

  println!("\n📊 End of Round Status:");
  for entity_id in [1, 2, 3, 11, 12, 13] {
    if let Some(res) = resources.get_resources(entity_id) {
      let name = match entity_id {
        1 => "Fighter",
        2 => "Mage",
        3 => "Cleric", 
        11 => "Orc Warrior 1",
        12 => "Orc Warrior 2",
        13 => "Orc Shaman",
        _ => "Unknown"
      };
      
      let status = if res.health.current <= 0.0 { " [DEFEATED]" } else { "" };
      println!("  {}: {:.0}/{:.0} HP, {:.0}/{:.0} MP{}",
        name, res.health.current, res.health.maximum,
        res.mana.current, res.mana.maximum, status);
    }
  }

  // Check for defeated enemies and update quest progress
  let mut orcs_defeated = 0;
  for orc_id in [11, 12, 13] {
    if let Some(res) = resources.get_resources(orc_id) {
      if res.health.current <= 0.0 {
        orcs_defeated += 1;
      }
    }
  }

  if orcs_defeated > 0 {
    quest_manager.update_objective("orc_encounter", "defeat_orcs", orcs_defeated);
    println!("\n📜 Quest Progress: {} orcs defeated", orcs_defeated);
  }

  // Check if battle is won
  let party_alive = [1, 2, 3].iter().any(|&id| {
    resources.get_resources(id).unwrap().health.current > 0.0
  });
  
  let enemies_alive = [11, 12, 13].iter().any(|&id| {
    resources.get_resources(id).unwrap().health.current > 0.0
  });

  if !enemies_alive && party_alive {
    println!("\n🎉 Victory! All orcs defeated!");
    state_machine.process_event(GameStateEvent::VictoryAchieved);
    
    if quest_manager.is_quest_completed("orc_encounter") {
      event_bus.publish(QuestCompletedEvent {
        quest_id: "orc_encounter".to_string(),
        rewards: vec![QuestReward::Experience(200)],
      });
    }
  } else if !party_alive {
    println!("\n💀 Defeat! The party has fallen...");
    state_machine.process_event(GameStateEvent::PlayerDefeated);
  } else {
    println!("\n⚔️ Battle continues! Both sides still fighting.");
  }

  println!("Final game state: {:?}", state_machine.current_state());
  
  event_bus.process_events();

  println!("\n🎯 Integration Summary:");
  println!("This demonstrated how all systems work together:");
  println!("• Turn-based combat managed participant order and actions");
  println!("• Resource system tracked health/mana for all entities");
  println!("• Quest system monitored combat objectives");
  println!("• Event system coordinated between systems");
  println!("• State machine tracked overall game flow");
  println!("• Debug visualization showed tactical positions");
}