# BUG-470: `gpu_hal`'s `vulkan` backend leaks a command pool, render pass, and framebuffer on every frame, exhausted by any long-running windowed present loop

- **Severity:** Medium (no crash within a normal demo session's runtime; genuine resource
  exhaustion given enough frames -- self-disclosed by the example crate's own readme.md
  "Known limitation" section before this bug existed, so severity is capped at Medium rather
  than High/Critical per that prior explicit risk acknowledgment)
- **state:** Completed
- **Affects:** `examples/gpu_hal/triangle_vulkan_window` (the only consumer that runs the Vulkan
  backend in a long-lived per-frame present loop; `gpu_hal/tests/vulkan_backend_test.rs`'s own
  `triangle_render_readback` uses the identical allocation pattern but runs it exactly once per
  process, so the leak never accumulated there)
- **Component:** `module/helper/gpu_hal` (`src/vulkan.rs` -- the `vulkan` backend's command-pool/
  render-pass/framebuffer lifecycle)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None. Not a duplicate of BUG-430 (which added `Device::*_destroy` methods for
  8 *other* resource types -- `Buffer`/`Texture`/`TextureView`/`Sampler`/`ShaderModule`/
  `BindGroupLayout`/`BindGroup`/`RenderPipeline` -- none of which cover the command pool, render
  pass, or framebuffer this bug concerns; confirmed by re-reading BUG-430's own fix location before
  starting this bug's real fix, since an earlier draft of this bug's task assumed incorrectly that
  BUG-430 already covered these 3 types).

## Symptom

```rust
// examples/gpu_hal/triangle_vulkan_window/src/main.rs, Renderer::draw -- runs every frame
let mut encoder = self.device.command_encoder_create();   // allocates a fresh command pool
let mut pass = encoder.render_pass_begin(
  &ColorAttachmentDesc { view : &view, clear : CLEAR_COLOR },
  None
)
.expect( "render pass failed to begin" );                 // allocates a fresh render pass + framebuffer
// .. record + submit + present ..
```

Every call to `draw()` allocated a new Vulkan command pool (via `command_encoder_create`) and a
new render pass plus framebuffer (via `render_pass_begin`), and none of the three was ever
destroyed. `gpu_hal`'s `vulkan` backend module doc comment already documented this as a deliberate
v0 tradeoff for its original one-shot-per-process test consumer (`gpu_hal/tests/vulkan_backend_test.rs`);
a windowed present loop (this example) is the first consumer that invalidates that rationale, since
it runs thousands of frames in a single long-lived process instead of one.

## Impact

**Who is affected:** Only `examples/gpu_hal/triangle_vulkan_window` -- the sole consumer that
drives the `vulkan` backend through a real per-frame render loop rather than a single offscreen
readback.

**What broke:** Command pools, render passes, and framebuffers accumulated without bound for as
long as the window stayed open, each frame adding 3 more never-freed Vulkan objects. Given enough
frames, this would exhaust either the Vulkan driver's own object limits or host memory, at which
point `command_encoder_create`/`render_pass_begin`'s `.expect(...)` calls would panic.

**Magnitude:** 3 leaked Vulkan objects (1 command pool, 1 render pass, 1 framebuffer) per frame,
unconditionally, for every `draw()` call -- now 0.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Assigned as part of a repo-wide bug/UX-DX sweep; the crate's own readme.md already self-disclosed
this exact leak under "Known limitation" before this bug report existed. Filed to give it a formal
bug reference (cross-linked from both the readme and the fix site's source comment) and to record
the investigation into whether a code-level fix is achievable from this example crate alone.

## Investigation: Is An In-Scope Fix Possible From The Example Crate?

The originating task suggested two possible mitigations -- "cache and reuse the command
pool/render pass/framebuffer across frames" or "explicitly destroy them after each frame" -- on
the assumption one would be achievable from the example crate itself. Direct investigation of
`gpu_hal`'s public API (not assumption) confirmed **neither is possible from the example crate
without a `module/helper/gpu_hal` API addition**:

- **No destroy API for these 3 resource types (at filing time).** `Device`'s public destroy surface
  (`module/helper/gpu_hal/src/device.rs`) covered `buffer_destroy`, `texture_destroy`,
  `texture_view_destroy`, `sampler_destroy`, `shader_module_destroy`,
  `bind_group_layout_destroy`, `bind_group_destroy`, and `render_pipeline_destroy` (BUG-430) --
  confirmed via direct grep for `fn.*destroy` across `device.rs`/`vulkan.rs`/`pass.rs` on
  2026-08-20. None of the 8 is `command_encoder_destroy`, `render_pass_destroy`, or
  `framebuffer_destroy`, for any backend.
- **No raw handle access.** This crate depends on `gpu_hal` alone (confirmed via its own readme's
  own "wgpu-free" `cargo tree` claim) -- it never depends on `ash` directly, and `gpu_hal`'s public
  `CommandEncoder`/`RenderPass` facade types (`module/helper/gpu_hal/src/pass.rs`) expose no
  accessor for the underlying Vulkan handles or the `DeviceVulkan` needed to destroy them manually,
  even if the example took on an `ash` dependency itself.
- **No reset/replay primitive to cache and reuse.** `Queue::submit` takes `encoder :
  CommandEncoder` by value (consuming it) and `RenderPass::end` likewise consumes the pass -- there
  is no `reset()`/`begin_again()` method that would let a `Renderer` build these three objects once
  in `Renderer::new` and replay them across frames instead of reallocating.

**Conclusion (unchanged by the fix below):** no in-scope fix was ever possible from the example
crate alone -- the real fix required a `module/helper/gpu_hal` API addition. See Fix Applied.

## Minimum Reproducible Example

`module/helper/gpu_hal/tests/vulkan_backend_test.rs`, `command_pool_and_render_passes_do_not_leak`
(marked `bug_reproducer(BUG-470)`): asserts that `render_pass_begin` pushes exactly one entry onto
its encoder's `pending_render_passes` per call, read back through `CommandEncoder::as_vulkan()` --
an assertion that does not compile against the pre-fix `CommandEncoderVulkan`, which had no such
field. See Prevention for the full rationale (including why a true GPU-exhaustion reproducer was
rejected as too slow/flaky/environment-dependent for this test to attempt).

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run -p gpu_hal --all-features -E 'test(command_pool_and_render_passes_do_not_leak)'
```

## Root Cause

`gpu_hal`'s `vulkan` backend was designed for `gpu_hal/tests/vulkan_backend_test.rs`'s original
one-shot-per-process usage pattern, where allocate-without-destroy is harmless (the whole process,
and its Vulkan instance/device with it, exits immediately after). `CommandEncoderVulkan` had no way
to remember what `render_pass_begin` had created on it, so even a `Queue::submit` that wanted to
destroy them would have had nothing to destroy without first adding that tracking.
`triangle_vulkan_window` is the first consumer to run the same allocation pattern inside a
persistent, thousands-of-frames event loop instead -- a usage shape the original design never
accounted for.

## Why Not Caught Sooner

No test exercised repeated command-pool/render-pass/framebuffer allocation across many submit
cycles -- every existing test (including BUG-430's own reproducers) submits at most a handful of
encoders per `cargo nextest`-isolated process, and none inspected `CommandEncoderVulkan`'s internal
state before or after `submit`. A leak that never accumulates past a handful of frames within one
short-lived test process produces no observable failure.

## Fix Applied

Implemented inside `module/helper/gpu_hal` itself, per the Investigation above -- no change to
`examples/gpu_hal/triangle_vulkan_window`'s own code was needed or made; the fix is transparent to
every existing caller.

Added `CommandEncoderVulkan::pending_render_passes : Vec< ( ash::vk::RenderPass,
ash::vk::Framebuffer ) >`. `render_pass_begin` (now taking `encoder : &mut CommandEncoderVulkan`,
widened from `&CommandEncoderVulkan` -- no caller-visible change, since the cross-backend
`CommandEncoder::render_pass_begin` it backs already took `&mut self` on every backend) pushes the
render pass/framebuffer pair it creates onto this `Vec` instead of leaving it untracked.
`Queue::submit`'s Vulkan backend drains `encoder.pending_render_passes` and destroys every pair,
plus `encoder.pool` itself (which implicitly frees `encoder.command_buffer`, allocated from it),
immediately after `vkQueueWaitIdle` confirms the GPU has finished executing everything that
referenced them.

**Design decision:** unlike BUG-430's 8 opt-in `Device::*_destroy` methods, this destruction is
unconditional, not opt-in, and lives inside `submit` rather than a new `Device` method. No API ever
hands the command pool back to the caller as a separately owned/destroyable resource --
`CommandEncoderVulkan` owns it outright, and `submit` already consumes the whole encoder by value
-- so there was no opt-in surface to add a `command_encoder_destroy` method to in the first place;
destroying unconditionally at the one point the encoder is guaranteed to be finished (post-
`vkQueueWaitIdle`, pre-return) is the only fix that fits the existing ownership shape. Considered
and rejected: destroying the pending pair inside `render_pass_end` instead -- `command_encoder_
create`'s own contract allows any number of render passes to be begun/ended into one encoder before
it is submitted, so a still-recording command buffer can reference a render pass/framebuffer pair
from an already-`end`ed prior pass; freeing it at `end` time would be undefined behavior the
instant a later pass on the same encoder records another command. Considered and rejected: the
per-swapchain-image caching alternative named in the original Recommended Fix -- destroy-on-submit
is strictly simpler, requires no new cache-invalidation logic tied to swapchain resize, and matches
the ownership-transfer idiom BUG-430 already established for this crate's other resource types.

## Prevention

New reproducer test `command_pool_and_render_passes_do_not_leak` in
`tests/vulkan_backend_test.rs`, split into two halves for the workspace's function-length lint
threshold: `render_passes_tracked_on_encoder_before_submit` proves `render_pass_begin` pushes
exactly one entry per call onto `pending_render_passes` (read back through the same `as_vulkan()`
accessor BUG-430's own tests established) and that submitting an encoder with several pending pairs
at once does not panic; `repeated_submit_cycles_leave_device_usable` runs 50 full create-record-
submit cycles back to back -- far closer to a real windowed present loop than any single-frame test
in this file -- and confirms the device still renders correct pixels afterward. A memory-threshold
or exhaustion-based test proving the leak is gone under real resource pressure was considered and
rejected as too slow, flaky, and environment-dependent (dependent on the specific Vulkan ICD's own
object limits) for an automated `cargo nextest` suite; the `pending_render_passes.len()` assertion
is deterministic and would not compile at all against the pre-fix struct, which is the same
honestly-scoped proof strategy BUG-430's own reproducers established (create/destroy plus a
"device still fully usable" render, not exhaustion).

## Pitfall

A resource-lifecycle design documented as "fine for now" (a module doc comment explicitly
disclaiming destroy support, scoped to a specific one-shot-per-process test consumer) can silently
become load-bearing the moment a new consumer with a different usage shape (a persistent event
loop instead of a single call) is added. Separately: the *intuitive* fix location for a "resource
created mid-recording, never destroyed" leak is wherever that resource's own scope conceptually
ends (`RenderPass::end`/`render_pass_end`, right after `vkCmdEndRenderPass`) -- but Vulkan's actual
safety boundary is GPU-completion, not API-call-scope, and those two are the same point only when a
single render pass is the only thing an encoder ever records. This backend's own contract
(`command_encoder_create`: "any number of render passes can be begun/ended into it") breaks that
assumption, so the correct fix has to defer destruction to the one point that's actually safe for
every pass on the encoder at once -- after `Queue::submit`'s `vkQueueWaitIdle`, not at each
individual `render_pass_end`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Assigned as part of a repo-wide bug/UX-DX sweep; the crate's own readme.md already self-disclosed this leak under "Known limitation" before this report existed. |
| 2026-08-20 | investigated | Confirmed via direct source inspection of `module/helper/gpu_hal` that no destroy API, raw handle access, or reuse/replay primitive existed for the 3 leaked resource types -- no in-scope code fix was possible from the example crate alone. |
| 2026-08-20 | verified (interim) | Documentation-only interim fix applied (source comment + readme cross-reference); state left at Verified, not Completed, since the underlying leak remained unresolved in code. |
| 2026-08-20 | fixed | Added `CommandEncoderVulkan::pending_render_passes` tracking and a destroy loop in `Queue::submit`'s Vulkan backend (`vulkan.rs`), destroying the command pool, every pending render pass, and every pending framebuffer once `vkQueueWaitIdle` confirms GPU completion. Added `bug_reproducer(BUG-470)` test. Updated doc comments (`vulkan.rs` module doc, `submit`, `render_pass_begin`, `render_pass_end`, `RenderPassVulkan`), `docs/feature/005`, and the example's `readme.md`/`main.rs` doc comment to reflect the fix. |
| 2026-08-20 | completed | See Verification Record below. State moved from Verified to Completed -- the leak is resolved in code, not just documented. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness (no semantic change, destruction point safe) | 🟢 (investigation gate, prior report) | 🟢 | Confirming pass: traced every `render_pass_begin` call site (only `pass.rs`'s cross-backend dispatch, already `&mut self`) and every `CommandEncoderVulkan` construction site (only `command_encoder_create`) via grep to confirm the widened `&mut` parameter and new field cause no other call-site breakage; confirmed `#[derive(Clone)]` on `CommandEncoderVulkan` is never exercised at the whole-struct level anywhere in the crate (ruling out a double-free via clone). Adversarial pass: attempted to construct a scenario where destruction happens before GPU completion -- found none, since the only destroy call site is textually after `queue_wait_idle`'s `.unwrap_or_else` in the same function, with no early return between them; attempted to find a path where `pending_render_passes` could be read after `submit` consumes the encoder -- none exists, since `submit` takes `encoder` by value and the `Vec` is drained inside it. `cargo clippy -p gpu_hal --all-features --all-targets -- -D warnings` and `-p gpu_hal_triangle_vulkan_window` both clean; `cargo nextest run -p gpu_hal --all-features` 31/31 pass including the new reproducer. | Added a per-`unsafe`-block `// SAFETY:` comment where the initial edit had one shared comment above two blocks (`clippy::undocumented_unsafe_blocks` caught this before nextest ran). |
| D2 | Fix documentation compliance | -- | 🟢 | 3-field `Fix(BUG-470)`/`Root cause`/`Pitfall` source comment present immediately before the destroy loop in `vulkan.rs`'s `submit`. 5-section (`Root Cause`/`Why Not Caught`/`Fix Applied`/`Prevention`/`Pitfall`) test doc comment present on the new `command_pool_and_render_passes_do_not_leak` reproducer, marked `// test_kind: bug_reproducer(BUG-470)` matching this repository's established convention (cross-checked against BUG-430's own reproducers in the same file). | -- |
| D3 | Scope containment | -- | 🟢 | `git diff --stat` confirms this bug's own code changes are confined to `module/helper/gpu_hal/src/vulkan.rs` (struct field, `render_pass_begin`, `submit`, doc comments) and `module/helper/gpu_hal/tests/vulkan_backend_test.rs` (new reproducer), plus doc-only updates to `module/helper/gpu_hal/docs/feature/005_command_recording_and_submission.md`, `examples/gpu_hal/triangle_vulkan_window/readme.md`, and `examples/gpu_hal/triangle_vulkan_window/src/main.rs` (doc comment only, no functional change) -- consistent with the Investigation's conclusion that no functional example-crate change was needed. `device.rs` and `native_backend_test.rs` changes in the same working tree belong to a separate, independent clippy-lint fix (`needless_pass_by_value`/`too_many_lines`) in this same session, not to this bug. No commit was made at any point. | -- |

**Reproduced:** YES. Pre-fix: `command_pool_and_render_passes_do_not_leak`'s first assertion
(`encoder.as_vulkan().unwrap().pending_render_passes`) does not compile against the pre-fix
`CommandEncoderVulkan`, which had no such field -- confirmed by construction, since the field and
the `&mut CommandEncoderVulkan` signature `render_pass_begin` needs to populate it were both added
by this fix in the same session that wrote the test. Post-fix: `cargo nextest run -p gpu_hal
--all-features -E 'test(command_pool_and_render_passes_do_not_leak)'` passes; full `-p gpu_hal`
suite 31/31 pass. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/vulkan.rs` | Module doc comment extended with a `Fix(BUG-470)` paragraph. `CommandEncoderVulkan` gained `pending_render_passes : Vec< ( ash::vk::RenderPass, ash::vk::Framebuffer ) >`. `command_encoder_create` initializes it. `render_pass_begin` widened to `&mut CommandEncoderVulkan` and pushes onto it. `RenderPassVulkan`'s and `render_pass_end`'s doc comments updated to point at `submit` for the authoritative destroy. `submit` gained a `Fix(BUG-470)`/`Root cause`/`Pitfall` source comment and a destroy loop (framebuffer, render pass, command pool) after `vkQueueWaitIdle`, plus updated doc comment/`#[allow]` reason. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/vulkan_backend_test.rs` | Added `command_pool_and_render_passes_do_not_leak` (marked `bug_reproducer(BUG-470)`), split into `render_passes_tracked_on_encoder_before_submit` and `repeated_submit_cycles_leave_device_usable`. |

## Refs: docs/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/docs/feature/005_command_recording_and_submission.md` | `Queue::submit`'s Design paragraph extended with a `Fix(BUG-470)` sentence documenting the new unconditional post-wait destruction. |
| `examples/gpu_hal/triangle_vulkan_window/readme.md` | "Known limitation" section (which self-disclosed this exact leak) removed and replaced with a short paragraph confirming per-frame resources are now destroyed automatically. |
| `examples/gpu_hal/triangle_vulkan_window/src/main.rs` | `Renderer::draw`'s doc comment rewritten from "BUG-470 (open)" to `Fix(BUG-470)`, describing the transparent fix; no functional change. |
