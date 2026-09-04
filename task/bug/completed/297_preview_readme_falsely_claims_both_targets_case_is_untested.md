# BUG-297: `shader_chunks_preview/readme.md` falsely claims the "both `name` and `file::` given" case is untested -- it's already covered by INT-5

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `shader_chunks_preview/readme.md`
- **Component:** module/shader/shader_chunks_preview
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`readme.md` stated: "**Disclosed gap:** no test covers giving *both* `name` and `file::`
simultaneously -- only the 'neither given' arm of the mutual-exclusivity check is exercised."
The crate's own test-coverage account
(`tests/docs/cli/command/cmd_001_preview.md`) already lists INT-5, a test covering exactly that
"both given" arm, alongside INT-3 for the "neither given" arm.

## Impact

**Who is affected:** any reader trusting this readme's disclosed-gap claim to decide whether
further test coverage is needed here.

**What breaks:** the claim is not just outdated -- it directly contradicts the crate's own
authoritative test-coverage document one directory below it, so a reader consulting both files
gets an internal contradiction instead of a consistent account of what's tested.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

During task #182's bug-hunting pass, cross-checked `shader_chunks_preview/readme.md`'s
"Disclosed gap" claim against its own cited test-coverage table
(`tests/docs/cli/command/cmd_001_preview.md`) rather than trusting the readme's prose at face
value -- found INT-5 already present and already covering the exact case the readme claimed was
untested.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "INT-3\|INT-5" module/shader/shader_chunks_preview/tests/docs/cli/command/cmd_001_preview.md
grep -c "Disclosed gap" module/shader/shader_chunks_preview/readme.md
```
**Expected** (fixed): both INT-3 and INT-5 rows print (the coverage genuinely exists), and the
`Disclosed gap` grep returns 0 (the false claim is gone). **Actual** (pre-fix): both INT-3/INT-5
rows already printed (proving the gap claim was already false), yet the readme's `Disclosed gap`
text was still present, contradicting them.

## Root Cause

The "Disclosed gap" note was accurate when originally written, but a later change added the
`subprocess_preview_with_both_targets_fails_loudly` test (INT-5) covering exactly that gap
without updating the readme's own disclosure -- the claim was correct at authoring time and
silently went stale as the actual test suite grew past it.

## Why Not Caught

No test or check ties the readme's prose claims to the actual contents of
`tests/docs/cli/command/cmd_001_preview.md` -- the two are maintained by hand independently, so a
new test can close a disclosed gap without anything forcing the disclosure itself to be updated
in the same change.

## Fix Applied (2026-08-18)

Replaced the false "Disclosed gap" paragraph in `readme.md` with an accurate statement naming
both INT-3 (neither given) and INT-5 (both given) as already covering both arms of the
mutual-exclusivity check, keeping the existing cross-reference to
`tests/docs/cli/command/cmd_001_preview.md`. Pure prose fix -- no test code changed, since the
coverage this correction describes already existed prior to this fix.

No new regression test was added for this specific bug: unlike BUG-294 (a factual claim with no
existing test tying doc text to source-of-truth data), this correction's own accuracy is
directly and immediately checkable by reading `cmd_001_preview.md`'s existing INT-3/INT-5 rows,
and the preview crate's existing `preview_cli_test.rs` integration tests
(`subprocess_preview_with_no_target_fails_loudly`,
`subprocess_preview_with_both_targets_fails_loudly`) already are the regression coverage for the
underlying claim -- adding a third, doc-text-reading test here would test only that this specific
sentence doesn't get re-broken, with no further behavioral signal beyond what BUG-294's
established `include_str!` pattern already demonstrates elsewhere in this task's own findings.

## Verification

Direct read/grep, no build or test run required for a pure prose correction; the pre-existing
`shader_chunks_preview` test suite (including INT-3/INT-5's own tests) was re-run as part of this
task's combined confirming sweep alongside its 3 sibling bugs.

- **Pre-fix:** `grep -c "Disclosed gap" module/shader/shader_chunks_preview/readme.md` → `1`.
- **Post-fix:** same command → `0`; `git show ab40a11d -- module/shader/shader_chunks_preview/readme.md`
  confirms the replacement text is exactly the corrected claim. Combined scoped suite
  (`shader_chunks_params_core` + `shader_chunks_params` + `shader_chunks_cli_core` +
  `shader_chunks_preview`, run together with sibling BUG-293/294/295): `48 tests run: 48 passed,
  0 skipped`, clean clippy across all 4 crates -- confirming INT-3/INT-5 and everything else in
  this crate remained green throughout.

## Generalized Version

A "disclosed gap" or "known limitation" note in documentation is itself a claim that can go
stale exactly like any other doc-text fact -- when closing a gap by adding a test, always search
the crate's own docs for prose that names that gap and update or remove it in the same change,
rather than leaving the correction implicit in the test suite alone.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found by a fork investigating `shader_chunks_cli_core`/`shader_chunks_preview`/`shader_chunks_preview_web` (task #182, parallel with 2 sibling forks). Fix applied directly by the fork (pure prose, no placeholder marker needed since no source/test file referenced a bug ID for this specific finding); this report and its real ID were assigned by the orchestrating session after independently reading the actual committed diff and confirming INT-3/INT-5's existence in the cited test-coverage table. |
