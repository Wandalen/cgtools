# BUG-326: `hexagonal_grid`'s `pathfind_demo` uploads to uniform name `"u_mvp"`, which the shader never declares -- a silent WebGL no-op masked only by an earlier draw call setting the real uniform first

- **Severity:** Medium (silently masked today, but a real broken uniform-upload call site)
- **state:** Completed
- **Affects:** `examples/minwebgl/hexagonal_grid/src/main.rs`
- **Component:** examples/minwebgl/hexagonal_grid
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`pathfind_demo`'s zoom-uniform upload called `hex_shader.uniform_upload("u_mvp", scale.as_slice())`
-- `"u_mvp"` is not declared anywhere in `main.vert`/`main.frag`. WebGL silently no-ops an unknown
uniform-name upload rather than erroring.

## Impact

**Who is affected:** dormant today -- masked because the obstacle-drawing block earlier in the
same frame already sets the correct `"u_zoom"` uniform to this same value, so `pathfind_demo`'s own
draw call happens to render correctly by inheriting the prior call's already-correct GPU state.

**What breaks:** `pathfind_demo`'s own zoom-uniform upload does nothing -- if the obstacle-drawing
block is ever removed, reordered, or given a different zoom value, `pathfind_demo`'s rendering
would silently use a stale/wrong zoom with no error.

**Entity Scope:** `None` -- confined to this crate's own uniform-upload call site.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by cross-checking every `uniform_upload` call site's name string against the shader
source's actual declared uniforms rather than assuming a plausible-looking name is correct.
Independently verified by the orchestrating session: `main.vert`/`main.frag` declare `u_zoom`, not
`u_mvp`; the obstacle-drawing block's own call correctly uses `"u_zoom"`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n 'uniform_upload( "u_zoom"' examples/minwebgl/hexagonal_grid/src/main.rs
grep -c 'uniform_upload( "u_mvp"' examples/minwebgl/hexagonal_grid/src/main.rs
```
**Expected** (fixed): `pathfind_demo`'s zoom upload uses `"u_zoom"`, and the `"u_mvp"` count is 0.
**Actual** (pre-fix): `pathfind_demo` uploaded to `"u_mvp"`, a name absent from both shader files.

## Root Cause

Stale/typo'd uniform name at this specific call site -- `main.vert`/`main.frag`'s actual declared
name is `u_zoom`, matching the correct call already present in this file's obstacle-drawing block;
never caught since WebGL doesn't error on an unknown uniform-name upload.

## Why Not Caught

WebGL uniform-location lookups fail silently for an unknown name -- `gl.get_uniform_location`
simply returns no location, and an upload against a missing location is a no-op with no console
warning or error by default; nothing in this crate's own test coverage exercised the call site's
name against the shader source.

## Fix Applied (2026-08-18)

Corrected `pathfind_demo`'s zoom-uniform upload from `"u_mvp"` to `"u_zoom"`, matching the shader's
actual declared uniform and the sibling obstacle-drawing block's own correct call. Added a new
`#[test]` to the crate's existing `tests/basic.rs`: `include_str!`-based structural assertion that
every `uniform_upload("...")` call-site name in `main.rs` appears as an actual `uniform` 
declaration in `main.vert`/`main.frag`.

## Verification

- **Pre-fix (RED):** reverted `pathfind_demo`'s upload to `"u_mvp"`; new test failed (call-site
  name not found in shader declarations).
- **Post-fix (GREEN):** `cargo test -p hexagonal_grid` -- all tests (existing `basic.rs` coverage
  plus the new assertion) pass; `cargo check --target wasm32-unknown-unknown -p hexagonal_grid`
  and `cargo clippy --all-targets --all-features -p hexagonal_grid -- -D warnings` both clean.

## Generalized Version

WebGL silently no-ops an `uniform_upload` call against an unknown uniform name -- a wrong name at
one call site can be invisibly masked by a different call site (earlier in the same frame, in this
case) coincidentally setting the same GPU state to the same value first. Cross-check every
uniform-upload call site's literal name string against the shader source's actual declarations
directly, rather than trusting that a rendering demo "looking correct" proves every individual
upload call is doing real work.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-326 after a fresh on-disk collision scan. |
