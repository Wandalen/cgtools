# BUG-329: `minimize_wasm`'s readme names a link-time-optimization technique the crate never actually enables -- no `[profile.release]` exists anywhere reachable from it

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/minimize_wasm/readme.md`
- **Component:** examples/minwebgl/minimize_wasm
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The readme's introduction named an unconfigured link-time-optimization technique this crate never
actually enables -- no `[profile.release]` section (LTO or otherwise) exists anywhere reachable
from this crate or the workspace root that would apply to it.

## Impact

**Who is affected:** any reader using the readme to understand which binary-size techniques this
crate actually demonstrates.

**What breaks:** the readme lists a technique with no corresponding configuration anywhere in the
build -- a demo whose purpose IS showcasing size-optimization techniques is exactly where a wrong
named technique goes unnoticed, since the demo still visibly "works" (produces a small binary via
its other, real techniques) either way.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by checking the readme's named techniques against the crate's and workspace's actual
`Cargo.toml` profile configuration rather than trusting the prose. Independently verified by the
orchestrating session: no `[profile.release]` block with an `lto` key exists in this crate's
`Cargo.toml` or the workspace root's.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -rn "profile.release\|lto" Cargo.toml examples/minwebgl/minimize_wasm/Cargo.toml
grep -n "wee_alloc\|wasm-opt\|wasm-strip" examples/minwebgl/minimize_wasm/readme.md
```
**Expected** (fixed): the readme names only the techniques the build pipeline actually configures
(`wee_alloc`, `wasm-opt -Os`, `wasm-strip`), and no LTO claim remains. **Actual** (pre-fix): the
readme's introduction named LTO despite no `[profile.release]`/`lto` key existing anywhere in
either `Cargo.toml`.

## Root Cause

Aspirational wording never checked against the actual build pipeline -- the readme's introduction
named a technique that was perhaps planned or used in an earlier revision, without verifying it
against the crate's real, currently-configured build steps.

## Why Not Caught

A demo whose purpose IS showcasing techniques is exactly where a wrong named technique goes
unnoticed, since the demo still visibly "works" (produces a small binary) either way -- no test
tied the readme's named-technique claims to the actual `Cargo.toml` configuration.

## Fix Applied (2026-08-18)

Removed the false LTO claim from the readme's introduction, keeping the three techniques already
correctly named and actually configured: the minimal global allocator (`wee_alloc`), post-build
size optimization (`wasm-opt -Os`), and debug-info stripping (`wasm-strip`). Added
`tests/readme_doc_test.rs`: `include_str!`-based assertion that the readme's named techniques
match what the crate's actual `Cargo.toml`/build configuration provides, and that no LTO claim
remains absent a corresponding `[profile.release]` block.

## Verification

- **Pre-fix (RED):** reverted the readme to include the LTO claim; new test failed (named
  technique with no corresponding build configuration detected).
- **Post-fix (GREEN):** `cargo test -p minwebgl_minimize_wasm` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p minwebgl_minimize_wasm` and
  `cargo clippy --all-targets --all-features -p minwebgl_minimize_wasm -- -D warnings` both clean.

## Generalized Version

A demo's readme naming a build-level optimization technique is a factual claim about the actual
`Cargo.toml`/build pipeline, not just about runtime behavior -- cross-check it against the real
profile configuration rather than trusting the prose, especially for a demo whose entire purpose
is showcasing exactly these techniques, where a wrong or stale one has zero visible symptom.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-329 after a fresh on-disk collision scan. |
