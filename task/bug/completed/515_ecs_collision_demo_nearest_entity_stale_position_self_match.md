# BUG-515: `ecs_collision_demo`'s "Nearest Entity Search" section queries from a stale pre-resolution player position and never excludes the player's own entity, so it reports the player as "nearest entity to the player"

- **Severity:** Medium (silent DX/demo-correctness defect -- no crash, no panic, but the demo's own "Nearest Entity Search" section produces a vacuous, self-referential, misleading result, and every other "of player" spatial query in the same function is built on the same stale-position pattern)
- **state:** Completed
- **Affects:** `ecs_collision_demo::entities_spawn`, `ecs_collision_demo::spatial_queries_run` (`src/main.rs`)
- **Component:** `examples/tiles_tools/ecs_collision_demo` (`src/main.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`main()` captures the player's `Position` exactly once, immediately after `entities_spawn`, and
passes that single snapshot into `spatial_queries_run` -- but `collisions_detect_and_resolve` runs
in between and moves entities (including the player itself) via
`CollisionSystem::collisions_resolve`. Every "of player" / "to player" query in
`spatial_queries_run` (the radius-3 circle query, both team-filtered queries, and the
nearest-entity search) is therefore centered on a location the player no longer occupies.
Independently, the library's `nearest_entity_find` is a pure "nearest entity to a coordinate"
primitive with no self-exclusion parameter -- so even a *correct*, live player position would still
trivially rediscover the player's own entity at distance 0, since the player is itself an entity
sitting exactly on that coordinate. The pre-fix real output combined both defects into one
observably nonsensical line: `"Nearest entity to player: 0v1 / Position: (5, 3) / Distance: 2"`,
where entity `0v1` **is** the player -- the demo reported the player as the entity nearest to
itself, at a distance that isn't even 0.

## Impact

**Who is affected:** anyone running or reading `cargo run -p ecs_collision_demo` to learn how the
`tiles_tools` spatial-query primitives compose -- the "Nearest Entity Search" section is supposed
to demonstrate finding *another* entity relative to the player, but instead demonstrates a
degenerate self-match, teaching nothing about the actual capability. The circle/team queries in the
same function are built on the identical stale-position pattern and are one collision-layout change
away from also returning visibly wrong entity sets (in this exact scenario the small 2-tile
resolution shift happened not to change which entities fall in/out of the fixed query radii, but
the underlying pattern -- querying "around the player" from a position the player has since moved
away from -- is wrong regardless of whether this particular seed happens to mask it).

**What breaks:** the "Nearest Entity Search" section's entire demonstrated value; no crash, no
panic, exit code stays 0, so nothing in normal CI/usage flags this except reading the output
critically.

**Entity Scope:** `None` -- source-level example-logic defect, not entity directory instances.

## How Discovered

Dedicated sweep of `examples/tiles_tools/` example binaries. Ran `cargo run -p ecs_collision_demo`
and inspected the full console output for internal consistency. The "=== Collision Resolution ==="
block prints each entity's new position after resolution (`Entity 0v1 now at (5, 3)`), and the later
"=== Nearest Entity Search ===" block reports `Nearest entity to player: 0v1` -- the same entity ID
-- with a nonzero `Distance: 2`. Since an entity's distance to itself is always 0, a nonzero
self-distance is only possible if the query origin doesn't actually match that entity's real
position, which traced directly to `spatial_queries_run` receiving a `Position` captured before
`collisions_detect_and_resolve` ran. Hand-deriving what the *correct* live-position query should
report (Manhattan distance from the player's real post-resolution position `(5, 3)` to every other
entity) additionally showed that fixing only the staleness would still self-match at distance 0
(the player's live position coincides exactly with the player's own entity), confirming a second,
independent defect: no self-exclusion in the nearest-entity call site.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p ecs_collision_demo --test collision_demo_test bug_reproducer_bug_515_nearest_entity_search_uses_live_position_and_excludes_self -- --nocapture
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed against pristine source): panicked at the self-match assertion --
`nearest entity to the player must not be the player's own entity -- self-match is a vacuous,
zero-information result: Nearest entity to player: 0v1`, exit 101.

Also directly observable without any test harness:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo run -p ecs_collision_demo 2>&1 | grep -A2 "Nearest entity"
```
Pre-fix: `Nearest entity to player: 0v1` / `Position: (5, 3)` / `Distance: 2` -- entity `0v1` is the
player itself (compare against the `Entity 0v1 now at (5, 3)` line printed a few lines earlier in
the same run). Post-fix: `Nearest entity to player: 1v1` / `Position: (6, 3)` / `Distance: 1` -- a
genuinely different entity, the correct unique nearest neighbor.

## Root Cause

`main()` (pre-fix), abbreviated:
```rust
let player_pos = entities_spawn(&mut world);      // Position captured at spawn time: (5, 5)
collisions_detect_and_resolve(&mut world);          // moves the player entity to (5, 3)
spatial_queries_run(&world, player_pos);            // still passed the stale (5, 5) snapshot
```
`spatial_queries_run` (pre-fix), abbreviated:
```rust
fn spatial_queries_run(world: &World, player_pos: Position<SquareCoord<FourConnected>>)
{
  // every query below uses the single stale `player_pos` parameter
  let nearby_entities = SpatialQuerySystem::circle_query(&world.hecs_world, &player_pos, 3);
  ...
  if let Some((nearest_entity, nearest_pos, distance)) = nearest_entity_find(&world.hecs_world, &player_pos) {
    // nearest_entity_find has no self-exclusion param -- with a *correct* live
    // player_pos this always finds the player itself, at distance 0
    println!("Nearest entity to player: {nearest_entity:?}");
    ...
  }
}
```
Two compounding defects: (1) `player_pos` is a one-time snapshot taken before a mutating system
call, never refreshed, so every query in `spatial_queries_run` risks operating on a stale origin;
(2) `nearest_entity_find` is a general "nearest entity to a coordinate" primitive (by design, no
entity-to-exclude parameter -- correct and reusable as documented), and the call site never filters
out the player's own entity, so the "Nearest Entity Search" section can never report anything but
the player itself once the position is accurate.

## Why Not Caught

The demo had zero test coverage prior to this bug (bin-only crate; nothing exercised `main()`'s own
composition logic). Every individual spatial-query primitive (`circle_query`, `by_team_query`,
`nearest_entity_find`) is correct in isolation against whatever position it's handed -- the defect
is purely in what `main.rs` passes those primitives, invisible to any library-level test. The
specific collision layout in this demo also happens not to change the circle/team query result
*sets* between the stale and live player positions (the 2-tile resolution shift is too small
relative to the fixed radii 3/5/8 to flip any entity in or out), so only the nearest-entity section
-- which is sensitive to exact distance, not just in/out-of-range -- exposed the defect visibly.

## Fix Applied (2026-08-21)

**`src/main.rs`:**
- `entities_spawn` now returns the player's `hecs::Entity` handle instead of a one-time `Position`
  snapshot (the position can always be re-derived live from a stable entity handle; the handle
  itself never goes stale).
- `spatial_queries_run` now takes `player_entity: hecs::Entity` and re-fetches the player's live
  `Position` from the world at the top of the function, immediately before any query uses it.
- The nearest-entity search no longer calls `nearest_entity_find` directly; it queries
  `(hecs::Entity, &Position<...>)`, filters out `player_entity`, and picks the minimum-distance
  remaining candidate via `Position::distance_to` (the same Manhattan-distance method the library
  primitive itself uses).
- `main()`'s two call sites updated: `player_pos` renamed to `player_entity` throughout.
- Removed the now-unused `nearest_entity_find` import.

**`tests/collision_demo_test.rs`** (new file -- crate is bin-only, no lib target):
`bug_reproducer_bug_515_nearest_entity_search_uses_live_position_and_excludes_self` runs the
compiled binary as a subprocess (`Command::new(env!("CARGO_BIN_EXE_ecs_collision_demo"))`) and
asserts on its stdout: the player's real post-resolution position, that the reported nearest entity
is not the player's own entity, and the exact hand-derived unique correct answer (entity `1v1`,
position `(6, 3)`, distance `1`) -- the exact-answer assertion additionally catches a fix that
excludes self but forgets to also fix the staleness (which would instead produce an ambiguous tie
between two different entities at distance 3, never landing on this unique distance-1 result).

## Verification

`longrun`-detached, from repo root:
- `cargo test -p ecs_collision_demo --test collision_demo_test bug_reproducer_bug_515_nearest_entity_search_uses_live_position_and_excludes_self -- --nocapture`
  -- pre-fix (pristine source): 1 failed, exact assertion message confirming the self-match
  (`Nearest entity to player: 0v1`), exit 101. Post-fix: 1 passed, exit 0.
- Adversarial re-check (Tier 2 Dual-Role Self-Check): temporarily neutered the self-exclusion filter
  (`.filter(|(entity, _)| *entity != player_entity)` replaced with an always-true no-op filter),
  re-ran the same scoped test -- failed again for the same reason, confirming the test genuinely
  detects the defect. Fix restored immediately after; grep confirmed no leftover marker text and
  `git diff --stat` showed only the intended fix lines.
- `cargo test -p ecs_collision_demo` (full scoped suite, post-rename): 1/1 passed, plus 0 unit /
  0 doc tests (none defined), all green.
- `cargo clippy -p ecs_collision_demo --all-targets --all-features -- -D warnings`: clean, exit 0.
- `cargo run -p ecs_collision_demo`: exit 0; `Nearest entity to player: 1v1 / Position: (6, 3) /
  Distance: 1` -- a genuinely distinct entity with a unique, correct Manhattan-distance answer.

## Generalized Version

**Broken assumption:** a value captured once from a mutable ECS world can be treated as "current"
for the rest of a function's lifetime, even after another system call that's known to mutate that
same world runs in between. Any position (or other mutable-component snapshot) taken before a
system call that can move/modify entities must be re-derived from a stable identifier (an `Entity`
handle) immediately before each subsequent use, not threaded through as a stale value parameter.
Separately: a general-purpose "nearest entity to a coordinate" primitive with no self-exclusion
option is a legitimate, correctly-scoped API -- but any call site that supplies the coordinate of an
entity *that is itself a candidate* must explicitly filter that entity out; the primitive cannot and
should not guess the caller's intent to exclude "self."

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed + fixed + verified | Found during a dedicated sweep of `examples/tiles_tools/`'s 12 example binaries: `ecs_collision_demo`'s own `cargo run` output showed the player entity (`0v1`) reported as the "nearest entity" to itself, at a nonzero distance. Root cause: `player_pos` was captured once before collision resolution moved entities and never refreshed, compounded by `nearest_entity_find` having no self-exclusion parameter. Fixed by tracking the player's `hecs::Entity` handle instead of a `Position` snapshot, re-deriving its live position at the top of `spatial_queries_run`, and explicitly filtering the player's own entity out of the nearest-entity search. Verified via 1 new regression test (bin-only crate, so implemented as a subprocess-output integration test -- confirmed fail pre-fix with the exact predicted self-match message, pass post-fix), a Tier 2 adversarial re-introduction of the bug that the test caught a second time, the full scoped suite green, clean clippy, and the real binary's own output now reporting a genuinely distinct nearest entity. Filed as BUG-515 after a fresh on-disk scan immediately before filing found 512, 513, and 514 already claimed by concurrent session actors (including a same-number 512 collision between two other concurrent actors) for what had been provisionally tracked as 512 mid-session.
