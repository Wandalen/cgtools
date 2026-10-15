# BUG-294: 3 doc files claim "46 of the 50" bundled chunks declare `//@ param:` lines and list only 4 exceptions -- real count is 45, missing `palette_cosine`

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `shader_chunks_params_core/readme.md`, `shader_chunks_params/readme.md`,
  `shader_chunks_params/docs/cli/command/01_tunables.md`
- **Component:** module/shader/shader_chunks_params_core, module/shader/shader_chunks_params
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

All 3 files claimed 46/50 bundled chunks carry `//@ param:` lines, naming exactly 4 leaf/
infrastructure exceptions (`hash21`, `hash22`, `srgb`, `fullscreen_triangle`). Direct enumeration
via `chunk_discover` against every real bundled chunk (`shader_chunks_core::CHUNKS`, 50 total)
shows the real count is 45, with `palette_cosine` a 5th exception missing from all 3 copies.

## Impact

**Who is affected:** any reader trusting these docs' stated coverage numbers.

**What breaks:** one fact, copy-pasted 3x, drifted identically in all 3 with nothing to catch it --
undermines trust in the crate's own claimed self-consistency between docs and the real bundled
chunk set.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

An initial wrong-path grep against a nonexistent `shader_chunks_core/shader/` directory falsely
suggested 0 real chunks use `//@ param:`; traced `build.rs`'s actual `shader_dir` resolution
(`manifest_dir/../../../shader` -> repo-root `shader/`), re-ran against the correct 50-file
location, and cross-checked `chunk_discover`'s own empty/non-empty split for every bundled chunk
against the docs' claim. Found during the same task #182 bug-hunting pass as BUG-293.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p shader_chunks_params_core --test discovery_test -- \
  exactly_5_bundled_chunks_declare_no_tunable_params \
  readme_chunk_annotations_reflect_palette_cosine_and_corrected_count
cargo test -p shader_chunks_params --test tunables_test -- \
  docs_reflect_palette_cosine_and_corrected_count
```
**Expected** (fixed): all pass. **Actual** (pre-fix docs): the 2 doc-text tests fail (readme/
CLI-doc text lacks `palette_cosine` / still says "46 of"). Note: the codebase-fact test
(`exactly_5_bundled_chunks_declare_no_tunable_params`) already passed even pre-fix, since it never
reads doc text -- an insufficient reproducer for a doc-only bug on its own, which is why the 2
`include_str!`-based doc-text tests exist (see Prevention).

## Root Cause

A specific numeric/list claim was authored once and copy-pasted into 2 further files without being
derived from or checked against `shader_chunks_core::CHUNKS` at any site.

## Why Not Caught

No existing test in either crate read any of the 3 files' own text -- `shader_chunks_params_core`'s
tests deliberately use only self-contained fixture WGSL (its own docs' Out of Scope note), so real-
chunk adoption state was never asserted against the docs' claim.

## Fix Applied (2026-08-18)

Corrected all 3 files from "46 of the 50" to "45 of the 50", adding `palette_cosine` to each
file's exception list (now `hash21`, `hash22`, `palette_cosine`, `srgb`, `fullscreen_triangle`).

**New regression tests:**
- `shader_chunks_params_core/tests/discovery_test.rs`:
  `readme_chunk_annotations_reflect_palette_cosine_and_corrected_count`
  (`include_str!`-based, asserts the readme's own text).
- `shader_chunks_params/tests/tunables_test.rs`: `docs_reflect_palette_cosine_and_corrected_count`
  (`include_str!`-based, asserts both the readme's and the CLI doc's own text).

## Verification

`longrun`-detached, from repo root, no `git stash`.

- **Pre-fix (RED):** combined `cargo nextest run -p shader_chunks_params_core -p
  shader_chunks_params`: `25/32 tests run: 22 passed, 3 failed` -- the 2 doc-text tests above plus
  BUG-293's own test failed; no other test affected.
- **Post-fix (GREEN):** same scope: `32 tests run: 32 passed, 0 skipped`, clean clippy. Wider
  combined scoped suite (adding `shader_chunks_cli_core` + `shader_chunks_preview`, run together
  with sibling BUG-293/295/297): `48 tests run: 48 passed, 0 skipped`, clean clippy across all 4
  crates -- independently re-run and confirmed by the orchestrating session, not only the
  investigating fork.

## Generalized Version

Every copy of a restated fact needs its own direct doc-text regression test (`include_str!` +
substring assertion) -- a codebase-fact test alone can pass forever even if the prose was never
actually corrected, matching this session's BUG-287/288/290 `include_str!` precedent for doc-only
defects that resist black-box runtime testing.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed together with sibling BUG-293 by the same fork investigating `shader_chunks_params`/`shader_chunks_params_core`/`shader_chunks_compose` (task #182). Fix and regression tests written by the fork with a `BUG-XXX-B` placeholder (forks in this batch were instructed not to self-file, to avoid a 3-way concurrent-write race on the shared bug ledger across 3 parallel forks); this report and its real ID were assigned by the orchestrating session after independently reading the actual committed diff (all 3 files) and re-running the full scoped test suite. |
