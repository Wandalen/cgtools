# BUG-322: `diamond`'s Beer-Lambert absorption-strength multiplication was commented out, making the `absorptionFactor` uniform have zero effect on the rendered result

- **Severity:** Medium (visible rendering defect -- a user-controllable parameter silently does nothing)
- **state:** Completed
- **Affects:** `examples/minwebgl/diamond/shaders/shader.frag`
- **Component:** examples/minwebgl/diamond
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`getRefractionColor`'s absorption-strength multiplication (`* absorptionFactor` applied to the
ray-segment length feeding the Beer-Lambert attenuation term) was commented out, leaving the
`absorptionFactor` uniform declared and uploaded from Rust but with zero effect on the shader's
output -- any value assigned to it produced identical rendered output.

## Impact

**Who is affected:** any user of this demo adjusting the absorption-strength control, expecting
the diamond's internal light attenuation to visibly respond.

**What breaks:** `absorptionFactor` is a dead, cosmetic-only uniform -- the CPU-side value is
computed and uploaded correctly, but the GPU-side computation that was supposed to consume it
never multiplies it in, so the rendered diamond looks identical regardless of the control's value.

**Entity Scope:** `None` -- confined to this crate's own fragment shader.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by cross-checking every uploaded uniform against its actual use inside the shader source
rather than assuming an uploaded value is necessarily consumed. Independently verified by the
orchestrating session: `absorptionFactor` is declared, uploaded from Rust, but the shader-side
multiplication line was present only in comment form.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "absorptionFactor" examples/minwebgl/diamond/shaders/shader.frag
```
**Expected** (fixed): the `r = length(...) * absorptionFactor` line is live code, not a comment.
**Actual** (pre-fix): the multiplication existed only inside a comment; the live line omitted
`* absorptionFactor` entirely.

## Root Cause

A debugging-time comment-out of the absorption-strength multiplication was never restored --
likely disabled temporarily while isolating a different rendering issue, then left in place.

## Why Not Caught

No test exercised the fragment shader's actual use of the `absorptionFactor` uniform against its
declaration -- a demo that "renders something plausible" either way gives no visible signal that
one of its own controls has become inert.

## Fix Applied (2026-08-18)

Restored the multiplication: `float r = length(rayOrigin - oldOrigin) * absorptionFactor;`,
re-enabling the Beer-Lambert attenuation term's dependence on the uploaded uniform. The uniform
itself was left untouched -- only the missing multiplication was restored. Added
`tests/absorption_factor_test.rs`: `include_str!`-based structural assertion that the shader
source's `getRefractionColor` function actually multiplies by `absorptionFactor` in live (not
commented-out) code.

## Verification

- **Pre-fix (RED):** reverted the multiplication back to its commented-out form; new test failed
  (uniform declared but unused in live code, detected).
- **Post-fix (GREEN):** `cargo test -p minwebgl_diamond` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p minwebgl_diamond` and
  `cargo clippy --all-targets --all-features -p minwebgl_diamond -- -D warnings` both clean.

## Generalized Version

An uploaded shader uniform having a correct CPU-side computation and upload call proves nothing
about whether the GPU-side shader actually consumes it -- cross-check every uniform's declaration
against its actual use inside the shader body, since a debugging-time comment-out of the consuming
line leaves the uniform silently inert with no compiler or runtime error on either side.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-UUU` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-322 after a fresh on-disk collision scan. |
