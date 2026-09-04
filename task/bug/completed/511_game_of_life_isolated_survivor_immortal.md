# BUG-511: `SquareGameOfLife::step`/`HexGameOfLife::step` never re-evaluate a living cell that has zero living neighbors, so it stays alive forever instead of dying of isolation

- **Severity:** Medium (silent simulation-correctness defect -- no crash, no panic, but the Game of Life demo's own advertised rule is violated and its own printed statistics contradict its own world state)
- **state:** Completed
- **Affects:** `game_of_life::SquareGameOfLife::step`, `game_of_life::HexGameOfLife::step` (`src/lib.rs`)
- **Component:** `examples/tiles_tools/game_of_life` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Both `step()` implementations build a `neighbors_count : HashMap<Coord, u32>` by iterating every
currently-*alive* cell and incrementing the count for each of **its neighbors**' coordinates. A
cell's own coordinate is only ever written into this map as a side effect of being someone else's
neighbor. A living cell that currently has **zero living neighbors** is therefore never written
into `neighbors_count` at all -- not even with a `0` count -- so it is never inserted into
`next_generation`, so `world_state_update` never touches its `Cell` component. The cell silently
remains alive forever, completely bypassing the Conway rule that would otherwise correctly kill it
(`(true, 0)` falls through to the `_ => false` arm in both `step`s' match expressions, but that arm
only runs for cells that made it into the map in the first place).

## Impact

**Who is affected:** any caller of `SquareGameOfLife::step` or `HexGameOfLife::step` whose
simulation runs long enough for a living cell to end up with zero living neighbors -- which is the
normal long-run fate of a spreading-out pattern, not a contrived edge case. The shipped
`game_of_life` binary's own default hexagonal seed pattern hits this by generation 3.

**What breaks:** an "immortal" cell that should have died of isolation keeps occupying its cell
forever, silently corrupting all subsequent generations built on top of it (it counts as a live
neighbor for adjacent cells' birth/survival checks, so the corruption compounds). Concretely
observable in the demo binary's own output: `step()`'s own printed
`"Hex Generation 3: 13 living cells"` line (derived from `next_generation`) did not match the real
ECS world state 15 cells later inspected by `state_print()` -- a self-contradiction within a single
`cargo run -p game_of_life` invocation, no test infrastructure required to notice.

**Entity Scope:** `None` -- source-level simulation-logic defect, not entity directory instances.

## How Discovered

Dedicated sweep of `examples/tiles_tools/` example binaries. Ran all 12 example binaries
end-to-end (`cargo run -p <crate>`, all exit 0, no panics) and inspected their console output for
internal consistency. `game_of_life`'s hexagonal section prints both a `step()`-computed
`"Hex Generation N: X living cells"` preview line and, on the next `state_print()` call, an
independently-queried `"Living cells: [...]"` list straight from the ECS world -- these two counts
must always agree (they describe the same generation), and for generation 3 they did not (13 vs
15). Hand-deriving the hex simulation generation-by-generation from the crate's own axial neighbor
offsets (cross-checked against the pre-existing, still-passing `bug_reproducer_bug_486` test's
gen0→gen1 derivation) pinpointed exactly which two coordinates, `(-1, 2)` and `(-2, 1)`, go isolated
(zero living neighbors) at generation 2 and are consequently skipped by generation 3's rule
application.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p game_of_life --test game_test bug_reproducer_bug_511_hex_game_isolated_survivor_never_reevaluated -- --nocapture
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary direct-source-edit revert of the fix, real isolated
run): `thread '...' panicked ... (-1,2) has 0 living neighbors at generation 2 and must die of
isolation by generation 3, not persist forever unevaluated`, exit 101.

Also directly observable without any test harness, comparing two adjacent lines of real program
output:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo run -p game_of_life 2>&1 | grep -A1 "Hex Generation 3"
```
Pre-fix: `Hex Generation 3: 13 living cells` immediately followed by a `Hexagonal Generation 3`
`Living cells:` list containing **15** entries (2 more than advertised). Post-fix: both agree at 13.

## Root Cause

`HexGameOfLife::step` (pre-fix), abbreviated -- `SquareGameOfLife::step` has the identical shape:
```rust
let mut neighbors_count = HashMap::new();
for ( pos, cell ) in &mut query {
  if cell.is_alive() {
    for neighbor_coord in pos.neighbors() {
      *neighbors_count.entry( neighbor_coord.coord ).or_insert( 0 ) += 1;   // only neighbors get an entry
    }
  }
}
let mut next_generation = HashMap::new();
for ( &coord, &neighbor_count ) in &neighbors_count {                       // a 0-neighbor living cell is never a key here
  let currently_alive = self.is_cell_alive( coord );
  let should_be_alive = match ( currently_alive, neighbor_count ) {
    ( true, 2 | 3 ) | ( false, 2 ) => true,
    _ => false,                                                             // this arm -- correct for isolation -- never runs for it
  };
  next_generation.insert( coord, should_be_alive );
}
self.world_state_update( &next_generation );                                // never told about the isolated cell -> never touches it
```
`neighbors_count` is populated exclusively by *incrementing* each living cell's neighbors, never by
registering the living cell's own coordinate. Since a `HashMap::entry(..).or_insert(0) += 1` only
ever runs from the perspective of the *other* cell's neighbor list, a living cell with literally no
living neighbors never appears as a key -- not even with a `0` -- so `next_generation` never
receives a verdict for it, so `world_state_update`'s `for (&coord, &should_be_alive) in
next_generation` loop never reaches it, so its `Cell` component is never aged, revived, or killed.
It simply persists in whatever state it was already in.

## Why Not Caught

The pre-existing `bug_reproducer_bug_486_hex_game_step_applies_rules` test (from the prior BUG-486
fix, which made `step()` apply rules *at all*) only advances the built-in seed one generation, at
which point every living cell already has 2+ living neighbors -- none isolated yet, since the seed
is a tightly clustered 6-cell patch. The isolation case only emerges once the pattern has spread out
over multiple generations, which no existing test exercised. `SquareGameOfLife`'s own demo glider
never triggers it either (a glider's 5 cells stay mutually adjacent throughout its whole cycle), so
`cargo run`'s square-grid section showed no visible symptom despite carrying the identical bug --
only the hex section's independent `step()`-count vs. `state_print()`-query cross-check exposed it.

## Fix Applied (2026-08-21)

**`src/lib.rs`:** in both `SquareGameOfLife::step` and `HexGameOfLife::step`, added
`neighbors_count.entry( pos.coord ).or_insert( 0 );` for every currently-alive cell, immediately
before that cell's neighbor-increment loop. This guarantees every living cell is always a key of
`neighbors_count` (with a `0` count when genuinely isolated), so it always receives a verdict in
`next_generation` and is always visited by `world_state_update` -- letting the existing, already
correct `_ => false` rule arm actually run for it instead of being silently bypassed.
`.or_insert(0)` is a no-op when the cell already has a nonzero count from being someone else's
neighbor too, so no existing (non-isolated) transition changes behavior.

**`tests/game_test.rs`** (new test):
`bug_reproducer_bug_511_hex_game_isolated_survivor_never_reevaluated` runs the real default seed
through the same 3 `step()` calls the `game_of_life` binary's own `main()` performs, then asserts
the two hand-derived isolated coordinates `(-1, 2)` and `(-2, 1)` are dead (not merely that a
printed count changed), plus a regression guard that a legitimately-always-connected cell `(1, 0)`
still survives, to confirm the fix does not over-kill.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p game_of_life --test game_test bug_reproducer_bug_511_hex_game_isolated_survivor_never_reevaluated -- --nocapture`
  -- pre-fix (temporary direct-source-edit revert of the fix, real isolated run): 1 failed, exact
  assertion message confirming `(-1,2)` remained alive, exit 101. Post-fix (restored): 1 passed,
  exit 0.
- Adversarial re-check (Tier 2 Dual-Role Self-Check): temporarily reverted only the `HexGameOfLife`
  half of the fix (removed the added `entry(..).or_insert(0)` line, left a marker comment), re-ran
  the same scoped test -- failed again for the same reason, confirming the test genuinely detects
  the defect rather than passing vacuously. Fix restored immediately after; `git diff` confirmed no
  leftover marker text.
- `cargo test -p game_of_life` (full scoped suite): 6/6 passed (`test_cell_lifecycle`,
  `test_square_game_creation`, `test_hex_game_creation`, `test_triangular_game_creation`,
  `bug_reproducer_bug_486_hex_game_step_applies_rules`,
  `bug_reproducer_bug_511_hex_game_isolated_survivor_never_reevaluated`), plus 0 unit tests / 0 doc
  tests (none defined), all green.
- `cargo clippy -p game_of_life --all-targets --all-features -- -D warnings`: clean, exit 0.
- `cargo run -p game_of_life`: exit 0; `Hex Generation 3: 13 living cells` now matches the real
  `Hexagonal Generation 3` `Living cells:` list, which also has exactly 13 entries (previously 15).

## Generalized Version

**Broken assumption:** a `HashMap` built by "increment this cell's neighbors" alone can be trusted
to contain every cell that logically needs re-evaluating, on the theory that adjacency is symmetric
so "if I'm alive, my neighbors will see me." That's true only for cells with at least one living
neighbor -- a fully isolated living cell has no one to be "seen by," so it needs to register
*itself* as a candidate, not rely on being registered by someone else. Any per-cell simulation step
built on "populate a candidate set purely by cross-referencing from other entities" should
explicitly seed candidates for every entity of interest first (here: every currently-alive cell,
via `.entry(self_coord).or_insert(default)`), then layer cross-referencing increments on top --
never assume cross-referencing alone reaches every cell that matters.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed + fixed + verified | Found during a dedicated sweep of `examples/tiles_tools/`'s 12 example binaries: `game_of_life`'s own `cargo run` output was internally inconsistent (`step()`'s printed count vs. `state_print()`'s real ECS query disagreed, 13 vs 15, for the hexagonal grid's generation 3). Root cause hand-derived from the crate's own axial neighbor offsets: living cells with zero living neighbors never became keys of `neighbors_count`, so they were never re-evaluated by the Conway rule and stayed alive forever. Fixed identically in both `SquareGameOfLife::step` and `HexGameOfLife::step` by seeding `neighbors_count` with a `0` entry for every living cell's own coordinate before the neighbor-increment loop. Verified via 1 new regression test (confirmed fail pre-fix with the exact predicted assertion message, pass post-fix), a Tier 2 adversarial re-introduction of the bug that the test caught a second time, the full scoped 6-test suite green, clean clippy, and the real binary's own output now self-consistent. Filed as BUG-511 after a fresh on-disk scan immediately before filing found 510 already claimed by a concurrent session actor (a `shader_chunks_validate_core` bug) for what had been provisionally tracked as 510 mid-session.