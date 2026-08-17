# BUG-267: `direction_alignment_calculate`'s spurious `current_distance == 0.0` guard makes every ray-casting FOV hop converge onto the same neighbor regardless of direction

- **Severity:** High (breaks `FOVAlgorithm::RayCasting` directional precision for every call --
  every ray's first hop is misdirected, silently degrading visibility results with no panic)
- **state:** Completed
- **Affects:** `tiles_tools::field_of_view::direction_alignment_calculate` (private fn, called via
  `directional_ray_cast` from `ray_casting_fov_calculate`) (`src/field_of_view.rs`)
- **Component:** `module/helper/tiles_tools` (`src/field_of_view.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`direction_alignment_calculate(viewer, current, next, direction_target)` guarded against division
by zero with `if target_distance == 0.0 || current_distance == 0.0 { return 0.0; }`. Only
`target_distance` is ever used as a divisor anywhere in the function -- `current_distance` is
computed and then never divided by. `directional_ray_cast` always starts its walk with `current =
viewer.clone()`, so `current_distance == 0.0` (`viewer.distance(current)` with `current == viewer`)
is true on the very first hop of every single ray, for every candidate neighbor, regardless of
which direction that ray is aimed at.

## Impact

**Who is affected:** any caller of `FieldOfView::fov_calculate` (or the lower-level
`ray_casting_fov_calculate`) using `FOVAlgorithm::RayCasting`.

**What breaks:** on the first hop of every ray, `direction_alignment_calculate` returns `0.0` for
every candidate neighbor (the guard fires before the real alignment math runs). The caller in
`directional_ray_cast` selects the best-aligned neighbor via a strict `>` comparison over
iteration order, so when every candidate ties at `0.0`, the same first-iterated neighbor is chosen
every time -- independent of `direction_target`. Concretely, at `max_range == 1`, only one of the
viewer's neighbors becomes visible instead of all of them, and for longer ranges every ray's first
step (and everything built on it) is misdirected toward one fixed neighbor rather than the
neighbor closest to its own intended direction. This silently produces wrong, direction-blind
visibility results with no panic or error signal.

**Entity Scope:** `None` -- source-level control-flow defect, not entity directory instances.

## How Discovered

During this session's Group J review of `tiles_tools/src/field_of_view.rs`, a manual trace of
`direction_alignment_calculate`'s divisor usage against its zero-guards showed `current_distance`
guarded but never divided by, while `directional_ray_cast`'s call pattern
(`current = viewer.clone()` at the start of every ray) meant the guard's condition was
unconditionally true on every ray's first hop.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tiles_tools --all-features --test integration_tests test_ray_casting_reaches_every_immediate_neighbor_at_range_one
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary direct-source-edit revert of the fix, real run
alongside this session's other 2 then-reverted bugs, `--no-fail-fast`): 1 failed --
`thread '...' panicked at .../field_of_view_tests.rs:640:5: expected immediate neighbor Coordinate
{ x: 9, y: 10, .. } to be visible at range 1` (`integration_tests` target: 196 passed, 1 failed).

## Root Cause

`direction_alignment_calculate` (pre-fix), abbreviated:
```rust
fn direction_alignment_calculate<C>(viewer: &C, current: &C, next: &C, direction_target: &C) -> f32
where
  C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
{
  let target_distance = viewer.distance( direction_target ) as f32;
  let current_distance = viewer.distance( current ) as f32;
  let next_distance = viewer.distance( next ) as f32;
  let target_to_next = direction_target.distance( next ) as f32;

  if target_distance == 0.0 || current_distance == 0.0 { return 0.0; }
  // .. real alignment math, divides only by target_distance ..
}
```
`current_distance` is computed and referenced nowhere else in the function body -- it is never a
divisor, so guarding on it being zero protects nothing. Since `directional_ray_cast` always begins
each ray with `current = viewer.clone()`, `viewer.distance(current)` is `0.0` by construction on
every ray's first call, making the spurious guard fire unconditionally at the start of every ray
and forcing every first-hop candidate to tie at alignment `0.0`.

## Why Not Caught

The existing `field_of_view_tests.rs` coverage for `RayCasting` (`test_ray_casting_fov`,
`test_single_light_source_calculation`, and others) asserts on aggregate visibility properties
(count, symmetry, presence of specific far cells) rather than checking that every one of a
viewer's immediate neighbors becomes visible at `max_range == 1` -- the exact case that isolates
the first-hop misdirection most starkly, since at that range there is no second hop to
coincidentally "recover" a wrong first choice.

## Fix Applied (2026-08-17)

**`src/field_of_view.rs`:** removed the `|| current_distance == 0.0` clause from the guard,
leaving only `if target_distance == 0.0 { return 0.0; }` (the one guard that actually protects a
real divisor in this function).

**`tests/integration/field_of_view_tests.rs`** (new test):
`test_ray_casting_reaches_every_immediate_neighbor_at_range_one` builds a `FieldOfView` with
`FOVAlgorithm::RayCasting`, calls `fov_calculate(&viewer, 1, |_| false)`, and asserts every one of
`viewer.neighbors()` is present in `visible_coordinates()`.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tiles_tools --all-features --test integration_tests
  test_ray_casting_reaches_every_immediate_neighbor_at_range_one` -- pre-fix (temporary
  direct-source-edit revert, real run): panics, `expected immediate neighbor .. to be visible at
  range 1`. Post-fix (restored): 1 passed.
- `cargo test -p tiles_tools --all-features --no-fail-fast` (full scoped suite, this session's
  other 3 bugs simultaneously reverted): `integration_tests` target 196 passed, 1 failed --
  exactly and only the new test, with no collateral damage to any of the other 196 pre-existing
  cases in that binary. Post-fix (all 4 restored): full suite green across all 10 test binaries
  (`integration_tests`: 197/197) plus 40 doctests, 0 failed.
- `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a zero-value guard placed alongside a legitimate divisor guard is itself
protecting a divisor, just because it shares the same `if` condition and "looks like" defensive
code. A variable used as a divisor elsewhere in the same function does not imply every
similarly-named or similarly-computed variable is also a divisor -- verify each guarded value
against an actual division site before trusting the guard's necessity, especially when (as here)
one of the guarded values is structurally guaranteed to be zero on a common, non-error code path
(the first hop of a walk that starts at its own reference point), which turns a merely-redundant
guard into a total-coverage-breaking one.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group J review of `tiles_tools/src/field_of_view.rs`. Root cause: `direction_alignment_calculate`'s `current_distance == 0.0` guard protected no actual divisor and fired unconditionally on every ray's first hop (since `directional_ray_cast` always starts `current = viewer.clone()`), forcing every candidate neighbor to tie at alignment 0.0 and collapsing all rays onto the same first-iterated neighbor. Fixed by removing the spurious guard clause, keeping only the genuine `target_distance == 0.0` check. Verified via 1 new native unit test (confirmed fail pre-fix via a combined `--no-fail-fast` run with this session's other 2 then-reverted bugs -- real panic, exact expected message -- and pass post-fix), the full scoped suite (197/197 in `integration_tests`, all 10 binaries + 40 doctests green), and clean clippy. Filed as BUG-267 after a fresh on-disk scan immediately before filing found 266 (this session's own debug.rs bug) as the highest existing ID. |
