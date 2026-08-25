# BUG-238: `Line::point_add_back`/`point_add_front` compare a squared distance against an
unsquared `EPSILON`, silently dropping distinct points ~2900x closer than intended

- **Severity:** Medium (no crash, no panic -- silent data corruption instead: a legitimately
  distinct point is dropped with no error signal, reachable via a widely-used, actively-tested
  public API with no validation gap needed to hit it)
- **state:** Completed
- **Affects:** Any `d3::Line::point_add_back`/`point_add_front` (and their batch counterparts
  `points_add_back`/`points_add_front`) call where consecutive points are closer than
  `sqrt( f32::EPSILON )` (~3.4527e-4) apart, built with `feature = "distance"`.
- **Component:** `module/helper/line_tools` (`src/lib.rs`, `impl_basic_line!` macro, consumed by
  `src/d3/line.rs`'s `impl_basic_line!( Line, f32, 3 );`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Same macro (`impl_basic_line!` in `lib.rs`) as
  [BUG-154](./154_point_set_color_set_doc_panic_mismatch.md) (doc/behavior mismatch on
  `point_set`/`color_set`) and the pre-`BUG-NNN`-era `issue-001`/`issue-002`/`issue-003` fixes
  (all in this same distance-bookkeeping region) -- a different defect class each time, but
  confirms this macro's distance/point arithmetic has a recurring history of subtle bugs worth
  extra scrutiny. Not related to BUG-236/BUG-237 (different crate region, different defect shape
  -- those are unguarded-divisor NaN bugs; this is a squared-vs-linear comparison unit mismatch).

## Symptom

```rust
// pre-fix, inside impl_basic_line!'s point_add_back/point_add_front
if ( last - point ).mag2() <= $primitive_type::EPSILON   // mag2() is SQUARED distance
{
  return;   // silently drops `point` -- treated as a duplicate of `last`
}
```

`mag2()` returns the squared Euclidean distance, but was compared directly against
`f32::EPSILON` (~1.1920929e-7), a linear-scale tolerance. Solving `d² <= t` for `d` gives
`d <= sqrt(t)`, so the *actual* dedup radius is `sqrt( f32::EPSILON )` (~3.4527e-4) -- about
2900x larger than `f32::EPSILON` itself, and ~34,527x larger than the sibling `d2::Line::point_add`
implementation's own linear per-axis threshold (`1e-8`, hand-written, not from this macro).

## Impact

**Who is affected:** Any caller of `d3::Line::point_add_back`/`point_add_front` (or the batch
`points_add_back`/`points_add_front`, which loop-call these) built with `feature = "distance"`
-- an actively-used, actively-tested feature (`tests/webgl/distance.rs`'s 60+ tests,
`tests/webgl/dash.rs`), not a dead/unused one. Any two consecutive points closer than
~3.45e-4 apart -- a small but easily-crossed distance for finely-sampled paths, small-scale
scenes, or normalized coordinate systems -- get silently merged: the second point is dropped
entirely, `total_distance`/`distances` are never incremented for it, and no error or panic
signals the loss.

**What breaks:** The line's point list silently loses fidelity/detail with no diagnostic --
worse than a `NaN` in one sense, since `is_finite()` cannot detect a *missing* point the way it
can flag a corrupted one.

**Magnitude:** 1 macro (`impl_basic_line!`), 2 call sites (`point_add_back`, `point_add_front`),
1 fix shape (square the comparison threshold) applied to both.

**Entity Scope:** None -- a code-level defect.

## How Discovered

While scouting `line_tools` for task #169 (systematic bug hunt), reached `d3/line.rs` and found
it delegated its point-adding logic to the `impl_basic_line!` macro in `lib.rs` rather than
hand-writing it (unlike `d2/line.rs`'s hand-written `point_add`). Reading the macro definition in
full, `point_add_back`/`point_add_front`'s near-duplicate dedup guard used `.mag2()` (a squared
quantity, per `ndarray_cg::Vector::mag2`'s own doc comment: "Compute the squared length of the
vector") compared directly against `$primitive_type::EPSILON` with no squaring -- a classic
squared-distance-vs-linear-threshold unit mismatch. Confirmed `mag2()`/`mag()`'s exact semantics
by reading `ndarray_cg/src/vector/arithmetics.rs` directly rather than assuming from the name.
Confirmed the existing test suite (`tests/webgl/distance.rs`) only ever exercises *exact*
duplicate points (distance 0.0, which correctly dedups under both the buggy and fixed
comparison), never a near-but-genuinely-distinct point in the gap between the intended and
accidentally-inflated thresholds.

## Minimum Reproducible Example

```rust
use line_tools::d3::Line;

let mut line = Line::default();
line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
line.point_add_back( &[ 0.0001, 0.0, 0.0 ] );   // 1e-4 apart -- well inside the buggy ~3.45e-4 radius
assert_eq!( 2, line.distances_get().len() );    // pre-fix: fails, only 1 point survives
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && cargo nextest run --all-features -E 'test(bug_238)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `point_add_back`/`point_add_front` compare `mag2()` (squared distance) directly against `$primitive_type::EPSILON` (a linear-scale tolerance) with no squaring, making the effective dedup radius `sqrt(EPSILON)` instead of `EPSILON`, so two genuinely distinct points closer than ~3.45e-4 apart get silently merged. | ✅ Root Cause | Direct read of pre-fix macro body confirms the unsquared comparison; `ndarray_cg`'s own doc comments confirm `mag2()`/`mag()`'s exact semantics; confirmed empirically via temporary-revert-and-rerun (both new tests failed with `left: 2, right: 1` -- the second point silently dropped, exactly as predicted). | E1, E2, E3, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/line_tools/src/lib.rs`, `impl_basic_line!` macro, `point_add_back`/`point_add_front` (pre-fix, direct read) | Both functions gate their near-duplicate check on `( last - point ).mag2() <= $primitive_type::EPSILON` -- a squared quantity compared against an unsquared constant. | H1 ✅ |
| E2 | `module/math/ndarray_cg/src/vector/arithmetics.rs` lines 12-19 and 75-77 (direct read) | `mag2()`'s doc comment: "Compute the squared length of the vector." `mag()`'s doc comment: "Compute the length of the vector." Confirms `mag2()` is genuinely squared, not a naming coincidence. | H1 ✅ |
| E3 | `module/helper/line_tools/tests/webgl/distance.rs`, `test_distance_add_back_duplicate_ignored`/`test_distance_add_at_the_same_position` (pre-existing tests, direct read) | Both only ever add EXACT duplicate points (distance 0.0 exactly), which dedups correctly under both the buggy and fixed comparison -- confirms no existing test exercised the actual defect's failure zone. | H1 ✅ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting both call sites back to the unsquared `$primitive_type::EPSILON` reproduced `assertion left == right failed: ... left: 2, right: 1` on both new tests -- the second point (1e-4 away) silently dropped, exactly as predicted. | H1 ✅ |

## Root Cause

`point_add_back`/`point_add_front` use `( last - point ).mag2()` -- the squared Euclidean
distance -- to avoid an unnecessary `sqrt()` call on every point-add (`mag2()` is a legitimate,
deliberate optimization for what can be a hot per-point path). But the comparison threshold was
left as `$primitive_type::EPSILON` unchanged, rather than squared to match. Since `d² <= t` is
equivalent to `d <= sqrt(t)` (for non-negative `d`, `t`), the actual linear-distance dedup radius
silently became `sqrt( f32::EPSILON )` (~3.4527e-4) instead of the evidently-intended
`f32::EPSILON` (~1.1920929e-7) -- roughly 2900x too permissive.

## Why Not Caught

Every existing test that exercises the dedup path (`test_distance_add_back_duplicate_ignored`,
`test_distance_add_at_the_same_position`) adds points at *exactly* identical coordinates
(distance 0.0), which correctly triggers the early-return under both the buggy and the fixed
comparison (`0.0 <= anything non-negative` is always true) -- no test ever added a point that
was close-but-genuinely-distinct in the ~1.19e-7-to-3.45e-4 gap where the two comparisons
actually diverge.

## Fix Location

`module/helper/line_tools/src/lib.rs`: both `point_add_back` (line 109) and `point_add_front`
(line 143) inside the `impl_basic_line!` macro now compare against
`$primitive_type::EPSILON * $primitive_type::EPSILON`, keeping the effective linear-distance
dedup radius at `$primitive_type::EPSILON` as evidently intended, while preserving the `mag2()`
optimization (no added `sqrt()` call).

## Prevention

`tests/webgl/distance.rs::test_distance_add_back_near_duplicate_not_dropped_bug_238` and
`test_distance_add_front_near_duplicate_not_dropped_bug_238` each add two points 1e-4 apart
(well inside the old buggy ~3.45e-4 radius, comfortably outside the fixed ~1.19e-7 one) and
assert both survive. A third test, `test_distance_add_back_true_near_zero_still_deduped`, adds a
point 1e-10 away (well under the fixed radius) and confirms it is still correctly treated as a
duplicate and dropped -- locking in that the fix narrows the radius without breaking genuine
near-exact-duplicate detection.

## Pitfall

Any `mag2()`/`distance_squared()`-based comparison must square its threshold to match: `d² <= t`
means `d <= sqrt(t)`, not `d <= t`. Using a squared distance as a cheap `sqrt()`-avoidance
optimization is legitimate and common, but the comparison constant must be re-derived
(`threshold² `, not `threshold`) whenever the underlying quantity being compared changes from
linear to squared -- copying an existing linear-scale constant (here, `$primitive_type::EPSILON`,
already a familiar "near-zero tolerance" idiom in Rust) across to a squared-quantity comparison
without re-deriving it is a natural, easy-to-miss mistake precisely because both sides of the
comparison still type-check and compile without any error.

## Generalized Version

**Broken assumption:** "`EPSILON` is a universal near-zero tolerance constant, safe to compare
against any small quantity regardless of whether that quantity is linear or squared."

**Confirmed general rule:** A squared quantity (`mag2()`, `distance_squared()`, etc.) must only
ever be compared against an already-squared threshold. When reusing an existing linear-scale
tolerance constant for a squared comparison, square it first
(`threshold * threshold` or `threshold.powi(2)`) -- never compare it unchanged. This is worth a
deliberate check (`grep -n "mag2()\|distance_squared()" -A2` and eyeball the RHS of each
comparison) whenever a codebase mixes squared- and linear-distance APIs in nearby code, since
both compile cleanly either way and only produce numerically wrong (not type-wrong) results.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found while scouting `d3/line.rs` for task #169, tracing its point-adding logic into the shared `impl_basic_line!` macro in `lib.rs`. |
| 2026-08-17 | fixed | Both `point_add_back` and `point_add_front` now compare against `$primitive_type::EPSILON * $primitive_type::EPSILON` (full `Fix(BUG-238)` comment block on the first occurrence, cross-referencing comment on the second). |
| 2026-08-17 | verified | `cargo nextest run --all-features` (scoped to `line_tools`): 101/101 passed, 0 skipped. `cargo clippy --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (`left: 2, right: 1` on both new tests pre-fix, matching the exact predicted symptom; both pass post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, exact integer length assertion (`2` vs `1`) is a non-flaky check. Adversarial pass: re-verified `mag2()`'s squared semantics against `ndarray_cg`'s own source (not just inferred from the name) before relying on it as the root cause -- confirmed at `module/math/ndarray_cg/src/vector/arithmetics.rs:12-19,75-77`. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified BUG-154 and the issue-001/002/003 trio as prior fixes in the same macro region (different defect class each time, cited for context, not as duplicates); correctly distinguished this defect from BUG-236/BUG-237 (different region, different shape). | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of the macro body, `ndarray_cg`'s `mag2`/`mag` doc comments, and empirical revert-rerun proof showing the exact predicted `left: 2, right: 1` failure on both call sites. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to squaring the threshold at both call sites, no other logic touched. Adversarial pass: re-checked `points_add_back`/`points_add_front` (batch variants) are pure delegators with no independent copy of the bug needing a separate fix, and confirmed via grep that `impl_basic_line!` has exactly one invocation site in the workspace (`d3::Line`, `f32`), so no other instantiation was missed. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely inside the macro definition; no call-site signature changed, so no caller needed updating. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with the exact predicted `left: 2,
right: 1` symptom on both call sites, pass post-fix) and temporary direct-source-edit
revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/lib.rs` | `impl_basic_line!` macro: `point_add_back` (line 109) and `point_add_front` (line 143) now compare `mag2()` against `$primitive_type::EPSILON * $primitive_type::EPSILON` instead of the unsquared `$primitive_type::EPSILON` (full `Fix(BUG-238)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/distance.rs` | Added `test_distance_add_back_near_duplicate_not_dropped_bug_238`, `test_distance_add_front_near_duplicate_not_dropped_bug_238` (both `bug_reproducer(BUG-238)`, 5-section doc comment on the first, cross-referencing comment on the second) and `test_distance_add_back_true_near_zero_still_deduped`. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects. |
