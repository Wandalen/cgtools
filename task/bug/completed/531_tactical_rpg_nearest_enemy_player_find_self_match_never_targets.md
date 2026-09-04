# BUG-531: `tactical_rpg`'s `nearest_enemy_find`/`nearest_player_find` always self-match and return `None`, so no unit ever attacks or moves toward a target for the entire simulation

- **Severity:** High (the demo's entire core mechanic -- turn-based combat -- is completely dead code; every simulated battle runs to completion with zero attacks and zero movement)
- **state:** Completed
- **Affects:** `tactical_rpg::TacticalRPG::nearest_enemy_find`, `tactical_rpg::TacticalRPG::nearest_player_find` (`src/main.rs`)
- **Component:** `examples/tiles_tools/tactical_rpg` (`src/main.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`player_turn_handle` and `ai_turn_handle` both call `nearest_enemy_find`/`nearest_player_find` to
locate a target, then only act (`attack_execute` or `execute_move_toward`) `if let Some(target) =
target`. Both finder functions always return `None`, on every call, for the entire simulation --
so neither branch's action body ever runs. A full `cargo run -p tactical_rpg` (10 simulated turns,
2 full rounds) prints "Player turn - planning actions..." / "AI turn - calculating optimal action..."
for every unit every turn, but never once prints "Targeting enemy at distance", "AI targeting player
at distance", "Moving from...to...", or "Attack!". Every unit's printed position is identical across
all 8 processed turns; the battle ends after 10 turns with "⏰ Battle continues..." having done
nothing at all.

## Impact

**Who is affected:** anyone running or reading `cargo run -p tactical_rpg` to see the
`tiles_tools` ECS framework's turn-based combat, pathfinding, and leveling systems in action --
the demo's headline features (AI decision-making, attack resolution, experience gain) never
execute even once, silently. No crash, no panic, exit code stays 0.

**What breaks:** `player_turn_handle`, `ai_turn_handle`, `attack_execute`, `execute_move_toward`,
and the entire `Experience`/leveling system are all unreachable in every normal run of this demo --
the only way any of that code has run recently is the pre-existing `bug_reproducer_bug_485` unit
test, which calls `execute_move_toward` directly, bypassing the broken target-finding that gates it
in the real game loop.

**Entity Scope:** `None` -- source-level example-logic defect, not entity directory instances.

## How Discovered

Dedicated sweep of `examples/tiles_tools/` example binaries. Read `nearest_enemy_find` and
`nearest_player_find` and noticed both call `self.world.nearest_entity_find(&our_pos)` -- the same
self-exclusion-free library primitive already found misused in `ecs_collision_demo` (BUG-515) --
with `our_pos` being the *querying entity's own position*. Since no two units in this demo spawn on
the same tile, `nearest_entity_find` is mathematically guaranteed to return the querying entity
itself (distance 0, the unique global minimum). Confirmed against the library's `Team::is_hostile_to`
(`module/helper/tiles_tools/src/ecs/components.rs`): `if self.id == other.id { false // Same team is
never hostile }` -- so the post-hoc team filter in both functions always rejects this guaranteed
self-match and returns `None`, never falling back to consider any other entity. Ran
`cargo run -p tactical_rpg` and confirmed the real output exactly matches this prediction: zero
"Targeting"/"Attack!"/"Moving" lines across the entire 10-turn run, and every unit's position frozen
at its spawn coordinate.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tactical_rpg bug_reproducer_bug_531_nearest_enemy_and_player_find_always_self_match_to_none -- --nocapture
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed against pristine source): panicked at the first assertion --
`player_warrior must find one of the 2 living hostile enemies on the board -- got None (self-match
filtered to nothing)`, exit 101.

Also directly observable without any test harness:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo run -p tactical_rpg 2>&1 | grep -c "Targeting enemy\|targeting player\|Attack!\|Moving from"
```
Pre-fix: `0` -- not one of these lines appears in the entire 10-turn run. Post-fix: `10` -- units
path toward each other, attack on contact, and take real damage every turn.

## Root Cause

`nearest_enemy_find` (pre-fix), abbreviated:
```rust
fn nearest_enemy_find(&self, entity: hecs::Entity) -> Option<(hecs::Entity, Position<...>)> {
  if let Ok(our_team) = self.world.get::<Team>(entity) {
    if let Ok(our_pos) = self.world.get::<Position<...>>(entity) {
      return self.world.nearest_entity_find(&our_pos)   // always finds `entity` itself: distance 0
        .and_then(|(nearest_entity, nearest_pos, _distance)| {
          if let Ok(their_team) = self.world.get::<Team>(nearest_entity) {
            if our_team.is_hostile_to(&their_team) {     // self vs self -> always false
              Some((nearest_entity, nearest_pos))
            } else { None }                              // -> always taken
          } else { None }
        });
    }
  }
  None
}
```
`nearest_player_find` has the identical structure/defect (its filter is `their_team.id ==
self.player_team.id`, which is likewise always false when called with an AI entity, since an AI
entity's own team is never the player team). The library's `nearest_entity_find` is a pure
"nearest entity to a coordinate" primitive by design (confirmed in BUG-515: no self-exclusion
parameter, correct and reusable as documented) -- but both call sites here query it with the
*querying entity's own position*, guaranteeing a self-match, and then rely on a *single*
post-hoc filter check that always rejects that one candidate instead of continuing on to the
next-nearest entity.

## Why Not Caught

The demo's only pre-existing test (`bug_reproducer_bug_485_execute_move_toward_persists_position`)
calls `execute_move_toward` directly with a hardcoded target coordinate, bypassing
`nearest_enemy_find`/`nearest_player_find` entirely -- so it never exercises the actual target-finding
path used by `player_turn_handle`/`ai_turn_handle` in the real game loop. `simulation_run` prints a
"planning actions..." / "calculating optimal action..." message on every turn regardless of whether
a target was found, so a manual run *looks* like it's doing something at a glance; only checking
whether any unit's position or HP ever actually changes across the run reveals every turn is a
no-op.

## Fix Applied (2026-08-21)

**`src/main.rs`:**
- `nearest_enemy_find` and `nearest_player_find` no longer call the library's `nearest_entity_find`
  primitive (single global nearest, self included, then post-hoc filter). Both now query
  `(hecs::Entity, &Position<...>, &Team)` directly, filter out the querying `entity` itself *and*
  apply the team condition (hostility / player-team membership) *before* computing distances, then
  pick the minimum-distance remaining candidate via `Position::distance_to` -- so a self-match can
  never be the sole candidate considered, and a genuinely eligible entity is found even when it
  isn't the single globally-nearest one.
- Both functions carry a `Fix(BUG-531)` 3-field comment; `nearest_player_find`'s comment
  cross-references `nearest_enemy_find`'s for the shared root cause (both functions had the
  identical defect).

**`src/main.rs`'s `#[cfg(test)] mod tests`** (bin-only crate; matches the existing
`bug_reproducer_bug_485` test's in-file placement, since there is no `src/lib.rs` for `tests/` to
link against):
`bug_reproducer_bug_531_nearest_enemy_and_player_find_always_self_match_to_none` constructs a fresh
`TacticalRPG` (2 player units, 2 hostile enemy units, no two units sharing a tile) and asserts that
both `nearest_enemy_find` (called from a player unit) and `nearest_player_find` (called from an AI
unit) return `Some`, not `None`.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tactical_rpg bug_reproducer_bug_531_nearest_enemy_and_player_find_always_self_match_to_none -- --nocapture`
  -- pre-fix (pristine source): 1 failed, exact assertion message confirming the self-match-to-`None`
  behavior on the very first check, exit 101. Post-fix: 1 passed, exit 0.
- Adversarial re-check (Tier 2 Dual-Role Self-Check): temporarily reverted `nearest_enemy_find`'s
  body to the exact pre-fix structure (single nearest-of-all via `self.world.nearest_entity_find`,
  then post-hoc hostility filter), re-ran the same scoped test -- failed again for the same reason,
  confirming the test genuinely detects the defect (a naive "just remove the self-exclusion
  condition" mutation was tried first and did *not* reproduce a failure, since the team filter alone
  already happens to exclude self in this codebase's actual call patterns -- the adversarial
  reproduction had to restore the original *single-candidate* structure, not merely tweak one
  clause, to genuinely reproduce the bug). Fix restored immediately after; grep confirmed no leftover
  marker text and `git diff --stat` showed only the intended fix lines.
- `cargo test -p tactical_rpg` (full scoped suite, all targets): 2/2 unit tests passed
  (`bug_reproducer_bug_531_...` and the pre-existing `bug_reproducer_bug_485_...`), 1/1 doc test
  (`readme_doc_test.rs`) passed, 0 failed.
- `cargo clippy -p tactical_rpg --all-targets --all-features -- -D warnings`: clean, exit 0.
- `cargo run -p tactical_rpg`: exit 0; real output now shows units pathing toward each other
  ("Targeting enemy at distance 4", "🚶 Moving from (-2, 1) to (1, 0)"), landing attacks
  ("💥 Attack! 8 damage dealt (120 -> 112 HP)"), and taking real damage over the full 10-turn run --
  a qualitative before/after contrast (0 action lines pre-fix vs. 10 post-fix across the same run).

## Generalized Version

**Broken assumption:** a general-purpose "nearest entity to a coordinate" primitive with no
self-exclusion option can be safely queried with an entity's *own* position and trusted to return a
*different* entity, because a downstream filter will "sort it out." When the primitive returns only
the single globally-nearest match (not a ranked list), and no two entities occupy the same tile, the
querying entity is mathematically guaranteed to be that unique nearest match -- so any single-shot
post-hoc filter that would legitimately reject a self-match (same team is never hostile; an AI unit
is never on the player team) rejects the *only* candidate it was ever given, degenerating to "always
`None`" rather than "falls through to the next-nearest candidate." Any call site building a filtered
"nearest other entity satisfying condition X" query on top of such a primitive must exclude the
querying entity *before* selecting the minimum, not filter a single already-selected candidate
after the fact -- this is the same class of defect as BUG-515 (`ecs_collision_demo`), against the
exact same underlying library primitive, but here the missing self-exclusion made the *entire*
demo's core mechanic silently inert rather than producing one misleading printed line.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed + fixed + verified | Found during a dedicated sweep of `examples/tiles_tools/`'s 12 example binaries: `tactical_rpg`'s own `cargo run` output showed zero attacks and zero movement across a full 10-turn, 2-round simulation. Root cause: `nearest_enemy_find`/`nearest_player_find` both queried the library's self-exclusion-free `nearest_entity_find` primitive with the querying entity's own position, guaranteeing a self-match that a subsequent one-shot team filter always rejected, returning `None` instead of falling through to a genuinely eligible candidate. Fixed by filtering candidates (self-exclusion plus team condition) before selecting the minimum-distance match, instead of selecting-then-filtering a single candidate. Verified via 1 new regression test (confirmed fail pre-fix with the predicted assertion message, pass post-fix), a Tier 2 adversarial re-introduction of the bug's exact original structure that the test caught a second time, the full scoped suite green (2 unit + 1 doc test), clean clippy, and the real binary's own output now showing genuine targeting, movement, and combat across the entire run. |
