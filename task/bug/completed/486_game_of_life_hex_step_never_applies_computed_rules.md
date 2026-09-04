# BUG-486: `game_of_life`'s `HexGameOfLife::step` computes its own documented survive/birth rule but never applies it -- the hex simulation never evolves

- **Severity:** Medium (no crash, no data corruption -- but one of the demo's three grid-type
  simulations, hexagonal, does nothing at all across any number of generations: the seed
  pattern is permanently frozen while `SquareGameOfLife`'s equivalent works correctly)
- **state:** Completed
- **Affects:** `examples/tiles_tools/game_of_life` (standalone lib+bin demo, no downstream consumers)
- **Component:** `examples/tiles_tools/game_of_life/src/lib.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None. Not the same defect class as BUG-484/BUG-485 (those never persist a
  *computed value that already exists*; this one never *derives* the next-generation decision
  from the count it does compute) -- filed separately despite being found in the same sweep.

## Symptom

```rust
// pre-fix -- game_of_life/src/lib.rs, HexGameOfLife::step (git show HEAD)
pub fn step( &mut self )
{
  // Hexagonal Game of Life uses different rules due to 6 neighbors instead of 8
  // Common rule: survive with 2-3 neighbors, born with 2 neighbors

  let mut neighbors_count = HashMap::new();
  {
    let mut query = self.world.query::< ( &Position< HexCoord< Axial, Pointy > >, &Cell ) >();
    for ( pos, cell ) in &mut query
    {
      if cell.is_alive()
      {
        for neighbor_coord in pos.neighbors()
        {
          *neighbors_count.entry( ( neighbor_coord.coord.q, neighbor_coord.coord.r ) ).or_insert( 0 ) += 1;
        }
      }
    }
  }

  println!( "Hex Generation {}: {} positions with neighbors", self.generation + 1, neighbors_count.len() );
  self.generation += 1;
  // <- function ends here. `neighbors_count` is computed and its *length* printed, but the
  // survive/birth rule stated two lines above is never applied to any `Cell`.
}
```

The function's own comment states the exact rule to implement ("survive with 2-3 neighbors,
born with 2 neighbors"), immediately above code that computes a per-coordinate neighbor count --
but nothing ever compares that count against the stated thresholds or touches a `Cell` component.
`self.generation` increments every call, so `generation()` reports progress while the simulation
itself is frozen.

## Impact

**Who is affected:** Anyone running the demo (`cargo run -p game_of_life`, or any code
constructing a `HexGameOfLife` directly) -- `step` is this type's entire simulation-advance
logic; there is no other path to evolve state.

**What breaks:** `HexGameOfLife::state_print`/any caller of `is_cell_alive` sees the exact same
6-cell ring pattern from `HexGameOfLife::new()` no matter how many times `step()` is called --
no cell is ever born, aged, or killed. This is a functional regression relative to its own
sibling: `SquareGameOfLife::step` (same file, ~line 135) correctly derives and applies a next
generation via its own `world_state_update`; `HexGameOfLife::step` stops one step short of doing
the same thing its own neighbor-counting logic was clearly building toward. The crate's own
readme/demo banner text describes all three grid types as having "proper neighbor calculations
and grid-aware game logic" -- false for the hex case pre-fix.

**Consumer audit:** `HexGameOfLife` has no external callers (`grep -rn "HexGameOfLife"`
workspace-wide: only this crate's own `src/lib.rs`, `src/main.rs`, and `tests/game_test.rs`).
Standalone demo crate, so nothing outside it can be affected.

**Magnitude:** 1 function, missing the derive-and-apply half of its own stated algorithm.

**Entity Scope:** None -- a code-level defect confined to this demo crate.

**Note on `TriangularGameOfLife`:** the sibling `TriangularGameOfLife::step` (~line 508) has the
same "increments `generation` without touching any `Cell`" shape, but was deliberately **not**
fixed here -- unlike the hex case, no rule is documented anywhere in this crate for a
12-neighbor triangular grid's survive/birth thresholds, so implementing one would mean inventing
game-design values with no source to verify against, not a traceable bug fix. Left as-is; not
filed as a bug (see this report's own filing session notes / the sweep's final report for
rationale).

## How Discovered

Assigned as part of a repo-wide bug/UX sweep of `examples/tiles_tools/*`, checking (among other
things) for hex-grid vs. triangle-grid coordinate confusion. No coordinate/adjacency confusion
was found -- `pos.neighbors()` correctly delegates to each coordinate type's own `Neighbors`
trait impl -- but tracing `HexGameOfLife::step` end-to-end to rule that hypothesis out surfaced
this separate, concretely-confirmed defect: the neighbor count it computes is discarded after
being used only for its `.len()` in a log line.

## Minimum Reproducible Example

```rust
// examples/tiles_tools/game_of_life/tests/game_test.rs (this crate has a real lib.rs, so
// per the workspace rulebook's Test Placement rule, this belongs in tests/, not an inline
// #[cfg(test)] module).
let mut game = HexGameOfLife::new();
let center = HexCoord::< Axial, Pointy >::new( 0, 0 ); // alive in the seed pattern
game.step();
assert_eq!( game.generation(), 1 );
// pre-fix: game.is_cell_alive( center ) is still true (nothing ever changed) -- but
// is_cell_alive did not even exist pre-fix; see Fix Location.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd examples/tiles_tools/game_of_life && cargo nextest run -p game_of_life -E 'test(bug_reproducer_bug_486)'
```

## Root Cause

The function stopped immediately after populating `neighbors_count`, printing only its `.len()`.
Nothing translated the per-coordinate neighbor counts into survive/birth/death decisions, and
nothing persisted such a decision into the ECS world -- unlike `SquareGameOfLife::step` earlier
in this same file, which derives a `next_generation` map from its own neighbor count and calls a
`world_state_update` helper to apply it via `world.get_mut::<Cell>`/`world.spawn`.

## Why Not Caught

The pre-fix `test_hex_game_creation` only asserted `generation() == 0` immediately after
construction -- nothing called `step()` and then inspected any cell's alive state, so a `step()`
that silently discarded its own computation produced no test failure. The demo's console output
("Hex Generation N: M positions with neighbors") also looked like a plausible progress line on a
casual read, giving no visible signal that nothing was actually evolving.

## Fix Location

`examples/tiles_tools/game_of_life/src/lib.rs`, `HexGameOfLife::step` (~line 340): after
computing `neighbors_count` (unchanged), added a `next_generation` derivation applying the
already-documented rule (`(true, 2|3) | (false, 2) => true`, matching `SquareGameOfLife`'s same
match shape) via a new call to `is_cell_alive` per coordinate, then calls a new private
`world_state_update` (~line 405, mirroring `SquareGameOfLife`'s own private equivalent) which
kills/ages/revives existing cells via `world.get_mut::<Cell>` and spawns new ones via
`world.spawn`. `is_cell_alive` (~line 389) was made **`pub`** -- unlike `SquareGameOfLife`'s
private equivalent -- specifically so external regression tests in `tests/game_test.rs` can
observe simulation state directly instead of parsing console output.
`TriangularGameOfLife::step` was deliberately left unchanged (see Impact's note above).

## Prevention

New test `bug_reproducer_bug_486_hex_game_step_applies_rules` in `tests/game_test.rs`: seeds
from the crate's own built-in pattern (`[(0,0),(1,0),(0,1),(-1,1),(-1,0),(0,-1)]`, all alive),
hand-derives the expected outcome from the crate's own axial neighbor offsets
(`(1,0),(1,-1),(0,-1),(-1,0),(-1,1),(0,1)`) -- tallying all 36 neighbor contributions across the
6 seed cells -- and asserts all three branches actually fire after one `step()`: the center
`(0,0)` dies of overcrowding (5 neighbors), `(1,0)` survives (2 neighbors), and `(1,1)` is newly
born (2 neighbors). Exercises death, survival, and birth in one call, not a single pinned
snapshot.

## Pitfall

A neighbor-counting loop that never writes its result anywhere is easy to mistake for a working
simulation, especially when it still prints a plausible-looking progress line every call and its
own generation counter still increments on schedule. Always confirm a computed value is actually
persisted into the state other code queries (here, the ECS `Cell` component via `get_mut`), not
merely logged -- a `.len()` or count derived from a computation is not proof the computation's
*result* was applied anywhere.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `examples/tiles_tools/*`, while tracing `HexGameOfLife::step` to rule out a hypothesized hex/triangle coordinate-confusion defect (that hypothesis was not confirmed -- adjacency is correctly wired for all grid types). |
| 2026-08-20 | fixed | `step` now derives `next_generation` from its own already-documented rule and persists it via a new `world_state_update` helper (mirroring `SquareGameOfLife`'s); `is_cell_alive` made `pub` for test observability. `TriangularGameOfLife::step` deliberately left untouched -- no documented rule exists to implement it faithfully. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass (performed earlier this session): temporarily reverted `step` to its pre-fix (log-only) body, reran the scoped suite, and confirmed the new test failed (center/survivor/newborn cells all reported unchanged from the seed state); restored the fix and reconfirmed pass. Freshly reconfirmed this run: `cargo nextest run -p stealth_game -p tactical_rpg -p game_of_life` (scoped to all 3 touched crates, launched via `longrun`) -- 9/9 pass, including `bug_reproducer_bug_486_hex_game_step_applies_rules`, at 2026-08-20 11:26. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-486)`/`Root cause`/`Pitfall` 3-field comment applied immediately above `HexGameOfLife::step`. | — |
| D3 | Scope containment | — | 🟢 | `git status --porcelain` confirms only `examples/tiles_tools/game_of_life/src/lib.rs` and `tests/game_test.rs` changed in this crate; `TriangularGameOfLife`/`SquareGameOfLife` untouched (`test_square_game_creation`/`test_triangular_game_creation` both still pass unmodified). `module/helper/tiles_tools/` (out of scope for this task) untouched. `cargo clippy -p stealth_game -p tactical_rpg -p game_of_life --all-targets --all-features -- -D warnings` is currently blocked by an unrelated, in-progress concurrent edit to `module/helper/tiles_tools/src/debug.rs:1253` (a `clippy::format_push_string` violation in a JSON-escape helper a different live session is actively adding) -- confirmed via fresh `cargo clippy` output; not caused by, or fixable within, this bug's own scope. | Clippy re-run deferred to whoever completes the `tiles_tools` work in progress; not actionable from this crate's own source. |

**Reproduced:** YES -- adversarial revert of `step` to its pre-fix body (this session) caused the
new test to fail (no cell state changed from the seed pattern); restoring the fix passes.
Freshly reconfirmed via a full scoped `cargo nextest run` on 2026-08-20 at 11:26 (9/9 pass).

## Refs: src/

| File | Change |
|------|--------|
| `examples/tiles_tools/game_of_life/src/lib.rs` | `HexGameOfLife::step`: now derives `next_generation` from the already-documented rule and calls a new `world_state_update`. Added `pub fn is_cell_alive` and private `fn world_state_update`, both mirroring `SquareGameOfLife`'s existing equivalents. `TriangularGameOfLife::step` unchanged. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/tiles_tools/game_of_life/tests/game_test.rs` | Added `bug_reproducer_bug_486_hex_game_step_applies_rules`, hand-deriving expected death/survival/birth outcomes from the crate's own seed pattern and axial neighbor offsets, and asserting all three actually landed in the ECS world after one `step()`. |
