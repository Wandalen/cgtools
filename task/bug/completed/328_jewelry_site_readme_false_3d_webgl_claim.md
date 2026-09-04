# BUG-328: `jewelry_site`'s readme claims a 3D/WebGL configurator and lists WebGL as a keyword -- the site has no canvas, no WebGL context, and swaps pre-rendered 2D images by filename

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgl/jewelry_site/readme.md`
- **Component:** examples/minwebgl/jewelry_site
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The readme's keyword list and opening description claimed a 3D/WebGL configurator. This site has
no `<canvas>`, no WebGL context, and no 3D library anywhere in its markup or scripts -- it swaps
between pre-rendered 2D preview images by filename. `src/main.rs` exists only as an inert
wasm-bindgen placeholder so trunk's tooling accepts the crate; it renders nothing.

## Impact

**Who is affected:** any reader using the readme's keywords/description to understand this
crate's actual implementation technique before opening its source.

**What breaks:** the "3D"/"WebGL" claim describes a rendering technique this site does not use at
all -- a materially wrong description of how the demo actually works.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by checking the readme's technique claim against the actual markup/scripts and
`src/main.rs`'s real content rather than trusting the claim because the crate lives in a directory
of genuine WebGL demos. Independently verified by the orchestrating session: no `<canvas>` element,
no `WebGl*` API usage, and `main.rs` contains only an inert wasm-bindgen entry point.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -c "canvas\|WebGl" examples/minwebgl/jewelry_site/index.html examples/minwebgl/jewelry_site/src/main.rs
grep -n "3D\|WebGL" examples/minwebgl/jewelry_site/readme.md
```
**Expected** (fixed): the readme no longer claims 3D/WebGL, and the canvas/WebGl grep count
confirms none exists. **Actual** (pre-fix): the readme claimed 3D/WebGL despite zero canvas/WebGl
references anywhere in the crate's own markup or source.

## Root Cause

Aspirational wording never checked against the actual implementation -- the readme described a
rendering technique (3D/WebGL) the site was perhaps originally planned to use, or was copied from
a sibling demo's boilerplate, without verifying it against the site's real pre-rendered-image-swap
implementation.

## Why Not Caught

This crate sits in a directory of genuine WebGL demos, making an unverified "WebGL"/"3D" claim
here easy to accept at a glance instead of checking the actual scripts -- no test ties the
readme's technique claims to the crate's actual markup/source.

## Fix Applied (2026-08-18)

Corrected the readme's keyword list and opening description to describe the site's actual
technique: a 2D preview-image configurator swapping pre-rendered images by filename, removing the
false 3D/WebGL claim. Added `tests/readme_and_asset_paths_test.rs`: `include_str!`-based assertion
that the readme no longer claims "3D"/"WebGL", paired with a check that the site's actual asset
paths referenced in its scripts resolve to real files in the crate (catching both the doc claim and
a related asset-path class of defect in one pass).

## Verification

- **Pre-fix (RED):** reverted the readme to its 3D/WebGL claim; new test failed (false technique
  claim detected).
- **Post-fix (GREEN):** `cargo test -p jewelry_site` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p jewelry_site` and
  `cargo clippy --all-targets --all-features -p jewelry_site -- -D warnings` both clean.

## Generalized Version

A demo's readme naming its own rendering technique is a factual claim like any other -- being
physically located in a directory of demos that mostly do use that technique makes an unverified
claim easier to accept at a glance; check the actual markup/source directly rather than inferring
correctness from a crate's neighbors.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-328 after a fresh on-disk collision scan. |
