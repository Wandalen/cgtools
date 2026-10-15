# BUG-303: `debug_demo`'s module doc comment and readme both falsely claim SVG export -- every SVG/CSV export call site is commented out, only ASCII-art rendering actually runs

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/tiles_tools/debug_demo/src/main.rs` + `readme.md`
- **Component:** examples/tiles_tools/debug_demo
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Both `src/main.rs`'s module doc comment ("ASCII art rendering and SVG export capabilities") and
`readme.md` ("both ASCII-art and SVG rendering of the same grid state") claimed the demo actively
exercises SVG export. Every `svg_export`/`csv_export` call site in `main.rs` is commented out
(e.g. `// square_grid.svg_export(...)`); only `ascii_render()` is actually called at runtime.

## Impact

**Who is affected:** any reader trusting either doc claim to understand what the demo actually
outputs.

**What breaks:** the same false claim was copy-pasted into 2 files (module doc comment and
readme.md), so a reader consulting either gets a wrong expectation about the demo's runtime
output.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Disclosed by a fork bug-hunting `tiles_tools`'s 12 native example crates (task #183).
Independently verified via `grep`: every `svg_export`/`csv_export` call site in `main.rs` is
commented out, only `ascii_render()` is live.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "svg_export\|csv_export\|ascii_render" examples/tiles_tools/debug_demo/src/main.rs
grep -c "SVG" examples/tiles_tools/debug_demo/readme.md
```
**Expected** (fixed): every `svg_export`/`csv_export` line is commented out, `ascii_render()` is
live, and the readme's "SVG" count is 0. **Actual** (pre-fix): same call-site pattern, but the
readme's "SVG" count was >= 1 and the module doc comment also claimed it.

## Root Cause

SVG/CSV export was once live in this demo, then disabled (commented out) without updating either
doc claim that described it as an active capability.

## Why Not Caught

This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero pre-existing test
coverage, so nothing tied either doc claim to which export call sites were actually live vs.
commented out.

## Fix Applied (2026-08-18)

Corrected both `src/main.rs`'s module doc comment and `readme.md` to describe only the ASCII-art
rendering the demo actually performs, dropping the false SVG-export claim from both.

Added `tests/readme_doc_test.rs` with 2 tests
(`main_rs_module_doc_comment_does_not_claim_svg_export`, `readme_does_not_claim_svg_rendering`):
pure `include_str!` + substring assertions, one per file, since the same false claim was
copy-pasted into both and each needed its own direct doc-text assertion -- fixing one without
the other would leave a real, independently-checkable contradiction behind.

## Verification

- **Pre-fix (RED):** both doc claims present -- tests would fail against the pristine text.
- **Post-fix (GREEN):** `cargo test -p debug_demo --test readme_doc_test` → 2 passed. `cargo
  clippy -p debug_demo --all-targets --all-features -- -D warnings` → clean. Independently
  re-run by the orchestrating session as part of this task's combined confirming sweep.

## Generalized Version

Commented-out code that still contains a real, once-working call site is easy to miss when
auditing doc claims -- `grep` for the claimed capability's call site itself (not just its mention
in doc text) to confirm it is actually live, not dead. A doc claim naming a specific capability
needs its own doc-text regression test; re-enabling a commented-out call site in a future change
should be paired with restoring the claim, not the other way around (claiming a capability that
was quietly disabled).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting `tiles_tools`'s 12 native crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (diff read, call sites cross-checked via grep, both tests independently re-run) before this report and its real ID were assigned; placeholder replaced with BUG-303 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |
