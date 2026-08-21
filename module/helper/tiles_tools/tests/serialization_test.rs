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

// BUG-348 task/bug/348_save_manager_meta_compressed_flag_desync.md -- reproducer
// for the .meta sidecar's compressed flag desync.
// test_kind: bug_reproducer(BUG-348)
/// ## Root Cause
/// `SaveManager::game_state_save` clones `state.metadata` as-is and writes it
/// to the `.meta` sidecar, but never sets `metadata.compressed =
/// self.serializer.compress` before writing it. `GameStateSerializer.compress`
/// (which actually controls whether the `.save` file is compressed) and
/// `SaveMetadata.compressed` (an independent field, set only via the
/// unrelated `SaveMetadata::with_compression` builder) are never
/// synchronized, so the `.meta` file's `compressed` flag can silently
/// disagree with the real compression state of the `.save` file it describes.
/// ## Why Not Caught
/// `test_compression` only round-trips through `game_state_serialize`/
/// `game_state_deserialize` directly -- it never goes through `SaveManager`
/// or reads the `.meta` sidecar back with `save_metadata_load`, so the
/// desynchronized flag was never observed.
/// ## Fix Applied
/// `game_state_save` now sets `metadata.compressed = self.serializer.compress`
/// on the cloned metadata before serializing and writing the `.meta` file.
/// ## Prevention
/// n/a -- covered by this test.
/// ## Pitfall
/// Two independently-settable fields that are supposed to describe the same
/// underlying fact (whether the save data is compressed) will drift apart
/// silently unless one is derived from the other at the one place they are
/// both written -- `game_state_load` happens to still round-trip correctly
/// here because it consults the serializer's own `compress` field, not the
/// metadata, which is exactly what let this desync stay invisible.
#[test]
fn test_game_state_save_meta_compressed_flag_matches_actual_compression() {
  let temp_dir = TempDir::new().unwrap();
  let save_manager = SaveManager::new(temp_dir.path())
    .with_serializer(GameStateSerializer::new().with_compression(true));

  let game_state = GameStateSerializer::basic_game_state_create("Compressed Save".to_string());
  assert!(
    !game_state.metadata.compressed,
    "fixture must start with metadata.compressed == false to exercise the desync"
  );

  save_manager.game_state_save("compressed_save", &game_state).unwrap();

  let loaded_metadata = save_manager.save_metadata_load("compressed_save").unwrap();
  assert!(
    loaded_metadata.compressed,
    "the .save file is actually compressed (serializer.compress == true), but the \
     .meta sidecar's compressed flag was not synchronized to match"
  );
}

// test_kind: bug_reproducer(BUG-475)
/// ## Root Cause
/// `GameStateSerializer::data_compress` never compressed anything -- it
/// prepended a 7-byte `"CMP"` marker + length header to the input bytes and
/// returned them unmodified, so `with_compression(true)` always produced
/// output 7 bytes LARGER than uncompressed, the opposite of what the name
/// and the module's documented "Compression Support" feature promise.
/// ## Why Not Caught
/// `test_compression` only asserts the compress/decompress round-trip
/// recovers the original description -- since the stub's compress and
/// decompress were exact mutual inverses (just neither one compressed
/// anything), the round-trip passed regardless of whether real compression
/// happened. No existing test compared compressed-vs-uncompressed output
/// size.
/// ## Fix Applied
/// `data_compress`/`data_decompress` now use `flate2`'s DEFLATE
/// encoder/decoder (`rust_backend`, no C toolchain / system zlib
/// dependency, confirmed wasm32-unknown-unknown compatible). The existing
/// 7-byte marker+header format is kept, but the header now stores the
/// DECOMPRESSED size (validated after inflating) instead of the
/// compressed-payload length (which was only ever meaningful when
/// "compression" was a no-op copy).
/// ## Prevention
/// A builder flag whose name promises a transformation (`with_compression`)
/// needs a test that checks the transformation actually happened (output
/// smaller than input for compressible data), not just that compress/
/// decompress remain inverses of each other -- a no-op stub trivially
/// satisfies round-trip correctness while completely failing its stated
/// purpose.
/// ## Pitfall
/// "The round trip still works" is not evidence a `with_*(true)` toggle
/// does anything at all -- an identity-function stub passes every
/// round-trip test a real implementation would, which is exactly what let
/// this ship silently.
#[test]
fn test_compression_actually_shrinks_compressible_data() {
  let game_state = GameStateSerializer::basic_game_state_create("Compression Size Test".to_string());

  let uncompressed = GameStateSerializer::new()
    .with_compression(false)
    .game_state_serialize(&game_state)
    .unwrap();
  let compressed = GameStateSerializer::new()
    .with_compression(true)
    .game_state_serialize(&game_state)
    .unwrap();

  assert!(
    compressed.len() < uncompressed.len(),
    "compressed output ({} bytes) must be smaller than uncompressed ({} bytes) for \
     `basic_game_state_create`'s highly-compressible 1024-zero-byte world_data; \
     the old stub always produced uncompressed_len + 7 bytes instead",
    compressed.len(),
    uncompressed.len()
  );

  // Round-trip correctness must still hold through the real compressor.
  let restored = GameStateSerializer::new()
    .with_compression(true)
    .game_state_deserialize(&compressed)
    .unwrap();
  assert_eq!(game_state.metadata.description, restored.metadata.description);
  assert_eq!(game_state.world_data, restored.world_data);
}

// test_kind: bug_reproducer(BUG-476)
/// ## Root Cause
/// `data_decompress`'s old corruption check, `data.len() != original_size +
/// 7`, added an attacker/corruption-controlled `u32` (`original_size`, cast
/// to `usize`) to a constant. On a 64-bit host this can never overflow
/// (`u32::MAX + 7` is nowhere near 64-bit `usize::MAX`), but on a 32-bit
/// target -- wasm32-unknown-unknown, this crate's stated primary target,
/// where `usize` is also 32 bits -- a crafted `original_size` near
/// `u32::MAX` makes `original_size + 7` wrap around, which can corrupt the
/// corruption check itself (either panicking via debug-mode
/// overflow-checks, or silently wrapping to a small value in release mode
/// and letting a bogus length check pass).
/// ## Why Not Caught
/// No existing test fed `data_decompress`/`game_state_deserialize` a
/// corrupted or adversarial header; all decompression tests only ever
/// round-tripped through `data_compress`'s own well-formed output. This bug
/// is also architecture-specific -- it cannot be triggered at all on the
/// 64-bit host this test suite actually runs on (see Pitfall), so no amount
/// of running `cargo test` here would ever have caught it either.
/// ## Fix Applied
/// Fixed structurally as a side effect of BUG-475's redesign, not via a
/// `checked_add` patch: the new `data_decompress` no longer performs any
/// arithmetic on the untrusted `original_size` at all. It inflates the
/// payload first, then compares the *inflated* length directly against
/// `original_size` (`decompressed.len() != original_size`) -- an equality
/// check between two already-bounded `usize` values, with no addition of
/// untrusted input in the vulnerable path.
/// ## Prevention
/// A length/bounds check written as `data.len() != untrusted_value +
/// constant` should be re-derived, not just re-verified, whenever the
/// surrounding data format changes -- the addition was only ever safe
/// because the old format guaranteed `untrusted_value` was small relative
/// to `data.len()`. Once real compression decoupled the on-disk length from
/// the original size, that guarantee silently stopped holding.
/// ## Pitfall
/// This overflow is real on the crate's stated wasm32-unknown-unknown
/// target but categorically unreachable on the x86_64 host this test runs
/// on -- `usize` is 64 bits here, so `u32::MAX + 7` never overflows
/// regardless of which code path runs. This test therefore cannot be a
/// classic "panics before, doesn't after" reproducer on this host for
/// either the old or the new code (both return `Err` here). Its value is
/// (a) permanent regression coverage that a hostile/corrupted header with
/// `original_size` near `u32::MAX` is always rejected gracefully via `Err`,
/// never a panic, on every target; and (b) documenting -- in case this
/// arithmetic is ever reintroduced -- that the overflow class is
/// architecture-specific and will NOT be caught by this repo's ordinary
/// x86_64 `cargo nextest` runs; a wasm32 target run (or a manual 32-bit
/// arithmetic audit, as performed for this fix) is required to verify that
/// class of fix directly.
#[test]
fn test_deserialize_rejects_corrupted_header_with_near_max_original_size() {
  // Marker + a header claiming a ~4GiB decompressed size, followed by a
  // short payload that is not a valid DEFLATE stream for that claim.
  let mut corrupted = vec![0xC0, 0x4D, 0x50];
  corrupted.extend_from_slice(&(u32::MAX - 1).to_le_bytes());
  corrupted.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

  let result = GameStateSerializer::new()
    .with_compression(true)
    .game_state_deserialize(&corrupted);

  assert!(
    matches!(result, Err(SerializationError::CorruptedData)),
    "a corrupted header claiming a near-u32::MAX original size must be rejected \
     cleanly as CorruptedData, never panic or succeed; got {result:?}"
  );
}
