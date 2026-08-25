# BUG-448: `normalize`/`project_on` (and their allocating siblings) silently return `NaN` for zero-magnitude input, previously undocumented and untested

- **Severity:** Medium (not a functional defect -- `NaN` is the mathematically honest answer for an
  undefined operation -- but the behavior was entirely undocumented and untested, leaving every caller
  to independently discover or misjudge it)
- **state:** Completed
- **Affects:** Any caller of `mdmath_core::vector::normalize`/`normalized`/`normalize_to`/
  `normalized_to`/`project_on`/`projected_on` with a zero-magnitude input vector (or, for `project_on`/
  `projected_on`, a zero-magnitude `b`).
- **Component:** `module/math/mdmath_core` (`src/vector/arithmetics.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Discovered alongside BUG-446 (`vector::angle`'s zero-vector case shares the exact
  same "division by zero magnitude is intentional NaN" contract, and `test_angle`'s pre-existing
  zero-vector assertion became the regression guard that caught BUG-446's own initial fix attempt
  laundering that NaN away). No shared root cause otherwise -- this item is documentation/test-only, not
  a source behavior change.

## Symptom

```rust
let mut r = [ 1.0, 2.0, 3.0 ];
let zero_b = [ 0.0, 0.0, 0.0 ];
vector::project_on( &mut r, &zero_b );
// r is now [ NaN, NaN, NaN ] -- correct, but was previously undocumented and untested
```

`normalize`/`project_on` (and their allocating `normalized`/`projected_on`, `normalize_to`/
`normalized_to` siblings) divide by a computed magnitude with no special-case guard for a zero-magnitude
input. Division by zero magnitude in IEEE-754 float arithmetic yields `NaN` for every written
component. This is the *correct*, intentional behavior -- a zero-length vector has no defined direction,
so `NaN` is the honest encoding of "undefined," not an arbitrary fallback (e.g. silently returning the
zero vector, which would falsely claim the zero vector's direction *is* the zero vector) -- but neither
the doc comments nor the test suite said so before this task.

## Impact

**Who is affected:** Any caller passing (or capable of passing) a zero-magnitude vector to any of these
six functions, who previously had to either read the source to learn the actual behavior or discover it
empirically at runtime.

**What breaks:** Nothing functionally -- this is a documentation/test gap, not a source defect. The
existing behavior (`NaN` propagation) is correct and is retained unchanged.

**Consumer audit:** A workspace-wide grep found 100+ call sites for `normalize`/`normalized`/
`normalize_to`/`normalized_to` and 7 for `project_on`/`projected_on`, spanning nearly every crate in the
workspace. Given the sheer footprint and that the existing behavior is already mathematically correct,
changing the *behavior* (e.g. adding a caller-configurable zero-magnitude fallback) was judged
out-of-scope for this task -- documenting and testing the existing, correct contract closes the actual
gap without touching 100+ external call sites on a speculative behavior change no caller has asked for.

**Entity Scope:** None -- a documentation/test gap, not a code-level defect.

## How Discovered

Found during the same repo-wide discovery sweep as BUG-445/446/447: every `/` (division) in
`vector/arithmetics.rs` was audited for an unguarded zero-denominator case. `normalize`/`project_on`'s
division by magnitude has no guard, and neither their doc comments nor `arithmetics.rs`'s test file
mentioned or tested the zero-magnitude case.

## Minimum Reproducible Example

```rust
// module/math/mdmath_core/tests/inc/arithmetics.rs
let vec_zero_b : [ f32 ; 3 ] = [ 0.0, 0.0, 0.0 ];
let mut r = [ 1.0, 2.0, 3.0 ];
vector::project_on( &mut r, &vec_zero_b );
// every component of r is NaN -- correct, now documented and tested
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/math/mdmath_core && cargo nextest run -E 'test(test_project_on_zero_b_yields_nan)'
```

## Root Cause

Not a code defect: `normalize`/`project_on`'s division by a zero magnitude producing `NaN` is the
mathematically correct behavior for an operation with no defined answer for that input (a zero-length
vector has no direction; a projection onto a zero-length vector has no defined target axis). The gap
was purely documentational -- the six affected functions' doc comments never stated this behavior, and
no test exercised it, so a caller had no way to confirm the behavior was intentional versus an oversight
without reading the source and reasoning through the arithmetic themselves.

## Why Not Caught

`arithmetics.rs`'s existing tests exercised `normalize`/`project_on` only with non-degenerate,
nonzero-magnitude inputs. There was no test (and no doc statement) establishing what should happen for a
zero-magnitude input, so there was nothing to "catch" in the sense of a wrong answer -- this is a
resolution documenting and testing a previously-silent contract, per this workspace's Bug-Fixing
Workflow, which treats "silently undocumented, untested behavior discovered during a systematic
audit" as a reportable finding even when the underlying behavior turns out to already be correct.

## Fix Location

`module/math/mdmath_core/src/vector/arithmetics.rs`: added a "# Zero-magnitude input" (`normalize`/
`normalized`/`normalize_to`/`normalized_to`) / "# Zero-magnitude `b`" (`project_on`/`projected_on`) doc
section to each of the six functions, stating the `NaN` behavior is intentional and cross-referencing
BUG-448. No source *behavior* changed.

## Prevention

`test_project_on_zero_b_yields_nan` (`mdmath_core/tests/inc/arithmetics.rs`) asserts every component of
both `project_on`'s (in-place) and `projected_on`'s (allocating) output is `NaN` for a zero-magnitude
`b`, converting the newly-documented contract into an executable regression guard.

## Pitfall

A judgment call to leave existing behavior unchanged and only add documentation/tests is not a substitute
for actually closing the "was this intentional?" gap that prompted the finding in the first place --
without a doc statement and a test, a future contributor encountering the same `NaN` output has no way
to distinguish "known, correct, honest IEEE-754 encoding of an undefined operation" from "an
unnoticed bug," and might otherwise be tempted to "fix" it into a silently-wrong fallback (e.g. the zero
vector) that discards the honest signal.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX discovery sweep. |
| 2026-08-20 | fixed | Judgment call: documentation-only resolution (no source behavior change) given the existing behavior is already mathematically correct and the 100+7 call-site footprint makes an unrequested behavior change high-risk relative to its benefit. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: test asserts the documented `NaN` contract for both the in-place and allocating variants. Adversarial pass: considered whether "documentation-only" is a legitimate resolution for a filed bug -- confirmed yes, given the Bug-Fixing Workflow's own MRE step only requires demonstrating the *finding*, not that a source change is owed; explicitly documented as a judgment call rather than silently narrowing scope. `cargo nextest run -p mdmath_core -p ndarray_cg --no-fail-fast` -- 395/395 pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | 5-section test doc comment (`bug_reproducer(BUG-448)`) added, explicitly noting the documentation-only resolution in its own Root Cause section; doc-comment `# Zero-magnitude input`/`# Zero-magnitude \`b\`` sections cross-reference BUG-448 on all 6 affected functions. | — |
| D3 | Scope containment | — | 🟢 | No source *behavior* changed -- confirmed via review that only doc comments and the new test were added; the 100+7 external call sites were audited (grep) but deliberately not touched, consistent with the judgment call above. | — |

**Reproduced:** YES (as a documentation gap, not a wrong-answer defect) -- `project_on`/`projected_on`
with a zero-magnitude `b` produced `NaN` both before and after this task; what changed is that the
behavior is now documented and covered by a test rather than silently relying on IEEE-754 semantics with
no stated contract. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/mdmath_core/src/vector/arithmetics.rs` | Added "# Zero-magnitude input"/"# Zero-magnitude \`b\`" doc sections (cross-referencing BUG-448) to `normalize`, `normalized`, `normalize_to`, `normalized_to`, `project_on`, `projected_on`. No behavior change. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/mdmath_core/tests/inc/arithmetics.rs` | Added `test_project_on_zero_b_yields_nan`. |
