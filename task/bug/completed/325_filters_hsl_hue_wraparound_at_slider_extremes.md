# BUG-325: `filters`'s HSL-adjustment shader under-wraps the hue channel at slider extremes because it fed `hue2rgb` a value outside the ±1-unit domain the helper assumes

- **Severity:** Medium (visible color-shift artifact at slider extremes)
- **state:** Completed
- **Affects:** `examples/minwebgl/filters/src/filters/hsl_adjustment.rs`
- **Component:** examples/minwebgl/filters
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The hue-adjustment line read `hsl.x += u_hsl.x;` with no wraparound. `hue2rgb`'s single-step ±1
wraparound assumes its phase-shifted (±1/3) input is at most 1 unit outside `[0,1)` -- which no
longer held once an external shift of up to ±1.0 was added on top, under-wrapping the r/b channels
at slider extremes.

## Impact

**Who is affected:** any user dragging the hue-adjustment slider to (or near) its extremes.

**What breaks:** the rendered color shifts incorrectly at slider extremes -- the r/b channels
under-wrap relative to the correct hue rotation, producing a visibly wrong color instead of the
expected full-range hue cycle.

**Entity Scope:** `None` -- confined to this crate's own HSL-adjustment fragment shader.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by checking `hue2rgb`'s own domain assumption (single-step ±1 wraparound) against the
actual range of values `main()` could feed it, rather than assuming an additive adjustment is
always safe. Independently verified by the orchestrating session: `u_hsl.x`'s slider range allows
a shift of up to ±1.0, exceeding the ±1/3 phase-shift domain `hue2rgb`'s single-step wraparound was
written for.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p filters --test hsl_wraparound_test
```
**Expected** (fixed): the hue-adjustment line normalizes into `[0,1)` via `mod(hsl.x + u_hsl.x, 1.0)`
before being passed onward. **Actual** (pre-fix): the raw sum was passed through unwrapped,
falling outside the domain `hue2rgb`'s single-step wraparound correctly handles.

## Root Cause

`hue2rgb` is correct only for a hue value already normalized to `[0,1)`; `main()` broke that
precondition by adding an external shift of up to ±1.0 before calling `hsl2rgb`, without
renormalizing the result first.

## Why Not Caught

No test exercised the HSL-adjustment shader across its full slider range -- the shader still
renders *a* color at every slider position, so an incorrect wraparound at the extremes has no
symptom short of visually comparing the rendered hue against the expected rotation.

## Fix Applied (2026-08-18)

Changed the hue-adjustment line to `hsl.x = mod(hsl.x + u_hsl.x, 1.0);`, renormalizing the summed
hue back into `[0,1)` before it reaches `hue2rgb`, restoring the precondition the helper assumes.
Added `tests/hsl_wraparound_test.rs`: `include_str!`-based structural assertion that the shader
source wraps the hue sum via `mod(..., 1.0)` rather than passing the raw, unbounded sum through.

## Verification

- **Pre-fix (RED):** reverted the hue line to its unwrapped `+=` form; new test failed (no
  `mod`-based wraparound detected).
- **Post-fix (GREEN):** `cargo test -p filters` -- new test passes (alongside sibling BUG-324's own
  test in the same crate); `cargo check --target wasm32-unknown-unknown -p filters` and
  `cargo clippy --all-targets --all-features -p filters -- -D warnings` both clean.

## Generalized Version

A helper function correct only for an assumed input domain (here, `hue2rgb`'s single-step ±1
wraparound, valid for inputs at most 1 unit outside `[0,1)`) fails silently once a caller widens
that domain by feeding it a value from a wider range -- check the caller's actual value range
against the helper's documented or implicit domain assumption, not just the helper's own
correctness in isolation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-325 after a fresh on-disk collision scan. Related: BUG-324, a second, unrelated-root-cause bug in the same crate's Stack Blur filter. |
