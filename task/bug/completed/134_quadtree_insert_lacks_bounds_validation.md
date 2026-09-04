# BUG-134: `Quadtree::insert` lacks bounds validation, silently misfiling out-of-bounds entities

- **Severity:** Medium (silently unqueryable entity — structurally present via `all_entities()`
  but invisible to every spatially-scoped query — no panic, no compile error)
- **state:** Completed
- **Affects:** Any caller of `Quadtree::insert` with an entity position outside the tree's own
  declared `bounds`
- **Component:** `module/helper/tiles_tools` (`src/spatial.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — fourth bug filed for this crate this session; independent of
  BUG-131/132/133 (different module, different mechanism)

## Symptom

```rust
let mut tree = Quadtree::new( SpatialBounds::new( 0, 0, 100, 100 ), 1 );
tree.insert( SpatialEntity::new( 1, Coord::new( 5, 5 ), 0 ) );

// Wrong (pre-fix): insert() returns () -- no signal -- and silently accepts
tree.insert( SpatialEntity::new( 2, Coord::new( 1000, 1000 ), 0 ) ); // "succeeds"
tree.all_entities().len() == 2;                                     // present here...
tree.region_query( &SpatialBounds::new( 900, 900, 1100, 1100 ) ).is_empty(); // ...but unfindable here

// Correct (post-fix): insert() returns bool, rejects out-of-bounds positions
tree.insert( SpatialEntity::new( 2, Coord::new( 1000, 1000 ), 0 ) ) == false;
tree.all_entities().len() == 1; // entity 2 was never stored at all
```

## Impact

**Who is affected:** Any caller of `Quadtree::insert` whose entity position can, for any
reason, fall outside the tree's declared bounds — a moved entity, a stale/mis-sized tree, or
any producer whose coordinate range isn't independently guaranteed to match the tree's.

**What breaks:** `insert_recursive_static`'s quadrant routing selects a quadrant via unbounded
`entity_x >= center_x` / `entity_y <= center_y` comparisons, which always succeeds for any
position — in or out of bounds. An out-of-bounds entity is filed into a leaf whose real
sub-bounds don't contain it. `region_query`'s pruning (`query_bounds.intersects(node_bounds)`,
walked from the tree's fixed `self.bounds`, which never grows) then silently excludes that
entity from every spatially-scoped query, while it remains fully visible via `all_entities()`.

**Magnitude:** Not a crash — a structurally-present entity that is unreachable by the very
queries (`region_query`, `circle_query`) the data structure exists to serve, with zero error
signal at insert time or query time.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #66, a targeted code review of `tiles_tools` under the standing bug-hunt mandate. The
reviewing agent flagged that `insert`/`insert_recursive_static` never validates an entity's
position against the tree's declared bounds. Independently confirmed by hand-tracing the exact
scenario: inserting entity A at `(1000,1000)` into a `Quadtree::new(SpatialBounds::new(0,0,100,100), 1)`,
then B at `(5,5)` (triggering subdivision), routes A into the southeast leaf (bounds become
`(50,50,100,100)` via unbounded center-comparison routing), after which
`region_query(&SpatialBounds::new(900,900,1100,1100))` returns empty because the top-level
`query_bounds.intersects(node_bounds=self.bounds=(0,0,100,100))` check fails immediately and
prunes before ever descending into the tree.

## Minimum Reproducible Example

```bash
cd module/helper/tiles_tools && cargo test --test spatial_test --features enabled test_quadtree_insert_rejects_out_of_bounds_entity 2>&1 | tail -10
```

**Expected** (post-fix):
```
test test_quadtree_insert_rejects_out_of_bounds_entity ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `insert`'s bounds-check guard clause
only, keeping the `-> bool` signature so the test still compiles unmodified against the
reverted logic, then restoring the fix immediately after capturing the failure):
```
thread 'test_quadtree_insert_rejects_out_of_bounds_entity' panicked at
module/helper/tiles_tools/tests/spatial_test.rs:179:3:
insert() must reject a position outside the quadtree's own declared bounds
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo test --test spatial_test --features enabled test_quadtree_insert_rejects_out_of_bounds_entity
# 1 passed = fixed; 1 failed (assertion panic) = bug present
```

**Known MRE limitation (check 205):** none — `Quadtree::insert`/`region_query` are pure,
synchronous, dependency-free data structure operations; runs as an ordinary native `cargo test`
against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `insert`/`insert_recursive_static` never validates an entity's position against the tree's declared bounds before routing it into a quadrant. | ✅ Root Cause | Direct read of `src/spatial.rs`'s `insert`, `insert_recursive_static`, and `node_subdivide_static` (lines 236-378 pre-fix): no call to `SpatialBounds::contains_point` or any bounds check anywhere in the insert path. | E1 |
| H2 | The bug is in `region_query`'s pruning logic itself (a wrong intersects check), not in `insert`. | ❌ Falsified | `query_recursive`'s `query_bounds.intersects(node_bounds)` check is correct for its own contract (prune subtrees whose declared bounds can't overlap the query) — it behaves exactly as designed; the defect is that `insert` let an entity violate the very invariant `region_query` depends on (that a leaf's real bounds contain everything filed under it). | E1, E2 |
| H3 | The defect only manifests after subdivision (a single unsubdivided root leaf is unaffected). | ⚠️ Partially true, doesn't change the fix | A single-leaf tree with no subdivision would still return the out-of-bounds entity from `all_entities()` and even from a query whose bounds happen to include the leaf's synthetic root-level bounds check — but `region_query`'s very first check is against `self.bounds` regardless of subdivision, so a query far outside `self.bounds` is pruned at the root in both cases. Subdivision isn't required to reproduce; the MRE's `max_entities: 1` forces it purely to also confirm quadrant-routing accepts the bad position, not just the trivial root-leaf case. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/spatial.rs:236-378`, pre-fix | `insert` delegates straight to `insert_recursive_static`; the `Internal` branch computes `bounds.center()` then routes via `entity_x >= center_x`/`entity_y <= center_y` — both comparisons well-defined (and always resolve to some quadrant) for any `i32`, not just values inside `bounds`. | H1 ✅ |
| E2 | `src/spatial.rs:407-415` (`query_recursive`) | `if !query_bounds.intersects(node_bounds) { return; }` — correct pruning logic given its precondition (every entity under a node lies within that node's bounds), a precondition `insert` was the one violating. | H2 ❌ |
| E3 | MRE run, reverted guard-clause-only code | Test fails exactly at the `assert!(!quadtree.insert(out_of_bounds), ...)` line — confirms unconditional acceptance is the defect, independent of the tree having already subdivided by that point (`max_entities: 1` in the MRE forces the first insert to subdivide before the out-of-bounds second insert runs). | H1 ✅, H3 |

## Root Cause

```
insert(entity):
  insert_recursive_static(root, entity, self.bounds, ...)   // <- no bounds.contains_point check

insert_recursive_static (Internal branch):
  center = bounds.center()
  in_north = entity_y <= center_y      // well-defined (and quadrant-selecting) for ANY entity_y
  in_east  = entity_x >= center_x      // well-defined (and quadrant-selecting) for ANY entity_x
  -> always picks a quadrant, never rejects
```

The recursive quadrant-selection comparisons are total functions over all of `i32` — they never
fail to produce an answer, so an out-of-bounds position is routed exactly as confidently as an
in-bounds one, silently violating the invariant every other method (`region_query` in
particular) depends on.

## Why Not Caught

Every existing test in `tests/spatial_test.rs` inserted entities well within the declared
`(0,0,100,100)` bounds — none exercised a position outside them, so the always-succeeds
quadrant routing never had a chance to misbehave visibly.

## Fix Location

`module/helper/tiles_tools/src/spatial.rs`, `Quadtree::insert`:

```rust
// before
pub fn insert(&mut self, entity: SpatialEntity<C>) {
    let bounds = self.bounds;
    let max_entities = self.max_entities;
    Self::insert_recursive_static(&mut self.root, entity, &bounds, 0, max_entities, &mut self.max_depth);
}

// after
#[must_use]
pub fn insert(&mut self, entity: SpatialEntity<C>) -> bool {
    let (x, y) = entity.position.to_spatial_coords();
    if !self.bounds.contains_point(x, y) {
        return false;
    }

    let bounds = self.bounds;
    let max_entities = self.max_entities;
    Self::insert_recursive_static(&mut self.root, entity, &bounds, 0, max_entities, &mut self.max_depth);
    true
}
```

The check lives once, at the entry point — the recursive quadrant split already preserves
containment correctly for any position confirmed to start inside `bounds` (by construction:
each split's sub-bounds are exactly the half of the parent's bounds the comparison selects), so
a bounds check at every recursion level would be redundant. `insert`'s signature changes from
`()` to `#[must_use] bool` (mirroring `std::collections::HashSet::insert`'s own convention) so a
caller cannot silently ignore a rejected insert the way the original `()` return made
unavoidable. Only workspace call sites are this crate's own `tests/spatial_test.rs` (9 sites,
all pre-existing in-bounds inserts) — updated to `assert!(quadtree.insert(...))`.

## Prevention

Added `test_quadtree_insert_rejects_out_of_bounds_entity` to `tests/spatial_test.rs`, covering
an out-of-bounds insert's rejected return value, its absence from `all_entities()`, and a
region query for the area it would have silently occupied.

**Pitfall:** invisible under any workload that only ever inserts positions already known to be
within the tree's declared bounds — the defect only surfaces once a caller's own
bounds-computation and the tree's own bounds can disagree (e.g. an entity that moved, or a tree
sized from a stale world extent).

## Generalized Version

**Broken assumption:** "a routing function that always returns *some* valid destination is
therefore always *correct*." Silently false whenever the routing function's domain (here, all
of `i32 × i32`) is wider than the precondition its result depends on elsewhere (here, "this
leaf's real bounds contain everything filed under it," relied on by `region_query`'s pruning).

**Confirmed general rule:** any data structure whose query-side optimization prunes by
structural bounds (rather than checking every stored element directly) must validate that
invariant at the single point of insertion — a total, always-succeeding routing function is a
liability, not a convenience, whenever a downstream reader assumes routing implies containment.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #66's targeted code review of `tiles_tools`; confirmed by hand-tracing an out-of-bounds insert through both `insert_recursive_static`'s quadrant routing and `region_query`'s root-level pruning. |
| 2026-08-16 | fixed | `insert` now validates `self.bounds.contains_point` before routing, returning `#[must_use] bool` instead of `()`; 9 existing in-crate call sites updated to assert the (always-true, pre-existing-behavior-preserving) return value. |
| 2026-08-16 | verified | Added `test_quadtree_insert_rejects_out_of_bounds_entity`; confirmed it fails against the reverted pre-fix guard clause with the exact predicted panic, and passes against the fix; full crate suite (234 tests incl. doctests) + `cargo clippy --all-targets --features enabled,integration -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-16 earlier same day, this verifier 2026-08-16). Independently re-read `Quadtree::insert` (confirmed the `#[must_use] -> bool` signature and `self.bounds.contains_point(x, y)` guard genuinely present, 6-line `Fix(BUG-134)`/`Root cause`/`Pitfall` comment intact) and `test_quadtree_insert_rejects_out_of_bounds_entity` (non-tautological: asserts `insert()` returns `false`, entity absent from `all_entities()`, and absent from a `region_query` covering where it would have been silently filed). Fresh `cargo nextest run --all-features` via `longrun` (crate-wide, covering BUG-131 through BUG-137 together): 251/251 passed. `cargo clippy --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-131 through BUG-137 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-traced the desync; adversarial pass required an actual runtime FAIL against reverted logic (not just a compile-time signature mismatch) — closed by reverting only the guard clause (keeping the `bool` signature) so the identical test runs against both versions; captured panic matched exactly. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Fourth bug for `tiles_tools` this session; independent of BUG-131/132/133 — no cross-ref needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether the defect was in `region_query`'s pruning instead (H2) and whether subdivision was a precondition (H3) — both addressed via direct source read and MRE design. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped the full workspace for `Quadtree` usage outside its own source file — confirmed only `tests/spatial_test.rs` (9 sites) calls `insert`; all updated. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tiles_tools` `src/spatial.rs` + `tests/spatial_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Signature change (`()` → `bool`) is contained entirely within this crate — no downstream workspace crate/example calls `Quadtree::insert`. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public type added — `insert`'s existing contract (only ever accept entities the tree can actually serve back) is now honored, signaled via its return value. | — |

**Reproduced:** YES — reverting `insert`'s bounds-check guard clause (keeping the `bool`
signature) and running
`cargo test --test spatial_test --features enabled test_quadtree_insert_rejects_out_of_bounds_entity`
produced the exact predicted panic; restoring the fix returned the full suite to 234/234 passing
(including doctests) plus a clean
`cargo clippy --all-targets --features enabled,integration -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/spatial.rs` | `Quadtree::insert`: signature changed from `fn insert(&mut self, entity: SpatialEntity<C>)` to `#[must_use] fn insert(&mut self, entity: SpatialEntity<C>) -> bool`; added a `self.bounds.contains_point` guard that rejects (returns `false`, no mutation) an out-of-bounds position. `Fix(BUG-134)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/spatial_test.rs` | 9 existing `quadtree.insert(...)` call sites updated to `assert!(quadtree.insert(...))` (all pre-existing in-bounds inserts, behavior-preserving). New test (`bug_reproducer(BUG-134)`, 5-section doc comment) — `test_quadtree_insert_rejects_out_of_bounds_entity`. |
