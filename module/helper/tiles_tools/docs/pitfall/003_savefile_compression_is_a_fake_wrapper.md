# Pitfall: Save-File "Compression" Never Shrinks Data

### Scope

- **Purpose**: Warn that `GameStateSerializer::with_compression(true)` does not run a compression algorithm, so enabling it never reduces save-file size.
- **Responsibility**: Document the exact framing `compress_data` applies and why round-tripping is still safe despite the missing compression.
- **In Scope**: `GameStateSerializer::compress_data`, `GameStateSerializer::decompress_data`, `GameStateSerializer::with_compression`.
- **Out of Scope**: The 3 real serialization formats (JSON/Binary/RON) this wraps — see `persistence/001`.

### Trap

Calling `GameStateSerializer::new().with_compression(true)` before `serialize_game_state`, expecting a smaller output than the uncompressed form — the method name and the struct's own `compress: bool` field both suggest genuine compression.

### Failure

`compress_data` does not run any compression algorithm — it prepends a fixed 7-byte header to the **unmodified** input bytes (`src/serialization.rs:469-476`):

```rust
// In a real implementation, this would use flate2 or similar
// For now, just return the data unchanged with a marker
fn compress_data(&self, data: Vec<u8>) -> Vec<u8> {
  let mut compressed = vec![0xC0, 0x4D, 0x50]; // "CMP" marker
  compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
  compressed.extend(data);
  compressed
}
```

A "compressed" save is therefore always exactly 7 bytes **larger** than the uncompressed form, never smaller — regardless of how repetitive or compressible the underlying data actually is. `decompress_data` correctly reverses this framing (validates the 3-byte marker, reads the 4-byte little-endian length, strips the 7-byte header), so round-tripping through `GameStateSerializer` with `compress: true` is lossless — the defect is purely that no space is ever saved, not data corruption.

### Mitigation

Do not enable `.with_compression(true)` expecting reduced save-file size. If save-file size matters, compress `serialize_game_state`'s output externally (e.g. via `flate2`) before writing to disk, and reverse it before calling `deserialize_game_state`.

### Persistences

| File | Relationship |
|------|--------------|
| [persistence/001_save_file_model.md](../persistence/001_save_file_model.md) | Documents the full save-file format this stub sits inside; cross-references this pitfall for the compression option specifically |

### Sources

| File | Relationship |
|------|--------------|
| `src/serialization.rs` | `GameStateSerializer::compress_data`, `GameStateSerializer::decompress_data`, `GameStateSerializer::with_compression` |

### Tests

| File | Relationship |
|------|--------------|
| `src/serialization.rs:767+` | `#[cfg(test)]` module exists with 4 `#[test]` functions (`src/serialization.rs:773,784,797,807`); none currently asserts compressed output is smaller than uncompressed input for the same data |
