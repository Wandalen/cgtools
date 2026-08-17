# BUG-141: `CubicBezier::apply`'s Newton-Raphson `slope` is not `x_get`'s actual derivative

- **Severity:** Medium (silently inaccurate eased values for named curves at the crate's fixed
  8-iteration solve budget — not a crash, magnitude varies by curve and target `time`)
- **state:** Completed
- **Affects:** All 24 `CubicBezier`-based named easing curves (`EaseInSine` … `EaseInOutBack`) at
  mid-curve `time` values away from `0.0`/`1.0`/`0.5`-on-symmetric-curves
- **Component:** `module/helper/animation` (`src/easing/cubic/bezier.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Fourth bug filed for `animation` this session; independent of BUG-138/139/140
  (different file, different mathematical mechanism — a wrong derivative in a root-solve, not an
  accumulator or formula-shape defect).

## Symptom

```rust
use animation::easing::{ base::{ EasingFunction, EasingBuilder }, cubic::bezier::EaseInExpo };

let value = EaseInExpo::build().apply( 0.0_f32, 1.0_f32, 0.9 );
// Wrong (pre-fix):   value ≈ 0.5472
// Correct (post-fix): value ≈ 0.5056   (diff ≈ 0.0416 -- ~4% of the full [0,1] output range)
```

## Impact

**Who is affected:** Any caller using one of the 24 named `CubicBezier`-based easing curves
(`EaseInSine` … `EaseInOutBack`, everything in `easing/cubic/bezier.rs` except `Linear`/`Step`)
at a mid-curve `time` value.

**What breaks:** `apply`'s Newton-Raphson loop solves `x_get(bezier_t) == time` for `bezier_t`,
using `bezier_t -= x_val / slope` where `slope` is meant to approximate `x_get`'s derivative.
The code computed each of `x_get`'s three additive terms' derivative as if the terms were
independent constants being scaled, instead of applying the product rule — algebraically NOT
equal to `x_get`'s real derivative (confirmed both by direct differentiation of `x_get` and by
expanding the standard cubic-Bezier tangent formula `3(1-t)²(P1-P0) + 6(1-t)t(P2-P1) +
3t²(P3-P2)`; both derivations agree with each other and disagree with the code).

**Magnitude:** Newton-Raphson using a valid-but-wrong derivative estimate does NOT fail to
converge — it still converges to the true root of `x_get(t) = time` (the root is defined by
`x_val == 0`, independent of which slope was used to step toward it), just more slowly (linear
rather than quadratic convergence). The bug is entirely a symptom of the crate's **fixed**
8-iteration budget (no convergence check, no early exit on small `x_val`): 8 iterations of the
wrong slope is nowhere near converged for some curve/time combinations, while 8 iterations of the
correct slope already matches a 200-iteration bisection ground truth to 6 decimal places (i.e.
the correct derivative gives Newton-Raphson its expected fast quadratic convergence; the wrong
one doesn't). Numerically verified worst case: `EaseInExpo` at `time = 0.9`, error ≈ 0.0416.

**Entity Scope:** None — a code-level numeric defect, not an operational-entity concern.

## How Discovered

Task #74's targeted code review of `module/helper/animation`. An `Explore` subagent flagged that
`slope`'s formula didn't match a hand-derived expectation for `x_get`'s derivative; confirmed
independently by (1) direct term-by-term differentiation of `x_get` via the product rule, (2)
cross-checking against the standard cubic-Bezier tangent formula, and (3) a numeric Python
verification (bisection ground truth vs. both slope formulas at fixed 8 iterations) across 5
named curves and 19 `time` samples each, isolating `EaseInExpo`/`time=0.9` as the worst case.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test easing_test test_cubic_newton_raphson_slope_matches_true_derivative 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_cubic_newton_raphson_slope_matches_true_derivative ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `slope`'s three terms to the buggy
independent-scaling formula, then restoring the fix immediately after capturing the failure):
```
thread 'tests::test_cubic_newton_raphson_slope_matches_true_derivative' panicked at module/helper/animation/tests/easing_test.rs:53:5:
assertion failed: second - eps < first && first < second + eps
```
(the shared `assert_f_eq` helper's own `assert!` — no literal values printed; independently
confirmed via a Python re-implementation that the pre-fix output is ≈0.5472 against an expected
≈0.5056, error ≈0.0416, well outside the test's `eps = 0.001`.)

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test easing_test test_cubic_newton_raphson_slope_matches_true_derivative
# 1 passed = fixed; 1 failed (assert_f_eq panic) = bug present
```

**Known MRE limitation (check 205):** none — `CubicBezier<T>` is pure, synchronous,
dependency-free numeric code; runs as an ordinary native `cargo test` against the real crate.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `slope` is not `x_get`'s true derivative — the three terms' derivatives were computed as if independent, missing the product-rule cross terms. | ✅ Root Cause | Direct differentiation of `x_get` via the product rule, and independently the standard cubic-Bezier tangent formula, both yield `3(1-t)²(P1-P0) + 6(1-t)t(P2-P1) + 3t²(P3-P2)` — algebraically different from the code's `3(1-t)²P1 + 6(1-t)t·P2 + 3t²`, missing `-6(1-t)t·P1 - 3t²·P2`. | E1 |
| H2 | The wrong derivative makes Newton-Raphson diverge or oscillate (never reaches the correct answer regardless of iteration count). | ❌ Falsified | Re-ran the wrong-slope iteration up to 500 steps for the worst-case curve/time — it converges to the same root a 200-iteration bisection finds, just far slower (8 iters: err 0.0416; 20 iters: err 0.0034; 50 iters: err 0.000007; 100+ iters: err 0.0). | E2 |
| H3 | The bug is visible at every curve and every `time`, including the pre-existing pinned reference test (`test_cubic_mid_curve_accuracy`). | ❌ Falsified | That test's own two pinned values (`EaseInSine`@0.5, `EaseOutQuad`@0.5) diverge from true/correct-slope values by only 0.000038 and 0.000534 respectively — both still inside that test's `eps = 0.001`, which is why it kept passing throughout this bug's lifetime. The defect is curve/time-dependent, not universal. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/easing/cubic/bezier.rs`, pre-fix `apply`'s `slope` vs. hand-derived `x_get'(t)` | Term-by-term product-rule differentiation of `x_get`'s three summands, and independently the standard `3(1-t)²(P1-P0)+6(1-t)t(P2-P1)+3t²(P3-P2)` formula, both expand to `3A-12At+9At²+6Bt-9Bt²+3t²` (A=in_tangent[0], B=out_tangent[0]) — the pre-fix `slope` expands to `3A-6At+3At²+6Bt-6Bt²+3t²`, missing `-6At+6At²-3Bt²` net (i.e. missing `-6At(1-t)-3Bt²`). | H1 ✅ |
| E2 | Python re-implementation, `EaseInExpo` (A=0.7,B=0.84) at `time=0.9`, wrong-slope Newton-Raphson at 8/20/50/100/500 iterations vs. 200-iteration bisection ground truth (0.505609) | err: 0.041598 / 0.003411 / 0.000007 / 0.000000 / 0.000000 — monotonically converging, never diverging or oscillating. | H2 ❌ |
| E3 | `tests/easing_test.rs`, pre-fix `test_cubic_mid_curve_accuracy`'s two pinned values (`0.300_338`, `0.749_269`) vs. 200-iteration-bisection ground truth (`0.300376`, `0.748735`) | Both pinned values are themselves ≈8-iteration wrong-slope outputs from when that test was authored (TASK-041), not independently re-derived ground truth — but their divergence from truth (0.000038, 0.000534) happens to sit inside that test's own `eps = 0.001`, so it never caught this bug. Left untouched: still correctly passing, unrelated to this fix. | H3 ❌ |

## Root Cause

```
x_get(t) = 3*(1-t)^2*t*P1 + 3*(1-t)*t^2*P2 + t^3      (P0=0, P3=1 baked in)

True dx/dt (product rule, or standard Bezier tangent formula):
  3*(1-t)^2*(P1-P0) + 6*(1-t)*t*(P2-P1) + 3*t^2*(P3-P2)
  = 3*(1-t)^2*P1 + 6*(1-t)*t*(P2-P1) + 3*t^2*(1-P2)

Pre-fix `slope` (WRONG -- each term's coefficient copied as-is, no product-rule cross terms):
  3*(1-t)^2*P1 + 6*(1-t)*t*P2 + 3*t^2
```

The pre-fix formula looks structurally similar to the correct one (same `3(1-t)²`/`6(1-t)t`/`3t²`
weights) but drops the `(P2-P1)` and `(1-P2)` factors in favor of bare `P2`/`1`, silently
supplying a different (but still positive, still roughly-in-the-right-direction) slope estimate.

## Why Not Caught

The only existing mid-curve accuracy test (`test_cubic_mid_curve_accuracy`, from TASK-041) pins
two curve/time combinations whose wrong-vs-correct divergence happens to be smaller than its own
`eps = 0.001` tolerance — not because the bug doesn't apply there, but because the error magnitude
is curve/time-dependent and those two particular samples are near the low end of it.
`test_cubic_boundaries_and_properties` only checks `t = 0.0`/`t = 1.0`, where `apply`'s
early-return guards bypass the solve loop (and thus `slope`) entirely.

## Fix Location

`module/helper/animation/src/easing/cubic/bezier.rs`, `CubicBezier::apply`'s `slope` computation
inside the Newton-Raphson loop:

```rust
// before
let slope = 3.0 * ( 1.0 - bezier_t ).powi( 2 ) * self.in_tangent[ 0 ]
+ 6.0 * ( 1.0 - bezier_t ) * bezier_t * self.out_tangent[ 0 ]
+ 3.0 * bezier_t.powi( 2 );

// after
let slope = 3.0 * ( 1.0 - bezier_t ).powi( 2 ) * self.in_tangent[ 0 ]
+ 6.0 * ( 1.0 - bezier_t ) * bezier_t * ( self.out_tangent[ 0 ] - self.in_tangent[ 0 ] )
+ 3.0 * bezier_t.powi( 2 ) * ( 1.0 - self.out_tangent[ 0 ] );
```

No signature change — pure internal-logic fix, now the algebraically correct derivative of
`x_get` with respect to `bezier_t`.

## Prevention

Added `test_cubic_newton_raphson_slope_matches_true_derivative` to `tests/easing_test.rs`, pinning
`EaseInExpo::build().apply(0.0, 1.0, 0.9)` — the numerically-verified worst-case curve/time
combination — against a value independently derived via 200-iteration bisection (a
derivative-free method, structurally immune to this class of bug).

**Pitfall:** the existing `test_cubic_mid_curve_accuracy` test's own pinned reference values were
themselves generated using the buggy slope (per its Pitfall note claiming "100 Newton-Raphson
iterations" — true, but with the wrong derivative), and happened to land within its tolerance
band anyway. A derivative bug in a fixed-iteration numeric solver needs either a
derivative-free ground truth (bisection, as used here) or a very large iteration count to expose
reliably — a handful of arbitrary curve/time samples is not guaranteed to catch it.

## Generalized Version

**Broken assumption:** "if a Newton-Raphson loop's numeric output still passes existing spot-check
tests, its derivative computation must be correct." False — a plausible-but-wrong derivative
doesn't fail to converge (so the code visibly "works" for many inputs), it converges *slower*,
and a fixed (not adaptive) iteration count silently trades accuracy for whatever the wrong
derivative's convergence rate happens to deliver at that budget, for that specific input.

**Confirmed general rule:** when reviewing a numeric root-solver, re-derive the slope/derivative
expression independently (by hand or via a symbolic/numeric cross-check) rather than trusting
that it "looks like" the expected shape — and validate against a derivative-free ground truth
(bisection, golden-section search) rather than only against the solver's own prior output at a
different iteration count, since both would share the same systematic bias.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #74's targeted code review of `module/helper/animation`; confirmed by independent product-rule differentiation of `x_get`, cross-checked against the standard cubic-Bezier tangent formula, and numerically verified via a Python bisection-ground-truth comparison across 5 curves and 19 `time` samples each. |
| 2026-08-16 | fixed | Corrected `slope` to the true derivative of `x_get`. |
| 2026-08-16 | verified | Added `test_cubic_newton_raphson_slope_matches_true_derivative`; confirmed it fails against the reverted pre-fix logic with the exact predicted `assert_f_eq` panic and passes against the fix; full crate suite (39 tests incl. 6 doctests) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `CubicBezier::apply`'s Newton-Raphson `slope` (confirmed the product-rule-corrected expression with `( self.out_tangent[0] - self.in_tangent[0] )` and `( 1.0 - self.out_tangent[0] )` factors genuinely present, `Fix(BUG-141)` comment intact) and `test_cubic_newton_raphson_slope_matches_true_derivative` (non-tautological: pins `EaseInExpo::build().apply(0.0,1.0,0.9)` against an independently bisection-derived value, `eps = 0.001`). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass relied on hand-derived calculus; adversarial pass demanded independent numeric proof — built a Python bisection ground truth across multiple curves/times before trusting the hand derivation, then closed the loop with revert-test-restore against the real Rust test. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-138/139/140 (different file, different defect class); explicitly checked whether the pre-existing `test_cubic_mid_curve_accuracy` test's pinned values needed correcting too and confirmed they don't (still within their own tolerance) — left untouched rather than scope-creeping into an unrelated test's precision. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass specifically hunted for "does the wrong slope actually cause non-convergence, or just slower convergence" (H2) via a 500-iteration numeric check, rather than assuming the first plausible explanation. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Only the one `slope` expression changed; verified via direct read that `x_get`/`y_get` themselves are correct and unrelated to this defect. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` `src/easing/cubic/bezier.rs` + `tests/easing_test.rs` + this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one expression inside one loop body. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added — existing "solve for bezier_t" contract now actually converges at its documented iteration budget. | — |

**Reproduced:** YES — temporarily reverting the fixed `slope` expression back to the buggy
independent-scaling formula and running
`cargo test --test easing_test test_cubic_newton_raphson_slope_matches_true_derivative` produced
the exact predicted `assert_f_eq` panic at `easing_test.rs:53:5`; restoring the fix returned the
full suite (39 tests incl. doctests) to passing plus a clean
`cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/easing/cubic/bezier.rs` | `CubicBezier::apply`'s Newton-Raphson `slope`: corrected to the true derivative of `x_get`. `Fix(BUG-141)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/easing_test.rs` | New test (`bug_reproducer(BUG-141)`, 5-section doc comment) — `test_cubic_newton_raphson_slope_matches_true_derivative`. |
