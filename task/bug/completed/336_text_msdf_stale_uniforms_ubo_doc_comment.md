# BUG-336: `text_msdf`'s module doc comment was copy-pasted from `uniforms_ubo`'s main.rs and never updated -- describes a plain UBO-driven triangle, not the MSDF text rendering this crate actually does

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/text_msdf/src/main.rs`
- **Component:** examples/minwebgl/text_msdf
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The module-level doc comment above the crate's own MSDF description was a leftover from
`uniforms_ubo`'s `main.rs`, describing a plain UBO-driven triangle rather than this crate's actual
Multi-Channel Signed Distance Field (MSDF) text rendering (glyph geometry, UV rects, and
per-character offsets computed from a parsed font atlas, uploaded as instanced quad attributes).

## Impact

**Who is affected:** any reader relying on the module doc comment to understand what this crate
demonstrates before reading further.

**What breaks:** the doc comment describes an unrelated, simpler technique (`uniforms_ubo`'s own
demo) instead of this crate's actual MSDF text-rendering pipeline, directly contradicting the
crate's own (correct) second doc-comment block describing MSDF rendering that follows it.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by noticing the file's doc comment was internally inconsistent with its own immediately
following MSDF description. Independently verified by the orchestrating session: the stale
sentence is a verbatim match for `uniforms_ubo`'s own module doc comment, confirming the copy-paste
origin.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl_text_msdf --test doc_comment_test
```
**Expected** (fixed): the module doc comment describes MSDF text rendering, not a UBO-driven
triangle. **Actual** (pre-fix): the leading doc-comment line described a plain UBO triangle,
contradicting the crate's own following MSDF description.

## Root Cause

Stale doc comment copy-pasted from `uniforms_ubo`'s `main.rs` file header and never updated to
describe this crate's own actual MSDF text-rendering implementation.

## Why Not Caught

No test tied the module doc comment's factual claim to the crate's actual rendering technique --
the crate's own immediately-following correct MSDF description masked the contradiction from a
quick skim.

## Fix Applied (2026-08-18)

Removed the stale, copy-pasted UBO-triangle sentence, leaving the crate's own accurate MSDF
description intact as the sole module doc comment. Added `tests/doc_comment_test.rs`:
`include_str!` + substring assertion confirming the doc comment no longer references a UBO-driven
triangle and does describe MSDF rendering.

## Verification

- **Pre-fix (RED):** reverted the doc comment to include the stale UBO-triangle sentence; new test
  failed (stale claim detected).
- **Post-fix (GREEN):** `cargo test -p minwebgl_text_msdf` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p minwebgl_text_msdf` and
  `cargo clippy --all-targets --all-features -p minwebgl_text_msdf -- -D warnings` both clean.

## Generalized Version

Copy-pasted boilerplate doc comments across sibling example crates drift silently once one
crate's actual behavior diverges from the source it was copied from -- when a file's own doc
comment contains two internally-inconsistent descriptions (a stale leading one and an accurate
following one), that inconsistency itself is a strong signal worth checking directly rather than
assuming only the more prominent/first description matters.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-336 after a fresh on-disk collision scan. |
