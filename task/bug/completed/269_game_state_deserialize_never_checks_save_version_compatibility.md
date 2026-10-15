# BUG-269: `GameStateSerializer::game_state_deserialize` never checks version compatibility -- `with_version` is a silent no-op and `IncompatibleVersion` can never be constructed

- **Severity:** Medium (no crash or data corruption, but a fully-implemented, documented feature
  -- version-gated save compatibility -- silently does nothing end to end)
- **state:** Completed
- **Affects:** `tiles_tools::serialization::GameStateSerializer::game_state_deserialize`,
  `SaveManager::game_state_load` (`src/serialization.rs`)
- **Component:** `module/helper/tiles_tools` (`src/serialization.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`GameStateSerializer` stores a `version: SaveVersion` field, settable via `with_version()`, and
the module implements `SaveVersion::is_compatible_with` plus a
`SerializationError::IncompatibleVersion { found, expected }` error variant with a full `Display`
impl. Despite all three pieces existing and being individually correct,
`game_state_deserialize` never called `is_compatible_with` and never constructed
`IncompatibleVersion` -- it deserialized the byte payload and returned the resulting state
unconditionally, regardless of what version that state declared or what version the reading
`GameStateSerializer` was configured with. `with_version()` therefore had no observable effect on
any code path.

## Impact

**Who is affected:** any caller relying on the module's documented "Version Management:
Backward compatibility for save files" feature (module-level doc comment) to reject save data
from an incompatible version -- e.g. a game shipping a breaking save-format change that expects
old saves to be rejected with a clear error rather than deserialized (potentially into a
`SerializableGameState` whose fields no longer match what the newer game logic expects).

**What breaks:** loading a save written by an incompatible version silently succeeds instead of
returning `SerializationError::IncompatibleVersion`. Since `SerializableGameState`'s own shape
did not change across the versions in this crate's test fixtures, no field-level corruption is
observable today -- the defect is that the *gate* itself never runs, not that a specific bad value
gets through right now.

**Entity Scope:** `None` -- source-level missing-validation defect, not entity directory
instances.

## How Discovered

During this session's Group J review of `tiles_tools/src/serialization.rs`, a grep for every call
site of `is_compatible_with` and every construction site of `IncompatibleVersion` found both
existed only at their own definitions (plus `is_compatible_with`'s own isolated unit test in
`tests/serialization_test.rs`) -- neither was reachable from `game_state_serialize` /
`game_state_deserialize`, the only real read/write path through the type.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tiles_tools --all-features --test serialization_test test_deserialize_rejects_incompatible_major_version
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary direct-source-edit revert of the fix, real run
alongside this session's other 2 then-reverted bugs, `--no-fail-fast`): 1 failed --
`thread '...' panicked at .../serialization_test.rs:195:3: expected IncompatibleVersion, got
Ok(SerializableGameState { metadata: SaveMetadata { version: SaveVersion { major: 1, minor: 0,
patch: 0, .. }, .. }, .. })` (`serialization_test` target: 9 passed, 1 failed).

## Root Cause

`GameStateSerializer::game_state_deserialize` (pre-fix), abbreviated:
```rust
pub fn game_state_deserialize(&self, bytes: &[u8]) -> Result<SerializableGameState, SerializationError> {
  let data = if self.compress { /* decompress */ } else { bytes.to_vec() };
  let state = match self.format {
    SerializationFormat::Json => serde_json::from_slice(&data)?,
    SerializationFormat::Bincode => { /* .. */ }
    SerializationFormat::Ron => { /* .. */ }
  };
  Ok(state)   // <- returned unconditionally, `self.version` never consulted
}
```
`self.version` (the field `with_version()` sets) was written but never read anywhere in the type's
implementation. `SaveVersion::is_compatible_with` and `SerializationError::IncompatibleVersion`
were both fully implemented and independently unit-tested, but nothing ever called the former or
constructed the latter -- the two halves of the feature (the check, and the thing that uses it)
were never wired together at their one real call site.

## Why Not Caught

`test_save_version_compatibility` tests `SaveVersion::is_compatible_with` in complete isolation,
never through `game_state_deserialize`. Every round-trip test in `serialization_test.rs` uses the
default `GameStateSerializer::new()` (current version) to both write and read
`GameStateSerializer::basic_game_state_create`-produced states (also current version), so a
version mismatch was never exercised end-to-end through the one function that should have
detected it.

## Fix Applied (2026-08-17)

**`src/serialization.rs`:** in `game_state_deserialize`, after the `match self.format { .. }`
deserialization and before returning, added a compatibility check: if
`!self.version.is_compatible_with(&state.metadata.version)`, return
`SerializationError::IncompatibleVersion { found: state.metadata.version, expected:
self.version.clone() }` instead of `Ok(state)`. Direction (`self`, the reader's configured
version, must be compatible-with `state.metadata.version`, the data's declared version) matches
`is_compatible_with`'s own existing test semantics ("v1.1 can read v1.0"). Also updated the `#
Errors` doc comments on `game_state_deserialize` and `SaveManager::game_state_load` to document
the new error condition.

**`tests/serialization_test.rs`** (new test):
`test_deserialize_rejects_incompatible_major_version` serializes a default-version state with a
default-version writer, then deserializes the resulting bytes with a reader configured via
`.with_version(SaveVersion::new(2, 0, 0))`, and asserts the result matches
`Err(SerializationError::IncompatibleVersion { .. })`.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tiles_tools --all-features --test serialization_test
  test_deserialize_rejects_incompatible_major_version` -- pre-fix (temporary direct-source-edit
  revert of the functional block, real run): fails, `expected IncompatibleVersion, got Ok(..)`.
  Post-fix (restored): 1 passed.
- `cargo test -p tiles_tools --all-features --no-fail-fast` (full scoped suite, this session's
  other 3 bugs simultaneously reverted): `serialization_test` target 9 passed, 1 failed -- exactly
  and only the new test, with all 9 other pre-existing round-trip/compatibility/manager cases
  still passing (confirming the fix cannot regress any existing round-trip test, since every one
  of them uses matching current-version writer/data). Post-fix (all 4 restored): full suite green
  across all 10 test binaries (`serialization_test`: 10/10) plus 40 doctests, 0 failed.
- `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a fully-implemented, individually-unit-tested helper
(`is_compatible_with`) and a fully-defined, `Display`-formatted error variant
(`IncompatibleVersion`) being present in a module is evidence that the feature they belong to
works end to end. Neither fact says anything about whether the one real call site that should
invoke them actually does -- "the pieces exist and each one is individually correct" is a
different, weaker claim than "the feature is wired together," and a builder setter
(`with_version`) whose stored value is never read back anywhere is the same silent-no-op pattern
regardless of how well-tested the unused value's own type is in isolation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group J review of `tiles_tools/src/serialization.rs`. Root cause: `GameStateSerializer::game_state_deserialize` never called `SaveVersion::is_compatible_with` or constructed `SerializationError::IncompatibleVersion`, despite both being fully implemented and unit-tested in isolation -- `with_version()` was a silent no-op, and the module's own documented "Version Management" feature never actually ran. Fixed by adding the compatibility check in `game_state_deserialize`, returning `IncompatibleVersion` on mismatch instead of the deserialized state. Verified via 1 new native unit test (confirmed fail pre-fix via a combined `--no-fail-fast` run with this session's other 2 then-reverted bugs -- real failure, exact expected `Ok(..)` payload shown -- and pass post-fix), the full scoped suite (10/10 in `serialization_test`, all 10 binaries + 40 doctests green), and clean clippy. Filed as BUG-269 after a fresh on-disk scan immediately before filing found 268 (this session's own events.rs bug) as the highest existing ID. |
