# BUG-479: `GameStateMachine` carried `state_enter_handlers`/`state_exit_handlers` fields that were never read, written, or reachable from any public API

- **Severity:** Low (no incorrect behavior -- the fields were provably inert -- but dead state
  in a public-facing struct misleads readers into believing a handler-registration feature
  exists)
- **state:** Completed
- **Affects:** Anyone reading `GameStateMachine`'s struct definition expecting
  `state_enter_handlers`/`state_exit_handlers` to be wired to `transition_to` or any other
  method.
- **Component:** module/helper/tiles_tools (`src/game_systems.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** None known.

## Symptom

```rust
// pre-fix -- src/game_systems.rs
pub struct GameStateMachine {
  // ...
  state_enter_handlers: HashMap<GameState, StateHandler>,
  state_exit_handlers: HashMap<GameState, StateHandler>,
}
```

Both maps were initialized empty in `new()` and never read or written anywhere else in the
struct's methods, including `transition_to` (the one method whose name suggests it should
invoke enter/exit handlers on a state change). `StateHandler` (the type alias for the map's
value type) had no public constructor or setter reachable from outside the module either.

## Impact

**Who is affected:** No runtime behavior was affected -- the fields were provably inert (see
Root Cause). Impact is purely to code readers: the fields' presence implies a
handler-registration feature that does not exist, which could mislead a caller into believing
`transition_to` invokes registered callbacks when it never did and never could have (no
registration API existed to populate the maps in the first place).

**What breaks:** Nothing at runtime.

**Consumer audit:** `state_enter_handlers`/`state_exit_handlers`/`StateHandler` are all private
to the struct with no accessor methods -- `grep -rn 'state_enter_handlers\|state_exit_handlers\|StateHandler'`
across the workspace, excluding this crate's own `src/game_systems.rs`, returns no matches.

**Magnitude:** 2 struct fields, 1 type alias, their initialization in `new()`.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/game_systems.rs` end to end and grepping for every
read/write site of `state_enter_handlers`/`state_exit_handlers` within the crate.

## Minimum Reproducible Example

N/A -- this is dead-code removal, not a runtime-behavior reproducer. See Prevention for how the
fix's correctness was instead verified (the struct's actually-used behavior, `transition_to`
tracking `previous_state`, has a permanent regression test).

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(game_systems_test) and test(transition_to_tracks_previous_state)'
```

## Root Cause

The two `HashMap` fields and the `StateHandler` type alias were scaffolded in anticipation of a
handler-registration feature that was never implemented -- no method was ever added to insert
into either map, and `transition_to` was never wired to look either map up on a state change.

## Why Not Caught

No test asserted on `state_enter_handlers`/`state_exit_handlers` in any way (they have no
accessors to assert against), so their permanent emptiness was invisible to the test suite --
there was nothing to fail regardless of whether the fields were wired up or not.

## Fix Location

`module/helper/tiles_tools/src/game_systems.rs`: removed `state_enter_handlers`/
`state_exit_handlers: HashMap<GameState, StateHandler>` fields and the `StateHandler` type
alias entirely from `GameStateMachine`; removed their `HashMap::new()` initialization in
`new()`. Judgment call: removed rather than implemented a real handler-registration feature --
wiring up actual enter/exit callback dispatch would require inventing an untested registration
API (what signature does a handler take? synchronous or queued? single or multiple handlers per
state?) nobody has specified, which is a feature request, not a bug fix. Since the fields were
never reachable from any public API to begin with, no caller could have been relying on them,
making removal the safe, minimal, "delete dead code, trust git history" choice per this
project's own code-management conventions.

## Prevention

New test `test_game_state_machine_transition_to_tracks_previous_state` in
`tests/game_systems_test.rs` verifies `previous_state()` correctly updates across two
`transition_to` calls -- the method's actual, real behavior, now that the dead
handler-invocation implication has been removed from the struct's shape. This is explicitly
**not** a classic fail-before/pass-after reproducer, since the removed fields were never
externally reachable and so never had observable behavior to pin -- the test instead covers the
refactored `transition_to` code path directly.

## Pitfall

A field with no accessor and no internal read/write site is not "private implementation detail
reserved for later" -- it is dead code that happens to compile, because Rust's
`#[warn(dead_code)]` lint does not fire on fields that are only ever *written* (the `HashMap::new()`
initialization in `new()` counts as a write), only on fields never referenced at all. A struct
field can be fully inert and still silently pass every lint this crate runs.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, grepping for read/write sites of `state_enter_handlers`/`state_exit_handlers`. |
| 2026-08-20 | fixed | Removed both dead fields and the `StateHandler` type alias; simplified `transition_to` accordingly. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Dead-field confirmation | — | 🟢 | Adversarial pass: grepped the entire crate (`src/`, `tests/`, `benches/`) for `state_enter_handlers`, `state_exit_handlers`, and `StateHandler` before removal -- confirmed zero read sites, zero write sites beyond `new()`'s own initialization, zero external accessors. Removal could not have broken any caller. | — |
| D2 | Full-crate regression | — | 🟢 | `cargo nextest run -p tiles_tools --all-features` -- 286/286 pass; `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` clean after removal. | — |

**Reproduced:** N/A -- dead-code removal with no observable pre-fix/post-fix behavioral delta
(see Minimum Reproducible Example and Prevention). 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/game_systems.rs` | Removed `state_enter_handlers`/`state_exit_handlers` fields and `StateHandler` type alias from `GameStateMachine`; simplified `new()` and `transition_to` accordingly; `Fix(BUG-479)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/game_systems_test.rs` | Added `test_game_state_machine_transition_to_tracks_previous_state`, covering `transition_to`'s actual (non-dead) behavior. |
