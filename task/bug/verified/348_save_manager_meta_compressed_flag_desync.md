# BUG-348: `SaveManager::game_state_save` never syncs the `.meta` sidecar's `compressed` flag with the serializer that actually compresses the `.save` file

- **Severity:** Medium
- **state:** Verified
- **Affects:** Every `SaveManager::game_state_save` call using a `GameStateSerializer` configured
  with `.with_compression(true)` (or `false`) whenever the saved `SerializableGameState`'s own
  `metadata.compressed` field does not already happen to match
- **Component:** `module/helper/tiles_tools` (`src/serialization.rs`, `SaveManager::game_state_save`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18
- **Fix Task:** [383](../../verifying/383_register_tiles_tools_savemanager_game_state_save_compressed_flag_sync_fix_closes_bug348.md)

## Symptom

The `.meta` sidecar file written alongside a `.save` file is supposed to accurately describe that
save — but its `compressed` field can silently disagree with whether the `.save` file's bytes are
actually compressed:

```
# save_manager with GameStateSerializer::new().with_compression(true)
# game_state.metadata.compressed starts false (default fixture state)

save_manager.game_state_save("compressed_save", &game_state)?;
# -> compressed_save.save   : actually compressed (serializer.compress == true)
# -> compressed_save.meta   : metadata.compressed == false   <- WRONG, desynced

save_manager.save_metadata_load("compressed_save")?.compressed  -> false   # wrong
```

## Impact

**Who is affected:** any consumer that reads a save's `.meta` sidecar (via
`save_metadata_load`, or any external tool/UI that inspects the `.meta` JSON directly) to decide
whether the corresponding `.save` file is compressed, without loading and decompressing the
`.save` file itself to check.

**What breaks:** the reported `compressed` flag can be the *opposite* of reality — a consumer that
branches on it (e.g., to decide whether to decompress before further processing, or just to
display accurate save-file metadata to a user/tool) gets a wrong answer. `game_state_load` itself
is unaffected, because it consults `self.serializer.compress` directly rather than the metadata's
`compressed` field — which is exactly what let this desync go unnoticed by any round-trip test.

**Magnitude:** 1 method (`game_state_save`); every save written through it carries a `.meta`
sidecar whose `compressed` field reflects the *caller's input* `SerializableGameState`, not the
`GameStateSerializer` that actually determines real compression behavior.

**Entity Scope:** `None` — a code-level field-sync defect, not entity directory instances.

## How Discovered

```bash
$ cargo test -p tiles_tools --all-features --test serialization_test \
    test_game_state_save_meta_compressed_flag_matches_actual_compression -- --exact

thread 'test_game_state_save_meta_compressed_flag_matches_actual_compression'
panicked at module/helper/tiles_tools/tests/serialization_test.rs:257:3:
the .save file is actually compressed (serializer.compress == true), but the .meta sidecar's
compressed flag was not synchronized to match
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

A prior investigation pass identified the missing sync by direct reading of `game_state_save`
(§ Hypothesis Table below); this report re-confirms it with the permanent reproducer test above,
run against the pre-fix source.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test serialization_test \
  test_game_state_save_meta_compressed_flag_matches_actual_compression -- --exact
```
**What:** after `game_state_save` with a serializer configured `with_compression(true)`, the
`.meta` sidecar loaded back via `save_metadata_load` must report `compressed == true`, matching
the serializer that actually compressed the `.save` file's bytes.

**Expected** (fixed): test passes — `test test_game_state_save_meta_compressed_flag_matches_actual_compression ... ok`.

**Actual** (pre-fix, directly confirmed by running the same test against the current, unfixed
source before applying the fix below):
```
thread 'test_game_state_save_meta_compressed_flag_matches_actual_compression'
panicked at module/helper/tiles_tools/tests/serialization_test.rs:257:3:
the .save file is actually compressed (serializer.compress == true), but the .meta sidecar's
compressed flag was not synchronized to match
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `game_state_save` (serialization.rs:569-609, pre-fix) clones `state.metadata` (line 579) and updates only `size_bytes` (line 580) before writing the `.meta` sidecar — `metadata.compressed` is never assigned anywhere in the method, so it silently retains whatever value the caller's input `SerializableGameState` happened to already carry | ✅ Root Cause | Direct read: only two field writes to the cloned `metadata` exist pre-fix (`size_bytes`); no `metadata.compressed = ...` assignment anywhere in the function body | E1 |
| H2 | `GameStateSerializer.compress` (the field that actually drives whether `game_state_serialize`'s output bytes are compressed) and `SaveMetadata.compressed` (an independent struct field, settable only via the unrelated `SaveMetadata::with_compression` builder) are two separate fields never synchronized anywhere in the crate | ✅ Verified | `grep -n "\.compress\b\|compressed" src/serialization.rs` shows `serializer.compress` consulted only inside `game_state_serialize`/`game_state_deserialize`; `metadata.compressed` set only by the `with_compression` builder and read only by external consumers of the `.meta` file | E2 |
| H3 | `game_state_load` (the read-side counterpart) never actually breaks on this desync because it decompresses based on `self.serializer.compress`, not on the loaded metadata's `compressed` field — which is exactly why no existing round-trip (save-then-load) test caught the desync | ✅ Verified | Direct read of `game_state_load`: decompression branches on `self.serializer.compress`, with no reference to `metadata.compressed` anywhere in the load path | E3 |
| H4 | The desync is directional: a `SerializableGameState` constructed via `GameStateSerializer::basic_game_state_create` (or any other fixture) always starts with `metadata.compressed == false` by default, so any save made through a `with_compression(true)` serializer reproduces the wrong-value case deterministically | ✅ Verified | Terminal evidence (E4): the reproducer test's own precondition assertion (`!game_state.metadata.compressed`) passes, confirming the fixture starts `false`, then the post-save assertion fails exactly as H1-H2 predict | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tiles_tools/src/serialization.rs:569-609` (`game_state_save`, pre-fix, direct read via `git show HEAD:...`) | Cloned `metadata` (579) has only `size_bytes` (580) assigned before being written to the `.meta` file (602-606); `compressed` untouched | H1 |
| E2 | `grep -n "\.compress\b\|compressed" src/serialization.rs` (direct read) | `serializer.compress` and `metadata.compressed` are read/written in disjoint sets of locations — no line assigns one from the other | H2 |
| E3 | `module/helper/tiles_tools/src/serialization.rs` (`game_state_load`, direct read) | Decompression decision uses `self.serializer.compress` exclusively; `metadata.compressed` (loaded from the `.meta` file) is never consulted | H3 |
| E4 | Terminal output (this report, MRE section; also captured in `-0001_longrun.log:366-376`, pre-fix combined test run) | Reproducer confirms the fixture starts `compressed == false`, then fails the post-save assertion exactly as predicted | H4 |

## Root Cause

```
GameStateSerializer { compress: true, ... }     SerializableGameState { metadata: { compressed: false, ... }, ... }
        |                                                          |
        v                                                          v
game_state_serialize(state)  -- compresses bytes using self.compress (true)
        |
        v
metadata = state.metadata.clone()   -- carries compressed: false forward, unmodified
metadata.size_bytes = ...           -- only field actually updated
        |
        v
write .meta (metadata.compressed == false)      <- WRONG: .save bytes ARE compressed
```
`GameStateSerializer.compress` and `SaveMetadata.compressed` describe the same underlying fact
(is this save's byte payload compressed) but are two independent fields with no code path that
derives one from the other. `game_state_save` clones the caller-supplied metadata and updates only
`size_bytes` — the one field it computes fresh from the actual serialized output — while leaving
`compressed` exactly as the caller's input happened to already have it, regardless of what the
serializer that just ran actually did.

## Why Not Caught

The crate's serialization tests exercise save→load round trips, but `game_state_load` never
reads `metadata.compressed` at all (it decides how to decompress from `self.serializer.compress`
directly) — so a round trip succeeding proves nothing about whether the `.meta` sidecar's
`compressed` field is accurate. No existing test loaded the `.meta` file's `compressed` value
independently and compared it against the serializer's own `compress` setting.

## Fix Location

**`module/helper/tiles_tools/src/serialization.rs:580`** (`game_state_save`, immediately after
the pre-fix `metadata.size_bytes = ...` assignment):

```rust
// Before:
let mut metadata = state.metadata.clone();
metadata.size_bytes = serialized_data.len() as u64;

// Write save file
let mut save_file = BufWriter::new(File::create(save_path)?);

// After:
let mut metadata = state.metadata.clone();
metadata.size_bytes = serialized_data.len() as u64;
// Fix(BUG-348): synchronize the .meta sidecar's `compressed` flag with
// the serializer that actually determines whether the .save file's
// bytes are compressed, instead of leaving whatever `compressed` value
// the caller's SerializableGameState happened to already carry.
// Root cause: `GameStateSerializer.compress` (drives real compression)
// and `SaveMetadata.compressed` (an independent field, set only via the
// unrelated `SaveMetadata::with_compression` builder) were never
// synchronized anywhere -- `game_state_load` still round-trips
// correctly because it consults the serializer's own `compress` field,
// not the metadata, which is exactly what let this desync go unnoticed.
// Pitfall: two fields describing the same underlying fact will drift
// apart unless one is derived from the other at the one place both are
// written -- verify every field a sidecar/report writes is actually
// sourced from the value that controls the real behavior it describes.
metadata.compressed = self.serializer.compress;

// Write save file
let mut save_file = BufWriter::new(File::create(save_path)?);
```

## Prevention

Detection command for the general pattern (a struct field written to a report/sidecar file
without being derived from the value that actually controls the behavior it claims to describe):
```bash
grep -n "compress" module/helper/tiles_tools/src/serialization.rs
```
This is a starting point for review, not a precise check — confirming correctness requires a test
that reads the sidecar/report back independently and compares it against the controlling value,
which is exactly what the new reproducer test adds. Any future metadata field added to
`SaveMetadata` that describes a fact also tracked by `GameStateSerializer` (or any other
serializer-side config) should be assigned from that authoritative source at write time, not left
to whatever the caller's input struct already contained.

**Pitfall:** two fields describing the same underlying fact will drift apart unless one is
explicitly derived from the other at the single place both are written — never assume a metadata
field is accurate just because it exists on the struct being saved.

## Generalized Version

**Broken assumption:** "a metadata/report field on the object being persisted already holds the
correct value, so the save/write path only needs to update the fields it explicitly computes
(like size), not re-derive every field from the actual state of the world."

Fails for any save/report-writing method whenever:
1. The output struct has a field claiming to describe a fact (compression, encoding, checksum,
   version) that is actually controlled by a *different* component (a serializer, a config,
   an environment setting), AND
2. That field can be set independently of the controlling component (e.g., via its own builder
   method, a default value, or caller-supplied input), AND
3. The write path never re-derives the field from the controlling component before persisting it.

**Detection invariant:**
```
for every metadata field F on a persisted object claiming to describe behavior controlled by
component C:
  F, as read back from the persisted output, must equal C's own current value at write time
  -- never merely "whatever F already held on the input object"
```
Confirmed as a single instance in this crate (`compressed` is the only `SaveMetadata` field with
an independent builder AND a corresponding `GameStateSerializer` field it should track; other
`SaveMetadata` fields like `size_bytes` are already correctly re-derived at write time — confirmed
via direct read of `game_state_save`). Dedup search:
`grep -rli "game_state_save\|SaveManager\|metadata.compressed" task/bug/` found one related but
distinct prior hit: `task/bug/completed/269_game_state_deserialize_never_checks_save_version_compatibility.md`
targets `game_state_deserialize`'s missing version-compatibility check — a different function and
a different defect (a missing validation, not a field-sync desync) in the same file; not a
duplicate of this bug.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Re-confirmed via direct source reading and a new permanent reproducer test, following up a prior investigation pass's finding |
| 2026-08-18 | note | SUBMIT: state Draft -> Unverified; reproducer confirmed FAIL pre-fix and PASS post-fix, fix applied, full scoped suite (`cargo test -p tiles_tools --all-features`) green |
| 2026-08-18 | VERIFY Gate | Reproducer test test_game_state_save_meta_compressed_flag_matches_actual_compression confirmed passing against current source (`cargo test -p tiles_tools --all-features --test serialization_test ... -- --exact`: 1 passed; 0 failed); fix in module/helper/tiles_tools/src/serialization.rs confirmed present at line 597 (`metadata.compressed = self.serializer.compress;`). state: Unverified -> Verified |
| 2026-08-18 | note | VERIFY Gate two-pass re-check (Tier 2 Dual-Role Self-Check, `governance/maav.rulebook.md`): adversarial pass found neither `src/serialization.rs` nor `tests/serialization_test.rs` carried the canonical FI027 backreference (only `Fix(BUG-348)`/`test_kind:` markers existed, matching the same gap BUG-298's own VERIFY Gate previously found and fixed in this repo); added `// BUG-348 task/bug/348_....md -- ...` backreference comment adjacent to each marker, re-verified via `grep -rn 'BUG-348' src/ tests/`; full `tiles_tools` scoped suite re-run (`cargo nextest run -p tiles_tools --all-features`: 272 passed / 0 failed, including this bug's reproducer); `## Verification Record` appended below |

## Refs: src/

- `module/helper/tiles_tools/src/serialization.rs` — `game_state_save` now assigns `metadata.compressed = self.serializer.compress` before writing the `.meta` sidecar

## Refs: tests/

- `module/helper/tiles_tools/tests/serialization_test.rs` — new reproducer: saves via a `with_compression(true)` serializer starting from a `metadata.compressed == false` fixture, then asserts the reloaded `.meta` sidecar's `compressed` field matches the serializer, not the stale input value

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE uses an in-repo `cargo test` command, not literal `/tmp/mreNNN/` paths -- deliberate, precedented local adaptation for a crate-internal algorithm defect (matches BUG-298/BUG-300's own already-verified shape in this repo), not an oversight | — |
| D3 | Cross-Reference Integrity | 🟡 | 🟢 | State was already flipped to Verified (with a History row) by a prior pass that left no `## Verification Record`, and neither `src/serialization.rs` nor `tests/serialization_test.rs` carried the canonical FI027 backreference (only `Fix(BUG-348)`/`test_kind:` markers existed) | Added canonical backreference comment adjacent to each existing marker in both files; re-verified via `grep -rn 'BUG-348' src/ tests/` |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 open | 1/1 |

**Reproduced:** YES -- exit 0 (`test_game_state_save_meta_compressed_flag_matches_actual_compression` ... ok), 2026-08-18. Full `tiles_tools` scoped suite (`cargo nextest run -p tiles_tools --all-features`, 272 passed / 0 failed) re-confirmed post-fix.
