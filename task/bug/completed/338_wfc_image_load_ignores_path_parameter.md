# BUG-338: `wfc`'s `image_load` ignored its own `path` parameter for element creation and DOM id, hardcoding `"tileset.png"` instead -- firing a wasted, 404ing network request on every page load

- **Severity:** Low (wasted network request + wrong initial `src`, self-corrected within the same function before render; no visible symptom for the crate's single current call site)
- **state:** Completed
- **Affects:** `examples/minwebgl/wfc/src/main.rs`
- **Component:** examples/minwebgl/wfc
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`image_load(path: &str, ...)` accepted a `path` parameter but ignored it for two of its three
uses: `image_element_create( "tileset.png" )` used a hardcoded literal instead of `path`, and
`image.set_id( "tileset.png" )` likewise. Only the later `set_src` call actually used a value
derived from `path`.

## Impact

**Who is affected:** every user of this demo -- a wasted network request fires on every page load.

**What breaks:** `image_element_create`'s hardcoded `"tileset.png"` resolves against the app root
(no `static/` prefix the real asset lives under), so the freshly created `<img>` element's initial
`src` points at a URL that 404s. This bad request is immediately overwritten a few lines later by
the correctly-computed `url` built from `path`, so the crate still renders correctly end-to-end --
but a real, failed HTTP request fires every time regardless.

**Entity Scope:** `None` -- confined to this crate's own `image_load` function.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by checking each use of the `path` parameter individually rather than assuming a
function that takes a parameter necessarily uses it everywhere its name suggests. Independently
verified by the orchestrating session: `image_element_create`'s resolution behavior (relative to
app root, no `static/` prefix) confirmed the hardcoded literal produces a 404ing URL distinct from
the correctly-prefixed one the later `set_src` call builds.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p wfc_example --test image_load_path_test
```
**Expected** (fixed): `image_element_create` is called with the `path` parameter.
**Actual** (pre-fix): `image_element_create` was called with the hardcoded literal `"tileset.png"`,
ignoring `path` entirely for that call.

## Root Cause

Literal copy-pasted in place of the parameter that was meant to drive it -- the function's own doc
comment already claimed `path` was "used to construct the image URL", which was only true for the
later `set_src` call, not for `image_element_create` or `set_id`.

## Why Not Caught

No test file existed for this crate -- it is a `fn main()`-only WebGL demo binary with no lib
target, and `image_load` touches `web_sys`/DOM APIs that only run inside an actual browser, so it
cannot be exercised by a plain `cargo test` against real behavior. The single current call site
happens to pass `"static/tileset.png"`, whose filename component coincidentally matches the
hardcoded `"tileset.png"` literal used for the DOM id, masking that specific half of the bug for
that one purpose; the wrong initial `src` request has no visible symptom besides an extra failed
network request, easy to miss without inspecting the browser's network panel.

## Fix Applied (2026-08-18)

`image_element_create` is now called with `path` (the real, resolvable path) instead of the
hardcoded literal. `image.set_id` now derives its id from `path`'s filename component
(`path.rsplit('/').next()`), matching the bare-filename ids `texture_array_prepare`'s
`get_element_by_id` lookups already expect elsewhere in this file. Added
`tests/image_load_path_test.rs`: `include_str!` + substring assertions confirming
`image_element_create` is called with `path` and never with the hardcoded `"tileset.png"` literal.

## Verification

- **Pre-fix (RED):** reverted `image_element_create`'s call to the hardcoded literal; new test
  failed (hardcoded-literal call detected).
- **Post-fix (GREEN):** `cargo test -p wfc_example --test image_load_path_test` -- new test passes;
  `cargo clippy -p wfc_example --all-targets --all-features -- -D warnings` and
  `cargo check --target wasm32-unknown-unknown -p wfc_example` both clean.

## Generalized Version

A parameter that is genuinely used for one of several related purposes (here, `set_src`) reads as
"used" at a glance -- silently ignoring it for the other purposes (element creation, id) hides
easily behind a single call site whose literal happens to coincide with the real value. Check every
use of a parameter individually, not just whether the parameter appears anywhere in the function.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-338 after a fresh on-disk collision scan. |
