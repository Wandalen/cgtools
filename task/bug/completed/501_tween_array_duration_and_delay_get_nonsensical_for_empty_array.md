# BUG-501: `[Tween<T>; N]`'s `AnimatablePlayer::duration_get`/`delay_get` return nonsensical values for `N = 0`

- **Severity:** Low (requires a caller to construct a zero-length tween array/group -- an
  unusual but type-valid input; no crash, but returns a nonsensical negative-huge duration and a
  `f64::MAX` delay instead of the "nothing to animate" answer of `0.0`)
- **state:** Completed
- **Affects:** Any caller constructing `[Tween<T>; 0]` (or any generic caller of
  `AnimatablePlayer` over an array type instantiated with `N = 0`).
- **Component:** `module/helper/animation` (`src/interpolation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same field (`[Tween<T>; N]`'s `duration_get`/`delay_get`) as TASK-015 (which
  fixed the min/max-reduction seed/direction for the *non-empty* case) but a different input
  class (`N = 0`, never exercised by that fix's own test) -- filed separately since TASK-015's
  fix is unrelated to and does not cover this case.

## Symptom

```rust
// pre-fix -- src/interpolation.rs, impl AnimatablePlayer for [ Tween<T>; N ]
fn duration_get( &self ) -> f64
{
  let mut min_start = f64::MAX;
  for tween in self { min_start = tween.delay.min( min_start ); } // loop body never runs for N=0

  let mut max_end = 0.0;
  for tween in self { max_end = ( tween.delay + tween.duration ).max( max_end ); } // same

  max_end - min_start // 0.0 - f64::MAX == -f64::MAX for N=0
}

fn delay_get( &self ) -> f64
{
  let mut min_delay = f64::MAX;
  for tween in self { min_delay = tween.delay.min( min_delay ); } // loop body never runs for N=0
  min_delay // f64::MAX for N=0
}
```

For `N == 0`, both reduction loops in each function never execute, so `duration_get` returns
`0.0 - f64::MAX == -f64::MAX` and `delay_get` returns its unreached `f64::MAX` seed unchanged.

## Impact

**Who is affected:** Any caller constructing a zero-length `[Tween<T>; 0]` and querying its
duration or delay -- e.g. a generic animation-group wrapper that happens to be instantiated with
zero tweens for some input.

**What breaks:** `duration_get()` returns `-f64::MAX` (a nonsensical, enormous negative number)
and `delay_get()` returns `f64::MAX` -- neither resembles the intuitively correct "nothing to
animate" answer of `0.0`, and either value would corrupt any downstream scheduling/sequencing
arithmetic that adds or compares against it.

**Consumer audit:** `AnimatablePlayer` is implemented generically for `[Tween<T>; N]` for any
`N` -- grepped for concrete instantiations; no current in-crate caller happens to construct an
`N = 0` array, so this is a hardening fix against a type-valid-but-unexercised input, not a
currently-triggered defect.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of `module/helper/animation`.

## Minimum Reproducible Example

```rust
// module/helper/animation/tests/interpolation_test.rs
let tweens : [ Tween< f32 >; 0 ] = [];
assert_eq!( tweens.delay_get(), 0.0 );    // pre-fix: f64::MAX
assert_eq!( tweens.duration_get(), 0.0 ); // pre-fix: -f64::MAX
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run -E 'test(duration_and_delay_get_for_empty_array)'
```

## Root Cause

Both functions use a min/max-reduction seeded for the non-empty case (`f64::MAX` for a
min-reduction, `0.0` for a max-reduction) -- a seed chosen to be "beaten" by any real element.
For `N == 0` there are no elements to beat it, so each seed is returned completely untouched
(`delay_get`) or combined with the other untouched seed (`duration_get`'s `max_end - min_start`),
producing values with no relationship to a meaningful "empty group" answer.

## Why Not Caught

TASK-015's own regression test (`test_tween_array_duration_and_delay_get`) only ever constructed
non-empty arrays -- nothing exercised `N == 0`, the one case where both reduction loops' seed
values are returned completely untouched instead of being compared against at least one real
element.

## Fix Location

`module/helper/animation/src/interpolation.rs`: added `if self.is_empty() { return 0.0; }` as the
first line of both `duration_get` and `delay_get` (arrays deref to slices, so `.is_empty()` is
available directly), short-circuiting before either reduction loop runs.

## Prevention

New test `test_tween_array_duration_and_delay_get_for_empty_array` in `interpolation_test.rs`,
placed immediately after the existing TASK-015 non-empty-array test, asserting both methods
return `0.0` for `[ Tween<f32>; 0 ]`.

## Pitfall

`max_end - min_start` reads as an obviously-safe non-negative subtraction for the non-empty case,
which is exactly what makes the empty case's `0.0 - f64::MAX` easy to miss in review -- the
formula's shape gives no visual signal that it depends on both loops having executed at least
once.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `module/helper/animation`. |
| 2026-08-20 | fixed | Added `is_empty()` short-circuit guards to both `duration_get` and `delay_get`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily removed both `is_empty()` guards and confirmed `test_tween_array_duration_and_delay_get_for_empty_array` fails; restored the fix and confirmed 49/49 crate tests pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-501)`/`Root cause`/`Pitfall` 3-field comment applied above both functions. | — |
| D3 | Scope containment | — | 🟢 | Changes confined to `interpolation.rs`'s 2 functions (1 guard line each); no unrelated files touched; the pre-existing non-empty-array test (TASK-015) still passes unmodified. | — |

**Reproduced:** YES -- temporarily removed both `is_empty()` guards;
`test_tween_array_duration_and_delay_get_for_empty_array` failed alongside 2 unrelated
BUG-502 tests also mid-revert (nextest fail-fast stopped further tests, all 3 confirmed as
expected failures). Restored the fix; full crate suite (49/49) passes with 0 warnings.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `duration_get`/`delay_get` on `impl AnimatablePlayer for [ Tween<T>; N ]` each gained an `if self.is_empty() { return 0.0; }` guard. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | Added `test_tween_array_duration_and_delay_get_for_empty_array`. |
