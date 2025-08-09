//! Behavior Tree AI system demonstration.

#![ allow( clippy::needless_return ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::items_after_statements ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::format_in_format_args ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::duplicated_attributes ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::trivially_copy_pass_by_ref ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unnested_or_patterns ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::redundant_else ) ]
#![ allow( clippy::cast_lossless ) ]
//!
//! This example demonstrates the comprehensive behavior tree system including:
//! - Composite nodes (sequence, selector, parallel)
//! - Decorator nodes (repeat, invert, cooldown)
//! - Condition and action nodes
//! - Blackboard data sharing
//! - Complex AI behavior patterns

use tiles_tools::{
  behavior_tree::*,
};
use std::time::Duration;

fn main()
{
  println!("Behavior Tree AI System Demonstration");
  println!("=====================================");

  // === Basic Behavior Tree Construction ===
  println!("\n=== Basic Behavior Tree Construction ===");

  // Create a simple sequence behavior
  let mut simple_sequence = BehaviorTreeBuilder::new()
    .sequence(vec![
      set_blackboard("task", "initialize"),
      wait(0.1),
      set_blackboard("task", "complete"),
    ])
    .build_named("SimpleSequence".to_string());

  println!("Created simple sequence behavior tree");

  let mut context = BehaviorContext::new();
  println!("Executing simple sequence...");
  
  // Execute until completion
  loop {
    let status = simple_sequence.execute(&mut context);
    println!("  Status: {:?}", status);
    
    if status != BehaviorStatus::Running {
      break;
    }
    
    // Simulate frame time
    std::thread::sleep(Duration::from_millis(50));
    context.update(Duration::from_millis(50));
  }

  println!("Final task status: {:?}", context.get_blackboard("task"));

  // === Advanced AI Behavior Pattern ===
  println!("\n=== Advanced AI Behavior: Guard Patrol ===");

  let mut guard_ai = create_guard_patrol_tree();
  let mut guard_context = BehaviorContext::for_entity(1);

  // Set initial game state
  guard_context.set_blackboard("health", 100);
  guard_context.set_blackboard("enemy_spotted", false);
  guard_context.set_blackboard("patrol_point", 0);
  guard_context.set_blackboard("alert_level", 0);

  println!("Created guard patrol AI with initial state");
  println!("  Health: {:?}", guard_context.get_blackboard("health"));
  println!("  Alert Level: {:?}", guard_context.get_blackboard("alert_level"));

  // Simulate several execution frames
  println!("\nSimulating guard AI behavior over time...");
  for frame in 1..=10 {
    println!("\n--- Frame {} ---", frame);
    
    // Simulate dynamic game events
    match frame {
      3 => {
        println!("  [EVENT] Enemy spotted!");
        guard_context.set_blackboard("enemy_spotted", true);
        guard_context.set_blackboard("alert_level", 3);
      }
      6 => {
        println!("  [EVENT] Taking damage!");
        guard_context.set_blackboard("health", 60);
      }
      8 => {
        println!("  [EVENT] Enemy lost, returning to patrol");
        guard_context.set_blackboard("enemy_spotted", false);
        guard_context.set_blackboard("alert_level", 1);
      }
      _ => {}
    }

    let status = guard_ai.execute(&mut guard_context);
    println!("  AI Status: {:?}", status);
    println!("  Current Task: {:?}", guard_context.get_blackboard("current_action"));
    
    // Simulate frame time
    std::thread::sleep(Duration::from_millis(100));
    guard_context.update(Duration::from_millis(100));
  }

  // === Complex Behavior Composition ===
  println!("\n=== Complex Behavior: Combat AI ===");

  let mut combat_ai = create_combat_ai_tree();
  let mut combat_context = BehaviorContext::for_entity(2);

  // Set combat scenario state
  combat_context.set_blackboard("health", 80);
  combat_context.set_blackboard("mana", 50);
  combat_context.set_blackboard("enemy_distance", 3);
  combat_context.set_blackboard("has_ranged_weapon", true);
  combat_context.set_blackboard("cover_available", true);

  println!("Created combat AI with tactical scenario");
  println!("Simulating tactical combat decision making...");

  for round in 1..=5 {
    println!("\n--- Combat Round {} ---", round);
    
    // Dynamic combat events
    match round {
      2 => {
        println!("  [COMBAT] Enemy closes distance!");
        combat_context.set_blackboard("enemy_distance", 1);
      }
      3 => {
        println!("  [COMBAT] Taking heavy damage!");
        combat_context.set_blackboard("health", 30);
      }
      4 => {
        println!("  [COMBAT] Found health potion!");
        combat_context.set_blackboard("health_potion_available", true);
      }
      _ => {}
    }

    let status = combat_ai.execute(&mut combat_context);
    println!("  Combat AI Status: {:?}", status);
    println!("  Tactical Decision: {:?}", combat_context.get_blackboard("combat_action"));
    println!("  Health: {:?}", combat_context.get_blackboard("health"));

    std::thread::sleep(Duration::from_millis(200));
    combat_context.update(Duration::from_millis(200));
  }

  // === Behavior Tree Performance Test ===
  println!("\n=== Performance Test: Multiple AI Entities ===");

  let ai_count = 50;
  let mut ai_entities: Vec<BehaviorTree> = Vec::new();
  let mut ai_contexts: Vec<BehaviorContext> = Vec::new();

  // Create multiple AI entities
  for i in 0..ai_count {
    let ai = create_simple_ai_tree(format!("AI_{}", i));
    let context = BehaviorContext::for_entity(i as u32);
    
    ai_entities.push(ai);
    ai_contexts.push(context);
  }

  println!("Created {} AI entities for performance testing", ai_count);

  let start_time = std::time::Instant::now();
  
  // Execute all AI entities
  for (ai, context) in ai_entities.iter_mut().zip(ai_contexts.iter_mut()) {
    ai.execute(context);
  }
  
  let execution_time = start_time.elapsed();
  
  println!("Executed {} AI entities in {:?}", ai_count, execution_time);
  println!("Average time per AI: {:.2}µs", execution_time.as_micros() as f64 / ai_count as f64);

  // === Decorator Demonstrations ===
  println!("\n=== Decorator Node Demonstrations ===");

  // Repeat decorator
  println!("\n--- Repeat Decorator ---");
  let mut repeat_tree = BehaviorTreeBuilder::new()
    .root(repeat(
      set_blackboard("counter", 1),
      3
    ))
    .build_named("RepeatDemo".to_string());

  let mut repeat_context = BehaviorContext::new();
  let status = repeat_tree.execute(&mut repeat_context);
  println!("Repeat 3 times status: {:?}", status);

  // Cooldown decorator
  println!("\n--- Cooldown Decorator ---");
  let mut cooldown_tree = BehaviorTreeBuilder::new()
    .root(cooldown(
      set_blackboard("can_use_ability", true),
      0.1 // 100ms cooldown
    ))
    .build_named("CooldownDemo".to_string());

  let mut cooldown_context = BehaviorContext::new();

  // First use - should succeed
  let status1 = cooldown_tree.execute(&mut cooldown_context);
  println!("First ability use: {:?}", status1);

  // Immediate second use - should fail
  let status2 = cooldown_tree.execute(&mut cooldown_context);
  println!("Immediate second use: {:?}", status2);

  // Wait and try again
  std::thread::sleep(Duration::from_millis(150));
  cooldown_context.update(Duration::from_millis(150));
  let status3 = cooldown_tree.execute(&mut cooldown_context);
  println!("After cooldown: {:?}", status3);

  println!("\n=== Behavior Tree Demonstration Complete ===");
  println!("\nKey features demonstrated:");
  println!("- Composite nodes (sequence, selector, parallel)");
  println!("- Decorator nodes (repeat, invert, cooldown)");
  println!("- Action and condition nodes");
  println!("- Blackboard data sharing");
  println!("- Dynamic behavior adaptation");
  println!("- Performance with multiple AI entities");
  println!("- Complex AI behavior patterns");
  println!("- Time-based behaviors and cooldowns");
}

/// Creates a guard patrol behavior tree for demonstration.
fn create_guard_patrol_tree() -> BehaviorTree
{
  BehaviorTreeBuilder::new()
    .selector(vec![
      // High priority: Handle low health
      sequence(vec![
        condition("health", BehaviorValue::Int(30)), // Health <= 30 (simplified condition)
        set_blackboard("current_action", "retreating"),
        set_blackboard("alert_level", 0),
      ]),
      
      // Medium priority: Combat behavior
      sequence(vec![
        condition("enemy_spotted", true),
        selector(vec![
          // Ranged combat
          sequence(vec![
            condition("alert_level", BehaviorValue::Int(3)),
            set_blackboard("current_action", "combat_ranged"),
            wait(0.5),
          ]),
          // Melee combat
          sequence(vec![
            set_blackboard("current_action", "combat_melee"),
            wait(0.3),
          ]),
        ]),
      ]),
      
      // Low priority: Normal patrol
      sequence(vec![
        set_blackboard("current_action", "patrolling"),
        selector(vec![
          sequence(vec![
            condition("patrol_point", BehaviorValue::Int(0)),
            set_blackboard("patrol_point", 1),
            wait(1.0),
          ]),
          sequence(vec![
            condition("patrol_point", BehaviorValue::Int(1)),
            set_blackboard("patrol_point", 2),
            wait(1.0),
          ]),
          sequence(vec![
            set_blackboard("patrol_point", 0),
            wait(1.0),
          ]),
        ]),
      ]),
    ])
    .build_named("GuardPatrol".to_string())
}

/// Creates a combat AI behavior tree for demonstration.
fn create_combat_ai_tree() -> BehaviorTree
{
  BehaviorTreeBuilder::new()
    .selector(vec![
      // Emergency: Use health potion if available and health is critical
      sequence(vec![
        condition("health", BehaviorValue::Int(30)), // Health <= 30
        condition("health_potion_available", true),
        set_blackboard("combat_action", "use_health_potion"),
        set_blackboard("health", 80), // Simulate healing
        set_blackboard("health_potion_available", false),
      ]),

      // Tactical retreat if health is low
      sequence(vec![
        condition("health", BehaviorValue::Int(40)), // Health <= 40
        condition("cover_available", true),
        set_blackboard("combat_action", "retreat_to_cover"),
      ]),

      // Ranged attack if enemy is far and we have ranged weapon
      sequence(vec![
        condition("enemy_distance", BehaviorValue::Int(2)), // Distance >= 2
        condition("has_ranged_weapon", true),
        set_blackboard("combat_action", "ranged_attack"),
      ]),

      // Melee attack if enemy is close
      sequence(vec![
        condition("enemy_distance", BehaviorValue::Int(1)), // Distance <= 1
        set_blackboard("combat_action", "melee_attack"),
      ]),

      // Default: Move closer to enemy
      set_blackboard("combat_action", "move_closer"),
    ])
    .build_named("CombatAI".to_string())
}

/// Creates a simple AI behavior tree for performance testing.
fn create_simple_ai_tree(name: String) -> BehaviorTree
{
  BehaviorTreeBuilder::new()
    .sequence(vec![
      set_blackboard("initialized", true),
      selector(vec![
        condition("active", true),
        set_blackboard("active", true),
      ]),
      set_blackboard("ready", true),
    ])
    .build_named(name)
}