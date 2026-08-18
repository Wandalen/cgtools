# BUG-317: `color_space_conversions`'s CSS `hsl()`/`hwb()` output omits the mandatory `%` suffix on saturation/lightness (and whiteness/blackness), producing invalid CSS the browser silently drops

- **Severity:** Medium (visible rendering defect -- the swatch using this string goes unstyled)
- **state:** Completed
- **Affects:** `examples/minwebgl/color_space_conversions/src/main.rs`
- **Component:** examples/minwebgl/color_space_conversions
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The "hsl" and "hwb" arms of this demo's color-space-conversion formatter emitted
`hsl( {hue:.2} {saturation:.2} {lightness:.2} )` / `hwb( {hue:.2} {whiteness:.2} {blackness:.2} )`
-- bare numbers for the second and third components, with no `%` suffix.

## Impact

**Who is affected:** any viewer of the demo relying on the "hsl"/"hwb" output strings to actually
style an element.

**What breaks:** CSS's `hsl()`/`hwb()` grammar requires saturation/lightness (and
whiteness/blackness) to be `<percentage>`, not a bare `<number>` -- unlike `lab()`/`lch()`, which
accept either. A bare number there is invalid syntax; the browser silently drops the whole
declaration rather than partially applying it, leaving the target element unstyled with no
visible error.

**Entity Scope:** `None` -- output-formatting defect confined to this crate's own display strings.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184). Independently verified by the orchestrating session: the `color` crate's `Hsl`/`Hwb` types
represent both components as `[0,100]` numeric ranges, but the CSS Color Module spec types
`hsl()`'s S/L and `hwb()`'s W/B strictly as `<percentage>`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n '"hsl( {hue:.2} {saturation:.2}% {lightness:.2}% )"\|"hwb( {hue:.2} {whiteness:.2}% {blackness:.2}% )"' \
  examples/minwebgl/color_space_conversions/src/main.rs
```
**Expected** (fixed): both format strings print with `%` on their second and third placeholders.
**Actual** (pre-fix): both arms formatted all three components as bare numbers, producing
CSS the browser rejects.

## Root Cause

The format strings for the "hsl" and "hwb" arms omitted the `%` suffix CSS requires for their
second and third components; the sibling "lab"/"lch"/"oklab"/"oklch" arms were correctly left bare
since those grammars accept `<number>` there, and the "hsl"/"hwb" arms were evidently written by
analogy without checking that their own grammar differs.

## Why Not Caught

No test exercised the generated CSS strings' validity against the CSS Color Module grammar --
the demo "works" visually as long as a viewer never inspects the actual unstyled swatch or the
browser console for a dropped-declaration warning.

## Fix Applied (2026-08-18)

Added the missing `%` suffix to the saturation/lightness placeholders in the "hsl" arm and the
whiteness/blackness placeholders in the "hwb" arm, matching CSS's `<percentage>` requirement for
both functions. The "lab"/"lch"/"oklab"/"oklch" arms were left untouched -- their grammars
correctly accept bare numbers.

Added `tests/hsl_hwb_css_percent_test.rs`: asserts (via the crate's own conversion + formatting
logic, exercised as a pure function) that the "hsl" and "hwb" arms' output strings contain `%` on
their second and third numeric components, and that the "lab"/"lch" arms do NOT gain a spurious
`%` (guards against an overcorrection blanket-appending `%` to every arm).

## Verification

- **Pre-fix (RED):** reverted the format strings to their bare-number form; new test failed as
  expected (missing `%`).
- **Post-fix (GREEN):** `cargo test -p color_space_conversions` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p color_space_conversions` and
  `cargo clippy --all-targets --all-features -p color_space_conversions -- -D warnings` both clean.

## Generalized Version

When a demo formats structured data into a target grammar (CSS, JSON, a shader-uniform block),
each field's required syntax must be checked against that grammar's own spec per function --
`hsl()`/`hwb()` and `lab()`/`lch()` share a visually similar function-call shape but have
different `<percentage>` vs `<number>` requirements on structurally analogous positions, and
copying one arm's format string to write a sibling arm silently carries over a grammar mismatch
that has no compiler-visible symptom, only a silently-dropped CSS declaration.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (fix diff read, CSS grammar cross-checked, test re-run) before this report and its real ID were assigned; placeholder replaced with BUG-317 after a fresh on-disk collision scan found IDs up to BUG-315/TASK-316 already claimed. |
