# BUG-323: `filter`'s readme names several post-processing filter categories the crate never implements -- it applies exactly one hardcoded emboss convolution kernel

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/filter/readme.md`
- **Component:** examples/minwebgl/filter
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The readme's opening description named several other post-processing filter categories this crate
never implements. `main.frag` applies exactly one hardcoded convolution kernel: emboss.

## Impact

**Who is affected:** any reader using the readme to understand which filter techniques this demo
actually shows before opening the shader source.

**What breaks:** a demo whose entire purpose is showing a filter technique is exactly where a
wrong named technique goes unnoticed -- the demo still visibly "works" (it does apply a
convolution kernel and does respond to the mouse-centered radius), so the specific wrong name has
no behavioral symptom.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by reading `main.frag`'s actual kernel and comparing it against the readme's named
technique list rather than trusting the prose. Independently verified by the orchestrating
session: `main.frag` contains exactly one 3x3 convolution kernel (emboss), no other filter
category's implementation.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "emboss\|convolution" examples/minwebgl/filter/shaders/main.frag examples/minwebgl/filter/readme.md
```
**Expected** (fixed): readme names only the emboss convolution-kernel technique this crate
actually implements. **Actual** (pre-fix): readme's opening line named additional filter
categories with no corresponding shader code anywhere in the crate.

## Root Cause

Aspirational wording never checked against the actual shader -- the readme described a broader
scope than the single hardcoded kernel `main.frag` actually applies.

## Why Not Caught

No test ties the readme's named-technique claims to the shader source's actual kernel -- the two
are maintained by hand independently.

## Fix Applied (2026-08-18)

Corrected the readme's opening description to name only the emboss convolution-kernel technique
this crate actually implements, keeping the existing description of the mouse-centered reveal
radius (which is accurate). Added `tests/readme_doc_test.rs`: `include_str!`-based assertion that
the readme's named filter technique is present as an actual kernel identifier in `main.frag`.

## Verification

- **Pre-fix (RED):** reverted the readme to its aspirational wording; new test failed (named
  technique not traceable to shader source).
- **Post-fix (GREEN):** `cargo test -p filter` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p filter` and
  `cargo clippy --all-targets --all-features -p filter -- -D warnings` both clean.

## Generalized Version

A demo's readme naming a technique category is itself a factual claim exactly like any other doc
text -- it must be cross-checked against the shader/source that actually implements it rather than
trusted at face value, especially for a demo whose entire purpose is showcasing that one
technique, where a wrong name has zero visible symptom.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-323 after a fresh on-disk collision scan. |
