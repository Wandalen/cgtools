# BUG-302: `field_of_view_demo/readme.md` describes the FOV algorithm lineup as 3 algorithms, omitting the 4th (flood fill) the demo actually exercises

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/tiles_tools/field_of_view_demo/readme.md`
- **Component:** examples/tiles_tools/field_of_view_demo
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`readme.md` described the demo's field-of-view algorithm lineup as "shadowcasting, ray casting,
and Bresenham line tracing" -- an enumerated, completeness-implying list. `src/main.rs` actually
exercises 4 algorithms via `FOVAlgorithm::{Shadowcasting, RayCasting, Bresenham, FloodFill}` (see
its own `"=== Flood Fill Algorithm ==="` output section), so the readme undercounted by one,
omitting flood fill entirely.

## Impact

**Who is affected:** any reader using the readme's algorithm list to understand what the demo
covers.

**What breaks:** the readme presents an enumerated, completeness-implying claim, but a reader
following it would not know a 4th algorithm (flood fill) is demonstrated at all.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Disclosed by a fork bug-hunting `tiles_tools`'s 12 native example crates (task #183).
Independently verified by reading `src/main.rs`, confirming `FOVAlgorithm::FloodFill` is
genuinely used at line 24 and its own dedicated output section exists.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "FloodFill" examples/tiles_tools/field_of_view_demo/src/main.rs
grep -c "flood fill" examples/tiles_tools/field_of_view_demo/readme.md
```
**Expected** (fixed): `FloodFill` genuinely appears in `main.rs`, and the readme's "flood fill"
count is >= 1. **Actual** (pre-fix): `FloodFill` was real and used, but the readme's count was 0.

## Root Cause

The readme's algorithm-lineup sentence was written before (or never updated after) flood fill was
added as a 4th demonstrated algorithm -- an enumerated/completeness-style doc claim left
unsynchronized with the actual `FOVAlgorithm` variants the demo exercises.

## Why Not Caught

This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero pre-existing test
coverage of any kind, so nothing tied the readme's enumerated claim to the actual `FOVAlgorithm`
variants demonstrated.

## Fix Applied (2026-08-18)

Added "and flood fill" to the readme's algorithm list so it names all 4 algorithms the demo
actually runs.

Added `tests/readme_doc_test.rs` (`readme_lists_all_four_fov_algorithms_including_flood_fill`):
pure `include_str!` + substring assertions confirming the readme mentions flood fill alongside
the other 3 algorithms.

## Verification

- **Pre-fix (RED):** readme lacked "flood fill" -- test would fail against the pristine text.
- **Post-fix (GREEN):** `cargo test -p field_of_view_demo --test readme_doc_test` → 1 passed.
  `cargo clippy -p field_of_view_demo --all-targets --all-features -- -D warnings` → clean.
  Independently re-run by the orchestrating session as part of this task's combined confirming
  sweep across all 4 of this fork's `tiles_tools` findings.

## Generalized Version

An enumerated/completeness-style doc claim ("compares X, Y, and Z") is a falsifiable claim, not
loose descriptive summary -- it needs its own doc-text regression test reading the file's actual
prose, since it can silently undercount again if a 5th algorithm is added to the demo without
updating the readme in the same change.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting `tiles_tools`'s 12 native crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (diff read, `FloodFill` usage cross-checked in source, test independently re-run) before this report and its real ID were assigned; placeholder replaced with BUG-302 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |
