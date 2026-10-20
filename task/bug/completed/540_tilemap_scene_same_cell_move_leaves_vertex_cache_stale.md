# BUG-540: A same-cell `placement_move` left the cached vertex resolve stale

- **Severity:** Low (a dual-grid corner could keep the wrong terrain until the next hex-cell change;
  only when two corner terrains share one cell; no crash)
- **state:** Completed
- **Affects:** `tilemap_scene` consumers that keep a `Renderer` across frames and call
  `Scene::placement_move` onto the cell an instance already stands in, on a cell that holds two
  objects competing for the same corner channel.
- **Component:** `module/helper/tilemap_scene` (`src/scene.rs`)
- **repo_identity:** self
- **Filed:** 2026-10-20
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-20
- **Related Bugs:** None. BUG-539 is filed on PR #219's branch, so this record takes 540.

## Symptom

`placement_move` always removed the instance from the spatial indexes and pushed it back, so a
move onto the same hex cell sent it to the end of `hex_instances`. The per-cell tile table is built
in that order, and the dual-grid vertex resolve gives each cell's corner to the first corner
terrain in its stack. The move changed that owner, but `tiles_revision` did not move (the cell's
membership is the same), so a long-lived `Renderer` kept serving the cached resolve with the old
owner while a fresh one showed the new owner.

## Impact

**Who is affected:** Scenes with two corner terrains stacked in one hex cell. `docs/format/005`
already tells spec authors not to rely on tie order, which keeps this rare.

**What breaks:** The cached frame and a fresh frame of the same scene disagree on one corner.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 4 of the PR #207 review (2026-10-20). The cache keyed on `tiles_revision` is new in that
PR (8686f8d7).

## Minimum Reproducible Example

```rust
// ( 0, 0 ) holds grass, then sand: grass owns the corner and makes the grass/sand/water blend.
let grass = scene.instances_at_hex( 0, 0 ).next().unwrap();
renderer.render( &scene, &camera );                       // blend drawn, resolve cached
scene.placement_move( grass, Placement::Hex { q : 0, r : 0 } );
// pre-fix: a fresh Renderer finds sand first and draws no blend,
// while the cached one still draws it
```

**Verify Command:**
```bash
cargo nextest run -p tilemap_scene --test vertex_resolve_cache_test
```

## Root Cause

Re-indexing on every move reordered the cell's stack, and the stack order decides the corner
owner, while the cache is keyed on `tiles_revision`, which tracks membership only.

## Why Not Caught

`tiles_revision_tracks_only_hex_cell_changes` checks that a same-cell move leaves the revision
alone, which is right, but no test put two corner terrains in one cell and compared the cached
resolve with a fresh one.

## Fix Location

- `module/helper/tilemap_scene/src/scene.rs`: `placement_move` leaves the spatial indexes alone
  when the instance stays in the hex cell it stands in. `Placement::Hex` carries only the cell, so
  nothing else changed. `Fix(BUG-540)` comment there.

## Prevention

`vertex_resolve_cache_matches_a_fresh_resolve_after_a_same_cell_move` in
`module/helper/tilemap_scene/tests/vertex_resolve_cache_test.rs` compares a fresh resolve and the
cached one after a same-cell move, on a cell whose corner owner depends on the stack order.

## Pitfall

The cache is keyed on `tiles_revision` alone, so anything that changes a cell's stack order, not
just its membership, must bump it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-20 | filed | Raised by round 4 of the PR #207 review. |
| 2026-10-20 | fixed | `placement_move` keeps the stack order on a same-cell move. |
| 2026-10-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `vertex_resolve_cache_test` passes 7/7. Adversarial pass: without the fix, the reproducer fails on the fresh resolve, which finds sand first and draws no blend (6/7). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-540)` / `Root cause` / `Pitfall` comment in `placement_move`; 5-section doc comment and `bug_reproducer(BUG-540)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the re-indexing in `placement_move`. | — |

**Reproduced:** YES -- the reverted run above. 2026-10-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/src/scene.rs` | `placement_move` keeps the stack order on a same-cell move; `Fix(BUG-540)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/tests/vertex_resolve_cache_test.rs` | `vertex_resolve_cache_matches_a_fresh_resolve_after_a_same_cell_move`, marked `bug_reproducer(BUG-540)`. |
