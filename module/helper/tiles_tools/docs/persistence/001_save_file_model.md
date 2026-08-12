# Persistence: Save File Model

### Scope

- **Purpose**: Document the on-disk save-file format `SaveManager` reads and writes, and the placeholder status of the game-state payload it currently persists.
- **Responsibility**: Document the 2-file-per-save layout, the 3 selectable serialization formats, and the durability gaps in the write path.
- **In Scope**: `SaveManager`, `GameStateSerializer`, `SerializableGameState`, `SaveMetadata`, `SerializationFormat`.
- **Out of Scope**: The compression option, which is a non-functional stub (see `pitfall/003`); the live `ecs::World` this format has no populated bridge to yet (see Data Layout below).

### Abstract

`SaveManager` is a real, working file-based save system — it genuinely creates directories, writes and reads files on disk via `std::fs`, and round-trips through 3 selectable serialization formats (JSON via `serde_json`, Binary via `bincode`, RON via `ron`), all real external crates, not stubs. What it persists, however, is presently always a fixed placeholder: `SerializableGameState::world_data` is populated only by `basic_game_state_create`, which sets it to `vec![0u8; 1024]` (`src/serialization.rs:461`, comment: `// Placeholder world data`) — no code path in this crate serializes a live `ecs::World` into this field. The save/load mechanics themselves are sound; there is currently nothing feeding them real game state.

### Storage Model

One named save occupies exactly 2 files inside `SaveManager`'s configured `saves_directory`:

| File | Content |
|------|---------|
| `{save_name}.save` | `GameStateSerializer::game_state_serialize`'s output bytes, written via `BufWriter`/`write_all` (`src/serialization.rs:535-537`) |
| `{save_name}.meta` | `SaveMetadata`, pretty-printed as JSON via `serde_json::to_string_pretty` (`src/serialization.rs:540-543`) — written independently of format selection, always JSON regardless of the `.save` file's own format |

`SaveManager::saves_list` enumerates `saves_directory`, filters by `.save` extension, and returns the sorted file stems (`src/serialization.rs:579-603`) — it does not read file contents, so a `.save` file with no matching `.meta` still appears in the listing.

### Data Layout

`SerializableGameState` (`src/serialization.rs:161+`):

| Field | Content |
|-------|---------|
| `metadata: SaveMetadata` | Save name, timestamp, and (after `game_state_save` runs) actual serialized byte size — see `src/serialization.rs:531-532`. |
| `world_data: Vec<u8>` | **Always a fixed 1024-byte zero-filled placeholder** when constructed via `basic_game_state_create` — the only constructor present in this file. No code in `src/serialization.rs` serializes a live `hecs::World`/`ecs::World` into this field. |
| `config: GameConfig` | Nested `GraphicsConfig`/`AudioConfig`/`ControlConfig`/`GameplayConfig`, each with a real `Default` impl. |
| `progress: PlayerProgress` | Player-progress tracking, including `Achievement`/`GameStatistics`. |
| `custom_data: HashMap<String, serde_json::Value>` | Open-ended extension slot for caller-defined save data. |

Format selection (`GameStateSerializer::game_state_serialize`, `src/serialization.rs:420-425`) is a real 3-way dispatch, not a stub:
- `SerializationFormat::Json` → `serde_json::to_vec`
- `SerializationFormat::Binary` → `bincode::serialize`
- `SerializationFormat::Ron` → `ron::ser::to_string(..).into_bytes()`

Optional compression (`with_compression(true)`) wraps the chosen format's bytes in a 7-byte marker/length header that does not actually compress anything — see `pitfall/003` for the exact mechanism.

### Durability Guarantees

`game_state_save` (`src/serialization.rs:521-546`) writes the `.save` and `.meta` files as **two independent, sequential** `File::create` + `write_all` + `flush()` operations — there is no temp-file-plus-rename pattern and no `sync_all`/`fsync` call anywhere in the write path. Consequences:
- `flush()` ensures bytes reach the OS's own buffers before `game_state_save` returns, but without an `fsync`-equivalent call, a hard crash or power loss immediately after return can still lose the write — durable against ordinary process behavior, not against a hard crash.
- Because the two files are written as separate operations with no atomic swap, a process interrupted between them can leave a `.save` file with no matching `.meta` (or a partially-written `.save`/`.meta` file on its own, since neither uses an atomic rename into place).

### Types

| File | Relationship |
|------|--------------|
| [type/002_ecs_component_vocabulary.md](../type/002_ecs_component_vocabulary.md) | The eventual (currently unbuilt) bridge target for `world_data` — the live component data this format has no path to persist yet |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/003_savefile_compression_is_a_fake_wrapper.md](../pitfall/003_savefile_compression_is_a_fake_wrapper.md) | Full detail on the non-functional compression option |

### Sources

| File | Relationship |
|------|--------------|
| `src/serialization.rs` | `SaveManager`, `GameStateSerializer`, `SerializableGameState`, `SaveMetadata`, `SerializationFormat`, `basic_game_state_create` |

### Tests

`src/serialization.rs:767+` — `#[cfg(test)]` module with 4 `#[test]` functions (`src/serialization.rs:773,784,797,807`). None currently constructs a `SerializableGameState` from a live `ecs::World` (consistent with no such bridge existing) or asserts `.save`/`.meta` durability under interruption.
