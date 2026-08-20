# BUG-233: `Step::new(0.0)`'s divide-by-zero silently produces `NaN` instead of erroring

- **Severity:** Medium (no crash, no panic -- `f64` division never panics -- but every
  interpolated value from an affected `Step` easing instance silently becomes `NaN`, which then
  propagates into whatever consumes it with no diagnostic at the source)
- **state:** Completed
- **Affects:** Any `Step::new( steps )` call with `steps <= 0.0` (most notably the exact
  boundary `0.0`, but any non-positive value reaches the same divide-by-zero-shaped defect).
- **Component:** `module/helper/animation` (`src/easing/base.rs`, `Step::new`/`Step::apply`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** None directly, but mirrors the already-fixed BUG-142
  (`Tween::with_duration(0.0)` reintroducing the exact division-by-zero `Tween::new` guards
  against) -- same defect shape (an unguarded `f64` constructor parameter later used as a
  division's divisor), different struct.

## Symptom

```rust
// pre-fix
pub fn new( steps : f64 ) -> Self
{
  Self { steps, _marker : PhantomData }   // no floor
}

fn apply( &self, start : A, end : A, time : f64 ) -> A
{
  let time = ( time * self.steps ).ceil() / self.steps;   // divides by self.steps directly
  start.interpolate( &end, time )
}
```

`Step::new( 0.0 ).apply( 0.0, 1.0, 0.5 )` computes `( 0.5 * 0.0 ).ceil() / 0.0` = `0.0 / 0.0` =
`NaN`. Rust's `f64` division never panics on a zero divisor -- it returns `NaN` (for `0.0/0.0`)
or `±inf` (for a nonzero numerator) -- so this reaches `start.interpolate(&end, NaN)` silently,
with no error and no diagnostic.

## Impact

**Who is affected:** Any caller constructing `Step::new` with a non-positive argument, whether
a literal `0.0`, a computed value that evaluates to `0.0`/negative, or a default/uninitialized
value.

**What breaks:** Every subsequent `apply()` call on that `Step` instance returns `NaN` (or, for
some `Animatable` types, an equally-corrupted value derived from `NaN`), silently propagating
into whatever consumes the interpolated value -- a rendered position, color, or any other
animated property -- with nothing at the source signaling the actual cause.

**Magnitude:** 1 constructor (`Step::new`), 1 missing floor.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `animation`'s `easing/base.rs`, reading `Step::new` and
`Step::apply` in full and comparing `Step::new`'s unguarded `steps` parameter against
`Tween::new`'s existing `duration.max( 0.001 )` floor for the structurally identical
divisor-of-a-division defect shape (both `f64` constructor parameters later divided into).

## Minimum Reproducible Example

```rust
let step_func = animation::easing::base::Step::< f64 >::new( 0.0 );
let value = step_func.apply( 0.0_f64, 1.0_f64, 0.5 );
assert!( !value.is_nan() ); // pre-fix: fails, value IS NaN
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/animation && cargo nextest run --all-features -E 'test(test_step_function_zero_steps_does_not_produce_nan)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Step::new` stores its `steps` argument with no floor, and `apply` divides by it directly, so `Step::new( 0.0 )` produces a `0.0` divisor and `apply` silently returns `NaN` instead of erroring. | ✅ Root Cause | Direct read of pre-fix `Step::new`/`Step::apply` shows the unguarded parameter and the direct division; confirmed empirically via temporary-revert-and-rerun (`Step::new( 0.0 )` produced NaN, test failed as predicted). | E1, E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/animation/src/easing/base.rs`, `Step::new` (pre-fix, direct read) | `Self { steps, _marker : PhantomData }` -- the argument is stored exactly as given, no clamp or validation. | H1 ✅ |
| E2 | `module/helper/animation/src/easing/base.rs`, `Step::apply` (direct read) | `( time * self.steps ).ceil() / self.steps` -- `self.steps` is the divisor of the final division with no zero-check anywhere in the method. | H1 ✅ |
| E3 | `module/helper/animation/src/interpolation.rs` line 120 (direct read) | `Tween::new`'s sibling constructor already guards the identical defect shape: `duration : duration.max( 0.001 ), // Minimum duration to avoid division by zero` -- confirms this codebase's own established convention for exactly this class of defect, which `Step::new` had not adopted. | H1 ✅ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting `Step::new`'s floor back to storing `steps` unguarded reproduced `Step::new( 0.0 ) produced NaN instead of a finite value` on the new test. | H1 ✅ |

## Root Cause

`Step::new` accepted an `f64` `steps` parameter and stored it unchanged, while `Step::apply`
later divides by that same stored value (`... / self.steps`) to normalize the ceiling-quantized
time back into `[0.0, 1.0]`. Rust's floating-point division never panics on a zero divisor --
`0.0 / 0.0` evaluates to `NaN` per IEEE 754 semantics -- so a `steps` value of exactly `0.0` (or
any non-positive value producing a degenerate divisor) silently corrupted every subsequent
`apply()` result instead of erroring at construction, where the invalid input actually
originated.

## Why Not Caught

The only existing `Step` test, `test_step_function`, exercises `Step::new( 5.0 )` exclusively --
no test ever constructed a `Step` with `0.0` or a negative value.

## Fix Location

`module/helper/animation/src/easing/base.rs`: `Step::new` now floors its argument with
`steps.max( 0.001 )`, mirroring `Tween::new`'s own `duration.max( 0.001 )` guard against the
identical division-by-zero shape.

## Prevention

`tests/easing_test.rs::test_step_function_zero_steps_does_not_produce_nan` constructs
`Step::new( 0.0 )` and asserts `apply()` returns a finite, non-`NaN` value.

## Pitfall

Rust's `f64` division never panics on a zero divisor -- it silently returns `NaN` or `±inf` --
so there is no language-level safety net that will surface a missing floor on a constructor
parameter that later becomes a division's divisor. Every such parameter needs its own explicit
guard; the mere presence of one sibling type's guard (`Tween::new`) does not protect a different
type (`Step::new`) with the identical shape.

## Generalized Version

**Broken assumption:** "an obviously-invalid constructor argument like `0.0` will surface as a
panic or an error somewhere downstream, so it doesn't need its own guard at the constructor."

**Confirmed general rule:** Any `f64`/`f32` constructor parameter that is later used as a
division's divisor must be floored away from `0.0` (and, if negative isn't meaningful either,
away from the entire non-positive range) at construction time -- floating-point division's
silent `NaN`/`±inf` semantics mean nothing else in the language will ever surface the mistake.
When a codebase has already established this guard for one type (here, `Tween::new`), audit
every sibling type with the same divisor-of-a-division shape rather than assuming the pattern
was applied everywhere it's needed.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `animation` scouting pass, comparing `Step::new`'s unguarded `steps` parameter against `Tween::new`'s existing `duration.max( 0.001 )` floor for the identical division-by-zero defect shape. |
| 2026-08-17 | fixed | `Step::new` now floors `steps` with `.max( 0.001 )`, mirroring `Tween::new`'s convention. |
| 2026-08-17 | verified | `cargo nextest run -p animation --all-features`: 43/43 passed, 0 skipped. `cargo test --doc -p animation --all-features`: 3/3 passed. `cargo clippy -p animation --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (NaN produced pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, `is_nan()` is an exact, non-flaky check. Adversarial pass: considered whether `0.001` as a floor could itself produce a surprising near-zero-but-nonzero result for `apply()` at `steps ≈ 0.001` -- confirmed this only affects the pathological near-zero input range already being guarded against, not any previously-valid input (mirrors `Tween::new`'s own accepted tradeoff). | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified and cited BUG-142 as the same-shape, different-struct sibling defect and precedent for the `.max( 0.001 )` convention. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `Step::new`, `Step::apply`, and `Tween::new`'s existing guard, plus empirical revert-rerun proof. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to `Step::new`'s single field assignment. Adversarial pass: grepped `easing/base.rs` and confirmed `Step` has no `with_steps`-style setter (unlike `Tween::with_duration`) needing a matching guard -- `new` is the only construction path. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `Step::new`; `Step`'s public signature is unchanged (still takes `f64`, returns `Self`), so no downstream caller needed updating. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with a NaN value, pass post-fix) and
temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/easing/base.rs` | `Step::new` now floors `steps` with `steps.max( 0.001 )` (full `Fix(BUG-233)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/easing_test.rs` | Added `test_step_function_zero_steps_does_not_produce_nan` (`bug_reproducer(BUG-233)`, 5-section doc comment), placed directly after `test_step_function`. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects (see BUG-230/231/232's own Refs: docs/ precedent). |
