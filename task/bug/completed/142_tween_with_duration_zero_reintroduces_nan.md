# BUG-142: `Tween::with_duration(0.0)` reintroduces the exact division-by-zero `new` guards against

- **Severity:** High (produces `NaN`, which silently poisons any downstream position/transform
  computation rather than failing loudly)
- **state:** Completed
- **Affects:** Any `Tween<T>` built via `Tween::new(...).with_duration(0.0)` (or any value
  `< 0.001`)
- **Component:** `module/helper/animation` (`src/interpolation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Fifth and final bug filed for `animation` this session (closes out task #74's
  review). Independent of BUG-138/139/140/141 — a builder method failing to preserve a sibling
  constructor's own documented invariant, not a formula or convergence defect.

## Symptom

```rust
use animation::{ Tween, easing::base::{ EasingBuilder }, easing::Linear };

let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() ).with_duration( 0.0 );
let value = tween.update( 0.1 );
// Wrong (pre-fix):   value.is_nan() == true
// Correct (post-fix): value is finite (duration silently floored to 0.001, same as `new` does)
```

## Impact

**Who is affected:** Any caller that constructs a `Tween` and then calls `.with_duration(0.0)` —
or any value below the `0.001` floor `Tween::new` itself enforces — whether directly or via a
data-driven/deserialized duration that happens to be `0.0`.

**What breaks:** `Tween::new` explicitly clamps its `duration` argument with
`duration.max( 0.001 )`, commented "Minimum duration to avoid division by zero" — `value_get`'s
`self.elapsed / self.duration` and `progress`'s equivalent (BUG-140) both divide by it. But the
`with_duration` builder method, which mutates `self.duration` after construction, clamped with
only `duration.max( 0.0 )` — no floor against zero at all. `with_duration(0.0)` therefore sets
`self.duration = 0.0` directly, undoing `new`'s own protection. On the very next `update()` call,
`self.elapsed` becomes some positive value `>= self.duration (0.0)`, completing the tween
immediately; `value_get`'s `self.elapsed / self.duration` then computes `0.0 / 0.0` (`elapsed` is
reset to `self.duration == 0.0` on completion) — IEEE 754 `NaN`. `f64::clamp` passes `NaN` through
unchanged (neither `<` nor `>` comparison is true for `NaN`), and the easing function's
`interpolate` call propagates it into the returned value with no panic and no error signal.

**Magnitude:** Not a crash — silent `NaN` propagation into whatever the tween drives (position,
scale, opacity, etc.), which then poisons every downstream computation that touches it (a `NaN`
position typically renders as invisible/off-screen with no error message).

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #74's targeted code review of `module/helper/animation`. An `Explore` subagent flagged that
`with_duration`'s clamp used `0.0` where `new`'s sibling clamp uses `0.001` for the identical
field, with the same documented "avoid division by zero" purpose; confirmed by direct trace of
`update`'s completion path and `value_get`'s division, and by checking `with_duration` had zero
existing test coverage of any kind.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test interpolation_test test_tween_with_duration_zero_does_not_produce_nan 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_tween_with_duration_zero_does_not_produce_nan ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `with_duration`'s clamp back to
`duration.max( 0.0 )`, then restoring the fix immediately after capturing the failure):
```
thread 'tests::test_tween_with_duration_zero_does_not_produce_nan' panicked at module/helper/animation/tests/interpolation_test.rs:151:5:
Tween::update produced NaN after with_duration( 0.0 )
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test interpolation_test test_tween_with_duration_zero_does_not_produce_nan
# 1 passed = fixed; 1 failed (NaN produced) = bug present
```

**Known MRE limitation (check 205):** none — `Tween<T>` is pure, synchronous, dependency-free
state; runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `with_duration`'s `.max( 0.0 )` clamp fails to preserve `new`'s `.max( 0.001 )` division-by-zero floor for the same field. | ✅ Root Cause | Direct read of both methods: `new` clamps `duration.max( 0.001 )` with an explicit "avoid division by zero" comment; `with_duration` clamps the identical field with `duration.max( 0.0 )` — a real floor (non-negative) but not THE floor the field's own invariant requires. | E1 |
| H2 | The resulting `0.0 / 0.0` actually panics (integer-style division-by-zero) rather than silently producing `NaN`. | ❌ Falsified | `f64` division by `0.0` is IEEE 754-defined (`NaN`, no panic) in Rust, unlike integer division; confirmed via the regression test observing `value.is_nan() == true` with no panic from the division itself — the only panic is the test's own explicit assertion. | E2 |
| H3 | `f64::clamp` normalizes `NaN` back to a boundary value (`0.0` or `1.0`), masking the issue before it reaches the easing function. | ❌ Falsified | `clamp`'s implementation (`if self < min { min } else if self > max { max } else { self }`) returns `self` unchanged for `NaN`, since both comparisons are false for `NaN` per IEEE 754 — `NaN` passes through `.clamp( 0.0, 1.0 )` untouched. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/interpolation.rs`, `Tween::new` (duration clamp, `.max( 0.001 )`, "Minimum duration to avoid division by zero") vs. pre-fix `with_duration` (`.max( 0.0 )`) | Same field, same division-by-zero risk documented at construction time, different (insufficient) clamp value at the builder site. | H1 ✅ |
| E2 | `tests/interpolation_test.rs`, new regression test | `Tween::new(...).with_duration(0.0)` then `.update(0.1)` returns a value where `.is_nan()` is `true`, no panic from the arithmetic itself. | H2 ❌ |
| E3 | `value_get`'s `( self.elapsed / self.duration ).clamp( 0.0, 1.0 )` | With `self.duration == 0.0` and `self.elapsed == 0.0` (post-completion reset), this divides `0.0 / 0.0`, and the resulting `NaN` survives `.clamp(...)` unchanged before being passed into `self.easing.apply(...)`. | H3 ❌ |

## Root Cause

```
Tween::new( ... ):
  self.duration = duration.max( 0.001 );   // "Minimum duration to avoid division by zero"

Tween::with_duration( ... ):   (pre-fix)
  self.duration = duration.max( 0.0 );     // BUG: doesn't reach new's own floor

update() on a with_duration(0.0) tween:
  self.elapsed += remaining_time;               // some positive value
  self.elapsed >= self.duration (0.0) -> true    // completes immediately
  repeat_count == 0 -> state = Completed; self.elapsed = self.duration; // both 0.0

value_get():
  normalized_time = ( self.elapsed / self.duration ).clamp( 0.0, 1.0 )
                  = ( 0.0 / 0.0 ).clamp( 0.0, 1.0 )
                  = NaN.clamp( 0.0, 1.0 )        // NaN passes through clamp unchanged
  self.easing.apply( start, end, NaN )           // propagates NaN into the returned value
```

`with_duration` re-derives `new`'s own "avoid division by zero" invariant but copies only the
clamp's polarity (`.max`), not its actual floor value (`0.001`) — `0.0` still satisfies
"non-negative" while reintroducing the exact division-by-zero the invariant exists to prevent.

## Why Not Caught

`with_duration` had zero existing test coverage of any kind prior to this bug — not just the
zero-duration edge case, but no test exercised the method at all.

## Fix Location

`module/helper/animation/src/interpolation.rs`, `Tween::with_duration`:

```rust
// before
self.duration = duration.max( 0.0 );

// after
self.duration = duration.max( 0.001 );
```

No signature change — pure internal-logic fix, now matching `new`'s own floor exactly.

## Prevention

Added `test_tween_with_duration_zero_does_not_produce_nan` to `tests/interpolation_test.rs`,
driving a `with_duration(0.0)` tween through one `update()` and asserting the result is finite.

**Pitfall:** invisible unless a caller specifically constructs a zero-duration tween via the
builder method rather than `new` directly (`new(0.0_dur, ...)` was already correctly floored) —
the bug is entirely in the builder's failure to preserve a sibling constructor's invariant, not in
the underlying division logic itself, which was already correct given a properly-floored
`duration`.

## Generalized Version

**Broken assumption:** "a builder method (`with_X`) that re-sets a field already validated at
construction time only needs to repeat the validation's general shape (e.g. `.max(...)`, `>= 0`),
not its specific value." False when the original clamp's value encodes a concrete invariant (here,
"never divide by this field") rather than a generic sanity bound — copying the shape without the
value silently reintroduces the exact failure mode the original clamp existed to prevent.

**Confirmed general rule:** when a field has a constructor-enforced floor/ceiling with a stated
reason (a comment, a doc line), every other mutator of that same field — builder methods, setters,
deserializers — must enforce the identical bound, not merely a bound in the same direction. Grep
for every mutation site of a field once its constructor comment reveals a non-obvious invariant.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered via task #74's targeted code review of `module/helper/animation`; confirmed by direct trace of `update`'s completion path and `value_get`'s division, and by noting `with_duration` had zero prior test coverage. |
| 2026-08-16 | fixed | Changed `with_duration`'s clamp from `.max( 0.0 )` to `.max( 0.001 )`, matching `new`. |
| 2026-08-16 | verified | Added `test_tween_with_duration_zero_does_not_produce_nan`; confirmed it fails against the reverted pre-fix logic with the exact predicted NaN assertion panic and passes against the fix; full crate suite (40 tests incl. 6 doctests) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `Tween::with_duration` (confirmed `duration.max( 0.001 )` genuinely present, matching `new`'s own floor, `Fix(BUG-142)` comment intact) and `test_tween_with_duration_zero_does_not_produce_nan` (non-tautological: asserts `!value.is_nan()` after `.with_duration(0.0)` and one `update()`). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass traced IEEE 754 `0.0/0.0` and `clamp`'s NaN-passthrough semantics directly; adversarial pass verified both claims independently (H2/H3) before trusting them, then closed via revert-test-restore with the real Rust test. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-138/139/140/141; final bug of this crate's review, cross-referenced against all four prior `animation` bugs to confirm no shared root cause. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass specifically checked whether `f64` division by zero panics (it doesn't) and whether `clamp` masks `NaN` (it doesn't) before accepting the propagation chain as real. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `with_duration` call site (none exist yet in this workspace) and every division by `self.duration` (`value_get`, `repeat_handle`, `progress`) to confirm the floor fix protects all three, not just the one exercised by the test. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one clamp's literal value. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing "avoid division by zero" contract now actually enforced at every mutation site of the field. | — |

**Reproduced:** YES — temporarily reverting the fixed clamp back to `duration.max( 0.0 )` and
running
`cargo test --test interpolation_test test_tween_with_duration_zero_does_not_produce_nan`
produced the exact predicted NaN-assertion panic at `interpolation_test.rs:151:5`; restoring the
fix returned the full suite (40 tests incl. doctests) to passing plus a clean
`cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `Tween::with_duration`: changed `duration.max( 0.0 )` to `duration.max( 0.001 )`, matching `new`. `Fix(BUG-142)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | New test (`bug_reproducer(BUG-142)`, 5-section doc comment) — `test_tween_with_duration_zero_does_not_produce_nan`. |
