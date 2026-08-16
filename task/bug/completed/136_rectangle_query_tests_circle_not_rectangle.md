# BUG-136: `rectangle_query` filters by a circular full-diagonal radius, not a rectangle

- **Severity:** Medium (silently over-includes entities well outside the documented rectangular
  area — no panic, no compile error, just a wrong result set)
- **state:** Completed
- **Affects:** Any caller of `SpatialQuerySystem::rectangle_query` (zero in-crate callers; one
  downstream example, `examples/tiles_tools/ecs_collision_demo`, already uses square coordinates
  and is unaffected by the narrowed signature)
- **Component:** `module/helper/tiles_tools` (`src/ecs/systems.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — sixth bug filed for this crate this session; independent of
  BUG-131/132/133/134/135 (different module, different mechanism)

## Symptom

```rust
let center = Position::new( SquareCoord::<FourConnected>::new( 0, 0 ) );
let outside = Position::new( SquareCoord::<FourConnected>::new( 5, 0 ) ); // dx=5, well outside a 4x4 rect

// Wrong (pre-fix):
SpatialQuerySystem::rectangle_query( &world, &center, 4, 4 )
  // includes the entity at (5, 0) -- Manhattan distance 5 <= floor(sqrt(4^2+4^2))=5

// Correct (post-fix):
SpatialQuerySystem::rectangle_query( &world, &center, 4, 4 )
  // excludes it -- dx=5 > half_width=2
```

## Impact

**Who is affected:** Any caller of `rectangle_query` expecting a genuine axis-aligned rectangular
selection — the function's own doc comment promises exactly that ("Finds all entities within a
rectangular area").

**What breaks:** `rectangle_query` computed `max_distance` as the rectangle's *full diagonal*
(`sqrt(width² + height²)`) and filtered by a single scalar `distance_to(pos) <= max_distance` —
a circular region, not a rectangle. Because the full diagonal is always at least double the
rectangle's own half-diagonal (the farthest any point actually inside the rectangle can be from
its center), the circular region is a strict superset of the true rectangle for every input: the
function can never produce a false negative (never wrongly excludes a point actually inside the
rectangle) but routinely produces false positives (wrongly includes points far outside it along
one axis, e.g. a very wide, short rectangle would also match entities far above/below it as long
as they're within the diagonal-radius circle).

**Magnitude:** Not a crash — a silently over-inclusive `Vec` of entities, e.g. a "select units in
this 4x4 area" gameplay query would also select units up to ~5 tiles away in a straight line.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #66, a targeted code review of `tiles_tools` under the standing bug-hunt mandate. The
reviewing agent flagged that `rectangle_query` (`src/ecs/systems.rs`, pre-fix) computed
`max_distance` from the diagonal formula and did a single-radius distance check, with its own
inline comment admitting the gap: `// Additional filtering could be added here for precise
rectangular bounds`. Confirmed by direct read of the full function body and by hand-computing the
divergence between the circular threshold and a true per-axis bounds check for a concrete input.

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test integration_tests --features enabled,integration test_rectangle_query_excludes_entity_outside_axis_aligned_bounds 2>&1 | tail -10
```

**Expected** (post-fix):
```
test integration::ecs_tests::test_rectangle_query_excludes_entity_outside_axis_aligned_bounds ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `rectangle_query`'s filter logic to its
exact pre-fix circular-threshold behavior, preserving the new signature so the same test compiles
against both versions, then restoring the fix immediately after capturing the failure):
```
assertion `left == right` failed: rectangle_query included an entity outside its axis-aligned bounds
  left: 3
 right: 2
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test integration_tests --features enabled,integration test_rectangle_query_excludes_entity_outside_axis_aligned_bounds
# 1 passed = fixed; 1 failed (left: 3, right: 2) = bug present
```

**Known MRE limitation (check 205):** none — `SpatialQuerySystem`/`hecs::World` are pure,
synchronous, dependency-free state; runs as an ordinary native `cargo test` against the real
crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `rectangle_query` filters by a circular (distance-threshold) region derived from the rectangle's diagonal, not a genuine per-axis rectangular bounds check. | ✅ Root Cause | Direct read of `src/ecs/systems.rs` pre-fix: `max_distance = sqrt(width²+height²)` then `distance_to(pos) <= max_distance` — no per-axis comparison anywhere, plus the function's own comment admitting "precise rectangular bounds" were never added. | E1 |
| H2 | The bug requires an asymmetric (non-square) `width`/`height` to be observable. | ❌ Falsified | The MRE uses a *square* 4x4 rectangle — the full-diagonal-vs-half-diagonal gap alone (a factor of ~2, independent of aspect ratio) is enough to expose the over-inclusion for a point positioned along one axis. | E2 |
| H3 | The function was written this way deliberately, as a cheap circular approximation, and the doc comment is simply outdated. | ❌ Falsified | The function's own inline comment (`// Additional filtering could be added here for precise rectangular bounds`) is a direct in-code admission that rectangular filtering was intended but not implemented — not a stale doc, an acknowledged gap. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/ecs/systems.rs`, pre-fix `rectangle_query` | `let max_distance = ((width * width + height * height) as f32).sqrt() as u32;` followed by a single `distance <= max_distance` check and the comment `// Additional filtering could be added here for precise rectangular bounds`. | H1 ✅, H3 ❌ |
| E2 | MRE run, reverted logic | `left: 3, right: 2` for a *square* 4x4 rectangle with an entity at `(5, 0)` (Manhattan distance 5, `max_distance` = `floor(sqrt(32))` = 5) — confirms no aspect-ratio asymmetry is needed to expose the defect. | H1 ✅, H2 ❌ |
| E3 | `grep -rn "rectangle_query"` across the full workspace | Zero callers within `tiles_tools` itself; exactly one downstream caller (`examples/tiles_tools/ecs_collision_demo/src/main.rs:106`), which already exclusively uses `SquareCoord::<FourConnected>` throughout — confirmed unaffected by narrowing the signature (`cargo check` on that example, clean). | Blast-radius check |

## Root Cause

```
rectangle_query(): // copy-pasted circle_query's distance-threshold shape
  max_distance = sqrt(width^2 + height^2)     // the rectangle's FULL diagonal
  for each entity:
    if distance_to(entity) <= max_distance:    // a circular test, not a rectangle
      include entity

// Missing: any per-axis (|dx| <= width/2, |dy| <= height/2) comparison at all.
```

A rectangle is a per-axis bounds check; no single scalar "distance" can express it. The function
was structured identically to the adjacent `circle_query` (single scalar threshold against
`Distance::distance`), which is correct for a genuine circle but was never adapted into a
per-axis test when repurposed for a rectangle — the sole change was reinterpreting the radius as
"the diagonal," which produces a circle roughly twice as generous as even the rectangle's own
corner-to-center distance, not a fix.

## Why Not Caught

`SpatialQuerySystem` (`circle_query`, `line_query`, `rectangle_query`, `by_team_query`) had zero
existing tests for any of its methods — this API surface was entirely untested prior to this fix.

## Fix Location

`module/helper/tiles_tools/src/ecs/systems.rs`, `SpatialQuerySystem::rectangle_query`:

```rust
// before
pub fn rectangle_query<C>(
  world: &hecs::World,
  center: &Position<C>,
  width: u32,
  height: u32,
) -> Vec<(hecs::Entity, Position<C>)>
where
  C: Distance + Clone + Send + Sync + 'static,
{
  let max_distance = ((width * width + height * height) as f32).sqrt() as u32;
  for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<C>)>() {
    if center.distance_to(pos) <= max_distance {
      entities.push((entity, pos.clone()));
    }
  }
  ...
}

// after
pub fn rectangle_query<Connectivity>(
  world: &hecs::World,
  center: &Position<SquareCoordinate<Connectivity>>,
  width: u32,
  height: u32,
) -> Vec<(hecs::Entity, Position<SquareCoordinate<Connectivity>>)>
where
  Connectivity: Clone + Send + Sync + 'static,
{
  let half_width = (width / 2) as i32;
  let half_height = (height / 2) as i32;
  for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<SquareCoordinate<Connectivity>>)>() {
    let dx = (pos.coord.x - center.coord.x).abs();
    let dy = (pos.coord.y - center.coord.y).abs();
    if dx <= half_width && dy <= half_height {
      entities.push((entity, pos.clone()));
    }
  }
  ...
}
```

The generic bound narrowed from `C: Distance` (any coordinate system) to the concrete
`square::Coordinate<Connectivity>` type, because a genuine rectangle test needs real x/y field
access — no scalar distance metric can express "within `width`/2 along x AND within `height`/2
along y" — and "rectangle" is only an unambiguous Cartesian concept for square-grid coordinates
in this crate (hexagonal/triangular grids have no natural orthogonal width/height). This is an
intentionally scoped API-breaking change: zero in-crate callers existed, and the one downstream
caller already used square coordinates exclusively (confirmed via workspace-wide grep and a clean
`cargo check` on that example).

## Prevention

Added `test_rectangle_query_excludes_entity_outside_axis_aligned_bounds` to
`tests/integration/ecs_tests.rs`, covering an entity positioned outside the rectangle along one
axis but inside the old circular full-diagonal radius.

**Pitfall:** invisible for entities near the rectangle's own corners (both the buggy circular test
and the correct per-axis test agree there) — only entities well outside the rectangle along one
axis, yet still within the full-diagonal circular radius, expose the defect.

## Generalized Version

**Broken assumption:** "a shape query can always be expressed as `distance <= threshold` against
some scalar metric." False for any shape that isn't itself a metric ball (a circle under the
chosen distance function) — an axis-aligned rectangle is the union of two independent per-axis
bounds, not a single-radius test under any scalar distance, Euclidean or otherwise.

**Confirmed general rule:** when adapting one geometric query (a distance-threshold circle) into
a differently-shaped query (an axis-aligned rectangle) by copying its structure and merely
reinterpreting the threshold value, verify the new threshold's *shape* matches the target
geometry, not just that some plausible scalar can be computed from the new parameters — a
diagonal-derived radius is a real number that *looks* like a reasonable adaptation but describes
a fundamentally different region.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #66's targeted code review of `tiles_tools`; confirmed by direct read of `rectangle_query`'s diagonal-radius computation and its own inline comment admitting the gap. |
| 2026-08-16 | fixed | `rectangle_query` narrowed to `Position<square::Coordinate<Connectivity>>` and rewritten to a genuine `\|dx\| <= width/2 && \|dy\| <= height/2` per-axis bounds check; workspace-wide grep confirmed zero in-crate callers and one already-compatible downstream example. |
| 2026-08-16 | verified | Added `test_rectangle_query_excludes_entity_outside_axis_aligned_bounds`; confirmed it fails against the reverted pre-fix logic with the exact predicted wrong value (`left: 3, right: 2`) and passes against the fix; full crate suite (234 tests incl. doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean; downstream example `cargo check` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `rectangle_query` (confirmed the narrowed `Position<SquareCoordinate<Connectivity>>` signature and genuine per-axis `dx <= half_width && dy <= half_height` check, replacing the old circular-diagonal threshold; 7-line `Fix(BUG-136)`/`Root cause`/`Pitfall` comment intact) and `test_rectangle_query_excludes_entity_outside_axis_aligned_bounds` (non-tautological: asserts exactly 2 results — center + inside — and explicitly asserts the `(5,0)` entity, inside the old circular radius but outside the true rectangle, is excluded). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-computed the expected over-inclusion; adversarial pass required actually observing the FAIL against the reverted pre-fix logic, not trusting the hand-computation — closed via signature-preserving revert-test-restore (BUG-134 technique), captured text (`left: 3, right: 2`) matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Sixth bug for `tiles_tools` this session; independent of BUG-131 through BUG-135 — no cross-ref needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether asymmetric width/height was required (H2, falsified — a square rectangle already exposes it) and whether this was an intentional approximation rather than a defect (H3, falsified by the function's own admitting comment). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Workspace-wide grep for `rectangle_query` found the one downstream caller the in-crate-only search missed; verified it compiles clean against the narrowed signature before finalizing the fix. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools` `src/ecs/systems.rs` + `tests/integration/ecs_tests.rs` + this bug file touched (plus a compile-only verification check against the downstream example, no edit needed there). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to `rectangle_query`'s body and signature; no other function touched. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing "rectangular area" contract now actually honored. | — |

**Reproduced:** YES — reverting `rectangle_query`'s filter logic to its exact pre-fix circular
full-diagonal-threshold behavior (signature preserved, so the same test compiles against both
versions) and running
`cargo test --test integration_tests --features enabled,integration test_rectangle_query_excludes_entity_outside_axis_aligned_bounds`
produced the exact predicted wrong value (`left: 3, right: 2`); restoring the fix returned the
full suite to 234/234 passing (including doctests) plus a clean
`cargo clippy --all-targets --features enabled,integration -- -D warnings`, and a clean
`cargo check` on the downstream `ecs_collision_demo` example, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/ecs/systems.rs` | `SpatialQuerySystem::rectangle_query`: narrowed to `Position<square::Coordinate<Connectivity>>`, replaced the circular full-diagonal distance filter with a genuine per-axis `\|dx\| <= width/2 && \|dy\| <= height/2` bounds check. `Fix(BUG-136)`/`Root cause`/`Pitfall` comment added. Added `use crate::coordinates::square::Coordinate as SquareCoordinate;`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/integration/ecs_tests.rs` | New test (`bug_reproducer(BUG-136)`, 5-section doc comment) — `test_rectangle_query_excludes_entity_outside_axis_aligned_bounds`. Added `SpatialQuerySystem` to the `tiles_tools::ecs` import list. |
