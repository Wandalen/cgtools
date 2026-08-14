//! Event system demonstration showing decoupled game logic communication.

//!
//! This example demonstrates the comprehensive event system including:
//! - Publishing and subscribing to events
//! - Event priorities and consumption
//! - Common game events (movement, health, collisions)
//! - Event statistics and performance monitoring
//! - Auto-unsubscribing listeners

use tiles_tools::{
  events::{EventBus, EventResult, EventPriority},
  events::common_events::{EntityMoved, MovementType, HealthChanged, EntitiesCollided, SpellCast, CollisionType, HealthChangeCause, HealingSource, GameStateChanged, GameState},
  coordinates::square::{Coordinate as SquareCoord, FourConnected},
};
use std::sync::{ Arc, Mutex };

#[derive(Debug, Clone)]
struct GameStarted {
  level_id: String,
  difficulty: u32,
}

#[derive(Debug, Clone)]
struct PlayerDied {
  player_id: u32,
  cause: String,
  position: SquareCoord<FourConnected>,
}

#[derive(Debug, Clone)]
struct AchievementUnlocked {
  achievement_id: String,
  points: u32,
}

fn basic_events_demonstrate(event_bus: &mut EventBus)
{
  // === BASIC SUBSCRIPTION DEMONSTRATION ===
  println!("\n📡 Basic Event Subscription");
  println!("----------------------------");

  // Subscribe to game events with different priorities
  let game_log = Arc::new(Mutex::new(Vec::<String>::new()));
  let log_clone = game_log.clone();

  // Critical priority logger - logs everything first
  event_bus.subscribe_with_priority(move |event: &GameStarted| {
  log_clone.lock().unwrap().push(format!("CRITICAL: Game started - Level: {}, Difficulty: {}", event.level_id, event.difficulty));
  EventResult::Continue
  }, EventPriority::Critical);

  // High priority achievement system
  event_bus.subscribe_with_priority(move |event: &AchievementUnlocked| {
  println!("🏆 Achievement Unlocked: {} (+{} points)", event.achievement_id, event.points);
  EventResult::Continue
  }, EventPriority::High);

  // Normal priority UI updates
  let ui_updates = Arc::new(Mutex::new(0u32));
  let ui_counter = ui_updates.clone();
  event_bus.subscribe(move |_event: &EntityMoved<SquareCoord<FourConnected>>| {
  *ui_counter.lock().unwrap() += 1;
  EventResult::Continue
  });

  // Low priority analytics
  event_bus.subscribe_with_priority(move |_: &GameStarted| {
  println!("📊 Analytics: Recording game session...");
  EventResult::Continue
  }, EventPriority::Low);

  // === EVENT PUBLISHING DEMONSTRATION ===
  println!("\n📤 Publishing Events");
  println!("--------------------");

  // Publish a game start event
  event_bus.publish(GameStarted {
  level_id: "forest_1".to_string(),
  difficulty: 3,
  });

  // Publish some movement events
  for i in 1..=5 {
  event_bus.publish(EntityMoved {
    entity_id: 42,
    from: SquareCoord::<FourConnected>::new(i, i),
    to: SquareCoord::<FourConnected>::new(i + 1, i + 1),
    movement_type: MovementType::Walk,
  });
  }

  // Publish an achievement event
  event_bus.publish(AchievementUnlocked {
  achievement_id: "first_steps".to_string(),
  points: 100,
  });

  println!("📊 Events published: {}", event_bus.statistics().events_published);
  println!("📊 Pending events: {}", event_bus.total_pending_count());

  // Process all events
  println!("\n⚡ Processing Events");
  println!("-------------------");
  event_bus.events_process();

  println!("📊 Events processed: {}", event_bus.statistics().events_processed);
  println!("📊 UI updates received: {}", *ui_updates.lock().unwrap());

  // Check game log
  let log_entries = game_log.lock().unwrap();
  println!("📝 Game log entries: {}", log_entries.len());
  for entry in log_entries.iter() {
  println!("  - {entry}");
  }
  drop(log_entries); // Release lock
}

fn combat_events_demonstrate(event_bus: &mut EventBus)
{
  // === HEALTH AND COMBAT EVENTS ===
  println!("\n⚔️ Combat Event System");
  println!("----------------------");

  let combat_log = Arc::new(Mutex::new(Vec::<String>::new()));

  // Subscribe to health changes
  let health_logger = combat_log.clone();
  event_bus.subscribe(move |event: &HealthChanged| {
  let change = event.new_health - event.old_health;
  let change_text = if change > 0 { "gained" } else { "lost" };
  health_logger.lock().unwrap().push(
    format!("Entity {} {} {} health ({} -> {})",
      event.entity_id, change_text, change.abs(),
      event.old_health, event.new_health)
  );
  EventResult::Continue
  });

  // Subscribe to collision events
  let collision_logger = combat_log.clone();
  event_bus.subscribe(move |event: &EntitiesCollided<SquareCoord<FourConnected>>| {
  collision_logger.lock().unwrap().push(
    format!("Collision between entities {} and {} at ({}, {})",
      event.entity1, event.entity2,
      event.position.x, event.position.y)
  );
  EventResult::Continue
  });

  // Subscribe to spell cast events
  let spell_logger = combat_log.clone();
  event_bus.subscribe(move |event: &SpellCast<SquareCoord<FourConnected>>| {
  spell_logger.lock().unwrap().push(
    format!("Entity {} cast spell {} (cost: {})",
      event.caster_id, event.spell_id, event.cost)
  );
  EventResult::Continue
  });

  // Simulate combat scenario
  println!("🎬 Simulating combat scenario...");

  // Spell casting
  event_bus.publish(SpellCast {
  caster_id: 1,
  spell_id: 101, // Fireball
  target_position: Some(SquareCoord::<FourConnected>::new(5, 5)),
  target_entity: Some(2),
  cost: 25,
  });

  // Collision between entities
  event_bus.publish(EntitiesCollided {
  entity1: 1,
  entity2: 2,
  position: SquareCoord::<FourConnected>::new(5, 5),
  collision_type: CollisionType::Projectile { damage: 30 },
  });

  // Health change from damage
  event_bus.publish(HealthChanged {
  entity_id: 2,
  old_health: 100,
  new_health: 70,
  cause: HealthChangeCause::Damage { attacker_id: Some(1) },
  });

  // Healing
  event_bus.publish(HealthChanged {
  entity_id: 2,
  old_health: 70,
  new_health: 85,
  cause: HealthChangeCause::Healing {
    source: HealingSource::Item { item_id: 201 }
  },
  });

  // Process combat events
  event_bus.events_process();

  println!("\n📜 Combat Log:");
  let combat_entries = combat_log.lock().unwrap();
  for entry in combat_entries.iter() {
  println!("  • {entry}");
  }
  drop(combat_entries);
}

fn event_consumption_demonstrate(event_bus: &mut EventBus)
{
  // === EVENT CONSUMPTION DEMONSTRATION ===
  println!("\n🔄 Event Consumption");
  println!("--------------------");

  let consume_count = Arc::new(Mutex::new(0u32));

  // First listener consumes every 3rd player death event
  let counter1 = consume_count.clone();
  event_bus.subscribe(move |event: &PlayerDied| {
  let mut count = counter1.lock().unwrap();
  *count += 1;
  println!("🔥 Player {} died at ({}, {}): {} (event #{})", event.player_id, event.position.x, event.position.y, event.cause, *count);

  if (*count).is_multiple_of(3) {
    println!("💀 Consumed death event #{}!", *count);
    EventResult::Consume
  } else {
    EventResult::Continue
  }
  });

  // Second listener should miss every 3rd event
  let respawn_count = Arc::new(Mutex::new(0u32));
  let respawn_counter = respawn_count.clone();
  event_bus.subscribe(move |event: &PlayerDied| {
  *respawn_counter.lock().unwrap() += 1;
  println!("🔄 Respawning player {} at safe location", event.player_id);
  EventResult::Continue
  });

  // Publish several player death events
  for i in 1..=7 {
  let coord = i32::try_from(i).unwrap_or(i32::MAX);
  event_bus.publish(PlayerDied {
    player_id: i,
    cause: format!("Defeated by enemy #{i}"),
    position: SquareCoord::<FourConnected>::new(coord, coord),
  });
  }

  event_bus.events_process();

  println!("📊 Death events processed: {}", *consume_count.lock().unwrap());
  println!("📊 Respawn events processed: {}", *respawn_count.lock().unwrap());
}

fn auto_unsubscribe_demonstrate(event_bus: &mut EventBus)
{
  // === AUTO-UNSUBSCRIBE DEMONSTRATION ===
  println!("\n🔄 Auto-Unsubscribe");
  println!("-------------------");

  let execution_count = Arc::new(Mutex::new(0u32));
  let exec_counter = execution_count.clone();

  // Subscribe with auto-unsubscribe after 2 executions
  event_bus.subscribe(move |event: &AchievementUnlocked| {
  let mut count = exec_counter.lock().unwrap();
  *count += 1;
  println!("🎯 Temporary achievement handler #{}: {}", *count, event.achievement_id);

  if *count >= 2 {
    println!("✅ Achievement handler unsubscribing itself");
    EventResult::Unsubscribe
  } else {
    EventResult::Continue
  }
  });

  // Publish achievements to test auto-unsubscribe
  for i in 1..=4 {
  event_bus.publish(AchievementUnlocked {
    achievement_id: format!("temp_achievement_{i}"),
    points: i * 10,
  });

  event_bus.events_process();
  println!("  Active achievement subscribers: {}",
    event_bus.subscriber_count::<AchievementUnlocked>());
  }
}

fn batch_publishing_demonstrate(event_bus: &mut EventBus)
{
  // === BATCH PUBLISHING ===
  println!("\n📦 Batch Publishing");
  println!("------------------");

  let batch_events = vec![
  EntityMoved {
    entity_id: 100,
    from: SquareCoord::<FourConnected>::new(0, 0),
    to: SquareCoord::<FourConnected>::new(1, 0),
    movement_type: MovementType::Walk,
  },
  EntityMoved {
    entity_id: 100,
    from: SquareCoord::<FourConnected>::new(1, 0),
    to: SquareCoord::<FourConnected>::new(2, 0),
    movement_type: MovementType::Run,
  },
  EntityMoved {
    entity_id: 100,
    from: SquareCoord::<FourConnected>::new(2, 0),
    to: SquareCoord::<FourConnected>::new(2, 1),
    movement_type: MovementType::Fly,
  },
  ];

  println!("📤 Publishing batch of {} movement events", batch_events.len());
  event_bus.batch_publish(batch_events);
  event_bus.events_process();
}

fn game_state_events_demonstrate(event_bus: &mut EventBus)
{
  // === GAME STATE EVENTS ===
  println!("\n🎮 Game State Management");
  println!("------------------------");

  // Subscribe to game state changes
  event_bus.subscribe(move |event: &GameStateChanged| {
  println!("🔄 Game state: {:?} -> {:?} ({})",
    event.old_state, event.new_state, event.reason);
  EventResult::Continue
  });

  // Simulate game state transitions
  let state_transitions = vec![
  (GameState::Initializing, GameState::MainMenu, "Initialization complete"),
  (GameState::MainMenu, GameState::Loading, "Player started new game"),
  (GameState::Loading, GameState::Playing, "Level loaded successfully"),
  (GameState::Playing, GameState::Paused, "Player pressed pause"),
  (GameState::Paused, GameState::Playing, "Player resumed game"),
  (GameState::Playing, GameState::GameOver, "Player health reached zero"),
  ];

  for (old_state, new_state, reason) in state_transitions {
  event_bus.publish(GameStateChanged {
    old_state,
    new_state,
    reason: reason.to_string(),
  });
  }

  event_bus.events_process();
}

fn final_statistics_print(event_bus: &EventBus)
{
  // === PERFORMANCE STATISTICS ===
  println!("\n📊 Final Statistics");
  println!("-------------------");
  let stats = event_bus.statistics();
  println!("Total events published: {}", stats.events_published);
  println!("Total events processed: {}", stats.events_processed);
  println!("Processing cycles: {}", stats.process_cycles);
  println!("Average events per cycle: {:.2}", stats.average_events_per_cycle());
  println!("Processing efficiency: {:.2}%", stats.processing_efficiency() * 100.0);
  println!("Active subscribers: {}", stats.total_subscribers);
  println!("Event channels: {}", event_bus.channel_count());

  println!("\n✨ Event System Demo Complete!");
  println!("\nKey features demonstrated:");
  println!("• Event publishing and subscription");
  println!("• Priority-based event processing");
  println!("• Event consumption and filtering");
  println!("• Auto-unsubscribing listeners");
  println!("• Batch event publishing");
  println!("• Common game events (movement, health, collisions)");
  println!("• Performance monitoring and statistics");
  println!("• Type-safe event channels");
}

fn main()
{
  println!("🎯 Event System Demonstration");
  println!("=============================");

  let mut event_bus = EventBus::new();

  basic_events_demonstrate(&mut event_bus);
  combat_events_demonstrate(&mut event_bus);
  event_consumption_demonstrate(&mut event_bus);
  auto_unsubscribe_demonstrate(&mut event_bus);
  batch_publishing_demonstrate(&mut event_bus);
  game_state_events_demonstrate(&mut event_bus);
  final_statistics_print(&event_bus);
}