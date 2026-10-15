# BUG-138: `Sequence::update` selects one player too far ahead via a `binary_search_by` off-by-one

- **Severity:** High (the first player in almost any multi-player `Sequence` is silently skipped
  entirely on the very first frame — not a rare edge case, the common/default path)
- **state:** Completed
- **Affects:** Any caller of `Sequence<T>::update` (via the `AnimatablePlayer` trait) with 2+
  players — i.e. every real use of `Sequence`, since `Sequence::new` rejects fewer than 2
- **Component:** `module/helper/animation` (`src/sequencer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same function as BUG-139 (independent root cause — index selection vs.
  delta-accumulation); first bug filed for `animation` this session

## Symptom

```rust
let players = vec!
[
  Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ),                 // delay=0.0
  Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_delay( 1.0 ),
];
let mut sequence = Sequence::new( players ).unwrap();
sequence.update( 0.1 );   // elapsed=0.1: player 0's delay has passed, player 1's has not

// Wrong (pre-fix): sequence.current_id_get() == 1 -- player 1 selected, player 0 never touched
// Correct (post-fix): sequence.current_id_get() == 0
```

## Impact

**Who is affected:** Any `Sequence<T>` with 2+ players (the only valid configuration —
`Sequence::new` rejects fewer than 2 via `SequenceError::NotEnough`).

**What breaks:** `binary_search_by`'s `Err( id )` return value is the *insertion point* — the
index of the first player whose `delay_get()` has NOT yet been reached. The player that should
actually be active is the one immediately before it (`id - 1`), since that's the last player
whose delay has already passed. The code used `id` directly for both the `Ok` and `Err` cases,
so on every frame where `elapsed` doesn't land on a player's delay *exactly* (the overwhelming
majority of frames, at any nonzero frame rate), the wrong — one-too-far-ahead — player is
selected. In the concrete two-player case above, this means player 0 (delay 0.0) is *never*
selected at all until `elapsed` first exceeds player 1's own delay, at which point `Sequence`
jumps straight to player 1 having skipped player 0's entire animation.

**Magnitude:** Not a crash — a silently wrong active player, and (combined with the dispatch
match's `Less` arm calling `current.update( self.elapsed )` on the wrong player) a silently wrong
animated value. For a two-player sequence this means the first clip is skipped outright; for
longer sequences, every transition is shifted one player ahead of where it should be.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #74, a targeted code review of `module/helper/animation` under the standing bug-hunt
mandate (fourth crate reviewed this session, following browser_input, gpu_hal, and tiles_tools).
An `Explore` subagent dispatch flagged the `Ok( id ) | Err( id ) => id` pattern as conflating two
different `binary_search_by` return semantics; confirmed by direct read of `Sequence::update` and
by tracing `binary_search_by`'s documented `Err` semantics (insertion point) against the
non-decreasing-by-delay invariant `Sequence::new` establishes and validates.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test sequencer_test test_sequence_update_selects_player_whose_delay_has_already_passed 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_sequence_update_selects_player_whose_delay_has_already_passed ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting the `Err( id ) => id.saturating_sub( 1 )`
adjustment back to `Ok( id ) | Err( id ) => id`, then restoring the fix immediately after
capturing the failure):
```
thread 'tests::test_sequence_update_selects_player_whose_delay_has_already_passed' panicked at module/helper/animation/tests/sequencer_test.rs:300:5:
assertion `left == right` failed: binary_search's Err(id) insertion-point was used directly instead of id-1, selecting the wrong (not-yet-started) player
  left: 1
 right: 0
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test sequencer_test test_sequence_update_selects_player_whose_delay_has_already_passed
# 1 passed = fixed; 1 failed (left: 1, right: 0) = bug present
```

**Known MRE limitation (check 205):** none — `Sequence<T>` is pure, synchronous,
dependency-free state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `binary_search_by`'s `Err( id )` insertion-point is used as-is instead of `id - 1`, selecting the wrong player whenever `elapsed` doesn't exactly match a player's delay. | ✅ Root Cause | Direct read of `Sequence::update`: `Ok( id ) \| Err( id ) => id` treats both return variants identically, despite `Err`'s documented "insertion point" semantics differing from `Ok`'s "exact match" semantics. | E1 |
| H2 | The bug requires more than two players to be observable. | ❌ Falsified | The MRE uses exactly two players — the very first frame after any nonzero, non-delay-exact `elapsed` already exposes it. | E2 |
| H3 | `Ok( id )` (exact delay match) also needs the same `- 1` adjustment. | ❌ Falsified | `Ok( id )` means `delay_get() == elapsed` for player `id` itself — that player has just reached its own delay and is correctly the active one; no adjustment needed there. | Direct trace of `binary_search_by`'s contract |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/sequencer.rs`, pre-fix `Sequence::update` | `let index = match index { Ok( id ) \| Err( id ) => id };` — no distinction between the two `Result` variants' actual index semantics. | H1 ✅ |
| E2 | MRE run, reverted logic | `left: 1, right: 0` for a two-player sequence at `elapsed = 0.1` (strictly between both players' delays) — confirms the very first non-exact-match frame already exposes the defect. | H1 ✅, H2 ❌ |

## Root Cause

```
Sequence::update():
  index = players.binary_search_by( |p| p.delay_get().partial_cmp( &elapsed ) )
  // Err( id ) means: id = count of players whose delay_get() < elapsed
  //                    = index of the first player whose delay has NOT yet passed
  // The correct active player is the LAST one whose delay HAS passed: id - 1
  index = match index { Ok( id ) | Err( id ) => id }   // BUG: Err case needs `- 1`
```

`binary_search_by` over a slice sorted ascending by `delay_get()` (an invariant `Sequence::new`
validates) returns `Err( id )` as the insertion point that keeps the slice sorted — i.e. the
index of the first element that is *not* less than the search target. For this search (
`player.delay_get().partial_cmp( &elapsed )`), that means `id` is the first player whose delay
hasn't been reached yet, not the player whose delay was most recently reached.

## Why Not Caught

The only existing `Sequence`-specific test (`test_sequence_new_rejects_unsorted_players`)
exercises solely the constructor's `Unsorted` error path — no test ever called `.update()` on a
valid, multi-player `Sequence`, so this entire code path had zero coverage.

## Fix Location

`module/helper/animation/src/sequencer.rs`, `Sequence::update`:

```rust
// before
let index = match index
{
  Ok( id ) | Err( id ) => id
};

// after
let index = match index
{
  Ok( id ) => id,
  Err( id ) => id.saturating_sub( 1 ),
};
```

No signature change — pure internal-logic fix.

## Prevention

Added `test_sequence_update_selects_player_whose_delay_has_already_passed` to
`tests/sequencer_test.rs`, exercising `Sequence::update` on a valid two-player sequence for the
first time in this crate's test suite.

**Pitfall:** invisible whenever `elapsed` exactly equals a player's delay (the `Ok` branch,
already correct) — only exposed when `elapsed` falls strictly between two players' delays, which
is the common case at any real frame rate but easy to miss with delay-exact hand-picked test
inputs.

## Generalized Version

**Broken assumption:** "`Ok`/`Err` from `binary_search_by` both carry the same index semantics
and can share a match arm." False in general — `Ok` is an exact match at that index; `Err` is an
insertion point one position past where a `Less`-comparing search would stop, which for a
"find the last element satisfying a condition" query (as opposed to "find an exact element") is
off by one from the desired result.

**Confirmed general rule:** when `binary_search`/`binary_search_by` is used to answer "which
element is the most recent one whose condition already holds" (rather than "does this exact
element exist"), the `Err` branch's insertion point must be adjusted by `- 1` (with a
`saturating_sub`/bounds check for the "before the first element" case) — collapsing `Ok`/`Err`
into one arm is a strong signal this distinction was overlooked.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #74's targeted code review of `module/helper/animation`; confirmed by direct read of `Sequence::update`'s collapsed `Ok`/`Err` match arm against `binary_search_by`'s documented insertion-point semantics. |
| 2026-08-16 | fixed | `Err( id )` now maps to `id.saturating_sub( 1 )`; `Ok( id )` unchanged. |
| 2026-08-16 | verified | Added `test_sequence_update_selects_player_whose_delay_has_already_passed`; confirmed it fails against the reverted pre-fix logic with the exact predicted wrong index (`left: 1, right: 0`) and passes against the fix. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `Sequence::update`'s `index` match (confirmed `Err( id ) => id.saturating_sub( 1 )` genuinely present, `Fix(BUG-138)`/`Root cause`/`Pitfall` comment intact) and `test_sequence_update_selects_player_whose_delay_has_already_passed` (non-tautological: asserts `current_id_get() == 0` and `value_get() == 1.0` on a two-player sequence). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-138 through BUG-143, BUG-147 through BUG-149 together): 40/40 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-138 through BUG-149 together (12-bug batch spanning `animation` and `behaviour_tree`). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass traced the insertion-point semantics by reasoning alone; adversarial pass required actually observing the FAIL against the reverted logic — closed via revert-test-restore, captured text (`left: 1, right: 0`) matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Shares a function with BUG-139 but an independent root cause (index selection, not delta accumulation) — cross-referenced explicitly in this report's header. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether `Ok( id )` also needed adjustment (H3, falsified by direct trace of exact-match semantics) and whether 2 players was sufficient to expose it (H2, confirmed). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped `binary_search` usage across `animation` — this is the only call site in the crate. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` `src/sequencer.rs` + `tests/sequencer_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to the `index` match; no other logic touched. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing player-selection contract now actually honored. | — |

**Reproduced:** YES — temporarily reverting the `Err` adjustment back to `Ok( id ) | Err( id ) =>
id` and running
`cargo test --test sequencer_test test_sequence_update_selects_player_whose_delay_has_already_passed`
produced the exact predicted wrong index (`left: 1, right: 0`); restoring the fix returned the
test to passing, 2026-08-16. Full-suite and clippy verification deferred to a combined run after
BUG-139 (same function, same file) — see BUG-139's Verification Record for the joint full-suite
result.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/sequencer.rs` | `Sequence::update`: `Err( id )` now maps to `id.saturating_sub( 1 )` instead of `id` directly. `Fix(BUG-138)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/sequencer_test.rs` | New test (`bug_reproducer(BUG-138)`, 5-section doc comment) — `test_sequence_update_selects_player_whose_delay_has_already_passed`. Added `AnimatablePlayer` to the `animation` import list. |
