//! Tests for the `serialization` module — save-version compatibility, metadata
//! builders, config round-trips, game-state serialization with and without
//! compression, save/config managers on real temp directories, player progress,
//! and error variants, driven purely through the public surface.
//!
//! Relocated from `src/serialization.rs` by task 072 (bodies verbatim; an unused
//! `use std::fs;` from the inline module was dropped).

#![ cfg( feature = "serialization" ) ]


use tiles_tools::serialization::*;
use tempfile::TempDir;

#[test]
fn test_save_version_compatibility() {
  let v1 = SaveVersion::new(1, 0, 0);
  let v2 = SaveVersion::new(1, 1, 0);
  let v3 = SaveVersion::new(2, 0, 0);

  assert!(v2.is_compatible_with(&v1)); // v1.1 can read v1.0
  assert!(!v1.is_compatible_with(&v2)); // v1.0 cannot read v1.1
  assert!(!v3.is_compatible_with(&v1)); // Major version difference
}

#[test]
fn test_save_metadata_creation() {
  let metadata = SaveMetadata::new("Test Save".to_string())
    .with_tag("level1".to_string())
    .with_custom("difficulty".to_string(), "hard".to_string())
    .with_compression(true);

  assert_eq!(metadata.description, "Test Save");
  assert!(metadata.tags.contains(&"level1".to_string()));
  assert_eq!(metadata.custom.get("difficulty"), Some(&"hard".to_string()));
  assert!(metadata.compressed);
}

#[test]
fn test_game_config_serialization() {
  let config = GameConfig::default();
  let json = serde_json::to_string(&config).unwrap();
  let deserialized: GameConfig = serde_json::from_str(&json).unwrap();

  assert_eq!(config.difficulty, deserialized.difficulty);
  assert_eq!(config.graphics.resolution_width, deserialized.graphics.resolution_width);
}

#[test]
fn test_game_state_serializer() {
  let serializer = GameStateSerializer::new()
    .with_format(SerializationFormat::Json)
    .with_compression(false);

  let game_state = GameStateSerializer::basic_game_state_create("Test Game".to_string());

  let bytes_out = serializer.game_state_serialize(&game_state).unwrap();
  let restored = serializer.game_state_deserialize(&bytes_out).unwrap();

  assert_eq!(game_state.metadata.description, restored.metadata.description);
  assert_eq!(game_state.world_data.len(), restored.world_data.len());
}

#[test]
fn test_compression() {
  let serializer = GameStateSerializer::new()
    .with_compression(true);

  let game_state = GameStateSerializer::basic_game_state_create("Compression Test".to_string());

  let compressed = serializer.game_state_serialize(&game_state).unwrap();
  let decompressed = serializer.game_state_deserialize(&compressed).unwrap();

  assert_eq!(game_state.metadata.description, decompressed.metadata.description);
}

#[test]
fn test_save_manager() {
  let temp_dir = TempDir::new().unwrap();
  let save_manager = SaveManager::new(temp_dir.path());

  let game_state = GameStateSerializer::basic_game_state_create("Test Save".to_string());

  // Save the game state
  save_manager.game_state_save("test_save", &game_state).unwrap();

  // Load it back
  let loaded_state = save_manager.game_state_load("test_save").unwrap();
  assert_eq!(game_state.metadata.description, loaded_state.metadata.description);

  // Test listing saves
  let saves = save_manager.saves_list().unwrap();
  assert!(saves.contains(&"test_save".to_string()));

  // Test metadata loading
  let metadata = save_manager.save_metadata_load("test_save").unwrap();
  assert_eq!(metadata.description, "Test Save");

  // Test saves info
  let saves_info = save_manager.saves_info_get().unwrap();
  assert_eq!(saves_info.len(), 1);
  assert_eq!(saves_info[0].0, "test_save");

  // Test deletion
  save_manager.save_delete("test_save").unwrap();
  let saves_after_delete = save_manager.saves_list().unwrap();
  assert!(!saves_after_delete.contains(&"test_save".to_string()));
}

#[test]
fn test_config_manager() {
  let temp_dir = TempDir::new().unwrap();
  let config_path = temp_dir.path().join("config.json");
  let config_manager = ConfigManager::new(&config_path);

  // Save default config
  let config = GameConfig::default();
  config_manager.config_save(&config).unwrap();

  // Load it back
  let loaded_config = config_manager.config_load().unwrap();
  assert_eq!(config.difficulty, loaded_config.difficulty);

  // Test reset
  config_manager.config_reset().unwrap();
  let reset_config = config_manager.config_load().unwrap();
  assert_eq!(reset_config.difficulty, 1);
}

#[test]
fn test_player_progress_serialization() {
  let mut progress = PlayerProgress {
    level: 5,
    experience: 1500,
    ..Default::default()
  };
  progress.achievements.push(Achievement {
    id: "first_kill".to_string(),
    name: "First Kill".to_string(),
    description: "Defeat your first enemy".to_string(),
    unlocked_at: 1_234_567_890,
    points: 10,
  });

  let json = serde_json::to_string(&progress).unwrap();
  let deserialized: PlayerProgress = serde_json::from_str(&json).unwrap();

  assert_eq!(progress.level, deserialized.level);
  assert_eq!(progress.achievements.len(), deserialized.achievements.len());
  assert_eq!(progress.achievements[0].id, deserialized.achievements[0].id);
}

// test_kind: bug_reproducer(BUG-269)
/// ## Root Cause
/// `GameStateSerializer::game_state_deserialize` never compared the
/// deserialized state's `metadata.version` against `self.version` (default
/// `SaveVersion::current()`, or a caller-supplied one via `with_version`).
/// `SaveVersion::is_compatible_with` and `SerializationError::IncompatibleVersion`
/// were both fully implemented, but nothing ever called the former or
/// constructed the latter.
///
/// ## Why Not Caught
/// `test_save_version_compatibility` only tests `SaveVersion::is_compatible_with`
/// in isolation. Every round-trip test in this file uses the default
/// `GameStateSerializer::new()` (current version) against
/// `basic_game_state_create` (also current version), so a version mismatch
/// was never exercised end-to-end through `game_state_deserialize`.
///
/// ## Fix Applied
/// Added a version-compatibility check in `game_state_deserialize`, after
/// deserializing but before returning the state: if
/// `self.version.is_compatible_with(&state.metadata.version)` is false,
/// return `SerializationError::IncompatibleVersion` instead of the state.
///
/// ## Prevention
/// A builder setter (`with_version`) whose stored value is never read back
/// anywhere is a silent no-op -- grep for every `self.<field>` read site
/// when adding a `with_*` method, not just the field's own declaration.
///
/// ## Pitfall
/// A fully-implemented, unit-tested helper (`is_compatible_with`) and a
/// fully-defined, `Display`-formatted error variant (`IncompatibleVersion`)
/// can coexist in a crate for a long time without ever being wired together
/// at the one call site that would actually use them -- "the pieces exist"
/// is not evidence that "the feature works end to end".
#[test]
fn test_deserialize_rejects_incompatible_major_version() {
  let writer = GameStateSerializer::new();
  let game_state = GameStateSerializer::basic_game_state_create("Version Test".to_string());
  let bytes = writer.game_state_serialize(&game_state).unwrap();

  let reader = GameStateSerializer::new().with_version(SaveVersion::new(2, 0, 0));
  let result = reader.game_state_deserialize(&bytes);

  assert!(
    matches!(result, Err(SerializationError::IncompatibleVersion { .. })),
    "expected IncompatibleVersion, got {result:?}"
  );
}

#[test]
fn test_error_handling() {
  let temp_dir = TempDir::new().unwrap();
  let save_manager = SaveManager::new(temp_dir.path());

  // Test loading non-existent save
  let result = save_manager.game_state_load("nonexistent");
  assert!(matches!(result, Err(SerializationError::SaveNotFound(_))));

  // Test loading non-existent metadata
  let result = save_manager.save_metadata_load("nonexistent");
  assert!(matches!(result, Err(SerializationError::MetadataNotFound(_))));
}
