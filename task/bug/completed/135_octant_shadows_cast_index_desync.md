# BUG-135: `octant_shadows_cast` desyncs direction index from filtered neighbor position

- **Severity:** Medium (silently mis-shadows tiles under shadowcasting FOV for specific
  single-obstacle placements — no panic, no compile error, just a wrong visibility result)
- **state:** Completed
- **Affects:** Any caller of `FieldOfView::fov_calculate`/`line_of_sight` using the default
  `FOVAlgorithm::Shadowcasting` algorithm, once any obstacle is present within `max_range`
- **Component:** `module/helper/tiles_tools` (`src/field_of_view.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — fifth bug filed for this crate this session; independent of
  BUG-131/132/133/134 (different module, different mechanism)

## Symptom

```rust
let fov = FieldOfView::with_algorithm( FOVAlgorithm::Shadowcasting );
let viewer = SquareCoord::<EightConnected>::new( 0, 0 );
let blocker = SquareCoord::<EightConnected>::new( -3, -1 ); // single tile, not a wall

let visibility = fov.fov_calculate( &viewer, 4, | coord | *coord == blocker );

// Wrong (pre-fix):
visibility.is_visible( &SquareCoord::<EightConnected>::new( -4, -1 ) ) == false // falsely shadowed

// Correct (post-fix):
visibility.is_visible( &SquareCoord::<EightConnected>::new( -4, -1 ) ) == true // reachable around the single blocker
```

## Impact

**Who is affected:** Any caller of the default `Shadowcasting` FOV algorithm — the common case
for roguelike exploration, tactical vision, and fog-of-war — once terrain has at least one
blocking tile within range.

**What breaks:** `octant_shadows_cast` derives each neighbor's direction-slot index `i` from
`neighbors.iter().filter(...).enumerate()` — filtering already-visited neighbors *before*
enumerating desyncs `i` from the fixed slot that neighbor actually occupies in the unfiltered
`pos.neighbors()` array. The subsequent octant-membership test then evaluates the wrong slot,
which can make a legitimately-reachable tile silently absent from the visibility map for a
specific octant even though a different, unblocked path to it exists.

**Magnitude:** Not a crash — a specific tile within range and not behind any continuous
obstacle chain is silently marked not-visible (or, for other index shifts, visible when it
should be shadowed). The 8-fold redundant per-octant sweep design means most inputs are
unaffected — empirical search found divergence requires a specific obstacle placement relative
to the target tile, not any single obstacle anywhere.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #66, a targeted code review of `tiles_tools` under the standing bug-hunt mandate. The
reviewing agent flagged that `octant_shadows_cast` (`src/field_of_view.rs` line 393, pre-fix)
computed `i` via `neighbors.iter().filter(...).enumerate()`, decoupling the index from the
neighbor's true fixed position. The code-level defect was unambiguous on inspection, but no
divergent output scenario was immediately obvious by hand-tracing (the algorithm's 8-octant
redundancy makes the interaction hard to predict analytically). Confirmed observable via a
from-scratch Python simulation implementing both the buggy and fixed variants side by side
(`fov_sim.py`, scratch file, not committed) — an unobstructed field and two straight-wall
obstacle shapes showed **no** divergence across 13 range/obstacle combinations; an adversarial
random-obstacle search (300 trials, 3–10 random blocking tiles each) then found genuine
divergence starting at trial 6, which a greedy minimization pass reduced to a single blocking
tile.

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test integration_tests --features enabled,integration test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile 2>&1 | tail -10
```

**Expected** (post-fix):
```
test integration::field_of_view_tests::test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `octant_shadows_cast`'s index
computation to its exact pre-fix `filter().enumerate()` form, then restoring the fix immediately
after capturing the failure):
```
thread '...test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile' panicked at
module/helper/tiles_tools/tests/./integration/field_of_view_tests.rs:597:3:
target at distance 4 was falsely shadowed by a single non-blocking-path obstacle
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test integration_tests --features enabled,integration test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile
# 1 passed = fixed; 1 failed (falsely shadowed) = bug present
```

**Known MRE limitation (check 205):** none — `FieldOfView` is pure, synchronous,
dependency-free state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `octant_shadows_cast` computes the octant-membership index from a filtered iterator, decoupling it from the neighbor's true fixed array position. | ✅ Root Cause | Direct read of `src/field_of_view.rs` line 393 pre-fix: `neighbors.iter().filter( \| n \| !visited_positions.contains( *n ) ).enumerate()` — `enumerate()` runs *after* `filter()`, so `i` numbers only the surviving neighbors, not their original slots. | E1 |
| H2 | The bug requires a continuous multi-tile wall to produce observable divergence. | ❌ Falsified | Simulation across an unobstructed field (5 ranges) and two straight-wall shapes (4 ranges × 2 shapes = 8 combinations) found **zero** divergence in all 13 cases — walls happen to be a case the 8-octant redundancy compensates for symmetrically. A single, non-wall blocking tile at a specific offset is what exposed it. | E2, E3 |
| H3 | The defect can only ever suppress a tile (never wrongly reveal a shadowed one), so it is a strict subset of correct output. | ❌ Falsified as a general claim (not exercised by this MRE, but not established either) | Not directly tested — the minimized MRE demonstrates the suppression direction only; the index-shift mechanism is symmetric enough that the opposite direction (wrongly revealing a tile that should be shadowed) is not ruled out by this evidence. Scoped out of this bug's MRE per the concrete-reproduction bar; the fix (restoring correct indexing) closes both directions regardless of which one this specific MRE demonstrates. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/field_of_view.rs:393`, pre-fix | `for ( i, neighbor ) in neighbors.iter().filter( \| n \| !visited_positions.contains( *n ) ).enumerate()` — `i` is assigned post-filter, never matching the neighbor's real slot in `pos.neighbors()` once any neighbor has already been visited. | H1 ✅ |
| E2 | `fov_sim.py` simulation, 300 adversarial random-obstacle trials (seed 42) | First divergence at trial 6 (8 random blockers, `max_range=4`): `missing_in_buggy=[(-4,-1)]`. Greedy single-obstacle removal minimized this to one blocker at `(-3,-1)` — same divergence, same target tile. | H1 ✅, H2 ❌ |
| E3 | `fov_sim.py` simulation, open field (5 ranges) + 2 wall shapes × 4 ranges | Zero divergence in all 13 combinations — confirms the bug is real but not exposed by every obstacle configuration, only specific ones. | H2 ❌ |
| E4 | MRE run, reverted code | Assertion failure text matches exactly: `target at distance 4 was falsely shadowed by a single non-blocking-path obstacle`, for viewer `(0,0)`, blocker `(-3,-1)`, target `(-4,-1)`, `max_range=4`. | H1 ✅ |

## Root Cause

```
octant_shadows_cast(): // per-octant, per-ring loop
  for pos in current_positions:
    neighbors = pos.neighbors()                                    // fixed 8-element array
    for (i, neighbor) in neighbors.iter()
        .filter(|n| !visited_positions.contains(*n))                // <- filters FIRST
        .enumerate():                                                // <- indexes AFTER filtering
      if octant_membership_test(i, octant, total_directions): ...    // tests the WRONG slot
```

`i` is meant to identify which of the 8 fixed compass directions `neighbor` occupies (the octant
math assumes `pos.neighbors()`'s array order is stable and meaningful). Filtering before
enumerating silently renumbers the survivors from 0, so `i` reflects "the Nth unvisited
neighbor," not "the neighbor at fixed slot N" — a different quantity entirely once any neighbor
of `pos` has already been visited, which is true for every ring beyond the first.

## Why Not Caught

Every existing FOV test used either fully open terrain or a multi-tile straight wall — both
happen to be cases the algorithm's 8-fold redundant octant sweep compensates for symmetrically,
so the aggregate `VisibilityMap` output is unaffected even though each individual octant call's
internal computation is wrong. No existing test used a single, isolated blocking tile at a
position capable of exposing the index shift's effect on the final unioned output.

## Fix Location

`module/helper/tiles_tools/src/field_of_view.rs`, `FieldOfView::octant_shadows_cast`:

```rust
// before
for ( i, neighbor ) in neighbors.iter().filter( | n | !visited_positions.contains( *n ) ).enumerate()

// after
for ( i, neighbor ) in neighbors.iter().enumerate().filter( | ( _, n ) | !visited_positions.contains( *n ) )
```

Swapping the order so `enumerate()` runs on the unfiltered iterator captures `i` from each
neighbor's real, fixed position in `pos.neighbors()`; `filter()` afterward still drops
already-visited neighbors from the loop body without renumbering the survivors.

## Prevention

Added `test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile` to
`tests/integration/field_of_view_tests.rs`, covering a single non-wall obstacle at a position
that requires the correct fixed-slot index to reach a nearby diagonal tile.

**Pitfall:** invisible under open terrain, straight multi-tile walls, or any obstacle shape the
8-octant redundant sweep happens to compensate for — only specific single/sparse obstacle
placements (found here via adversarial random search, not hand-derivation) expose the shift in
the final `VisibilityMap`, even though the per-octant-call computation is wrong for essentially
every ring beyond the first regardless of terrain.

## Generalized Version

**Broken assumption:** "the position of an item in a filtered iterator corresponds to its
position in the original collection." Silently false whenever `filter()` precedes `enumerate()`
and downstream logic (here, octant-membership arithmetic) depends on the index matching a fixed,
externally-meaningful slot rather than a transient enumeration of survivors.

**Confirmed general rule:** when an index must identify a fixed structural position (a compass
direction, an array slot, a protocol field number) rather than merely a scan position, `enumerate()`
must run *before* any `filter()`/`skip()`/similar iterator adapter that can change which elements
survive — reversing the order silently renumbers survivors from zero, decoupling the index from
the meaning code downstream assumes it still carries. Iterator-adapter ordering bugs of this
shape can be entirely invisible in aggregate output when redundant computation elsewhere
(here, the 8-fold octant sweep) happens to compensate for the specific inputs tested — absence of
observed divergence under hand-picked test terrain is not proof of correctness; an adversarial
search over the input space was required to find the actual failure case.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #66's targeted code review of `tiles_tools`; code-level defect confirmed immediately by direct read, but a concrete divergent-output scenario required a from-scratch Python simulation and a 300-trial adversarial random-obstacle search, minimized to a single blocking tile. |
| 2026-08-16 | fixed | `octant_shadows_cast` now runs `enumerate()` before `filter()` so the direction index reflects each neighbor's true fixed slot in `pos.neighbors()`. |
| 2026-08-16 | verified | Added `test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile`; confirmed it fails against the reverted pre-fix code with the exact predicted symptom and passes against the fix; full crate suite (233 tests incl. doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `octant_shadows_cast` (confirmed `.enumerate().filter(...)` ordering genuinely reversed from the pre-fix `.filter(...).enumerate()`, 8-line `Fix(BUG-135)`/`Root cause`/`Pitfall` comment intact) and `test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile` (non-tautological: asserts a specific diagonal tile stays visible around one non-wall blocker, the exact minimized adversarial-search scenario). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass initially hand-traced the expected shift; adversarial pass required actually finding an observable-divergence scenario empirically (2 rounds of hand-picked scenarios found none) before accepting the finding as filing-worthy — closed via a 300-trial adversarial random search, minimization, then revert-test-restore against the minimized MRE, captured text matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Fifth bug for `tiles_tools` this session; independent of BUG-131/132/133/134 — no cross-ref needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether a multi-tile wall was required (H2, falsified by 13 zero-divergence combinations) and whether the defect direction was fully characterized (H3, explicitly scoped out rather than overclaimed). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Checked the fix preserves the loop body's use of `neighbor` (still `&C` after reordering `.enumerate()`/`.filter()`, no type change) — confirmed via successful compilation and full suite pass. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools` `src/field_of_view.rs` + `tests/integration/field_of_view_tests.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to `octant_shadows_cast`'s loop header; no public API/signature change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing shadowcasting contract now actually honored for all obstacle placements, not just symmetric ones. | — |

**Reproduced:** YES — reverting `octant_shadows_cast`'s index computation to its exact pre-fix
`filter().enumerate()` form and running
`cargo test --test integration_tests --features enabled,integration test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile`
produced the exact predicted failure (`target at distance 4 was falsely shadowed by a single
non-blocking-path obstacle`); restoring the fix returned the full suite to 233/233 passing
(including doctests) plus a clean
`cargo clippy --all-targets --features enabled,integration -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/field_of_view.rs` | `FieldOfView::octant_shadows_cast`: reorders `.filter()`/`.enumerate()` so the direction index reflects each neighbor's true fixed slot. `Fix(BUG-135)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/integration/field_of_view_tests.rs` | New test (`bug_reproducer(BUG-135)`, 5-section doc comment) — `test_shadowcasting_single_obstacle_does_not_falsely_shadow_diagonal_tile`. |
