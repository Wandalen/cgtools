# BUG-304: `serialization_demo/readme.md`'s feature list omits compression support -- the 6th feature `src/main.rs`'s own module doc comment lists and the demo actually exercises

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/tiles_tools/serialization_demo/readme.md`
- **Component:** examples/tiles_tools/serialization_demo
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`readme.md` enumerated 5 features (3-format serialization, config management, save-file
management, version-compatibility checking) but omitted "Compression support" -- the 6th item in
`src/main.rs`'s own module doc comment list. `compression_demonstrate()` exists in `src/main.rs`
and is actively called from `main()`, so the omission undercounted a real, exercised feature.

## Impact

**Who is affected:** any reader using the readme's feature list to understand what the demo
covers.

**What breaks:** the readme is missing a real, exercised feature that its own sibling source file
already documents -- a reader consulting only the readme would not know compression is
demonstrated at all.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Disclosed by a fork bug-hunting `tiles_tools`'s 12 native example crates (task #183).
Independently verified: `compression_demonstrate()` is real and genuinely called from `main()`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "compression_demonstrate" examples/tiles_tools/serialization_demo/src/main.rs
grep -c "compression support" examples/tiles_tools/serialization_demo/readme.md
```
**Expected** (fixed): `compression_demonstrate` is defined and called, and the readme's
"compression support" count is >= 1. **Actual** (pre-fix): the function was real and called, but
the readme's count was 0.

## Root Cause

The readme's feature list was written before (or never updated after) compression support was
added as the 6th feature in `src/main.rs`'s own module doc comment -- an enumerated feature-list
claim left unsynchronized between the two files.

## Why Not Caught

This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero pre-existing test
coverage, so nothing tied the readme's enumerated feature list to `src/main.rs`'s own module doc
comment or to which `*_demonstrate` functions `main()` actually calls.

## Fix Applied (2026-08-18)

Added "compression support" to the readme's feature list so it matches all 6 items in
`src/main.rs`'s own module doc comment.

Added `tests/readme_doc_test.rs`
(`readme_lists_compression_support_alongside_other_five_features`): pure `include_str!` +
substring assertions confirming the readme mentions compression support alongside the other 5
features.

## Verification

- **Pre-fix (RED):** readme lacked "compression support" -- test would fail against the pristine
  text.
- **Post-fix (GREEN):** `cargo test -p serialization_demo --test readme_doc_test` → 1 passed.
  `cargo clippy -p serialization_demo --all-targets --all-features -- -D warnings` → clean.
  Independently re-run by the orchestrating session as part of this task's combined confirming
  sweep.

## Generalized Version

An enumerated feature-list claim ("exercises: A, B, C, and D") is a falsifiable completeness
claim, not loose descriptive summary -- it needs its own doc-text regression test reading the
file's actual prose, since it can silently undercount again if the readme and the module doc
comment are edited independently.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting `tiles_tools`'s 12 native crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (diff read, `compression_demonstrate` usage cross-checked in source, test independently re-run) before this report and its real ID were assigned; placeholder replaced with BUG-304 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |
