# BUG-186: `scaled_rotation_apply` continuity rebase only ran when `scale < 1.0`

- **Severity:** High (visual defect -- a discontinuous jump at every segment boundary of a
  grouped node's rotation animation, present for the GUI's own default amplitude and its entire
  "amplify" range, not just an edge case)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::animation::Scaler` that groups a node whose
  rotation animation has more than one segment and is scaled with `scale >= 1.0`.
- **Component:** `module/helper/renderer` (`src/webgl/animation/scaling.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** Fixed in the same session pass as BUG-184 (same file, same function family) --
  reported separately since the two are independent root causes. The two new functions BUG-184
  adds use this bug's corrected (unconditional) guard form from the start, rather than copying
  the wrong-guard version.

## Symptom

```rust
// before
if scale < 1.0 && i > 0
{
  tweens[ i ].start_value = tweens[ i - 1 ].end_value;
}
```

The continuity rebase -- which makes segment `i`'s start value pick up segment `i - 1`'s own
freshly-scaled end value, rather than sampling the tween's originally-authored, un-scaled start
-- was gated on `scale < 1.0` in addition to `i > 0`. Any segment after the first, scaled with
`scale >= 1.0`, kept its stale original `start_value`, producing a visible discontinuity at that
segment's boundary.

## Impact

**Who is affected:** Every caller driving a multi-segment rotation animation through a scaled
group with `scale >= 1.0` -- which includes `examples/minwebgl/animation_amplitude_change`'s own
GUI default (1.0) and its entire documented "amplify" range (1.0 to 3.0); `scale < 1.0`
("dampen") was the only range where the rebase actually ran.

**What breaks:** At every segment boundary after the first, the rotation visibly snaps from
wherever the current (correctly continuity-rebased) segment's scaled interpolation left off, back
to the next segment's originally-authored (un-rebased, un-scaled) start orientation.

**Magnitude:** Every multi-segment rotation sequence scaled at or above the GUI's own default
amplitude -- not a rare configuration.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Continuing task #136's investigation into `Scaler::set`'s grouped-node handling
(`scaled_rotation_apply`). Reading the per-segment loop for the BUG-184 fix surfaced the
`scale < 1.0 && i > 0` guard; manually tracing its interaction with the GUI's own default/range
(1.0 to 3.0, i.e. always `>= 1.0`) showed continuity was already broken at the tool's own default
setting, not just some unusual configuration.

## Minimum Reproducible Example

```rust
// two-segment rotation sequence, both segments reached ( current_id_get() == 1 ), scale = 1.5
scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 1.5, 1.0, 1.0 ) );
scaler.update( 0.002 ); // advances current_id_get() to 1
scaler.set( &nodes );
// pre-fix: node1's rotation samples segment 1's STALE original start_value, not
// segment 0's own freshly-scaled end_value -- a visible jump at the boundary.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test scaler_tests test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_one
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `scale < 1.0` clause in the continuity-rebase guard is unrelated to the rebase's actual correctness condition (`i > 0`, "is this not the first segment") and incorrectly narrows when the rebase runs. | ✅ Root Cause | Confirmed by tracing the guard against the GUI's own default/range and hand-deriving both branches' resulting quaternions. | E1 |
| H2 | The `scale < 1.0` clause is intentional -- e.g. amplitude-reduction ("dampen") is meant to preserve continuity while amplitude-increase ("amplify") is meant to intentionally snap. | ❌ Falsified | No comment, doc, or caller code anywhere suggests scale-direction-dependent continuity is intended; the GUI treats amplitude uniformly across its whole 0.0-3.0 range with no special-casing at 1.0. Nothing in `scaled_translation_apply`/`scaled_scale_apply`'s otherwise-identical structure (written fresh this session) has any such asymmetry either. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/scaling.rs`, `scaled_rotation_apply` (pre-fix) | `if scale < 1.0 && i > 0` -- hand-derived that with `scale = 1.5` the rebase is skipped, leaving `tweens[ 1 ].start_value` at its stale original value; empirically confirmed via a temporary revert-and-rerun ( see Prevention ). | H1 ✅ |
| E2 | `examples/minwebgl/animation_amplitude_change/src/gui_setup.rs` | Amplitude sliders range 0.0-3.0 with no special handling at or around 1.0; every value is treated uniformly via `F64x4::splat`. | H2 ❌ |

## Root Cause

The continuity-rebase guard combined an unrelated numeric condition (`scale < 1.0`) with the
actual correctness condition (`i > 0`, "not the first segment") via `&&`, without either clause
being individually justified against the other -- most plausibly a leftover from early
development/testing against only the "dampen" range, never widened once the GUI's actual default
and range were settled at `>= 1.0`.

## Why Not Caught

No pre-existing test drove a `Sequence` past its first segment boundary while asserting on the
resulting node rotation value -- `test_grouped_nodes_independence`, the one test exercising
multi-group rotation scaling, only checks `Scaler::scale_get`'s own weight bookkeeping, never a
post-`set()` node transform.

## Fix Location

`module/helper/renderer/src/webgl/animation/scaling.rs`, `scaled_rotation_apply`: changed the
guard to `if i > 0`, unconditional on `scale`.

## Prevention

New test `test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_one` drives a
two-segment rotation sequence past its first boundary with `scale = 1.5` and asserts the second
segment's sampled rotation is close to the first segment's own scaled end value (independently
recomputed in the test using the same axis-angle scaling formula `scaled_rotation_apply` itself
uses), rather than the stale original value. Verified empirically, not just by construction: the
guard was temporarily reverted to `scale < 1.0 && i > 0` via a direct source edit (no `git
stash` -- outside the git whitelist), the new test was re-run and failed with `|dot| = 0.922`
against a `got` value matching the hand-derived stale value almost exactly, then the fix was
restored and the test re-confirmed passing.

## Pitfall

A guard combining an unrelated numeric condition with the actual correctness condition via `&&`
silently narrows when a piece of otherwise-unconditional bookkeeping logic runs. When reading a
multi-clause guard, check every clause individually against what the guarded code is actually
*for* -- a clause that happens to be true during whatever scenario the guard was first written
against can silently persist long after that scenario stops being the only one that matters.

## Generalized Version

**Broken assumption:** "Every clause in an existing multi-condition guard is load-bearing for
correctness, since it's already there and the code otherwise works."

**Confirmed general rule:** Before trusting a multi-clause guard, verify each clause
independently against the guarded code's actual purpose -- an extra clause with no comment
explaining why it belongs is a candidate defect, not a given.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found while implementing BUG-184's fix; traced the existing guard against the GUI's own default/range. |
| 2026-08-16 | fixed | Changed the guard to `if i > 0`, unconditional on `scale`. |
| 2026-08-16 | verified | Empirically confirmed via temporary guard revert ( direct source edit, not git ): new test failed pre-fix (`|dot|=0.922`), passed post-fix. `cargo nextest run -p renderer --test scaler_tests --all-features`: 10/10 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1911/1911 passed, doctests all `ok`, `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: test passes against fixed code. Adversarial: attempted to show the test might pass for reasons unrelated to the fix (e.g. floating-point coincidence) -- ruled out by temporarily reverting the guard via direct source edit and confirming the test fails with a `got` value matching the hand-derived pre-fix expectation almost exactly (`|dot|=0.922`), then restoring and reconfirming PASS. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-184 (fixed in the same pass, same file, uses this fix's corrected guard form from the start). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct code reading, hand-derivation, and empirical revert-and-rerun confirmation, not assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is the one-line guard change only; no unrelated refactor. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `scaling.rs`. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `scaled_rotation_apply` has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix corrects the guard's own documented intent (rebase every non-first segment) without adding unrelated scope. | — |

**Reproduced:** YES -- new test fails pre-fix (`|dot|=0.922`, `got` matching the hand-derived
stale value almost exactly) and passes post-fix (`|dot| > 0.99`), confirmed via a temporary
direct-source-edit revert-and-rerun. Scoped suite (10/10), full workspace (1911/1911), doctests,
and clippy all clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/scaling.rs` | `scaled_rotation_apply`'s continuity-rebase guard changed from `scale < 1.0 && i > 0` to `i > 0`, with a `Fix(BUG-186)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/scaler_tests.rs` | Added `test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_one`. |
