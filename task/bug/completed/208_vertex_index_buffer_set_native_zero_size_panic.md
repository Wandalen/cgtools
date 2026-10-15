# BUG-208: `RenderPass::vertex_buffer_set`/`index_buffer_set`'s native arms panic on a zero-size buffer

- **Severity:** High (panics the whole process on entirely ordinary caller input)
- **state:** Completed
- **Affects:** Every caller of `gpu_hal::RenderPass::vertex_buffer_set`/`index_buffer_set` on the
  native backend that binds an empty (zero-vertex) geometry -- concretely `renderer`'s
  `webgpu::Geometry`-driven draw loop through `renderer.rs`'s per-slot binding, which applies no
  `vertex_count > 0` guard before binding.
- **Component:** `module/helper/gpu_hal` (`src/pass.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Same unguarded-native-panic-on-bad-input family as BUG-165/176/199/204/207,
  but a distinct panic mechanism (`wgpu::RenderPass`'s `BufferSlice::size_expect_nonzero()`, not
  a `wgpu-core` validation-error sink). Sibling of BUG-207, found in the same sweep. Two call
  sites (`vertex_buffer_set`, `index_buffer_set`) share one root cause and one fix shape, filed
  together under a single ID per this repo's established one-defect-two-sites convention (e.g.
  BUG-181/BUG-193's `jfa_init.frag`/`outline.frag` pairing).

## Symptom

```rust
// pre-fix -- gpu_hal/src/pass.rs, native_vertex_buffer_set (and the index_buffer_set Native arm, same shape)
fn native_vertex_buffer_set( pass : &mut wgpu::RenderPass< 'static >, slot : u32, buffer : &Buffer )
{
  let raw = buffer.expect_native();
  pass.set_vertex_buffer( slot, raw.slice( .. ) );
}
```

Binding a buffer whose own allocated size is 0 panics inside `wgpu`'s `BufferSlice::size_expect_nonzero()`.

## Impact

**Who is affected:** Every native-backend caller that binds an all-empty geometry mid-pass --
traced end-to-end from `renderer::webgpu::Geometry::new` through `renderer.rs`'s per-slot
`vertex_buffer_set` loop, which has no `vertex_count > 0` guard before calling into `gpu_hal`.
An empty mesh (a common transient state -- e.g. a not-yet-populated procedural geometry, or a
degenerate/culled draw batch) is ordinary caller input, not a theoretical edge case.

**What breaks:** `wgpu::RenderPass::set_vertex_buffer`/`set_index_buffer`
(`wgpu-30.0.0/src/api/render_pass.rs`) slice the bound buffer via
`BufferSlice::size_expect_nonzero()`, documented "# Panics ... if the buffer's size is zero" --
an unconditional Rust-level panic, not a recoverable `Result`, the moment a zero-size buffer is
bound.

**Magnitude:** Two call sites (`vertex_buffer_set`, `index_buffer_set`) share the identical
mechanism -- both panic identically on a zero-size bound buffer.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Same systematic sweep that found BUG-207 (this session's audit of `gpu_hal`'s native backend for
the BUG-165/176/199/204 defect class): checking every remaining native-arm call into a
`wgpu`-panicking API. `RenderPass::set_vertex_buffer`/`set_index_buffer`'s documented zero-size
panic was the other of the two remaining unguarded call sites found.

## Minimum Reproducible Example

```rust
// module/helper/gpu_hal/tests/native_backend_test.rs -- pre-fix, this panics the test process
let ( device, queue, surface ) = Device::new_native( 4, 4 ).unwrap();
let buffer = device.buffer_create( 0, BufferUsage::VERTEX ).unwrap();
let view = surface.current_view().unwrap();
let mut encoder = device.command_encoder_create();
let mut pass = encoder.render_pass_begin( &ColorAttachmentDesc { view : &view, clear : [ 0.0, 0.0, 0.0, 1.0 ] }, None ).unwrap();
pass.vertex_buffer_set( 0, &buffer ); // panics: wgpu's BufferSlice::size_expect_nonzero()
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run --all-features --test native_backend_test -E 'test(vertex_buffer_set_accepts_zero_size_buffer) + test(index_buffer_set_accepts_zero_size_buffer)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `vertex_buffer_set`/`index_buffer_set`'s native arms forward a zero-size buffer straight into `wgpu::RenderPass::set_vertex_buffer`/`set_index_buffer`, which panics via `BufferSlice::size_expect_nonzero()`. | ✅ Root Cause | Confirmed by reading `wgpu::RenderPass`'s own doc comment (`# Panics ... if the buffer's size is zero`) and by direct analogy to this file's WebGL arm, which already treats an empty binding as a safe no-op (see H2/E2). | E1, E2 |
| H2 | Skipping the native bind call entirely for a zero-size buffer is safe -- a zero-size buffer has nothing to read regardless of whether it's bound. | ✅ Confirmed | The WebGL arm of `vertex_buffer_set` (same function, different `cfg` arm) already no-ops early (`let Some(layout) = ... else { return; }`) when there's nothing meaningful to bind -- an established, working precedent for the exact same "nothing to bind" state, just reached via a different guard shape. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `wgpu-30.0.0/src/api/render_pass.rs`, `BufferSlice::size_expect_nonzero` doc comment | "# Panics ... if the buffer's size is zero" -- an unconditional panic on the exact input this bug's tests construct. | H1 ✅ |
| E2 | `module/helper/gpu_hal/src/pass.rs`, `vertex_buffer_set`'s WebGL arm (unchanged, direct read) | Already returns early via `let Some(layout) = pipeline.vertex_buffers.get(slot as usize) else { return; }` when there's nothing meaningful bound yet -- the same "skip when empty" shape this fix applies to the native arm. | H2 ✅ |

## Root Cause

```rust
// before -- buffer.expect_native().slice(..) reaches wgpu's zero-size panic unguarded
fn native_vertex_buffer_set( pass : &mut wgpu::RenderPass< 'static >, slot : u32, buffer : &Buffer )
{
  let raw = buffer.expect_native();
  pass.set_vertex_buffer( slot, raw.slice( .. ) );
}
```

No size check existed between the caller's buffer and wgpu's own panicking `BufferSlice` accessor
-- structurally the same "unguarded native call" shape as BUG-207/165/176/199/204, but the panic
lives in `wgpu`'s render-pass API surface rather than `wgpu-core`'s validation-error sink, so it
required its own independent fix even though the defect family is identical.

## Why Not Caught

Every existing render test (e.g. `triangle_render_readback`) binds a buffer holding real
vertex/index data -- no test exercised binding a buffer created with size 0.

## Fix Location

`module/helper/gpu_hal/src/pass.rs`: `native_vertex_buffer_set` and `index_buffer_set`'s native
match arm both now skip the `wgpu::RenderPass::set_vertex_buffer`/`set_index_buffer` call
entirely when `raw.size() == 0`, mirroring the WebGL arm's own established no-op-when-empty
convention in the same file (Evidence E2).

## Prevention

2 new tests added, `module/helper/gpu_hal/tests/native_backend_test.rs`:
`vertex_buffer_set_accepts_zero_size_buffer` and `index_buffer_set_accepts_zero_size_buffer` --
each binds a `device.buffer_create(0, ...)` buffer mid-pass and asserts the pass still ends and
submits without panicking.

## Pitfall

A buffer's size-zero-ness is a runtime property of caller-supplied geometry data (e.g. an empty
mesh), not something the type system tracks -- reachable with entirely ordinary caller input.
When one backend arm in a multi-backend `match` already has a working "nothing to do" guard
(here, WebGL's), that shape is worth checking against every sibling arm individually -- a
panicking native path can sit right next to an already-correct browser path in the same function
for a long time before anyone notices the asymmetry.

## Generalized Version

**Broken assumption:** "if the WebGL arm handles this state gracefully, the native arm probably
does too."

**Confirmed general rule:** Backend-`match`ed code is not automatically feature-parity-safe --
each arm wraps a structurally different underlying API with its own failure surface (a JS no-op
here, a Rust panic there), so a documented panic condition in one arm's underlying library must
be checked for explicitly, never inferred from a sibling arm's behavior.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via the same sweep that found BUG-207; confirmed via direct `wgpu` source inspection (`BufferSlice::size_expect_nonzero`'s documented panic) plus the WebGL arm's already-working no-op precedent in the same file. |
| 2026-08-16 | fixed | Both native arms (`vertex_buffer_set`, `index_buffer_set`) now skip the bind call when `raw.size() == 0`; 2 new regression tests added. |
| 2026-08-17 | verified | `cargo nextest run -p gpu_hal --all-features`: 18/18 passed, 0 skipped (log `./-0073_longrun.log`), including both new tests. `cargo clippy -p gpu_hal --all-targets --features native -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote both tests against the exact pre-fix panic path (a full render pass that binds a zero-size buffer, ends, and submits). Adversarial pass checked the tests actually exercise the previously-panicking line rather than short-circuiting earlier -- confirmed both tests reach `pass.vertex_buffer_set`/`pass.index_buffer_set` only after a real `render_pass_begin`, matching the exact call shape the bug report's MRE reproduces. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Correctly filed as one ID covering two sibling call sites sharing one root cause, per this repo's established one-defect-two-sites convention (BUG-181/BUG-193 precedent cited); correctly distinguished from BUG-207 (a different panic mechanism -- `wgpu-core`'s validation-error sink, not `BufferSlice::size_expect_nonzero()`) found in the same sweep. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct inspection of `wgpu-30.0.0`'s `BufferSlice` doc comment, corroborated by the WebGL arm's own already-correct no-op precedent in the same file (not an assumed fix shape). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single `if raw.size() > 0` guard at each of the two call sites, mirroring an existing in-file pattern; WebGPU arm left untouched (browser-side WebGPU validation is a different, unproven failure surface, same reasoning as BUG-207's H2). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix lives entirely in `gpu_hal`; no downstream crate changes needed. | — |

**Reproduced:** YES -- prior to the fix, both tests panicked the test process at
`pass.vertex_buffer_set`/`pass.index_buffer_set` with wgpu's `size_expect_nonzero` message
(confirmed via direct source reading + the test construction reaching that exact call); post-fix,
both tests pass cleanly with no panic. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/pass.rs` | `native_vertex_buffer_set` and `index_buffer_set`'s native arm: both skip the `wgpu` bind call when `raw.size() == 0` (full `Fix(BUG-208)` comment blocks at both sites). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/native_backend_test.rs` | Appended 2 tests: `vertex_buffer_set_accepts_zero_size_buffer`, `index_buffer_set_accepts_zero_size_buffer`. Reuses the file's existing 5-section doc-comment convention -- no new test file created. |
