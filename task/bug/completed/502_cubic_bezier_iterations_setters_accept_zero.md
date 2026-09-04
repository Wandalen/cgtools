# BUG-502: `CubicBezier::iterations_set`/`with_iterations` accept `0`, reintroducing the TASK-041 unsolved-curve defect via explicit caller action

- **Severity:** Low (requires an explicit caller call to either setter with `0` -- none of this
  crate's own 24 named-curve constructions do so; no crash, but silently disables the
  Newton-Raphson solve loop for any caller that does)
- **state:** Completed
- **Affects:** Any caller explicitly calling `CubicBezier::iterations_set( 0 )` or
  `.with_iterations( 0 )`.
- **Component:** `module/helper/animation` (`src/easing/cubic/bezier.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect *class* as TASK-041 (`iterations = 0` skips the Newton-Raphson
  solve loop, producing the wrong easing shape) but reached through a different path -- TASK-041
  fixed the constructor's *default*, this fixes the public *setters* that can override that
  default right back to the degenerate value. Filed as a bug, not folded into TASK-041, since the
  setters are a distinct code path with their own regression risk.

## Symptom

```rust
// pre-fix -- src/easing/cubic/bezier.rs
pub fn iterations_set( &mut self, iterations : usize )
{
  self.iterations = iterations; // no floor -- 0 silently accepted
}

pub fn with_iterations( mut self, iterations : usize ) -> Self
{
  self.iterations = iterations; // same
  self
}
```

`CubicBezier::new` defaults `iterations` to `8` specifically because `apply`'s Newton-Raphson
solve loop is `for _ in 0..self.iterations`, and `0` iterations means the loop body never runs,
leaving `bezier_t` at the raw input `time` instead of the solved Bezier parameter (Fix(TASK-041)).
Both setters wrote their `iterations` argument straight to the field with no floor, so an explicit
`.with_iterations( 0 )` (or `.iterations_set( 0 )`) call reintroduces the exact TASK-041 defect
via caller action, bypassing the constructor's safe default entirely.

## Impact

**Who is affected:** Any caller explicitly setting `iterations` to `0` via either public setter.

**What breaks:** `apply` silently returns `y_get( time )` evaluated at the raw input fraction
instead of the solved Bezier easing value -- the wrong easing shape, with no error or warning.

**Consumer audit:** Grepped this file's own 24 `impl_easing_function!` invocations (every named
curve, `EaseInSine` through `EaseInOutBack`) -- all pass `.with_iterations( 8 )`; none pass `0`.
No other in-crate or workspace caller calls either setter. The floor changes no existing
behavior; it only closes a currently-unused but publicly-reachable footgun.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of `module/helper/animation`, while reviewing
`CubicBezier::new`'s existing `Fix(TASK-041)` comment and checking whether its stated invariant
(`iterations` must be `>= 1` for the solve loop to run) was actually enforced everywhere the field
could be written, not just at construction.

## Minimum Reproducible Example

```rust
// module/helper/animation/tests/easing_test.rs
let curve = CubicBezier::< f32 >::new( [ 0.12, 0.0, 0.39, 0.0 ] ).with_iterations( 0 );
let result = curve.apply( 0.0, 1.0, 0.5 );
// pre-fix: result == 0.125 exactly (y_get(0.5) == 0.5^3, the raw unsolved pass-through)
assert!( ( result - 0.125 ).abs() > 0.01 );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run -E 'test(cubic_iterations_floored_at_one)'
```

## Root Cause

Fixing a bad default only closes the constructor path -- any public setter writing the same field
re-opens the identical defect unless it enforces the same constraint the default was protecting.
`iterations_set`/`with_iterations` were added (or left) without carrying forward the "must be
>= 1" invariant `CubicBezier::new`'s own `Fix(TASK-041)` comment documents.

## Why Not Caught

TASK-041's own test (`test_cubic_mid_curve_accuracy`) only exercises the *default* (`iterations:
8` from `new`, via the named-curve builders' `.with_iterations( 8 )` chains) -- nothing called
either setter with `0` to check whether the same degenerate case the constructor default was
chosen to avoid was still reachable through the public setter API.

## Fix Location

`module/helper/animation/src/easing/cubic/bezier.rs`: changed both `iterations_set` and
`with_iterations` to `self.iterations = iterations.max( 1 );`, flooring at `1` instead of writing
the caller's value unchecked. Verified against this file's own 24 call sites (all pass `8`) that
the floor changes no existing behavior.

## Prevention

Two new tests in `easing_test.rs`: `test_cubic_iterations_floored_at_one_via_with_iterations` and
`test_cubic_iterations_floored_at_one_via_iterations_set`, each constructing a `CubicBezier` with
the affected setter called with `0`, then asserting `apply`'s mid-curve result does *not* match
the known-exact pre-fix raw-pass-through value (`0.125`, independently derivable since this
curve's y-tangents are both `0.0`, making `y_get( t ) == t^3` exactly).

## Pitfall

`with_iterations( 0 )` compiles and runs with no error or warning -- the defect is silent exactly
like the original TASK-041 default was, just reachable through a different call path (explicit
caller action instead of an unset default). A field with a "degenerate value" needs that
constraint enforced at every write site, not just the one the original bug happened to be filed
against.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `module/helper/animation`. |
| 2026-08-20 | fixed | Floored both `iterations_set` and `with_iterations` at `1` via `.max( 1 )`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily removed `.max( 1 )` from both setters and confirmed both new tests fail; restored the fix and confirmed 49/49 crate tests pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-502)`/`Root cause`/`Pitfall` 3-field comment applied above `iterations_set`, with a pointer comment on `with_iterations`. | — |
| D3 | Scope containment | — | 🟢 | Changes confined to `bezier.rs`'s 2 setters (1 `.max(1)` each); confirmed via grep that all 24 `impl_easing_function!` call sites pass `8`, so no other file needed changes. | — |

**Reproduced:** YES -- temporarily removed the `.max( 1 )` floor from both setters;
`test_cubic_iterations_floored_at_one_via_with_iterations` and
`test_cubic_iterations_floored_at_one_via_iterations_set` both failed alongside 1 unrelated
BUG-501 test also mid-revert (nextest fail-fast stopped further tests, all 3 confirmed as
expected failures). Restored the fix; full crate suite (49/49) passes with 0 warnings.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/easing/cubic/bezier.rs` | `iterations_set`/`with_iterations` now floor their argument at `1` via `.max( 1 )`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/easing_test.rs` | Added `test_cubic_iterations_floored_at_one_via_with_iterations` and `test_cubic_iterations_floored_at_one_via_iterations_set`; added `CubicBezier` to the file's existing `bezier` import block. |
