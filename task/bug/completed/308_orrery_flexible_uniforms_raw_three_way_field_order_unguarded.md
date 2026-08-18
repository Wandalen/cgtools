# BUG-308: `orrery_flexible`'s `UniformsRaw` struct order, its own `to_bytes()` push order, and `scene_fragment.wgsl`'s `Uniforms` struct order had no 3-way guard tying them together

- **Severity:** Medium (silent GPU-side data corruption risk, not currently manifesting -- all
  three currently agree)
- **state:** Completed
- **Affects:** `examples/orrery/flexible/src/uniforms.rs`
- **Component:** examples/orrery/flexible
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Unlike sibling crate `orrery_webgpu` (BUG-307), which uploads `UniformsRaw` via
`bytemuck::Pod`/`minwebgpu`, this crate's `gpu_hal`-based buffer API takes a raw `&[u8]`
(`Queue::buffer_write`), so its byte layout is produced by an explicit `to_bytes()` walk instead
of a transmute-style derive. This adds a 3rd point of potential divergence: the struct's own
declared field order, `to_bytes()`'s push-call order, and `orrery_webgpu`'s
`shader/scene_fragment.wgsl` `Uniforms` struct order (the same shared shader contract both orrery
crates target) must all three stay in lockstep, but nothing tied them together. A same-shape
field swap on any of the 3 (e.g. `ring_colors`/`ring_params`) compiles cleanly, preserves
`to_bytes()`'s length-only `debug_assert_eq!(.., 704, ..)`, and produces no render-test failure --
only a silently corrupted uniform buffer past the divergence point.

## Impact

**Who is affected:** anyone editing `UniformsRaw`'s declared field order, `to_bytes()`'s push
order, or the shared `Uniforms` WGSL struct in `orrery_webgpu`, without updating the other two in
lockstep.

**What breaks:** a mismatch between any 2 of the 3 silently corrupts every uniform value in the
GPU buffer after the divergence point, undetected by the existing pixel-landmark render tests.

**Entity Scope:** `None` -- all 3 currently agree; this is a structural guard against future
drift, not a currently-manifesting corruption.

## How Discovered

Disclosed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer`
crates (task #183), which also found and fixed the closely-related BUG-307 in the sibling
`orrery_webgpu` crate. Independently verified by reading `UniformsRaw`'s declaration, `to_bytes`'s
push order, and the WGSL `Uniforms` struct, confirming all 3 currently match.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p orrery_flexible --test uniforms_layout_test
```
**Expected** (fixed): `uniforms_raw_order_matches_to_bytes_order_matches_wgsl_order` passes,
confirming all 3 orders match. **Actual** (pre-fix): no test existed at all -- per the fork's own
account, manually transiently swapping the `grid_color`/`grid_params` declaration lines in a
scratch edit compiled, kept `to_bytes()`'s 704-byte length assertion valid (total length is
unchanged by a same-shape swap), and passed both existing render tests (neither sampled pixel
depends on the grid) -- nothing caught it.

## Root Cause

The buffer crosses to the GPU as raw bytes with no per-field validation anywhere, and this
crate's explicit `to_bytes()` walk adds a 3rd independently-maintained ordering (beyond the 2 that
BUG-307's sibling crate already has) with no mechanism tying any of the 3 together.

## Why Not Caught

No test in this crate compared any 2 of the 3 orderings; `native_render_test.rs`/
`vulkan_render_test.rs` only sample 2 landmark pixels (sun-disc center, background corner), so a
same-shape field swap outside those 2 regions compiles cleanly and passes both.

## Fix Applied (2026-08-18)

Added a `Fix(BUG-308)` comment above `UniformsRaw` documenting the 3-way lockstep requirement and
pointing to the new test. Added `tests/uniforms_layout_test.rs`
(`uniforms_raw_order_matches_to_bytes_order_matches_wgsl_order`): `struct_field_names()` (shared
parser with the sibling `orrery_webgpu` test) extracts the struct's declared order and the WGSL
struct's order; a new `to_bytes_field_order()` extracts every `self.<field>` access, in order of
appearance, from `to_bytes()`'s body -- the exact order fields land in the byte buffer,
independent of the struct's own declaration order. Asserts struct-declared order ==
`to_bytes()`-push order == WGSL order, a genuinely more thorough 3-way check than the sibling
`orrery_webgpu` test's 2-way one, reflecting this crate's extra `to_bytes()` indirection.

## Verification

RED proof (per the fork's own account, manually confirmed by transiently swapping the
`grid_color`/`grid_params` declaration lines in a scratch edit of `uniforms.rs` before writing the
fix, then reverting): see Minimum Reproducible Example above.

- **Post-fix (GREEN), independently re-run by the orchestrating session:** `cargo test -p
  orrery_flexible -p orrery_webgpu --tests` (combined `longrun`-detached sweep) →
  `orrery_flexible`: `native_render_test.rs` 1/1, `uniforms_layout_test.rs` 1/1,
  `vulkan_render_test.rs` 0/0 (correctly gated out -- no Vulkan driver in this sandbox), all
  passed. `cargo clippy -p orrery_flexible --all-targets -- -D warnings` → clean (note: this
  crate enforces exactly one backend feature via a `compile_error!` guard, so clippy must run
  with default features only, not `--all-features`).

## Generalized Version

A `#[repr(C)]`-equivalent struct uploaded as raw bytes to a GPU, read back by an
independently-declared shader-side struct, has zero compiler-level linkage between the two --
this is a distinct defect class from ordinary doc drift. When a crate adds its own explicit
byte-serialization walk (rather than a transmute-style derive), that walk becomes a 3rd
independently-maintained ordering needing its own parity check, not just a 2-way one. A
field-order-parity text test (not a runtime pixel test, which only samples a handful of landmark
points) is the proportionate guard.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer` crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); reported via both an `<agent-message from="fork">` cross-session channel and the standard task-notification for the same agent ID (corroborating, confirming genuineness); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (diff read, all 3 orderings compared, test independently re-run via a `longrun`-detached sweep after resolving 2 separate log-auto-discovery collisions) before this report and its real ID were assigned; placeholder replaced with BUG-308 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |
