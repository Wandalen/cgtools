# BUG-207: `Queue::buffer_write`'s native arm performs no validation on misaligned or oversized `data`

- **Severity:** High (panics the whole process on entirely ordinary caller input)
- **state:** Completed
- **Affects:** Every caller of `gpu_hal::Queue::buffer_write` on the native backend --
  concretely `tilemap_renderer`'s native adapter and the crate's own test fixtures.
- **Component:** `module/helper/gpu_hal` (`src/device.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Fifth occurrence of the same unguarded-native-panic-on-bad-input defect
  class in this crate/its siblings: BUG-165 (`minwgpu::Surface::configure`, sibling crate),
  BUG-176 (`gpu_hal::Device::texture_create`), BUG-199 (`gpu_hal::Device::new_native`) and
  BUG-204 (`gpu_hal::Queue::texture_write`) all fixed earlier this session, all in this same
  file. Found via this session's systematic sweep of every remaining native-arm call into a
  `()`-returning `wgpu` API for the same gap, after BUG-204 closed the fourth occurrence.
  Sibling of BUG-208, found in the same sweep.

## Symptom

```rust
// pre-fix -- gpu_hal/src/device.rs, Queue::buffer_write, native arm
Self::Native( queue ) =>
{
  let raw = buffer.expect_native();
  queue.write_buffer( raw, 0, data );
  Ok( () )
}
```

`data`'s length reaches `wgpu::Queue::write_buffer` with zero validation -- neither checked for
`wgpu::COPY_BUFFER_ALIGNMENT` (4-byte) alignment, nor checked against the destination buffer's
own allocated size.

## Impact

**Who is affected:** Every native-backend caller of `buffer_write` -- most directly
`tilemap_renderer`'s native adapter, which uploads caller-supplied vertex/uniform bytes through
this exact call chain. A caller serializing a non-4-byte-aligned struct, or writing into a buffer
sized for a since-shrunk resource, reaches this gap with entirely ordinary input.

**What breaks:** `wgpu::Queue::write_buffer` is documented (`wgpu-30.0.0/src/api/queue.rs`) to
require the write stay fully in-bounds, and -- per wgpu-core's own `validate_write_buffer_impl`
(`wgpu-core-30.0.0/src/device/queue.rs`) -- that `data.len()` be a multiple of
`wgpu::COPY_BUFFER_ALIGNMENT`. The method's signature returns `()`, not `Result`, so a violation
has nowhere to go except wgpu's own internal error sink. `gpu_hal` installs no custom
`on_uncaptured_error` handler anywhere, so the failure reaches wgpu-core's `default_error_handler`
(`wgpu-core-30.0.0/src/backend/wgpu_core.rs`), which panics unconditionally -- the same class as
BUG-165/176/199/204, just a fifth, independent call site.

**Magnitude:** Single chokepoint -- every native-backend `buffer_write` call site was exposed
identically, for two independent violation conditions (misalignment, overrun).

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's systematic sweep of `gpu_hal`'s native backend for the same defect class already
found 4 times in this file (BUG-165/176/199/204: a `wgpu` call documented as fallible but
signature-`()`, unguarded against `wgpu-core`'s panicking default error handler), checking every
remaining native-arm call into a `()`-returning `wgpu` API for a similar gap. `Queue::buffer_write`
(`write_buffer`) was one of two remaining unguarded call sites found (the other is BUG-208).

## Minimum Reproducible Example

```rust
// module/helper/gpu_hal/tests/native_backend_test.rs -- pre-fix, this panics the test process
let ( device, queue, _surface ) = Device::new_native( 64, 64 ).unwrap();
let buffer = device.buffer_create( 8, BufferUsage::UNIFORM | BufferUsage::COPY_DST ).unwrap();
let _ = queue.buffer_write( &buffer, &[ 0u8, 0, 0 ] ); // 3 bytes: not a multiple of 4
// panics inside wgpu-core's default_error_handler with a COPY_BUFFER_ALIGNMENT validation message
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run --all-features --test native_backend_test -E 'test(buffer_write_rejects_misaligned_data) + test(buffer_write_rejects_oversized_data)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `buffer_write`'s native arm never validates `data.len()` against `wgpu::COPY_BUFFER_ALIGNMENT` or the destination buffer's own allocated size, and wgpu's default error sink panics on either violation since no custom handler is installed. | ✅ Root Cause | Confirmed by reading `wgpu::Queue::write_buffer`'s doc comment plus `wgpu-core`'s `validate_write_buffer_impl`/`default_error_handler` source directly -- the identical mechanism already confirmed empirically for BUG-204's sibling `texture_write` defect in the same file. | E1, E2 |
| H2 | The WebGPU and WebGL arms of `buffer_write` have the same gap and need the same fix. | Unproven, not fixed | WebGPU's arm forwards through `minwebgpu`'s own binding into browser-side WebGPU validation (a different runtime and error-reporting path than `wgpu-core`'s Rust-side panic); WebGL's arm goes through `bufferSubData`, already investigated and fixed independently as BUG-200. No concrete evidence of an unguarded *native-style* panic was found for either browser backend for this specific call, so neither was touched here -- scope kept to the one backend with proven evidence. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `wgpu-30.0.0/src/api/queue.rs`, `Queue::write_buffer` doc comment + `wgpu-core-30.0.0/src/device/queue.rs`, `validate_write_buffer_impl` | Write must stay in-bounds and `data.len()` must be a multiple of `COPY_BUFFER_ALIGNMENT` -- yet the method returns `()`. | H1 ✅ |
| E2 | `wgpu-30.0.0/src/backend/wgpu_core.rs`, `default_error_handler` | No custom `uncaptured_handler` is ever installed by this crate (confirmed identically for BUG-204), so any validation `Err` from `write_buffer` falls through to the unconditional `panic!`. | H1 ✅ |
| E3 | `module/helper/gpu_hal/src/device.rs`, `Queue::buffer_write`'s WebGPU/WebGL arms (unchanged, direct read) | WebGPU forwards to `minwebgpu`'s own `Result`-returning binding (browser-side validation, distinct failure surface); WebGL's `bufferSubData` path was already fixed independently at BUG-200. Neither shows the `wgpu-core` panic mechanism this bug is scoped to. | H2 (unproven) |

## Root Cause

```rust
// before -- data.len() reaches wgpu::Queue::write_buffer with zero validation
Self::Native( queue ) =>
{
  let raw = buffer.expect_native();
  queue.write_buffer( raw, 0, data );
  Ok( () )
}
```

`wgpu::Queue::write_buffer` is fallible by its own documentation but `()`-returning by its own
signature -- the same "infallible-looking wrapper around a fallible call" shape as BUG-204's
`write_texture`. No validation existed between the caller's `data` and this call to close that gap.

## Why Not Caught

Every existing call site writes a hardcoded, correctly-sized/aligned payload (e.g. the 16-byte
uniform write in `triangle_render_readback`) -- no test exercised a misaligned or oversized write.

## Fix Location

`module/helper/gpu_hal/src/device.rs`: new private helper `native_buffer_write_len_validate`,
called from `Queue::buffer_write`'s native arm before `wgpu::Queue::write_buffer` -- rejects a
`data` whose length isn't a multiple of `wgpu::COPY_BUFFER_ALIGNMENT`, or that overruns the
destination buffer's own allocated size, with `Error::InvalidInput` (BUG-176's existing variant,
reused).

This is a purely additive, native-arm-only fix. The WebGPU and WebGL arms were investigated (see
Hypothesis H2 / Evidence E3) and found to route through different, already-separately-handled
failure surfaces -- neither was touched, keeping the fix scoped to the one backend with proven
evidence.

## Prevention

2 new tests added, `module/helper/gpu_hal/tests/native_backend_test.rs`:
`buffer_write_rejects_misaligned_data` (3 bytes into an 8-byte buffer -- not 4-byte aligned) and
`buffer_write_rejects_oversized_data` (12 aligned bytes into an 8-byte buffer -- overruns).

## Pitfall

A `wgpu` API can be fallible by its own documentation while still returning `()` at the type
level -- the `Result` never existed to propagate in the first place. This is the same pitfall
BUG-204 already named for `texture_write`; `buffer_write` is an independent occurrence of the
identical shape, confirming this is a systemic pattern in how this crate wraps `wgpu`'s
error-sink-based native APIs, not a one-off.

## Generalized Version

**Broken assumption:** "this backend's write calls either return `Result` or can't fail."

**Confirmed general rule:** When a downstream API's own documentation admits a call "fails" or
requires an invariant (in-bounds, aligned) under some condition, but the binding's Rust signature
returns `()`, that condition is not actually unreachable -- it is a silent failure mode (here, an
unconditional panic via the runtime's default error sink) waiting for validation to be added at
the call site that wraps it. Confirmed independently five times now in this one file.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via this session's systematic sweep of `gpu_hal`'s native backend for the BUG-165/176/199/204 defect class; confirmed via direct `wgpu`/`wgpu-core` source inspection. |
| 2026-08-16 | fixed | Added `native_buffer_write_len_validate`, checking alignment and destination-size overrun, reusing BUG-176's `Error::InvalidInput` variant; 2 new regression tests added; WebGPU/WebGL arms investigated and left unchanged (no proven native-style gap). |
| 2026-08-17 | verified | `cargo nextest run -p gpu_hal --all-features`: 18/18 passed, 0 skipped (log `./-0073_longrun.log`), including both new tests. `cargo clippy -p gpu_hal --all-targets --features native -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote both tests against the exact pre-fix panic path. Adversarial pass specifically checked that the fix rejects BOTH violation conditions independently (misalignment alone, and overrun alone with a still-aligned length) rather than one test accidentally covering both at once -- confirmed the two tests use disjoint byte lengths (3 vs 12) so each isolates one condition. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Correctly identified as the fifth occurrence of the BUG-165/176/199/204 class at an independent call site, not a duplicate; correctly distinguished from BUG-208 (a different panic mechanism -- zero-size `BufferSlice`, not alignment/overrun -- found in the same sweep, filed separately). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct inspection of `wgpu-30.0.0` and `wgpu-core-30.0.0` source, consistent with the mechanism already empirically confirmed for the sibling BUG-204 fix in the same file. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is one validation helper called from one match arm, reusing an existing `Error` variant; WebGPU/WebGL arms deliberately left untouched after investigation found no evidence of the same gap -- no speculative defensive changes. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix lives entirely in `gpu_hal`; no downstream crate changes needed -- callers already propagate `Result` correctly. | — |

**Reproduced:** Confirmed via direct source inspection (`wgpu`/`wgpu-core`) rather than a live
fail-then-pass toggle this round, consistent with BUG-208's sibling verification in the same
sweep -- the panic mechanism is the same one already empirically reproduced for BUG-204 in this
file. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/device.rs` | New `native_buffer_write_len_validate` helper; `Queue::buffer_write`'s native arm calls it before `wgpu::Queue::write_buffer` (full `Fix(BUG-207)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/native_backend_test.rs` | Appended 2 tests: `buffer_write_rejects_misaligned_data`, `buffer_write_rejects_oversized_data`. Reuses the file's existing 5-section doc-comment convention -- no new test file created. |
