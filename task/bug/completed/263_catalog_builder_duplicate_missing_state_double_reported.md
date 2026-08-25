# BUG-263: `CatalogBuilder::build()` double-reports a missing `(object, state)` pair when `state_require` is called twice with the identical pair

- **Severity:** Low (no observed wrong pass/fail outcome -- `build()` still correctly returns `Err` -- but `CatalogError::missing_states` and its `Display` "N missing state(s)" count become inflated/inaccurate whenever a caller's `state_require` calls repeat the identical missing pair)
- **state:** Completed
- **Affects:** `tilemap_scene::catalog::CatalogBuilder::build` (`src/catalog.rs`)
- **Component:** `module/helper/tilemap_scene` (`src/catalog.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`CatalogBuilder::build()`'s objects-accumulation loop dedupes missing ids via a
`seen_missing_objects` set: calling `object_require("x")` (or, implicitly, `state_require("x",
..)`) for the same missing id `x` more than once reports the miss exactly once in
`CatalogError::missing_objects`. The parallel states-accumulation loop had no equivalent guard:
calling `state_require(obj, state)` twice with the exact same, missing `(obj, state)` pair pushed
the pair onto `missing_states` twice, so `CatalogError::missing_states` (and its `len()`) held
duplicate entries and `CatalogError`'s `Display` impl printed the identical `"state: (...)"` line
twice under one inflated "N missing state(s)" count.

## Impact

**Who is affected:** any caller assembling a `CatalogBuilder`'s requirements from more than one
independent code path that can legitimately declare the same `(object, state)` requirement -- e.g.
two subsystems each calling `.state_require("knight", "attack")` during adapter init without
cross-checking whether the other already declared it, which is exactly the scenario `Catalog`'s
own module doc describes it existing to serve ("Hot-path consumers cache handles in a `Catalog`
during init").

**What breaks:** the returned `CatalogError.missing_states` `Vec` and its `Display`-rendered "N
missing state(s)" count over-report the true number of distinct misses, and `.len()` is inaccurate
for any caller that inspects it programmatically (e.g. to decide how many typos to expect to fix).
This contradicts the crate's own stated design intent for `Catalog` ("all misses are reported
together so callers see the full picture") and is asymmetric with the objects loop's already-correct
dedup for the identical situation.

**Entity Scope:** `None` -- source-level accumulation-loop defect, not entity directory instances.

## How Discovered

During this session's Group L review of
`tilemap_scene/src/{anchor,catalog,coords,error,event,hash,instance,layer,lib,object}.rs`, direct
comparison of `CatalogBuilder::build()`'s two parallel accumulation loops (objects vs. states)
against each other showed the objects loop guards every push with `objects.contains_key( id ) ||
seen_missing_objects.contains( id )` while the states loop had no equivalent guard before
`missing_states.push(...)`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tilemap_scene --all-features catalog_build_does_not_double_report_duplicate_missing_state_require
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary direct-source-edit revert of the fix, real run): 1
failed -- `assertion left == right failed: duplicate identical state_require calls must not
double-report: [("knight", "attack"), ("knight", "attack")]` (`left: 2, right: 1`).

## Root Cause

`CatalogBuilder::build()` (pre-fix), abbreviated:
```rust
for ( obj, state ) in &self.states
{
  let Some( &obj_handle ) = objects.get( obj ) else { continue; };
  if let Some( h ) = self.scene.state( obj_handle, state )
  {
    states.insert( ( obj.clone(), state.clone() ), h );
  }
  else
  {
    missing_states.push( ( obj.clone(), state.clone() ) );
  }
}
```
`self.states` is a plain `Vec<(String, String)>` accumulated by every `state_require` call, with no
dedup at push time (mirroring `self.objects`, itself also an un-deduped `Vec`). The objects loop
compensates for this at resolution time with `seen_missing_objects`; the states loop's `else` branch
pushed onto `missing_states` unconditionally on every iteration, so N identical `state_require(obj,
state)` calls for a missing pair produced N identical `missing_states` entries.

## Why Not Caught

Every existing `catalog_test.rs` case calls `state_require` with distinct `(obj, state)` pairs --
none repeated the identical pair twice, so the missing dedup guard on the states loop was never
exercised. The bug produces no panic and no compiler warning; `build()` still correctly returns
`Err(..)` in every case, so any test only checking `is_err()`/`is_ok()` (rather than the exact
`missing_states` contents/length) would not have caught it either.

## Fix Applied (2026-08-17)

**`src/catalog.rs`:** added a `seen_missing_states : FxHashSet<(String, String)>` set and a
`states.contains_key( &key )` guard to the states loop in `CatalogBuilder::build()`, mirroring the
`seen_missing_objects` / `objects.contains_key` pattern the objects loop already uses immediately
above it. A `(obj, state)` pair now contributes to `missing_states` at most once regardless of how
many times `state_require` declared it.

**`tests/catalog_test.rs`** (new test): `catalog_build_does_not_double_report_duplicate_missing_state_require`
calls `.state_require( "knight", "attack" )` twice (identical, missing pair) and asserts
`err.missing_states.len() == 1` with the single expected entry.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tilemap_scene --all-features catalog_build_does_not_double_report_duplicate_missing_state_require`
  -- pre-fix (temporary direct-source-edit revert of the dedup guard, real run): 1 failed, `left: 2,
  right: 1`. Post-fix (guard restored): 1 passed, 0 failed.
- `cargo test -p tilemap_scene --all-features` (full scoped suite): all green across every test
  binary -- 176 passed, 0 failed, 2 ignored (pre-existing, unrelated -- both are doctests requiring
  a live GPU/renderer context) -- including all 7/7 `catalog_test.rs` cases (the 6 pre-existing plus
  the new one).
- `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** two parallel accumulation loops sharing the same "report every unique miss
exactly once" contract will both implement the dedup that contract requires, just because one of
them visibly does. Extending an existing loop's pattern to add a second, structurally-similar loop
(objects → states) without carrying over its dedup guard leaves a defect that is invisible on the
common path (a single `require` call per id) and only surfaces when a caller repeats an identical
requirement -- exactly the kind of caller behaviour a builder API designed to aggregate requirements
from multiple independent call sites should expect. When reviewing a builder/accumulator with more
than one structurally-parallel loop, check that every loop enforces the same invariants the others
do, not just that each loop is individually correct in isolation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group L review of `tilemap_scene/src/{anchor,catalog,coords,error,event,hash,instance,layer,lib,object}.rs`. Root cause: `CatalogBuilder::build()`'s states-accumulation loop had no dedup guard for repeated identical `(obj, state)` misses, unlike the objects loop's `seen_missing_objects` guard for the same situation. Fixed by adding an equivalent `seen_missing_states` set plus a `states.contains_key` guard. Verified via 1 new native unit test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun), the full scoped suite (176 passed / 0 failed / 2 pre-existing-unrelated ignored), and clean clippy. Filed as BUG-263 after a fresh on-disk scan immediately before filing confirmed no collision (highest prior id: 262). |
