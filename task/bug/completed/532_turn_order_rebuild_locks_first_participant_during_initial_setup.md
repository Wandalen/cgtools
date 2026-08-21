# BUG-532: `TurnBasedGame::turn_order_rebuild`'s identity-preserving remap applies during initial roster setup, locking play onto whichever participant was added first instead of the highest-initiative one

- **Severity:** Medium (silent simulation-correctness defect -- no crash, no panic, but a higher-initiative participant added after a lower-initiative one during setup is silently skipped for the entire first round, and only self-corrects once the round rolls over)
- **state:** Completed
- **Affects:** `tiles_tools::game_systems::TurnBasedGame::turn_order_rebuild`, `participant_add`, `participant_remove` (`src/game_systems.rs`)
- **Component:** `module/helper/tiles_tools` (`src/game_systems.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`examples/tiles_tools/game_systems_demo` builds its roster with 4 `participant_add` calls,
including `Enemy Archer` (initiative 110, the global highest) added *after* `Player` (initiative
80, added first). The demo's own printed "Turn Order (by initiative)" list correctly showed
`Enemy Archer` first -- but the actual simulated Round 1 skipped straight from setup to `Player`'s
turn, with `Enemy Archer` not acting until Round 2 onward. The advertised turn order and the
actually-simulated turn order disagreed for the entire first round.

## Impact

**Who is affected:** any consumer of `TurnBasedGame::participant_add` that adds participants in
anything other than strictly descending initiative order, which is the ordinary case (roster
membership is rarely known/sorted in advance) -- not a contrived edge case. Every current example
consumer (`game_systems_demo`) exercises it, and any future consumer inherits the same defect.

**What breaks:** the highest-initiative participant added after at least one other participant is
silently excluded from Round 1's turn order entirely, contradicting the crate's own advertised
"higher initiative goes first" contract for the very first round of play. No crash, no panic --
purely a silent scheduling-correctness defect, self-correcting from Round 2 onward once
`current_turn_index` has wrapped back to `0` through ordinary `turn_end()` progression.

**Entity Scope:** `None` -- source-level simulation-logic defect, not entity directory instances.

## How Discovered

Reported by a parallel sweep agent investigating `examples/tiles_tools/`'s example binaries: while
that agent's assigned scope was examples only, running `cargo run -p game_systems_demo` surfaced
`Enemy Archer` (highest initiative) missing from the printed Round 1 turn sequence, only appearing
from Round 2 onward. The agent traced this to `TurnBasedGame::turn_order_rebuild` in
`module/helper/tiles_tools/src/game_systems.rs` but correctly declined to fix it as out of its
assigned `examples/tiles_tools/` target area, reporting the finding back instead. Investigated and
fixed directly as a follow-up, since it is a genuine, distinct library defect with the entire
"entire cgtools repo" bug-hunting scope in play.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tiles_tools --test game_systems_test bug_reproducer_bug_532_turn_order_rebuild_locks_first_participant_during_setup -- --nocapture
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary adversarial revert -- see Verification): `left: Some(1),
right: Some(2)`, exit 101.

Also directly observable without any test harness:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo run -p game_systems_demo 2>&1 | grep -A6 "Round 1"
```
Pre-fix: `Round 1` opens with `Player's turn`, no `Enemy Archer's turn` line anywhere in Round 1.
Post-fix: `Round 1` opens with `Enemy Archer's turn`, matching the printed turn-order list.

## Root Cause

`turn_order_rebuild` (pre-fix), abbreviated:
```rust
fn turn_order_rebuild(&mut self) {
  let current_entity = self.turn_order.get(self.current_turn_index).copied();
  // ... sort participants by initiative, rebuild self.turn_order ...
  if !self.turn_order.is_empty() {
    self.current_turn_index = current_entity
      .and_then(|id| self.turn_order.iter().position(|&e| e == id))
      .unwrap_or_else(|| self.current_turn_index.min(self.turn_order.len() - 1));
  }
}
```
This identity-preserving remap (BUG-133's own fix) is correct once play has genuinely started --
it exists precisely to keep "whose turn it is" stable across a mid-round roster change. But it ran
unconditionally, including on the very first `participant_add` calls before `turn_end()` has ever
been called. The first `participant_add` on an empty game has no `current_entity` (old `turn_order`
is empty), so it falls through to the numeric fallback and lands on index `0`. Every subsequent
`participant_add` during the same setup phase then treated *that* arbitrary index-0 fallback as a
real identity worth preserving, remapping to wherever the first-added participant landed in the
newly-sorted order -- regardless of whether a later add introduced someone with higher initiative
who should now rightfully occupy index 0 instead.

## Why Not Caught

Both pre-existing tests exercising this path (`test_turn_based_participants`,
`test_turn_order_rebuild_preserves_current_entity_across_removal`) happen to add their
highest-initiative participant *first*, so the wrongly-locked-in entity and the correct one were
always identical by coincidence -- the defect had no way to manifest under either test's fixture.
It only surfaces when a higher-initiative participant is added *after* a lower-initiative one,
still during initial setup, which is exactly `game_systems_demo`'s own roster-construction order.

## Fix Applied (2026-08-21)

**`src/game_systems.rs`:**
- Added a `game_started: bool` field to `TurnBasedGame`, initialized `false` in `new()`.
- `turn_end()` now sets `self.game_started = true` immediately after confirming `turn_order` is
  non-empty (i.e., a real turn is genuinely about to be consumed).
- `turn_order_rebuild()`'s remap now branches on `self.game_started`: when `true`, applies the
  existing BUG-133 identity-preserving remap unchanged; when `false` (still in initial roster
  setup), resets `current_turn_index` to `0` -- always tracking whoever currently sorts first by
  initiative, matching the crate's own advertised ordering contract.

**`tests/game_systems_test.rs`** (new test, inserted directly after the BUG-133 regression test):
`bug_reproducer_bug_532_turn_order_rebuild_locks_first_participant_during_setup` adds a
low-initiative participant first, then a higher-initiative one, and asserts the higher-initiative
participant is immediately current -- reproducing `game_systems_demo`'s exact roster-construction
pattern in miniature.

## Verification

`longrun`-detached, from repo root:
- `cargo nextest run -p tiles_tools bug_reproducer_bug_532 turn_order`: both the new test and the
  pre-existing BUG-133 regression test pass, 2/2.
- Adversarial re-check (Tier 2 Dual-Role Self-Check): temporarily changed `if self.game_started` to
  `if true` (forcing the pre-fix unconditional remap), re-ran the new test alone -- failed exactly
  as predicted (`left: Some(1), right: Some(2)`), confirming genuine RED against the original
  defect. Restored the real fix immediately after; `git diff --stat` confirmed only the intended
  fix lines remain (no leftover marker), and a `grep` for the marker text returned zero matches.
- `cargo nextest run -p tiles_tools -p game_of_life -p ecs_collision_demo -p tactical_rpg -p
  game_systems_demo` (full scoped suite across the crate and every example consumer touched by
  this session's `tiles_tools` sweep): 84/84 passed, 0 skipped.
- `cargo clippy -p tiles_tools -p game_of_life -p ecs_collision_demo -p tactical_rpg -p
  game_systems_demo --all-targets --all-features -- -D warnings`: clean, exit 0.
- `cargo run -p game_systems_demo`: exit 0; Round 1 now opens with `Enemy Archer's turn`, matching
  the printed "Turn Order (by initiative)" list instead of silently skipping to Round 2.

## Generalized Version

**Broken assumption:** an "identity-preserving remap across a rebuild" fix is only correct once a
genuine prior identity actually exists to preserve. Applying it unconditionally from the very
first mutation of a collection silently promotes an arbitrary default/fallback value (here, index
`0` with no real "current" entity behind it yet) into sticky, load-bearing state that then
persists incorrectly through every subsequent mutation until an unrelated event (here, the first
`turn_end()`) happens to reset it. Any "preserve X across a rebuild" fix needs its own guard for
"is there genuinely an X yet," distinct from the numeric-fallback guard for "is the collection
non-empty."

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed + fixed + verified | Reported by a parallel `examples/tiles_tools/` sweep agent as an out-of-scope finding (library code, not examples) -- `game_systems_demo`'s highest-initiative participant (`Enemy Archer`) was silently skipped for all of Round 1. Root cause: `turn_order_rebuild`'s BUG-133 identity-preserving remap applied even during initial roster setup, before any real "current" participant existed, locking play onto whichever participant was added first. Fixed by gating the remap on a new `game_started` flag, set only once `turn_end()` has genuinely run; during setup, `current_turn_index` always resets to `0` (whoever currently sorts first by initiative). Verified via 1 new regression test (confirmed fail via temporary adversarial revert of the fix, pass restored), the pre-existing BUG-133 test unaffected, the full 84-test scoped suite across the crate and all 4 touched example consumers green, clean clippy, and the real `game_systems_demo` binary's own output now matching its advertised turn order in Round 1. |
