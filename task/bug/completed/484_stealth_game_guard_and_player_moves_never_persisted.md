# BUG-484: `stealth_game`'s `guard_move_toward` and `player_movement_simulate` compute moves but never persist them to the `Position` component

- **Severity:** Medium (no crash, no data corruption -- but the demo's two core movement
  systems, guard patrol and player advance-to-objective, silently do nothing every single
  turn: console output claims a move happened while the entity's actual `Position` component,
  the only state rendering/detection/the victory check ever read, never changes)
- **state:** Completed
- **Affects:** `examples/tiles_tools/stealth_game` (standalone demo binary, no downstream consumers)
- **Component:** `examples/tiles_tools/stealth_game/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-485 (`tactical_rpg`'s `execute_move_toward`,
  identical unpersisted-move pattern -- even shares the "in real implementation..." style
  smoking-gun absence of a persistence call) -- found together in the same sweep of
  `examples/tiles_tools/*`, no shared code between the two crates (independent fixes).

## Symptom

```rust
// pre-fix -- stealth_game/src/main.rs, guard_move_toward (git show HEAD)
fn guard_move_toward(&mut self, guard: hecs::Entity, target: SquareCoord<EightConnected>) {
  if let Ok(pos) = self.world.get::<Position<SquareCoord<EightConnected>>>(guard) {
    if let Ok(movable) = self.world.get::<Movable>(guard) {
      let path_result = astar( &pos.coord, &target, |coord| self.level_map.is_passable(*coord), |_| 1 );
      if let Some((path, _cost)) = path_result {
        let move_distance = movable.range.min(u32::try_from(path.len() - 1).unwrap_or(u32::MAX));
        if move_distance > 0 {
          let new_pos = path[move_distance as usize];
          println!("🚶 Guard moving from ({}, {}) to ({}, {})",
                   pos.coord.x, pos.coord.y, new_pos.x, new_pos.y);
          // <- function ends here. `new_pos` is computed and printed, never written back.
        }
      }
    }
  }
}
```

`player_movement_simulate`'s "safe to move" branch had the identical shape: it computed
`next_pos`, printed a "Player moving from X to Y" message, then went straight to updating the
`Stealth` component -- `next_pos` itself was never written into `Position`.

## Impact

**Who is affected:** Anyone running the demo (`cargo run -p stealth_game`) -- this is the
entire content of the simulation loop; there is no other caller.

**What breaks:** Both of the game's two movement systems are completely non-functional every
turn, for the demo's full 30-turn run:
- `guard_patrol_update` → `guard_move_toward`: guards compute a path toward their next patrol
  waypoint and print a "moving" line, but their `Position` component never changes, so they
  stay frozen at their spawn coordinate forever. `PatrolRoute::waypoint_advance` still runs on
  its own timer, so the *target* waypoint cycles even though the guard never physically
  approaches it.
- `player_movement_simulate`: same pattern for the player advancing toward the objective --
  `is_position_safe_for_player`/detection/`level_map_print` all keep reading the player's
  original spawn coordinate `(2, 2)`, so the player can never actually reach the objective and
  the victory condition (`turn_process`'s `distance(&objective) <= 1` check) can never fire.

**Consumer audit:** `guard_move_toward`/`player_movement_simulate` are both private methods
with a single call site each (`guard_patrol_update`, `stealth_turn_process`), both within this
same file -- `grep -rn "guard_move_toward\|player_movement_simulate"` workspace-wide confirms
zero external callers. `stealth_game` is a standalone demo binary (no `lib.rs`), so nothing
outside this crate can be affected.

**Magnitude:** 2 functions, 1 missing persistence call each.

**Entity Scope:** None -- a code-level defect confined to this demo binary.

## How Discovered

Assigned as part of a repo-wide bug/UX sweep of `examples/tiles_tools/*`, specifically checking
for a previously-flagged "unpersisted entity movement" pattern -- confirmed by reading both
movement functions end-to-end and grepping the whole file for `get_mut::<Position` (zero
matches pre-fix), while every *other* mutated component in the same file (`Vision`,
`PatrolRoute`, `Stealth`, `Health`) is correctly persisted via `world.get_mut::<T>()`.

## Minimum Reproducible Example

```rust
// examples/tiles_tools/stealth_game/src/main.rs, inline #[cfg(test)] mod tests (no lib.rs in
// this crate, so this is the only place a test can reach StealthGame's private methods -- see
// the workspace rulebook's Test Placement rule).
let mut game = StealthGame::new();
let guard = game.guard_entities[0]; // spawns at (8, 3)
let before = game.world.get::<Position<SquareCoord<EightConnected>>>(guard).unwrap().coord;
game.guard_move_toward(guard, SquareCoord::<EightConnected>::new(12, 3));
let after = game.world.get::<Position<SquareCoord<EightConnected>>>(guard).unwrap().coord;
// pre-fix: after == before (never moved); post-fix: after != before
```

**Verify Command** (<=3 lines, standalone):
```bash
cd examples/tiles_tools/stealth_game && cargo nextest run -p stealth_game -E 'test(bug_reproducer_bug_484)'
```

## Root Cause

Both functions borrowed `Position` read-only (`world.get::<Position<_>>`) to read the current
coordinate, computed a new coordinate via `astar`, and printed a human-readable "moving"
message -- but neither ever called `world.get_mut::<Position<_>>` to write the computed
coordinate back into the ECS world, the single authoritative store every other system in this
file queries. The `println!` gave the false impression that the move had taken effect.

## Why Not Caught

The demo prints a "🚶 ... moving from X to Y" line unconditionally whenever a move is computed,
so a manual `cargo run` looks correct at a glance -- the printed coordinates are the *intended*
destination, not a readback of the actual component state. Nothing in the codebase asserted on
the resulting `Position` component after a move, and `stealth_game` had no `tests/` directory
or any test coverage at all before this fix.

## Fix Location

`examples/tiles_tools/stealth_game/src/main.rs`:
- `guard_move_toward` (~line 494): restructured to extract `(current_pos, move_range)` into
  locals first (releasing the `Position`/`Movable` borrows before the pathfind call, matching
  the pattern already used elsewhere in this file), then added
  `self.world.get_mut::<Position<SquareCoord<EightConnected>>>(guard)?.set(new_pos)` right
  after computing `new_pos`.
- `player_movement_simulate` (~line 610): added the equivalent `get_mut(...).set(next_pos)`
  call in the "safe to move" branch, immediately before the existing `Stealth` component update.

## Prevention

Two new tests in the crate's inline `#[cfg(test)] mod tests` --
`bug_reproducer_bug_484_guard_move_toward_persists_position` (drives guard 1 from its spawn
`(8, 3)` toward its own second patrol waypoint `(12, 3)`, an unobstructed straight line, and
asserts the `Position` component changed) and
`bug_reproducer_bug_484_player_movement_simulate_persists_position` (relocates the player to
`(2, 12)` -- hand-verified via `square.rs`'s Chebyshev `EightConnected::distance` to be out of
both guards' vision range regardless of line-of-sight: 9 tiles from guard 1's range-6, 14 tiles
from guard 2's range-8 -- guaranteeing the very next call takes the "safe to move" branch
instead of "wait and hide", isolating the persistence bug from FOV geometry) -- both assert the
general invariant the fix restores (`Position` actually changes), not a pinned coordinate.

## Pitfall

A `println!` describing a computed move is not evidence the move was applied to the state other
systems query -- always trace whether a value that's about to be logged also gets written back
into the authoritative store (here, the ECS `Position` component) via a `get_mut` call, not just
read via `get` and discarded after the log line.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `examples/tiles_tools/*`, confirming a previously-flagged "unpersisted entity movement" pattern hypothesis. |
| 2026-08-20 | fixed | Added `get_mut::<Position<_>>(...).set(...)` calls to both `guard_move_toward` and `player_movement_simulate`, after restructuring the former to release its borrows first. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass (performed earlier this session): temporarily reverted both `get_mut(...).set(...)` call sites back to a no-op, reran the scoped suite, and confirmed exactly the 2 new tests for this bug failed (`left == right`, `Position` unchanged) while the crate's other tests were unaffected; restored the fix and reconfirmed pass. Freshly reconfirmed this run: `cargo nextest run -p stealth_game -p tactical_rpg -p game_of_life` (scoped to all 3 touched crates, launched via `longrun`) -- 9/9 pass, including both `bug_reproducer_bug_484_*` tests, at 2026-08-20 11:26. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-484)`/`Root cause`/`Pitfall` 3-field comment applied at `guard_move_toward`; `player_movement_simulate`'s call site cross-references it (`// Fix(BUG-484): see guard_move_toward above -- same root cause, same fix: ...`) rather than duplicating the full block, since both are the same defect in the same file. | — |
| D3 | Scope containment | — | 🟢 | `git status --porcelain` confirms only `examples/tiles_tools/stealth_game/src/main.rs` changed in this crate; `module/helper/tiles_tools/` (out of scope for this task) untouched. `cargo clippy -p stealth_game -p tactical_rpg -p game_of_life --all-targets --all-features -- -D warnings` is currently blocked by an unrelated, in-progress concurrent edit to `module/helper/tiles_tools/src/debug.rs:1253` (a `clippy::format_push_string` violation in a JSON-escape helper a different live session is actively adding) -- confirmed via fresh `cargo clippy` output and cross-checked against that concurrent session's own reported in-flight state; not caused by, or fixable within, this bug's own scope. | Clippy re-run deferred to whoever completes the `tiles_tools` work in progress; not actionable from this crate's own source. |

**Reproduced:** YES -- adversarial revert of both fix call sites (this session) caused both new
tests to fail with the expected "Position unchanged" assertion message; restoring the fix passes.
Freshly reconfirmed via a full scoped `cargo nextest run` on 2026-08-20 at 11:26 (9/9 pass).

## Refs: src/

| File | Change |
|------|--------|
| `examples/tiles_tools/stealth_game/src/main.rs` | `guard_move_toward`: restructured to release borrows before computing the move, added `get_mut::<Position<_>>(guard).set(new_pos)`. `player_movement_simulate`: added the equivalent `get_mut::<Position<_>>(player_entity).set(next_pos)` in the "safe to move" branch. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/tiles_tools/stealth_game/src/main.rs` (inline `#[cfg(test)] mod tests`, no `lib.rs` in this crate) | Added `bug_reproducer_bug_484_guard_move_toward_persists_position` and `bug_reproducer_bug_484_player_movement_simulate_persists_position`, both asserting the entity's `Position` component actually changes after the respective move function runs. |
