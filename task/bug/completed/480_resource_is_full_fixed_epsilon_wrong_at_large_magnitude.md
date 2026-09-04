# BUG-480: `Resource::is_full`'s fixed epsilon tolerance is meaningless at large resource magnitudes

- **Severity:** Low (only manifests for resources with large `maximum` values, where floating
  point precision loss near the maximum naturally exceeds a fixed absolute tolerance)
- **state:** Completed
- **Affects:** Any consumer of `Resource::is_full` where `maximum` is large enough that
  `f32` precision near that magnitude exceeds a fixed small epsilon (e.g. `maximum` in the
  hundreds of thousands or more).
- **Component:** module/helper/tiles_tools (`src/game_systems.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-349 (a different `Resource` defect -- negative `maximum` panic, already
  fixed, unrelated mechanism).

## Symptom

```rust
// pre-fix -- src/game_systems.rs
pub fn is_full(&self) -> bool {
  (self.current - self.maximum).abs() < f32::EPSILON
}
```

`f32::EPSILON` (~1.19e-7) is the smallest representable difference near `1.0` -- it is not a
meaningful tolerance near arbitrary magnitudes. For a `Resource` with `maximum = 1_000_000.0`,
`f32`'s representable precision near that value is roughly `0.0625` (the ULP at that magnitude),
far coarser than `f32::EPSILON` -- so a `current` value that is, for all practical floating-point
purposes, already at the maximum (e.g. `maximum - 0.06`) could still fail the `< f32::EPSILON`
check and report `is_full() == false`.

## Impact

**Who is affected:** Any consumer of `Resource` with a large `maximum` (health/mana/currency
pools scaled into the hundreds of thousands or beyond) relying on `is_full()` to detect the
resource has reached its cap.

**What breaks:** `is_full()` under-reports fullness for large-magnitude resources -- a resource
that is, within `f32`'s own representable precision at that magnitude, at its maximum, could
still report `false`.

**Consumer audit:** `is_full` is a public method; `grep -rln 'is_full' --include="*.rs" .` from
the repo root, excluding `tiles_tools` itself, returns no external call sites -- confirmed via
direct audit.

**Magnitude:** Single comparison expression; see Fix Location.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/game_systems.rs` end to end -- a fixed
`f32::EPSILON` tolerance in a comparison against values of arbitrary, caller-controlled
magnitude is a well-known floating-point anti-pattern.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/game_systems_test.rs
let mut resource = Resource::new(1_000_000.0);
resource.current = resource.maximum - 0.06;
assert!(resource.is_full());
// pre-fix: fails -- 0.06 > f32::EPSILON (~1.19e-7), so is_full() incorrectly returns false
// despite `current` being within f32's own representable precision of `maximum` at this magnitude
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(game_systems_test) and test(is_full_uses_magnitude_scaled_tolerance)'
```

## Root Cause

`f32::EPSILON` is a constant scaled for comparisons near magnitude `1.0` -- using it as a fixed
absolute tolerance against a difference computed from values of arbitrary magnitude (`self.maximum`,
which is caller-controlled and can be arbitrarily large) does not scale with the actual
floating-point precision available at that magnitude. This is a textbook instance of the general
floating-point pitfall: comparing `abs(a - b) < EPSILON` without scaling `EPSILON` by the
magnitude of `a`/`b`.

## Why Not Caught

No existing test exercised `is_full` with a large `maximum` value -- all prior test fixtures used
small, `1.0`-magnitude-adjacent resource values, where `f32::EPSILON` happens to be a reasonable
tolerance, masking the defect.

## Fix Location

`module/helper/tiles_tools/src/game_systems.rs`: `is_full` changed to
`(self.current - self.maximum).abs() <= self.maximum.abs() * f32::EPSILON` -- a magnitude-scaled
tolerance (the standard fix for this class of floating-point comparison: scale the epsilon by
the magnitude of the values being compared, not a fixed absolute constant).

## Prevention

New test `test_resource_is_full_uses_magnitude_scaled_tolerance` in `tests/game_systems_test.rs`
constructs a `Resource` with `maximum = 1_000_000.0`, sets `current` to `maximum - 0.06` (within
`f32`'s own representable precision at that magnitude but far outside the old fixed
`f32::EPSILON`), and asserts `is_full()` now returns `true`.

## Pitfall

`f32::EPSILON`/`f64::EPSILON` are frequently reached for as "the" floating-point comparison
tolerance, but they are only meaningful near magnitude `1.0` -- using them as a fixed absolute
tolerance against values of arbitrary, especially large, magnitude silently under-tolerates
(too strict) exactly where floating-point precision is naturally coarser. The general fix is to
scale the tolerance by the magnitude of the values being compared, not to reach for a smaller or
larger fixed constant.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, reading `src/game_systems.rs` end to end. |
| 2026-08-20 | fixed | `is_full` changed to a magnitude-scaled tolerance (`self.maximum.abs() * f32::EPSILON`) instead of the fixed `f32::EPSILON`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: confirmed the test's `maximum - 0.06` offset genuinely exceeds the old fixed `f32::EPSILON` (~1.19e-7) by roughly 6 orders of magnitude, so the test fails against the pre-fix comparison and passes against the magnitude-scaled fix -- not a vacuous margin. | — |
| D2 | Small-magnitude behavior preserved | — | 🟢 | Confirmed the fix does not regress small-magnitude resources: for `maximum` near `1.0`, `self.maximum.abs() * f32::EPSILON` is approximately the old fixed `f32::EPSILON`, so existing small-value behavior is unchanged -- `cargo nextest run -p tiles_tools --all-features` (286/286 pass) confirms no existing test regressed. | — |

**Reproduced:** YES -- `test_resource_is_full_uses_magnitude_scaled_tolerance`'s assertion
(`resource.is_full()` after setting `current = maximum - 0.06`) is false against the pre-fix
fixed-`f32::EPSILON` comparison (verified by direct calculation: `0.06 > f32::EPSILON`) and true
against the fix. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/game_systems.rs` | `Resource::is_full` changed to a magnitude-scaled epsilon tolerance; `Fix(BUG-480)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/game_systems_test.rs` | Added `test_resource_is_full_uses_magnitude_scaled_tolerance`, exercising a large-magnitude `Resource` where the old fixed epsilon under-tolerates. |
