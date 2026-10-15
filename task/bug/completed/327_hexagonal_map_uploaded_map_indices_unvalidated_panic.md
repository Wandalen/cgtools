# BUG-327: `hexagonal_map` deserializes a dropped-in map JSON straight into live state with no validation of its tile indices against the current config, panicking on the next render for any out-of-range upload

- **Severity:** High (panics on untrusted/user-supplied file input)
- **state:** Completed
- **Affects:** `examples/minwebgl/hexagonal_map/src/main.rs`
- **Component:** examples/minwebgl/hexagonal_map
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`loaded_map_sync` deserialized a dropped map JSON payload and assigned it straight to `map` with
no validation of its tiles' `owner_index`/`object_index` against the currently-loaded
`game_config`. An out-of-range value (a hand-edited file, or a map saved under a `Config` with more
players/objects than the one it's re-loaded into) panics on the very next render via
`game_config.player_colors[hex.owner_index.0 as usize]` /
`object_props[object_index.0]`.

## Impact

**Who is affected:** any user dropping in a map JSON file that doesn't match the currently-loaded
config's player/object counts.

**What breaks:** the whole demo panics on the render immediately following the upload -- a file
that deserializes successfully (well-typed JSON) can still carry index fields out of range for the
`Config` it's being applied against, and nothing catches that before the panicking index
expression runs.

**Entity Scope:** `None` -- confined to this crate's own map-upload boundary.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by tracing the file-upload boundary's deserialized value through to its first consuming
use rather than trusting `serde_json::from_str::<Map>()` returning `Ok` as proof the data is safe
to index with. Independently verified by the orchestrating session: `game_config.player_colors[...]`
and `object_props[...]` are unchecked slice-index operations with no bounds guard upstream of
`loaded_map_sync`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p hexagonal_map --test loaded_map_validation_test
```
**Expected** (fixed): a map JSON with an out-of-range `owner_index`/`object_index` is rejected (or
clamped/reported) before being assigned to live state, never reaching the panicking index
expression. **Actual** (pre-fix): the same malformed payload was assigned directly to `map` with
no check, panicking on the next render.

## Root Cause

The file-upload boundary trusted an uploaded map's indices to already be in range for whatever
`Config` happens to be currently loaded, with no cross-check between the two independently-sourced
values (the uploaded file and the live config).

## Why Not Caught

No test exercised `loaded_map_sync` against a malformed or config-mismatched map payload -- the
happy path (a map saved under the same config it's later loaded into) always worked, so nothing
exposed the missing boundary validation.

## Fix Applied (2026-08-18)

Added `map_tile_indices_in_range`, a new validator function checking every tile's
`owner_index`/`object_index` against `game_config`'s actual `player_colors`/`object_props` lengths
before the deserialized map is assigned to live state; `loaded_map_sync` now calls this validator
and rejects (logs and discards) an out-of-range payload instead of assigning it. Added
`tests/loaded_map_validation_test.rs`: constructs both an in-range and an out-of-range map payload
against a fixed `Config`, asserting the validator accepts the former and rejects the latter.

## Verification

- **Pre-fix (RED):** reverted `loaded_map_sync` to assign the deserialized map unconditionally; new
  test failed (out-of-range payload accepted, would panic downstream).
- **Post-fix (GREEN):** `cargo test -p hexagonal_map` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p hexagonal_map` and
  `cargo clippy --all-targets --all-features -p hexagonal_map -- -D warnings` both clean.

## Generalized Version

A deserializer returning `Ok` only proves the input is well-typed JSON -- it says nothing about
whether index/reference fields inside that payload are in range for a *different*,
independently-loaded piece of state (here, a `Config` loaded separately from the map file itself).
Any file-upload or similar external-input boundary that later indexes into currently-live state
using fields from the uploaded data needs its own explicit range validation at the boundary, not
just a successful deserialize.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-327 after a fresh on-disk collision scan. |
