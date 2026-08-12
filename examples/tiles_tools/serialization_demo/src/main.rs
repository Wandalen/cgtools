//! Serialization system demonstration showing save/load functionality.

//!
//! This example demonstrates the comprehensive serialization system including:
//! - Game state serialization and deserialization
//! - Multiple serialization formats (JSON, Binary, RON)
//! - Configuration management and persistence
//! - Save file management with metadata
//! - Version compatibility checking
//! - Compression support

use tiles_tools::serialization::{GameStateSerializer, Achievement, SerializationFormat, SaveManager, SaveVersion, ConfigManager, GameConfig, PlayerProgress, SerializableGameState};

fn basic_game_state_build() -> SerializableGameState
{
  // === BASIC SERIALIZATION ===
  println!("\n📄 Basic Game State Serialization");
  println!("----------------------------------");

  // Create a basic game state
  let mut game_state = GameStateSerializer::create_basic_game_state("My First Save".to_string());

  // Add some custom data
  game_state.progress.level = 5;
  game_state.progress.experience = 2500;
  game_state.progress.playtime_seconds = 3600; // 1 hour
  game_state.progress.levels_completed.push("tutorial".to_string());
  game_state.progress.levels_completed.push("forest_1".to_string());

  // Add an achievement
  game_state.progress.achievements.push(Achievement {
  id: "first_level".to_string(),
  name: "First Steps".to_string(),
  description: "Complete your first level".to_string(),
  unlocked_at: 1_234_567_890,
  points: 50,
  });

  // Update game statistics
  game_state.progress.statistics.entities_defeated = 25;
  game_state.progress.statistics.distance_moved = 1500.0;
  game_state.progress.statistics.items_collected = 12;

  // Add custom metadata tags
  game_state.metadata = game_state.metadata
  .with_tag("demo".to_string())
  .with_tag("tutorial_complete".to_string())
  .with_custom("difficulty".to_string(), "normal".to_string())
  .with_custom("character_class".to_string(), "warrior".to_string());

  println!("Created game state:");
  println!("  Player Level: {}", game_state.progress.level);
  println!("  Experience: {}", game_state.progress.experience);
  println!("  Playtime: {}h {}m",
  game_state.progress.playtime_seconds / 3600,
  (game_state.progress.playtime_seconds % 3600) / 60);
  println!("  Achievements: {}", game_state.progress.achievements.len());
  println!("  Levels Completed: {}", game_state.progress.levels_completed.len());

  game_state
}

fn serialization_formats_demonstrate(game_state: &SerializableGameState)
{
  // === MULTIPLE FORMATS DEMONSTRATION ===
  println!("\n🔄 Multiple Serialization Formats");
  println!("----------------------------------");

  // Test JSON serialization
  let json_serializer = GameStateSerializer::new()
  .with_format(SerializationFormat::Json);

  let json_data = json_serializer.serialize_game_state(game_state)
  .expect("Failed to serialize to JSON");
  println!("JSON serialization: {} bytes", json_data.len());

  // Test Binary serialization
  let binary_serializer = GameStateSerializer::new()
  .with_format(SerializationFormat::Binary);

  let binary_data = binary_serializer.serialize_game_state(game_state)
  .expect("Failed to serialize to binary");
  println!("Binary serialization: {} bytes", binary_data.len());

  // Test RON serialization
  let ron_serializer = GameStateSerializer::new()
  .with_format(SerializationFormat::Ron);

  let ron_data = ron_serializer.serialize_game_state(game_state)
  .expect("Failed to serialize to RON");
  println!("RON serialization: {} bytes", ron_data.len());

  // Verify deserialization works
  let json_restored = json_serializer.deserialize_game_state(&json_data)
  .expect("Failed to deserialize JSON");
  let binary_restored = binary_serializer.deserialize_game_state(&binary_data)
  .expect("Failed to deserialize binary");
  let ron_restored = ron_serializer.deserialize_game_state(&ron_data)
  .expect("Failed to deserialize RON");

  println!("✅ All formats successfully roundtrip serialized");
  println!("  JSON player level: {}", json_restored.progress.level);
  println!("  Binary player level: {}", binary_restored.progress.level);
  println!("  RON player level: {}", ron_restored.progress.level);
}

fn compression_demonstrate(game_state: &SerializableGameState)
{
  // === COMPRESSION DEMONSTRATION ===
  println!("\n🗜️ Compression");
  println!("--------------");

  let uncompressed_serializer = GameStateSerializer::new()
  .with_compression(false);
  let compressed_serializer = GameStateSerializer::new()
  .with_compression(true);

  let uncompressed = uncompressed_serializer.serialize_game_state(game_state)
  .expect("Failed to serialize uncompressed");
  let compressed = compressed_serializer.serialize_game_state(game_state)
  .expect("Failed to serialize compressed");

  println!("Uncompressed size: {} bytes", uncompressed.len());
  println!("Compressed size: {} bytes", compressed.len());
  let ratio = if compressed.len() < uncompressed.len() {
  let saved = u32::try_from(uncompressed.len() - compressed.len()).unwrap_or(u32::MAX);
  let total = u32::try_from(uncompressed.len()).unwrap_or(u32::MAX);
  f64::from(saved) / f64::from(total) * 100.0
  } else {
  // No compression achieved (mock compression adds overhead)
  let overhead = u32::try_from(compressed.len() - uncompressed.len()).unwrap_or(u32::MAX);
  let total = u32::try_from(uncompressed.len()).unwrap_or(u32::MAX);
  -(f64::from(overhead) / f64::from(total) * 100.0)
  };
  println!("Compression ratio: {ratio:.1}%");

  // Verify compressed data can be decompressed
  let decompressed = compressed_serializer.deserialize_game_state(&compressed)
  .expect("Failed to decompress data");
  println!("✅ Compression/decompression successful");
  println!("  Restored player level: {}", decompressed.progress.level);
}

fn save_manager_demonstrate(game_state: &SerializableGameState, temp_dir: &std::path::Path) -> SaveManager
{
  // === SAVE MANAGER DEMONSTRATION ===
  println!("\n💾 Save Manager");
  println!("---------------");

  let save_manager = SaveManager::new(temp_dir)
  .with_serializer(GameStateSerializer::new().with_compression(true));

  // Save the game state
  save_manager.save_game_state("demo_save", game_state)
  .expect("Failed to save game state");
  println!("✅ Game saved as 'demo_save'");

  // Create additional saves with different states
  let mut quick_save = game_state.clone();
  quick_save.metadata.description = "Quick Save - Before Boss Fight".to_string();
  quick_save.progress.level = 6;
  quick_save.progress.experience = 3000;

  save_manager.save_game_state("quick_save", &quick_save)
  .expect("Failed to save quick save");

  let mut checkpoint = game_state.clone();
  checkpoint.metadata.description = "Checkpoint - Forest Entry".to_string();
  checkpoint.progress.level = 4;
  checkpoint.progress.experience = 1800;
  checkpoint.metadata = checkpoint.metadata.with_tag("checkpoint".to_string());

  save_manager.save_game_state("checkpoint_1", &checkpoint)
  .expect("Failed to save checkpoint");

  // List all saves
  println!("\n📂 Available Saves:");
  let saves = save_manager.list_saves()
  .expect("Failed to list saves");
  for save_name in &saves {
  println!("  - {save_name}");
  }

  // Get detailed save information
  println!("\n📊 Save Information:");
  let saves_info = save_manager.get_saves_info()
  .expect("Failed to get saves info");

  for (name, metadata) in &saves_info {
  let _created_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(metadata.created_at);
  println!("  Save: {name}");
  println!("    Description: {}", metadata.description);
  println!("    Size: {} bytes", metadata.size_bytes);
  println!("    Version: {}.{}.{}", metadata.version.major, metadata.version.minor, metadata.version.patch);
  println!("    Compressed: {}", metadata.compressed);
  println!("    Tags: {:?}", metadata.tags);
  if !metadata.custom.is_empty() {
    println!("    Custom data: {:?}", metadata.custom);
  }
  println!();
  }

  // Load a specific save
  println!("🔄 Loading save 'demo_save'...");
  let loaded_state = save_manager.load_game_state("demo_save")
  .expect("Failed to load save");

  println!("✅ Save loaded successfully:");
  println!("  Player Level: {}", loaded_state.progress.level);
  println!("  Experience: {}", loaded_state.progress.experience);
  println!("  Description: {}", loaded_state.metadata.description);
  println!("  Achievements: {}", loaded_state.progress.achievements.len());

  save_manager
}

fn version_compatibility_demonstrate()
{
  // === VERSION COMPATIBILITY ===
  println!("\n🔄 Version Compatibility");
  println!("------------------------");

  let current_version = SaveVersion::current();
  let older_version = SaveVersion::new(1, 0, 0);
  let newer_version = SaveVersion::new(1, 1, 0);
  let incompatible_version = SaveVersion::new(2, 0, 0);

  println!("Current version: {}.{}.{}", current_version.major, current_version.minor, current_version.patch);
  println!("Older version compatibility: {}", current_version.is_compatible_with(&older_version));
  println!("Newer version compatibility: {}", older_version.is_compatible_with(&newer_version));
  println!("Incompatible version compatibility: {}", current_version.is_compatible_with(&incompatible_version));
}

fn configuration_demonstrate(temp_dir: &std::path::Path)
{
  // === CONFIGURATION MANAGEMENT ===
  println!("\n⚙️ Configuration Management");
  println!("---------------------------");

  let config_path = temp_dir.join("game_config.json");
  let config_manager = ConfigManager::new(&config_path);

  // Create custom configuration
  let mut custom_config = GameConfig::default();
  custom_config.difficulty = 3;
  custom_config.graphics.resolution_width = 2560;
  custom_config.graphics.resolution_height = 1440;
  custom_config.graphics.quality_level = 3;
  custom_config.audio.master_volume = 0.8;
  custom_config.gameplay.auto_save_interval = 600; // 10 minutes

  // Add custom keybindings
  custom_config.controls.key_bindings.insert("sprint".to_string(), "Shift".to_string());
  custom_config.controls.key_bindings.insert("inventory".to_string(), "Tab".to_string());

  // Save configuration
  config_manager.save_config(&custom_config)
  .expect("Failed to save configuration");
  println!("✅ Configuration saved");

  // Load configuration
  let loaded_config = config_manager.load_config()
  .expect("Failed to load configuration");
  println!("✅ Configuration loaded:");
  println!("  Difficulty: {}", loaded_config.difficulty);
  println!("  Resolution: {}x{}", loaded_config.graphics.resolution_width, loaded_config.graphics.resolution_height);
  println!("  Master Volume: {}", loaded_config.audio.master_volume);
  println!("  Auto-save Interval: {}s", loaded_config.gameplay.auto_save_interval);
  println!("  Key bindings: {}", loaded_config.controls.key_bindings.len());
}

fn player_progress_demonstrate()
{
  // === PLAYER PROGRESS TRACKING ===
  println!("\n👤 Player Progress Tracking");
  println!("---------------------------");

  let mut progress = PlayerProgress::default();

  // Simulate some gameplay progress
  progress.level = 8;
  progress.experience = 5500;
  progress.playtime_seconds = 7200; // 2 hours

  // Add completed levels
  progress.levels_completed.extend([
  "tutorial".to_string(),
  "forest_1".to_string(),
  "forest_2".to_string(),
  "caves_1".to_string(),
  ]);

  // Add achievements
  progress.achievements.extend([
  Achievement {
    id: "level_5".to_string(),
    name: "Experienced".to_string(),
    description: "Reach level 5".to_string(),
    unlocked_at: 1_234_567_890,
    points: 100,
  },
  Achievement {
    id: "cave_explorer".to_string(),
    name: "Cave Explorer".to_string(),
    description: "Complete the cave levels".to_string(),
    unlocked_at: 1_234_568_000,
    points: 150,
  },
  ]);

  // Update statistics
  progress.statistics.entities_defeated = 150;
  progress.statistics.distance_moved = 5000.0;
  progress.statistics.items_collected = 45;
  progress.statistics.spells_cast = 80;
  progress.statistics.deaths = 3;
  progress.statistics.levels_completed_count = 4;

  // Serialize progress
  let _progress_json = serde_json::to_string_pretty(&progress)
  .expect("Failed to serialize progress");

  println!("Player Progress Summary:");
  println!("  Level: {}", progress.level);
  println!("  Experience: {}", progress.experience);
  println!("  Playtime: {}h {}m", progress.playtime_seconds / 3600, (progress.playtime_seconds % 3600) / 60);
  println!("  Levels Completed: {}", progress.levels_completed.len());
  println!("  Achievements: {} (total {} points)",
  progress.achievements.len(),
  progress.achievements.iter().map(|a| a.points).sum::<u32>());
  println!("  Statistics:");
  println!("    Entities Defeated: {}", progress.statistics.entities_defeated);
  println!("    Distance Moved: {:.1}m", progress.statistics.distance_moved);
  println!("    Items Collected: {}", progress.statistics.items_collected);
  println!("    Spells Cast: {}", progress.statistics.spells_cast);
  println!("    Deaths: {}", progress.statistics.deaths);
}

fn cleanup_demonstrate(save_manager: &SaveManager, temp_dir: &std::path::Path)
{
  // === CLEANUP DEMONSTRATION ===
  println!("\n🧹 Cleanup");
  println!("----------");

  // Demonstrate save deletion
  save_manager.delete_save("checkpoint_1")
  .expect("Failed to delete save");
  println!("✅ Deleted checkpoint_1 save");

  let remaining_saves = save_manager.list_saves()
  .expect("Failed to list remaining saves");
  println!("Remaining saves: {remaining_saves:?}");

  // Clean up temporary directory
  std::fs::remove_dir_all(temp_dir).ok();
  println!("✅ Cleaned up temporary files");
}

fn performance_demonstrate(game_state: &SerializableGameState)
{
  // === PERFORMANCE DEMONSTRATION ===
  println!("\n⚡ Performance Test");
  println!("------------------");

  let mut large_state = game_state.clone();

  // Add lots of custom data to simulate a large game state
  for i in 0..100 {
  large_state.custom_data.insert(
    format!("large_data_{i}"),
    vec![0u8; 1024] // 1KB per entry
  );
  }

  let start_time = std::time::Instant::now();
  let serialized_large = GameStateSerializer::new()
  .with_compression(true)
  .serialize_game_state(&large_state)
  .expect("Failed to serialize large state");
  let serialize_duration = start_time.elapsed();

  let start_time = std::time::Instant::now();
  let _deserialized_large = GameStateSerializer::new()
  .with_compression(true)
  .deserialize_game_state(&serialized_large)
  .expect("Failed to deserialize large state");
  let deserialize_duration = start_time.elapsed();

  println!("Large game state performance:");
  println!("  Size: {} KB", serialized_large.len() / 1024);
  println!("  Serialize time: {:.2}ms", serialize_duration.as_secs_f64() * 1000.0);
  println!("  Deserialize time: {:.2}ms", deserialize_duration.as_secs_f64() * 1000.0);
}

fn main()
{
  println!("💾 Serialization System Demonstration");
  println!("=====================================");

  let game_state = basic_game_state_build();

  serialization_formats_demonstrate(&game_state);
  compression_demonstrate(&game_state);

  // Create a temporary directory for saves (in real usage, this would be a persistent directory)
  let temp_dir = std::env::temp_dir().join("tiles_tools_demo_saves");
  std::fs::create_dir_all(&temp_dir).expect("Failed to create saves directory");

  let save_manager = save_manager_demonstrate(&game_state, &temp_dir);

  version_compatibility_demonstrate();
  configuration_demonstrate(&temp_dir);
  player_progress_demonstrate();
  cleanup_demonstrate(&save_manager, &temp_dir);
  performance_demonstrate(&game_state);

  println!("\n✨ Serialization Demo Complete!");
  println!("\nKey features demonstrated:");
  println!("• Multiple serialization formats (JSON, Binary, RON)");
  println!("• Compression support with size reduction");
  println!("• Save file management with metadata");
  println!("• Configuration persistence");
  println!("• Player progress tracking");
  println!("• Version compatibility checking");
  println!("• Error handling and recovery");
  println!("• Performance optimization");
}