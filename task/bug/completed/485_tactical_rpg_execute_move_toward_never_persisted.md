# BUG-485: `tactical_rpg`'s `execute_move_toward` computes a move but never persists it to the `Position` component

- **Severity:** Medium (no crash, no data corruption -- but the demo's only movement action is
  completely non-functional every time it's invoked: units compute and print a destination but
  their `Position` component never changes, so battlefield rendering, targeting, and range
  checks all keep reading each unit's original spawn coordinate forever)
- **state:** Completed
- **Affects:** `examples/tiles_tools/tactical_rpg` (standalone demo binary, no downstream consumers)
- **Component:** `examples/tiles_tools/tactical_rpg/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-484 (`stealth_game`'s `guard_move_toward` /
  `player_movement_simulate`, identical unpersisted-move pattern) -- found together in the same
  sweep of `examples/tiles_tools/*`, no shared code between the two crates (independent fixes).

## Symptom

```rust
// pre-fix -- tactical_rpg/src/main.rs, execute_move_toward (git show HEAD)
fn execute_move_toward(&mut self, entity: hecs::Entity, target: HexCoord<Axial, Pointy>) {
  if let Ok(pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
    if let Ok(movable) = self.world.get::<Movable>(entity) {
      let path_result = astar( &pos.coord, &target, |&coord| Self::is_position_passable(coord), |_| 1 );
      if let Some((path, _cost)) = path_result {
        let path_len = u32::try_from(path.len()).unwrap_or(u32::MAX);
        let move_distance = movable.range.min(path_len - 1);
        if move_distance > 0 {
          let new_pos = path[move_distance as usize];

          // Update position (in real implementation would use proper ECS mutation)
          println!("🚶 Moving from ({}, {}) to ({}, {})",
                   pos.coord.q, pos.coord.r, new_pos.q, new_pos.r);
          // <- function ends here. `new_pos` is computed and printed, never written back.
        }
      }
    }
  }
}
```

The pre-fix code carried its own smoking-gun comment: `// Update position (in real
implementation would use proper ECS mutation)` -- documenting the exact gap without closing it.

## Impact

**Who is affected:** Anyone running the demo (`cargo run -p tactical_rpg`) -- `execute_move_toward`
is this game's only movement action; there is no other caller.

**What breaks:** Every unit's "move" turn action is a complete no-op on game state: the unit
computes a path, picks a destination up to `move_range` tiles away, and prints it -- but its
`Position` component never changes. `nearest_enemy_find`/`nearest_player_find` (used for AI
targeting) and any attack-range check all keep reading each unit's original spawn coordinate for
the entire battle, so units can never actually close distance to reach or flee an opponent via
this action.

**Consumer audit:** `execute_move_toward` is a private method with a single call site
(`grep -rn "execute_move_toward"` workspace-wide confirms zero external callers). `tactical_rpg`
is a standalone demo binary (no `lib.rs`), so nothing outside this crate can be affected.

**Magnitude:** 1 function, 1 missing persistence call.

**Entity Scope:** None -- a code-level defect confined to this demo binary.

## How Discovered

Assigned as part of a repo-wide bug/UX sweep of `examples/tiles_tools/*`, specifically checking
for a previously-flagged "unpersisted entity movement" pattern -- confirmed immediately by the
function's own "in real implementation would use proper ECS mutation" comment, then verified by
grepping the whole file for `get_mut::<Position` (zero matches pre-fix) while every *other*
mutated component in the same file (`Health`, `Experience`) is correctly persisted via
`world.get_mut::<T>()`.

## Minimum Reproducible Example

```rust
// examples/tiles_tools/tactical_rpg/src/main.rs, inline #[cfg(test)] mod tests (no lib.rs in
// this crate, so this is the only place a test can reach TacticalRPG's private methods -- see
// the workspace rulebook's Test Placement rule).
let mut game = TacticalRPG::new();
let entity = game.turn_queue[0]; // player_warrior, spawns at (-2, 1)
let before = game.world.get::<Position<HexCoord<Axial, Pointy>>>(entity).unwrap().coord;
game.execute_move_toward(entity, HexCoord::<Axial, Pointy>::new(2, -1));
let after = game.world.get::<Position<HexCoord<Axial, Pointy>>>(entity).unwrap().coord;
// pre-fix: after == before (never moved); post-fix: after != before
```

**Verify Command** (<=3 lines, standalone):
```bash
cd examples/tiles_tools/tactical_rpg && cargo nextest run -p tactical_rpg -E 'test(bug_reproducer_bug_485)'
```

## Root Cause

`pos`/`movable` were only ever borrowed read-only (`world.get::<_>`) to compute the destination;
no code path in this function called `world.get_mut::<Position<_>>` to write the computed
coordinate back into the ECS world, the single authoritative store every other query in this
file reads. The stale comment documented the intended mutation without ever implementing it.

## Why Not Caught

The demo prints a "🚶 Moving from X to Y" line unconditionally whenever a move is computed, so a
manual `cargo run` looks correct at a glance -- the printed coordinates are the *intended*
destination, not a readback of actual component state. Nothing in the codebase asserted on the
resulting `Position` component after a move, and `tactical_rpg`'s only pre-existing test
(`tests/readme_doc_test.rs`) checks the module doc comment's text, not simulation behavior.

## Fix Location

`examples/tiles_tools/tactical_rpg/src/main.rs`, `execute_move_toward` (~line 380): restructured
to extract `(current_pos, move_range)` into locals first (releasing the `Position`/`Movable`
borrows before the pathfind call, matching `stealth_game`'s sibling fix for BUG-484), replaced
the stale "in real implementation..." comment with the mandated fix-documentation format, and
added `self.world.get_mut::<Position<HexCoord<Axial, Pointy>>>(entity)?.set(new_pos)` right after
computing `new_pos`.

## Prevention

New test `bug_reproducer_bug_485_execute_move_toward_persists_position` in the crate's inline
`#[cfg(test)] mod tests`: grabs `turn_queue[0]` (the player warrior, spawned at `(-2, 1)`), moves
it toward `(2, -1)`, and asserts the `Position` component changed -- deterministic since this
board's `is_position_passable` always returns `true` (no obstacles configured), so the path and
resulting position are not contingent on map state. Asserts the general invariant the fix
restores, not a pinned coordinate.

## Pitfall

A comment describing intended future work ("in real implementation would use proper ECS
mutation") is not a substitute for the work -- it silently documents a known gap instead of
closing it, and reads as a deliberate, already-acceptable design choice on a casual pass rather
than as an open defect. Always grep a "TODO-shaped" comment like this for whether the described
mutation actually exists anywhere nearby before treating it as informational.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `examples/tiles_tools/*`, confirming a previously-flagged "unpersisted entity movement" pattern hypothesis; same sweep that found sibling BUG-484 in `stealth_game`. |
| 2026-08-20 | fixed | Added `get_mut::<Position<_>>(...).set(new_pos)` after restructuring the function to release its borrows first; replaced the stale comment with the mandated fix-documentation format. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass (performed earlier this session): temporarily reverted the `get_mut(...).set(...)` call site back to a no-op, reran the scoped suite, and confirmed the new test failed (`left == right`, `Position` unchanged) while the crate's pre-existing `readme_doc_test` was unaffected; restored the fix and reconfirmed pass. Freshly reconfirmed this run: `cargo nextest run -p stealth_game -p tactical_rpg -p game_of_life` (scoped to all 3 touched crates, launched via `longrun`) -- 9/9 pass, including `bug_reproducer_bug_485_execute_move_toward_persists_position`, at 2026-08-20 11:26. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-485)`/`Root cause`/`Pitfall` 3-field comment applied at `execute_move_toward`, replacing the stale "in real implementation..." comment entirely. | — |
| D3 | Scope containment | — | 🟢 | `git status --porcelain` confirms only `examples/tiles_tools/tactical_rpg/src/main.rs` changed in this crate; `tests/readme_doc_test.rs` untouched and still passes. `module/helper/tiles_tools/` (out of scope for this task) untouched. `cargo clippy -p stealth_game -p tactical_rpg -p game_of_life --all-targets --all-features -- -D warnings` is currently blocked by an unrelated, in-progress concurrent edit to `module/helper/tiles_tools/src/debug.rs:1253` (a `clippy::format_push_string` violation in a JSON-escape helper a different live session is actively adding) -- confirmed via fresh `cargo clippy` output; not caused by, or fixable within, this bug's own scope. | Clippy re-run deferred to whoever completes the `tiles_tools` work in progress; not actionable from this crate's own source. |

**Reproduced:** YES -- adversarial revert of the fix call site (this session) caused the new test
to fail with the expected "Position unchanged" assertion message; restoring the fix passes.
Freshly reconfirmed via a full scoped `cargo nextest run` on 2026-08-20 at 11:26 (9/9 pass).

## Refs: src/

| File | Change |
|------|--------|
| `examples/tiles_tools/tactical_rpg/src/main.rs` | `execute_move_toward`: restructured to release borrows before computing the move, replaced the stale comment with `Fix(BUG-485)`/`Root cause`/`Pitfall`, added `get_mut::<Position<_>>(entity).set(new_pos)`. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/tiles_tools/tactical_rpg/src/main.rs` (inline `#[cfg(test)] mod tests`, no `lib.rs` in this crate) | Added `bug_reproducer_bug_485_execute_move_toward_persists_position`, asserting the moved entity's `Position` component actually changes after `execute_move_toward` runs. |
