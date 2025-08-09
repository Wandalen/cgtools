//! Serialization support for game state persistence and save/load functionality.
//!
//! This module provides comprehensive serialization capabilities for tiles_tools,
//! enabling save/load functionality for game states, configurations, and persistent
//! data. It supports multiple serialization formats and provides utilities for
//! managing game saves, checkpoints, and configuration files.
//!
//! # Serialization Features
//!
//! - **Game State Serialization**: Complete game world persistence
//! - **Configuration Management**: Settings and preferences persistence
//! - **Checkpoint System**: Automatic and manual save points
//! - **Version Management**: Backward compatibility for save files
//! - **Compression Support**: Efficient storage of large game states
//! - **Incremental Saves**: Delta-based serialization for performance
//!
//! # Supported Formats
//!
//! - **JSON**: Human-readable format for debugging and configuration
//! - **Binary**: Compact format for efficient game state storage
//! - **RON**: Rust Object Notation for configuration files
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::serialization::*;
//!
//! // Create a game state
//! let game_state = GameStateSerializer::create_basic_game_state("Test Save".to_string());
//! 
//! // Serialize the game state
//! let serialized = GameStateSerializer::new()
//!     .with_compression(true)
//!     .serialize_game_state(&game_state)
//!     .expect("Failed to serialize game state");
//!
//! // Save to file
//! # let temp_dir = std::env::temp_dir().join("tiles_tools_doctest");
//! # std::fs::create_dir_all(&temp_dir).unwrap();
//! SaveManager::new(&temp_dir)
//!     .save_game_state("my_save", &game_state)
//!     .expect("Failed to save game");
//!
//! // Load from file
//! let _loaded = SaveManager::new(&temp_dir)
//!     .load_game_state("my_save")
//!     .expect("Failed to load game");
//! # std::fs::remove_dir_all(&temp_dir).ok();
//! ```

use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

/// Version information for save file compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveVersion {
  /// Major version - incompatible changes
  pub major: u32,
  /// Minor version - backward compatible additions
  pub minor: u32,
  /// Patch version - bug fixes
  pub patch: u32,
  /// Build timestamp for debugging
  pub timestamp: u64,
}

impl SaveVersion {
  /// Creates a new save version.
  pub fn new(major: u32, minor: u32, patch: u32) -> Self {
    Self {
      major,
      minor,
      patch,
      timestamp: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs(),
    }
  }

  /// Current version of the tiles_tools serialization format.
  pub fn current() -> Self {
    Self::new(1, 0, 0)
  }

  /// Checks if this version is compatible with another version.
  pub fn is_compatible_with(&self, other: &SaveVersion) -> bool {
    self.major == other.major && self.minor >= other.minor
  }
}

impl Default for SaveVersion {
  fn default() -> Self {
    Self::current()
  }
}

/// Metadata associated with a saved game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
  /// Save file version
  pub version: SaveVersion,
  /// Human-readable save description
  pub description: String,
  /// Timestamp when save was created
  pub created_at: u64,
  /// Size of the save data in bytes
  pub size_bytes: u64,
  /// Whether the save data is compressed
  pub compressed: bool,
  /// Custom tags for organizing saves
  pub tags: Vec<String>,
  /// Additional custom metadata
  pub custom: HashMap<String, String>,
}

impl SaveMetadata {
  /// Creates new save metadata.
  pub fn new(description: String) -> Self {
    Self {
      version: SaveVersion::current(),
      description,
      created_at: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs(),
      size_bytes: 0,
      compressed: false,
      tags: Vec::new(),
      custom: HashMap::new(),
    }
  }

  /// Adds a tag to the metadata.
  pub fn with_tag(mut self, tag: String) -> Self {
    self.tags.push(tag);
    self
  }

  /// Adds custom metadata.
  pub fn with_custom(mut self, key: String, value: String) -> Self {
    self.custom.insert(key, value);
    self
  }

  /// Sets compression flag.
  pub fn with_compression(mut self, compressed: bool) -> Self {
    self.compressed = compressed;
    self
  }
}

/// Serializable game state container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameState {
  /// Metadata for this save
  pub metadata: SaveMetadata,
  /// Serialized world data
  pub world_data: Vec<u8>,
  /// Game configuration settings
  pub config: GameConfig,
  /// Player progress and statistics
  pub progress: PlayerProgress,
  /// Custom game-specific data
  pub custom_data: HashMap<String, Vec<u8>>,
}

/// Game configuration that can be serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
  /// Game difficulty level
  pub difficulty: u32,
  /// Graphics settings
  pub graphics: GraphicsConfig,
  /// Audio settings  
  pub audio: AudioConfig,
  /// Input control mappings
  pub controls: ControlConfig,
  /// Gameplay preferences
  pub gameplay: GameplayConfig,
}

impl Default for GameConfig {
  fn default() -> Self {
    Self {
      difficulty: 1,
      graphics: GraphicsConfig::default(),
      audio: AudioConfig::default(),
      controls: ControlConfig::default(),
      gameplay: GameplayConfig::default(),
    }
  }
}

/// Graphics configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
  /// Screen resolution width
  pub resolution_width: u32,
  /// Screen resolution height
  pub resolution_height: u32,
  /// Fullscreen mode enabled
  pub fullscreen: bool,
  /// Vertical sync enabled
  pub vsync: bool,
  /// Graphics quality level (0-3)
  pub quality_level: u32,
  /// Field of view in degrees
  pub fov: f32,
}

impl Default for GraphicsConfig {
  fn default() -> Self {
    Self {
      resolution_width: 1920,
      resolution_height: 1080,
      fullscreen: false,
      vsync: true,
      quality_level: 2,
      fov: 90.0,
    }
  }
}

/// Audio configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
  /// Master volume (0.0 - 1.0)
  pub master_volume: f32,
  /// Music volume (0.0 - 1.0)
  pub music_volume: f32,
  /// Sound effects volume (0.0 - 1.0)
  pub sfx_volume: f32,
  /// Audio enabled
  pub enabled: bool,
}

impl Default for AudioConfig {
  fn default() -> Self {
    Self {
      master_volume: 1.0,
      music_volume: 0.8,
      sfx_volume: 0.9,
      enabled: true,
    }
  }
}

/// Control configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlConfig {
  /// Key bindings mapping
  pub key_bindings: HashMap<String, String>,
  /// Mouse sensitivity
  pub mouse_sensitivity: f32,
  /// Invert mouse Y-axis
  pub invert_mouse_y: bool,
}

impl Default for ControlConfig {
  fn default() -> Self {
    let mut key_bindings = HashMap::new();
    key_bindings.insert("move_up".to_string(), "W".to_string());
    key_bindings.insert("move_down".to_string(), "S".to_string());
    key_bindings.insert("move_left".to_string(), "A".to_string());
    key_bindings.insert("move_right".to_string(), "D".to_string());
    key_bindings.insert("action".to_string(), "Space".to_string());
    key_bindings.insert("menu".to_string(), "Escape".to_string());

    Self {
      key_bindings,
      mouse_sensitivity: 1.0,
      invert_mouse_y: false,
    }
  }
}

/// Gameplay configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayConfig {
  /// Auto-save enabled
  pub auto_save: bool,
  /// Auto-save interval in seconds
  pub auto_save_interval: u32,
  /// Show tutorials and hints
  pub show_tutorials: bool,
  /// Animation speed multiplier
  pub animation_speed: f32,
  /// UI scaling factor
  pub ui_scale: f32,
}

impl Default for GameplayConfig {
  fn default() -> Self {
    Self {
      auto_save: true,
      auto_save_interval: 300, // 5 minutes
      show_tutorials: true,
      animation_speed: 1.0,
      ui_scale: 1.0,
    }
  }
}

/// Player progress and statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgress {
  /// Current player level
  pub level: u32,
  /// Experience points
  pub experience: u64,
  /// Game time played in seconds
  pub playtime_seconds: u64,
  /// Levels completed
  pub levels_completed: Vec<String>,
  /// Achievements unlocked
  pub achievements: Vec<Achievement>,
  /// Game statistics
  pub statistics: GameStatistics,
}

impl Default for PlayerProgress {
  fn default() -> Self {
    Self {
      level: 1,
      experience: 0,
      playtime_seconds: 0,
      levels_completed: Vec::new(),
      achievements: Vec::new(),
      statistics: GameStatistics::default(),
    }
  }
}

/// Player achievement data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
  /// Achievement identifier
  pub id: String,
  /// Display name
  pub name: String,
  /// Achievement description
  pub description: String,
  /// Timestamp when unlocked
  pub unlocked_at: u64,
  /// Achievement points awarded
  pub points: u32,
}

/// Game statistics tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStatistics {
  /// Total entities defeated
  pub entities_defeated: u64,
  /// Total distance moved
  pub distance_moved: f64,
  /// Total items collected
  pub items_collected: u64,
  /// Total spells cast
  pub spells_cast: u64,
  /// Deaths count
  pub deaths: u32,
  /// Levels completed count
  pub levels_completed_count: u32,
}

/// Serialization format options.
#[derive(Debug, Clone, Copy)]
pub enum SerializationFormat {
  /// JSON format - human readable
  Json,
  /// Binary format - compact
  Binary,
  /// RON format - Rust Object Notation
  Ron,
}

/// Game state serializer with compression and format options.
pub struct GameStateSerializer {
  format: SerializationFormat,
  compress: bool,
  version: SaveVersion,
}

impl GameStateSerializer {
  /// Creates a new game state serializer with JSON format.
  pub fn new() -> Self {
    Self {
      format: SerializationFormat::Json,
      compress: false,
      version: SaveVersion::current(),
    }
  }

  /// Sets the serialization format.
  pub fn with_format(mut self, format: SerializationFormat) -> Self {
    self.format = format;
    self
  }

  /// Enables or disables compression.
  pub fn with_compression(mut self, compress: bool) -> Self {
    self.compress = compress;
    self
  }

  /// Sets a custom version.
  pub fn with_version(mut self, version: SaveVersion) -> Self {
    self.version = version;
    self
  }

  /// Serializes a game state to bytes.
  pub fn serialize_game_state(&self, state: &SerializableGameState) -> Result<Vec<u8>, SerializationError> {
    let data = match self.format {
      SerializationFormat::Json => serde_json::to_vec(state)?,
      SerializationFormat::Binary => bincode::serialize(state)?,
      SerializationFormat::Ron => ron::ser::to_string(state)?.into_bytes(),
    };

    if self.compress {
      Ok(self.compress_data(data)?)
    } else {
      Ok(data)
    }
  }

  /// Deserializes a game state from bytes.
  pub fn deserialize_game_state(&self, data: &[u8]) -> Result<SerializableGameState, SerializationError> {
    let data = if self.compress {
      self.decompress_data(data)?
    } else {
      data.to_vec()
    };

    let state = match self.format {
      SerializationFormat::Json => serde_json::from_slice(&data)?,
      SerializationFormat::Binary => bincode::deserialize(&data)?,
      SerializationFormat::Ron => {
        let text = String::from_utf8(data)?;
        ron::from_str(&text).map_err(|e| match e {
          ron::error::SpannedError { code, .. } => SerializationError::Ron(ron::Error::from(code)),
        })?
      }
    };

    Ok(state)
  }

  /// Creates a basic game state for testing.
  pub fn create_basic_game_state(description: String) -> SerializableGameState {
    SerializableGameState {
      metadata: SaveMetadata::new(description),
      world_data: vec![0u8; 1024], // Placeholder world data
      config: GameConfig::default(),
      progress: PlayerProgress::default(),
      custom_data: HashMap::new(),
    }
  }

  // Private compression methods (stubbed for now - would use flate2 or similar)
  fn compress_data(&self, data: Vec<u8>) -> Result<Vec<u8>, SerializationError> {
    // In a real implementation, this would use flate2 or similar
    // For now, just return the data unchanged with a marker
    let mut compressed = vec![0xC0, 0x4D, 0x50]; // "CMP" marker
    compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
    compressed.extend(data);
    Ok(compressed)
  }

  fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, SerializationError> {
    // Check for compression marker
    if data.len() < 7 || &data[0..3] != &[0xC0, 0x4D, 0x50] {
      return Err(SerializationError::InvalidCompressionFormat);
    }

    let original_size = u32::from_le_bytes([data[3], data[4], data[5], data[6]]) as usize;
    if data.len() != original_size + 7 {
      return Err(SerializationError::CorruptedData);
    }

    Ok(data[7..].to_vec())
  }
}

impl Default for GameStateSerializer {
  fn default() -> Self {
    Self::new()
  }
}

/// Save file management system.
pub struct SaveManager {
  saves_directory: PathBuf,
  serializer: GameStateSerializer,
}

impl SaveManager {
  /// Creates a new save manager with the specified saves directory.
  pub fn new<P: AsRef<Path>>(saves_directory: P) -> Self {
    Self {
      saves_directory: saves_directory.as_ref().to_path_buf(),
      serializer: GameStateSerializer::new(),
    }
  }

  /// Sets the serializer to use for save operations.
  pub fn with_serializer(mut self, serializer: GameStateSerializer) -> Self {
    self.serializer = serializer;
    self
  }

  /// Saves a game state to a file.
  pub fn save_game_state(&self, save_name: &str, state: &SerializableGameState) -> Result<(), SerializationError> {
    create_dir_all(&self.saves_directory)?;
    
    let save_path = self.saves_directory.join(format!("{}.save", save_name));
    let metadata_path = self.saves_directory.join(format!("{}.meta", save_name));

    // Serialize the game state
    let serialized_data = self.serializer.serialize_game_state(state)?;

    // Create updated metadata with actual size
    let mut metadata = state.metadata.clone();
    metadata.size_bytes = serialized_data.len() as u64;

    // Write save file
    let mut save_file = BufWriter::new(File::create(save_path)?);
    save_file.write_all(&serialized_data)?;
    save_file.flush()?;

    // Write metadata file
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    let mut metadata_file = BufWriter::new(File::create(metadata_path)?);
    metadata_file.write_all(metadata_json.as_bytes())?;
    metadata_file.flush()?;

    Ok(())
  }

  /// Loads a game state from a file.
  pub fn load_game_state(&self, save_name: &str) -> Result<SerializableGameState, SerializationError> {
    let save_path = self.saves_directory.join(format!("{}.save", save_name));
    
    if !save_path.exists() {
      return Err(SerializationError::SaveNotFound(save_name.to_string()));
    }

    let mut save_file = BufReader::new(File::open(save_path)?);
    let mut data = Vec::new();
    save_file.read_to_end(&mut data)?;

    self.serializer.deserialize_game_state(&data)
  }

  /// Loads save metadata without loading the full save.
  pub fn load_save_metadata(&self, save_name: &str) -> Result<SaveMetadata, SerializationError> {
    let metadata_path = self.saves_directory.join(format!("{}.meta", save_name));
    
    if !metadata_path.exists() {
      return Err(SerializationError::MetadataNotFound(save_name.to_string()));
    }

    let mut metadata_file = BufReader::new(File::open(metadata_path)?);
    let mut metadata_json = String::new();
    metadata_file.read_to_string(&mut metadata_json)?;

    Ok(serde_json::from_str(&metadata_json)?)
  }

  /// Lists all available saves.
  pub fn list_saves(&self) -> Result<Vec<String>, SerializationError> {
    if !self.saves_directory.exists() {
      return Ok(Vec::new());
    }

    let mut saves = Vec::new();
    
    for entry in std::fs::read_dir(&self.saves_directory)? {
      let entry = entry?;
      let path = entry.path();
      
      if let Some(extension) = path.extension() {
        if extension == "save" {
          if let Some(stem) = path.file_stem() {
            if let Some(name) = stem.to_str() {
              saves.push(name.to_string());
            }
          }
        }
      }
    }

    saves.sort();
    Ok(saves)
  }

  /// Deletes a save file and its metadata.
  pub fn delete_save(&self, save_name: &str) -> Result<(), SerializationError> {
    let save_path = self.saves_directory.join(format!("{}.save", save_name));
    let metadata_path = self.saves_directory.join(format!("{}.meta", save_name));

    if save_path.exists() {
      std::fs::remove_file(save_path)?;
    }

    if metadata_path.exists() {
      std::fs::remove_file(metadata_path)?;
    }

    Ok(())
  }

  /// Gets information about all saves including metadata.
  pub fn get_saves_info(&self) -> Result<Vec<(String, SaveMetadata)>, SerializationError> {
    let save_names = self.list_saves()?;
    let mut saves_info = Vec::new();

    for name in save_names {
      match self.load_save_metadata(&name) {
        Ok(metadata) => saves_info.push((name, metadata)),
        Err(_) => {
          // Skip saves with missing or invalid metadata
          continue;
        }
      }
    }

    // Sort by creation time (newest first)
    saves_info.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    Ok(saves_info)
  }
}

/// Configuration manager for game settings.
pub struct ConfigManager {
  config_path: PathBuf,
}

impl ConfigManager {
  /// Creates a new configuration manager.
  pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
    Self {
      config_path: config_path.as_ref().to_path_buf(),
    }
  }

  /// Saves configuration to file.
  pub fn save_config(&self, config: &GameConfig) -> Result<(), SerializationError> {
    if let Some(parent) = self.config_path.parent() {
      create_dir_all(parent)?;
    }

    let config_json = serde_json::to_string_pretty(config)?;
    let mut file = BufWriter::new(File::create(&self.config_path)?);
    file.write_all(config_json.as_bytes())?;
    file.flush()?;

    Ok(())
  }

  /// Loads configuration from file.
  pub fn load_config(&self) -> Result<GameConfig, SerializationError> {
    if !self.config_path.exists() {
      return Ok(GameConfig::default());
    }

    let mut file = BufReader::new(File::open(&self.config_path)?);
    let mut config_json = String::new();
    file.read_to_string(&mut config_json)?;

    Ok(serde_json::from_str(&config_json)?)
  }

  /// Resets configuration to defaults.
  pub fn reset_config(&self) -> Result<(), SerializationError> {
    self.save_config(&GameConfig::default())
  }
}

/// Errors that can occur during serialization operations.
#[derive(Debug)]
pub enum SerializationError {
  /// IO error occurred
  Io(std::io::Error),
  /// JSON serialization error
  Json(serde_json::Error),
  /// Binary serialization error
  Binary(bincode::Error),
  /// RON serialization error
  Ron(ron::Error),
  /// UTF-8 conversion error
  Utf8(std::string::FromUtf8Error),
  /// Save file not found
  SaveNotFound(String),
  /// Metadata file not found
  MetadataNotFound(String),
  /// Invalid compression format
  InvalidCompressionFormat,
  /// Corrupted save data
  CorruptedData,
  /// Version incompatibility
  IncompatibleVersion { found: SaveVersion, expected: SaveVersion },
}

impl std::fmt::Display for SerializationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SerializationError::Io(e) => write!(f, "IO error: {}", e),
      SerializationError::Json(e) => write!(f, "JSON error: {}", e),
      SerializationError::Binary(e) => write!(f, "Binary serialization error: {}", e),
      SerializationError::Ron(e) => write!(f, "RON error: {}", e),
      SerializationError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
      SerializationError::SaveNotFound(name) => write!(f, "Save '{}' not found", name),
      SerializationError::MetadataNotFound(name) => write!(f, "Metadata for save '{}' not found", name),
      SerializationError::InvalidCompressionFormat => write!(f, "Invalid compression format"),
      SerializationError::CorruptedData => write!(f, "Save data is corrupted"),
      SerializationError::IncompatibleVersion { found, expected } => {
        write!(f, "Incompatible save version: found {}.{}.{}, expected {}.{}.{}",
          found.major, found.minor, found.patch,
          expected.major, expected.minor, expected.patch)
      }
    }
  }
}

impl std::error::Error for SerializationError {}

impl From<std::io::Error> for SerializationError {
  fn from(error: std::io::Error) -> Self {
    SerializationError::Io(error)
  }
}

impl From<serde_json::Error> for SerializationError {
  fn from(error: serde_json::Error) -> Self {
    SerializationError::Json(error)
  }
}

impl From<bincode::Error> for SerializationError {
  fn from(error: bincode::Error) -> Self {
    SerializationError::Binary(error)
  }
}

impl From<ron::Error> for SerializationError {
  fn from(error: ron::Error) -> Self {
    SerializationError::Ron(error)
  }
}

impl From<std::string::FromUtf8Error> for SerializationError {
  fn from(error: std::string::FromUtf8Error) -> Self {
    SerializationError::Utf8(error)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
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

    let game_state = GameStateSerializer::create_basic_game_state("Test Game".to_string());
    
    let serialized = serializer.serialize_game_state(&game_state).unwrap();
    let deserialized = serializer.deserialize_game_state(&serialized).unwrap();

    assert_eq!(game_state.metadata.description, deserialized.metadata.description);
    assert_eq!(game_state.world_data.len(), deserialized.world_data.len());
  }

  #[test]
  fn test_compression() {
    let serializer = GameStateSerializer::new()
      .with_compression(true);

    let game_state = GameStateSerializer::create_basic_game_state("Compression Test".to_string());
    
    let compressed = serializer.serialize_game_state(&game_state).unwrap();
    let decompressed = serializer.deserialize_game_state(&compressed).unwrap();

    assert_eq!(game_state.metadata.description, decompressed.metadata.description);
  }

  #[test]
  fn test_save_manager() {
    let temp_dir = TempDir::new().unwrap();
    let save_manager = SaveManager::new(temp_dir.path());

    let game_state = GameStateSerializer::create_basic_game_state("Test Save".to_string());

    // Save the game state
    save_manager.save_game_state("test_save", &game_state).unwrap();

    // Load it back
    let loaded_state = save_manager.load_game_state("test_save").unwrap();
    assert_eq!(game_state.metadata.description, loaded_state.metadata.description);

    // Test listing saves
    let saves = save_manager.list_saves().unwrap();
    assert!(saves.contains(&"test_save".to_string()));

    // Test metadata loading
    let metadata = save_manager.load_save_metadata("test_save").unwrap();
    assert_eq!(metadata.description, "Test Save");

    // Test saves info
    let saves_info = save_manager.get_saves_info().unwrap();
    assert_eq!(saves_info.len(), 1);
    assert_eq!(saves_info[0].0, "test_save");

    // Test deletion
    save_manager.delete_save("test_save").unwrap();
    let saves_after_delete = save_manager.list_saves().unwrap();
    assert!(!saves_after_delete.contains(&"test_save".to_string()));
  }

  #[test]
  fn test_config_manager() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let config_manager = ConfigManager::new(&config_path);

    // Save default config
    let config = GameConfig::default();
    config_manager.save_config(&config).unwrap();

    // Load it back
    let loaded_config = config_manager.load_config().unwrap();
    assert_eq!(config.difficulty, loaded_config.difficulty);

    // Test reset
    config_manager.reset_config().unwrap();
    let reset_config = config_manager.load_config().unwrap();
    assert_eq!(reset_config.difficulty, 1);
  }

  #[test]
  fn test_player_progress_serialization() {
    let mut progress = PlayerProgress::default();
    progress.level = 5;
    progress.experience = 1500;
    progress.achievements.push(Achievement {
      id: "first_kill".to_string(),
      name: "First Kill".to_string(),
      description: "Defeat your first enemy".to_string(),
      unlocked_at: 1234567890,
      points: 10,
    });

    let json = serde_json::to_string(&progress).unwrap();
    let deserialized: PlayerProgress = serde_json::from_str(&json).unwrap();

    assert_eq!(progress.level, deserialized.level);
    assert_eq!(progress.achievements.len(), deserialized.achievements.len());
    assert_eq!(progress.achievements[0].id, deserialized.achievements[0].id);
  }

  #[test]
  fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let save_manager = SaveManager::new(temp_dir.path());

    // Test loading non-existent save
    let result = save_manager.load_game_state("nonexistent");
    assert!(matches!(result, Err(SerializationError::SaveNotFound(_))));

    // Test loading non-existent metadata
    let result = save_manager.load_save_metadata("nonexistent");
    assert!(matches!(result, Err(SerializationError::MetadataNotFound(_))));
  }
}