# Fix animation's CubicBezier default-iterations bug and CubicHermite silent truncation

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-09
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/animation
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-09
- **blocked_by:** null

## Goal

Fix 2 confirmed logic bugs found during the `animation` crate audit (2026-08-09), both confined to
`src/easing/cubic/`, bundled into one task since both are small and share a subsystem (matching
`TASK-015`'s own stated bundling rationale).

### Bugs (fixed, 2026-08-09)

1. **`CubicBezier::new` default iterations** (`src/easing/cubic/bezier.rs:63-73`) — `iterations`
   defaulted to `0`, which skips `apply`'s Newton-Raphson solve loop entirely: `y_get` was evaluated at
   the raw input time fraction instead of the solved Bezier parameter, silently producing the wrong
   easing shape for every one of the 24 named curves built through this constructor (`EaseInSine`,
   `EaseOutQuad`, etc. in the same file) that didn't separately call `set_iterations`. Boundary-only
   tests (`t = 0.0` / `1.0`) can't catch this since `apply`'s early-return guards bypass the solve loop
   at both boundaries regardless of `iterations` — only a mid-curve value exposes the wrong shape.
   Fixed: default changed to `iterations: 8`; all 24 `impl_easing_function!` invocations updated to
   chain `.with_iterations( 8 )` explicitly (new fluent builder method, alongside the pre-existing
   `&mut self` `set_iterations`). Fix documented with `Fix(TASK-041)`/`Root cause`/`Pitfall` source
   comments. Reproducer: `tests/easing_test.rs::test_cubic_mid_curve_accuracy` (mid-curve values for
   `EaseInSine`/`EaseOutQuad` against independently-solved reference values). This bug also silently
   made `tests/sequencer_test.rs::test_sequencer_ease_in`'s existing assertion wrong (it asserted the
   `iterations = 0` value, `1.25`); corrected to the real fixed-behavior value (`~3.00338`, epsilon
   comparison) in the same session.
2. **`CubicHermite<Vec<E>>` silent length truncation** (`src/easing/cubic/hermite.rs`) — both `new()`
   and `apply()` silently `.resize()`d mismatched-length vectors (`m1`/`m2` in the constructor;
   `start`/`end`/tangents in `apply`) down to the shortest length instead of surfacing the mismatch,
   discarding trailing data with no signal to the caller. `EasingFunction::apply`'s shared trait
   signature returns `Self::AnimatableType` directly (no `Result`) for every implementor, so a
   `Result`-based fix isn't possible without a much larger trait-wide change — fixed instead via loud
   `assert_eq!` panics (with `# Panics` doc sections) on both length-mismatch preconditions. Fix
   documented with `Fix(TASK-041)`/`Root cause`/`Pitfall` source comments. Reproducers:
   `tests/easing_test.rs::test_cubic_hermite_new_panics_on_mismatched_tangent_lengths` and
   `::test_cubic_hermite_apply_panics_on_mismatched_value_lengths`.

`cargo nextest run -p animation --all-features` confirms all tests pass (29/29), including the 3 new
reproducer tests above.

## Verification

### Checklist

- [x] C1 — Does `CubicBezier::new` default `iterations` to `8` (not `0`)? `bezier.rs:70` — `iterations : 8,`.
- [x] C2 — Do all 24 named curve constructors chain `.with_iterations( 8 )` explicitly? `grep -c "with_iterations( 8 )" bezier.rs` → `24`.
- [x] C3 — Does `CubicHermite` panic loudly (via `assert_eq!`) on mismatched lengths in both `new()` and `apply()`, instead of silently truncating? `hermite.rs` has 3× `assert_eq!` guards and 2× `# Panics` doc sections.
- [x] C4 — Do dedicated reproducer tests exist for both bugs? `tests/easing_test.rs` — `test_cubic_mid_curve_accuracy` (line 153), `test_cubic_hermite_new_panics_on_mismatched_tangent_lengths` (line 180), `test_cubic_hermite_apply_panics_on_mismatched_value_lengths` (line 203).
- [x] C5 — Was the pre-existing `test_sequencer_ease_in` assertion (which had baked in the old buggy `iterations=0` value) corrected to the real fixed-behavior value? `tests/sequencer_test.rs:195` — `assert_f_eq( f64::from( value.value_get() ), 3.00338, 0.001 );`, with a `Fix(TASK-041)` comment (lines 190-194) explaining the old `1.25` value's provenance.
- [x] C6 — Is each fix source-documented per the project's 3-field fix-comment convention? `grep -c "Fix(TASK-041)"` → `1` in `bezier.rs`, `2` in `hermite.rs`.

### Measurements

- [x] M1 — `animation` crate test count: `cargo nextest run -p animation --all-features` → `29 tests run: 29 passed, 0 skipped` (was: 26 tests pre-fix, before the 3 reproducer tests in C4 were added).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p animation --all-features` → exit 0, 29/29 passed.
- [x] I2 — Compiler/lints clean: `cargo clippy -p animation --all-targets --all-features -- -D warnings` → exit 0, zero warnings.

### Anti-faking checks

- [x] AF1 — Guards against the boundary-only-test blind spot recurring: `test_cubic_mid_curve_accuracy` (C4) asserts a **mid**-curve value specifically because `apply`'s `t = 0.0`/`1.0` early-return guards bypass the Newton-Raphson solve loop at both boundaries regardless of `iterations` — a boundary-only regression test would pass even if `iterations` silently reverted to `0`. Re-running `cargo nextest run -p animation --all-features` after any future edit to `bezier.rs`'s solve loop must still show this test passing, not merely present.

## History

- **[2026-08-09]** `FILED` — Filed from the same 2026-08-09 workspace audit re-verification pass as
  `TASK-015`'s bug list; split out into its own task since both bugs are confined to `easing/cubic/`
  specifically, distinct from `TASK-015`'s `Sequencer`/`Tween` scope.
- **[2026-08-09]** `RESOLVED` — Both bugs fixed in-place, source-documented, and covered by dedicated
  reproducer tests in the same session as filing. Verification performed as a Tier 2 Dual-Role
  Self-Check (same session, no independent dispatch) per
  `governance/maav.rulebook.md § MAAV : Verification Tier Selection`'s default — not an independent
  PROC16-style acceptance pass. State → ✅ Completed; filed directly to `task/completed/`.
