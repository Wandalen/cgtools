# BUG-307: `orrery_webgpu`'s `UniformsRaw` struct had no guard tying its Rust field order to `scene_fragment.wgsl`'s independently-declared `Uniforms` struct order

- **Severity:** Medium (silent GPU-side data corruption risk, not currently manifesting -- both
  sides happen to already agree)
- **state:** Completed
- **Affects:** `examples/orrery/webgpu/src/uniforms.rs`
- **Component:** examples/orrery/webgpu
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`uniforms.rs`'s `#[repr(C)]` `UniformsRaw` struct is uploaded to the GPU as raw bytes via
`bytemuck::Pod`, and `shader/scene_fragment.wgsl`'s independently-declared `Uniforms` struct reads
those same bytes back through its own field list. Nothing in the codebase compared the two field
orders -- a reorder on either side compiles cleanly and produces no runtime error; every field
after the divergence point would be silently read as the wrong value (e.g. `disc_params` bytes
interpreted as `ring_colors`), corrupting the rendered frame with no panic and no validation
failure to catch it. The existing render tests only sample 2 landmark pixels (sun-disc center,
background corner), so a same-shape field swap outside those 2 regions would pass every existing
test.

## Impact

**Who is affected:** anyone editing either `UniformsRaw`'s field list or
`scene_fragment.wgsl`'s `Uniforms` struct without also updating the other.

**What breaks:** a field reorder, insertion, or removal on either side compiles cleanly and
silently corrupts every uniform value after the divergence point in the rendered frame, with no
compiler error, no panic, and no existing test catching it.

**Entity Scope:** `None` -- both sides currently agree; this is a structural guard against future
drift, not a currently-manifesting corruption.

## How Discovered

Disclosed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer`
crates (task #183). Independently verified by reading both the Rust `UniformsRaw` struct and the
WGSL `Uniforms` struct in full and confirming they currently match field-for-field (26 fields,
`time`/`seed`/`node_count`/`grid_density` through `resolution`).

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p orrery_webgpu --test uniforms_layout_test
```
**Expected** (fixed): `uniforms_raw_field_order_matches_wgsl_uniforms_struct` passes, confirming
both structs' field order match. **Actual** (pre-fix): no test existed at all -- a manually
introduced field swap (e.g. `disc_params`/`ring_colors`) compiled cleanly and passed every
pre-existing test in this crate.

## Root Cause

`UniformsRaw` crosses the Rust/WGSL boundary as raw `Pod` bytes with no per-field validation on
either side -- the two structs are declared completely independently, and nothing in the build or
test pipeline ever compared their field orders.

## Why Not Caught

No test in this crate ever inspected `UniformsRaw`'s field order against the WGSL struct it's
paired with; the existing render tests only sample 2 landmark pixels, neither of which depends on
most of the struct's fields.

## Fix Applied (2026-08-18)

Added a `Fix(BUG-307)` comment above `UniformsRaw` documenting the lockstep requirement and
pointing to the new test. Added `tests/uniforms_layout_test.rs`
(`uniforms_raw_field_order_matches_wgsl_uniforms_struct`): a `struct_field_names()` text parser
(shared shape between Rust's and WGSL's `field_name : type,` spaced-colon style in this codebase)
extracts both structs' declared field order and asserts they match exactly, plus a sanity check
that the parser found the full ~27-field struct rather than stopping early.

## Verification

RED proof (per the fork's own account, manually confirmed by transiently swapping two field lines
in a scratch copy of `uniforms.rs` before writing the fix, then reverting): with the pre-fix
codebase, the swap compiled and passed every existing test in the crate -- nothing caught it. This
test closes that gap.

- **Post-fix (GREEN), independently re-run by the orchestrating session:** `cargo test -p
  orrery_flexible -p orrery_webgpu --tests` (combined `longrun`-detached sweep) →
  `orrery_webgpu`: `scene_test.rs` 2/2, `shader_source_test.rs` 6/6, `uniforms_layout_test.rs`
  1/1, all passed. `cargo clippy -p orrery_webgpu --all-targets --all-features -- -D warnings` →
  clean.

## Generalized Version

A `#[repr(C)]` struct uploaded as raw `Pod` bytes to a GPU, read back by an independently-declared
shader-side struct, has zero compiler-level linkage between the two -- this is a distinct defect
class from ordinary doc drift. A field-order-parity text test (not a runtime pixel test, which
only samples a handful of landmark points) is the proportionate guard: cheap, deterministic, and
covers every field rather than the 1-2 the render tests happen to sample.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer` crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); reported via both an `<agent-message from="fork">` cross-session channel and the standard task-notification for the same agent ID (corroborating, confirming genuineness); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (diff read, both structs' current field order compared field-by-field, test independently re-run via a `longrun`-detached sweep after resolving 2 separate log-auto-discovery collisions) before this report and its real ID were assigned; placeholder replaced with BUG-307 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |
