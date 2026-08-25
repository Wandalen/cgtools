# BUG-133: `turn_order_rebuild` desyncs `current_turn_index` from the acting entity

- **Severity:** Medium (silently reassigns whose turn it is, mid-round, with no explicit
  `turn_end()` call — no panic, no compile error, just a wrong active participant)
- **state:** Completed
- **Affects:** Any caller of `TurnBasedGame::participant_add`/`participant_remove` invoked while
  a round is already in progress (`current_turn_index > 0`, or any add/remove that reshuffles
  entities ahead of the current index)
- **Component:** `module/helper/tiles_tools` (`src/game_systems.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — third bug filed for this crate this session; independent of
  BUG-131/BUG-132 (different module, different mechanism)

## Symptom

```rust
let mut game = TurnBasedGame::new();
game.participant_add( 1, 100 );
game.participant_add( 2, 90 );
game.participant_add( 3, 80 );
game.turn_end();
assert_eq!( game.current_turn(), Some( 2 ) ); // participant 2's turn, correctly

game.participant_remove( 1 ); // removes an unrelated, EARLIER-ordered participant

// Wrong (pre-fix):
game.current_turn() == Some( 3 )  // silently skipped participant 2's turn entirely

// Correct (post-fix):
game.current_turn() == Some( 2 )  // still participant 2's turn -- nobody ended it
```

## Impact

**Who is affected:** Any caller of `participant_add`/`participant_remove` mid-round — the
realistic case of a new combatant joining, or an existing one dying/fleeing, while a turn-based
encounter is already underway.

**What breaks:** `turn_order_rebuild` re-sorts the turn order by initiative on every add/remove,
then clamps `current_turn_index` purely numerically against the new order's length — it never
remaps the index to the same entity. Any add/remove that shifts entities into or out of the
positions before the current index silently reassigns "whose turn it is" to a different entity,
with no `turn_end()` ever called.

**Magnitude:** Not a crash — a silently wrong `Option<u32>` consumed directly by game logic
(whose turn to render, whose input to accept) with no error signal. The skipped participant's
turn is lost entirely for that round; the wrongly-selected participant acts out of order.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #66, a targeted code review of `tiles_tools` under the standing bug-hunt mandate. The
reviewing agent flagged that `turn_order_rebuild` (`src/game_systems.rs` lines 273-285)
clamps `current_turn_index` via `.min(self.turn_order.len() - 1)` only — no identity remapping.
Independently confirmed by direct reading of the full `TurnBasedGame` impl (lines 125-296) and
by hand-tracing a `participant_remove` mid-round scenario against the exact sort/clamp logic.

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test game_systems_test --features enabled test_turn_order_rebuild_preserves_current_entity_across_removal 2>&1 | tail -10
```

**Expected** (post-fix):
```
test test_turn_order_rebuild_preserves_current_entity_across_removal ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `turn_order_rebuild` to its exact
pre-fix numeric-clamp-only form, then restoring the fix immediately after capturing the
failure):
```
assertion `left == right` failed: removing participant 1 shifted the current turn away from
participant 2, who was never removed and never had turn_end() called
  left: Some(3)
 right: Some(2)
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test game_systems_test --features enabled test_turn_order_rebuild_preserves_current_entity_across_removal
# 1 passed = fixed; 1 failed (left: Some(3), right: Some(2)) = bug present
```

**Known MRE limitation (check 205):** none — `TurnBasedGame` is pure, synchronous,
dependency-free state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `turn_order_rebuild` clamps `current_turn_index` numerically only, never remapping it to the same entity_id across a re-sort. | ✅ Root Cause | Direct read of `src/game_systems.rs` lines 273-285 pre-fix: `self.current_turn_index = self.current_turn_index.min(self.turn_order.len() - 1);` — no entity-identity lookup anywhere in the function. | E1 |
| H2 | The bug requires the newly added/removed participant to have higher initiative than the current entity. | ❌ Falsified | The MRE removes participant 1 (initiative 100, ahead of current participant 2's 90) — but the defect is any reshuffle that changes which entity occupies the current numeric slot, not specifically initiative comparison; removing any entity positioned before the current index reproduces it, regardless of the removed entity's own initiative relative to the current one. | E2 |
| H3 | `turn_end()` itself is where the desync originates (its own index arithmetic is wrong). | ❌ Falsified | `turn_end()`'s `current_turn_index += 1` / round-wrap logic is untouched by the fix and was already correct in isolation — the MRE's `turn_end()` call (step before the removal) produces the correct `Some(2)`; the defect only manifests on the subsequent `participant_remove` call. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/game_systems.rs:273-285`, pre-fix | `turn_order_rebuild` re-sorts by initiative then clamps `current_turn_index` via `.min(len-1)` alone — no entity_id ever read or compared. | H1 ✅ |
| E2 | MRE run, reverted code | `left: Some(3), right: Some(2)` after `add(1,100); add(2,90); add(3,80); turn_end(); remove(1)` — confirms the defect fires from a plain `participant_remove` of an entity that was never the current turn holder, with no initiative-magnitude precondition. | H1 ✅, H2 ❌, H3 ❌ |

## Root Cause

```
turn_order_rebuild():
  turn_order = participants sorted by initiative desc     // re-sorted every add/remove
  current_turn_index = current_turn_index.min(len - 1)    // <- numeric clamp only

// Missing: no lookup of "which entity_id was at current_turn_index before the
// re-sort, and where does that same entity_id land after it?"
```

`current_turn_index` is a position in an array that gets rebuilt from scratch on every
`participant_add`/`participant_remove` call — but the index itself was never re-derived from
the entity it was supposed to track, only bounds-checked against the new array's length.

## Why Not Caught

The crate's existing `test_turn_based_participants` only calls `participant_add` before any
`turn_end()` (so `current_turn_index` is always `0`, coincidentally stable across every rebuild
in that test) and never calls `participant_remove` at all — no existing test exercised a
rebuild happening while `current_turn_index` already pointed partway through the order.

## Fix Location

`module/helper/tiles_tools/src/game_systems.rs`, `TurnBasedGame::turn_order_rebuild`:

```rust
// before
self.current_turn_index = self.current_turn_index.min(self.turn_order.len() - 1);

// after
self.current_turn_index = current_entity
  .and_then(|id| self.turn_order.iter().position(|&e| e == id))
  .unwrap_or_else(|| self.current_turn_index.min(self.turn_order.len() - 1));
```

where `current_entity` is captured (`self.turn_order.get(self.current_turn_index).copied()`)
*before* `self.turn_order` is reassigned. The fallback (original numeric clamp) is preserved
for the one case with no identity to recover: the previously-current entity was itself the one
removed.

## Prevention

Added `test_turn_order_rebuild_preserves_current_entity_across_removal` to
`tests/game_systems_test.rs`, covering a `participant_remove` of an earlier-ordered,
not-currently-acting entity mid-round.

**Pitfall:** invisible whenever every `participant_add`/`participant_remove` call happens
before the first `turn_end()`, or whenever removed/added entities never precede the current
turn holder in initiative order — both leave the numeric index and the identity-correct index
coincidentally equal.

## Generalized Version

**Broken assumption:** "a positional index into a collection remains valid across a rebuild of
that collection, as long as it stays within bounds." Silently false whenever the rebuild
reorders or removes elements — an in-bounds index after a reorder points to *some* valid
element, just not necessarily the *same* one the index was tracking before.

**Confirmed general rule:** whenever a mutable index tracks a specific logical entity (not a
transient scan position) across an operation that can reorder or resize its backing collection,
the index must be re-derived from the entity's own identity after the operation, not merely
clamped to the new bounds — a bounds-only clamp only prevents a panic, it does not preserve
correctness.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #66's targeted code review of `tiles_tools`; confirmed by direct read of `turn_order_rebuild` and by hand-tracing a `participant_remove` mid-round scenario. |
| 2026-08-16 | fixed | `turn_order_rebuild` now captures the current entity_id before re-sorting and remaps the index to that entity's new position, falling back to the original numeric clamp only when that entity no longer exists. |
| 2026-08-16 | verified | Added `test_turn_order_rebuild_preserves_current_entity_across_removal`; confirmed it fails against the reverted pre-fix code with the exact predicted wrong value (`left: Some(3), right: Some(2)`) and passes against the fix; full crate suite (233 tests incl. doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `turn_order_rebuild` (confirmed it captures `current_entity` before re-sorting and remaps via `position(...)`, falling back to the original numeric clamp, 3-field comment intact) and `test_turn_order_rebuild_preserves_current_entity_across_removal` (non-tautological: asserts `current_turn()` stays `Some(2)` after removing an unrelated, earlier-ordered participant). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-traced the expected desync; adversarial pass required actually observing the FAIL against the reverted pre-fix clamp, not trusting the trace — closed via revert-test-restore, captured text (`Some(3)` vs `Some(2)`) matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Third bug for `tiles_tools` this session; independent of BUG-131/132 — no cross-ref needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether an initiative-magnitude precondition was required (H2) and whether `turn_end()` itself was implicated (H3) — both falsified by the MRE and by isolating `turn_end()`'s own correctness before the removal step. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Checked the fallback path (current entity itself removed) reduces to the exact original numeric-clamp expression — confirmed no behavior change for that case. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools` `src/game_systems.rs` + `tests/game_systems_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to `turn_order_rebuild`'s body; no public API/signature change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing turn-order contract now actually honored. | — |

**Reproduced:** YES — reverting `turn_order_rebuild` to its exact pre-fix numeric-clamp-only
form and running
`cargo test --test game_systems_test --features enabled test_turn_order_rebuild_preserves_current_entity_across_removal`
produced the exact predicted wrong value (`left: Some(3), right: Some(2)`); restoring the fix
returned the full suite to 233/233 passing (including doctests) plus a clean
`cargo clippy --all-targets --features enabled,integration -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/game_systems.rs` | `TurnBasedGame::turn_order_rebuild`: captures the current entity_id before re-sorting and remaps `current_turn_index` to that entity's new position, falling back to the original numeric clamp when the entity no longer exists. `Fix(BUG-133)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/game_systems_test.rs` | New test (`bug_reproducer(BUG-133)`, 5-section doc comment) — `test_turn_order_rebuild_preserves_current_entity_across_removal`. |
