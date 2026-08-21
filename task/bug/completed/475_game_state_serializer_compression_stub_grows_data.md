# BUG-475: `GameStateSerializer::with_compression(true)` never compresses anything -- it silently makes output 7 bytes larger

- **Severity:** Medium (no crash, no data loss -- round-trip correctness was preserved -- but
  the documented "Compression Support" feature does the opposite of its stated purpose for
  every caller who enables it)
- **state:** Completed
- **Affects:** Any consumer of `GameStateSerializer::with_compression(true)` /
  `SaveManager::with_serializer(GameStateSerializer::new().with_compression(true))` expecting
  smaller save files.
- **Component:** module/helper/tiles_tools (`src/serialization.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-476 (the corrupted-header overflow this same fix structurally
  eliminates), BUG-348 (a different `compressed`-flag desync bug in the same struct, already
  fixed).

## Symptom

```rust
// pre-fix -- src/serialization.rs
fn data_compress(data: Vec<u8>) -> Vec<u8> {
  // In a real implementation, this would use flate2 or similar
  // For now, just return the data unchanged with a marker
  let mut compressed = vec![0xC0, 0x4D, 0x50]; // "CMP" marker
  compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
  compressed.extend(data);
  compressed
}
```

`data_compress` prepends a 7-byte marker+length header to the input and returns it otherwise
unmodified -- output is always exactly `input.len() + 7` bytes, strictly larger than
uncompressed output, regardless of how compressible the input is.

## Impact

**Who is affected:** Any caller of `with_compression(true)`, directly or via `SaveManager`.

**What breaks:** Disk usage only -- `game_state_deserialize` round-trips correctly against the
stub's own output (compress/decompress remained exact mutual inverses), so no data was ever
lost or corrupted by this bug. The feature simply did not do what its name and the module's own
"Compression Support" doc section claimed.

**Consumer audit:** `data_compress`/`data_decompress` are both private; the only path to this
behavior is via `with_compression(true)`, used by this crate's own tests
(`test_compression`, `test_game_state_save_meta_compressed_flag_matches_actual_compression`)
and the module's own doc example. `grep -rln 'with_compression' --include="*.rs" .` from the
repo root, excluding `tiles_tools` itself, returns no external call sites.

**Magnitude:** Every compressed save is `original_size + 7` bytes instead of smaller; for
`basic_game_state_create`'s placeholder 1024-zero-byte `world_data`, uncompressed JSON output is
several KB and would DEFLATE-compress to well under 100 bytes -- the stub instead grew it by 7
bytes.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/serialization.rs` end to end -- the function's own
comment ("stubbed for now - would use flate2 or similar") flagged it directly.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/serialization_test.rs
let game_state = GameStateSerializer::basic_game_state_create("Compression Size Test".to_string());
let uncompressed = GameStateSerializer::new().with_compression(false).game_state_serialize(&game_state).unwrap();
let compressed = GameStateSerializer::new().with_compression(true).game_state_serialize(&game_state).unwrap();
assert!(compressed.len() < uncompressed.len());
// pre-fix: fails -- compressed.len() == uncompressed.len() + 7, always larger, never smaller
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(serialization_test) and test(compression_actually_shrinks)'
```

## Root Cause

`data_compress`/`data_decompress` were authored as placeholder stubs (per the function's own
comment) and never revisited once the surrounding `GameStateSerializer` API stabilized -- the
7-byte marker+length header made the stub's own round-trip self-consistent, which was
sufficient for every existing test to pass, masking that no actual compression algorithm was
ever wired in.

## Why Not Caught

`test_compression` only asserts the compress/decompress round-trip recovers the original
description (`assert_eq!(game_state.metadata.description, decompressed.metadata.description)`)
-- since the stub's compress and decompress were exact mutual inverses (neither one actually
compressed anything), the round-trip passed regardless of whether real compression happened. No
existing test compared compressed-vs-uncompressed output size.

## Fix Location

`module/helper/tiles_tools/src/serialization.rs`: `data_compress`/`data_decompress` rewritten
to use `flate2`'s `DeflateEncoder`/`DeflateDecoder` (`rust_backend` feature -- pure-Rust
miniz_oxide, no system zlib/C toolchain dependency, confirmed `wasm32-unknown-unknown`
compatible, this crate's stated primary target per `Cargo.toml`'s `[package.metadata.docs.rs]`
targets list). The existing 7-byte marker+header format is kept, but the header now stores the
*decompressed* size (validated after inflating), not the compressed-payload length -- see
BUG-476 for why this redesign also eliminates a separate overflow bug.
`Cargo.toml`: added `flate2 = { version = "1", default-features = false, features =
["rust_backend"], optional = true }` and `"dep:flate2"` to the `serialization` feature list.
Also fixed the module doc example (previously computed a `serialized` variable via
`with_compression(true)` that was never used -- the actual save/load calls below it used a
separate, uncompressed default `SaveManager`) to thread a compression-configured serializer
through both save and load via `SaveManager::with_serializer`.

## Prevention

New test `test_compression_actually_shrinks_compressible_data` in `tests/serialization_test.rs`
asserts compressed output is strictly smaller than uncompressed output for
`basic_game_state_create`'s highly-compressible placeholder data, in addition to re-confirming
round-trip correctness through the real compressor.

## Pitfall

"The round trip still works" is not evidence a `with_*(true)` toggle does anything at all -- an
identity-function stub passes every round-trip test a real implementation would, which is
exactly what let this ship silently. A test suite for a `with_compression` flag needs at least
one assertion that actually measures the thing compression is supposed to do (output size),
not just that encode/decode remain inverses of each other.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, reading `src/serialization.rs` end to end; the stub's own comment flagged it directly. |
| 2026-08-20 | fixed | Real DEFLATE compression via `flate2` (`rust_backend`, wasm32-compatible); header format reinterpreted to store decompressed size; module doc example fixed to actually exercise compression end to end. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: confirmed `test_compression_actually_shrinks_compressible_data` genuinely fails against the pre-fix stub (compressed.len() would be uncompressed.len() + 7, violating the `<` assertion) -- not vacuous. Full-crate pass: `cargo nextest run -p tiles_tools --all-features` -- 286/286 pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-475)`/`Root cause`/`Pitfall` 3-field format applied at `data_compress` in `src/serialization.rs`. | — |
| D3 | wasm32 compatibility | — | 🟢 | `flate2` added with `default-features = false, features = ["rust_backend"]`, selecting the pure-Rust miniz_oxide backend -- confirmed via `Cargo.toml` this is the same pattern already used for other pure-Rust wasm32-safe dependencies in this crate; no system zlib / C toolchain introduced. | — |

**Reproduced:** YES -- `test_compression_actually_shrinks_compressible_data`'s core assertion
(`compressed.len() < uncompressed.len()`) is false against the pre-fix stub (verified by
inspection of the stub's fixed `+7`-byte-growth formula, not a temporary revert-and-rerun, since
the fix was written and verified in the same pass) and true against the fix. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/serialization.rs` | `data_compress`/`data_decompress` rewritten to use real `flate2` DEFLATE compression; `Fix(BUG-475)`/`Root cause`/`Pitfall` comment added; module doc example fixed to thread compression through `SaveManager::with_serializer` (see also UX/DX fix log). |
| `module/helper/tiles_tools/Cargo.toml` | Added `flate2` optional dependency (`rust_backend`, wasm32-safe) and wired it into the `serialization` feature. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/serialization_test.rs` | Added `test_compression_actually_shrinks_compressible_data`, asserting compressed output is smaller than uncompressed for compressible data, plus round-trip correctness through the real compressor. |
