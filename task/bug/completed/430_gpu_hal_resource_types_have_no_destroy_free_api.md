# BUG-430: `gpu_hal` resource types have no destroy/free API -- WebGL and Vulkan backends leak every `Buffer`/`Texture`/`Sampler`/etc. by design

- **Severity:** Medium (no crash, no data corruption, every existing test still passes -- but a
  real-time consumer allocating per-frame transient GPU resources through the WebGL or Vulkan
  backend had no way to free any of them early, and would exhaust GPU memory over a long-running
  session; mitigated for consumers that only allocate resources once at startup and reuse them for
  the process lifetime)
- **state:** Completed
- **Affects:** Every consumer of `gpu_hal`'s WebGL or Vulkan backend that allocates
  `Buffer`/`Texture`/`TextureView`/`Sampler`/`ShaderModule`/`BindGroupLayout`/`BindGroup`/
  `RenderPipeline` more than once per process lifetime (e.g. per-frame or per-scene-load
  allocation) -- `renderer`/`tilemap_renderer` (L3, the crate's own documented consumers per
  `docs/layer/002_l1_gpu_hal.md`) and anything built on them. WebGPU and native `wgpu` consumers
  were never affected for 6 of the 8 types (GC/`Drop`-managed in both underlying APIs); `Buffer`
  and `Texture` specifically also gain an *earlier* free option on WebGPU/native, where before this
  fix the only way to reclaim their memory sooner than a full `Device` drop was `wgpu`'s own
  internal `Drop`, which this crate's resource wrapper types never exposed.
- **Component:** `module/helper/gpu_hal` (`src/device.rs`, `src/vulkan.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- first resource-lifecycle defect filed against `gpu_hal`.

## Symptom

```rust
// pre-fix -- src/resource.rs / src/vulkan.rs / src/webgl.rs
// grep -rn "impl Drop\|fn delete\|fn destroy\|fn free\|fn release" module/helper/gpu_hal/src
// -- zero matches anywhere in the crate.
```

`gpu_hal` had a full resource-CREATE API (`buffer_create`, `texture_create`, `sampler_create`,
`shader_module_create`, `bind_group_layout_create`, `bind_group_create`, `render_pipeline_create`)
but no destroy/free counterpart on any of the 4 backends. Every one of the 8 resource types
(`Buffer`, `Texture`, `TextureView`, `Sampler`, `ShaderModule`, `BindGroupLayout`, `BindGroup`,
`RenderPipeline`) had zero `impl Drop` and the crate exposed zero `delete`/`destroy`/`free`/
`release` method anywhere. The WebGL backend leaked every one of these unconditionally, with no
doc-comment disclosure at all. The Vulkan backend's own module doc comment *did* disclose the leak
("nothing in this module frees a resource early; the entire allocation ... is only reclaimed when
`vkDestroyDevice` runs"), but that disclosure never reached the public `Device::*_create` methods'
own doc comments in `device.rs` -- a caller reading only the public API surface had no way to
discover the leak short of independently reading `vulkan.rs`'s internals -- and no escape-hatch
method existed to free anything early on any backend.

## Impact

**Who is affected:** Any consumer of the WebGL or Vulkan backend that allocates these resource
types more than once per process lifetime. `docs/layer/002_l1_gpu_hal.md` names `renderer` and
`tilemap_renderer` (L3) as this crate's own documented consumers -- a scene that streams new
textures/buffers as it loads, or a UI that rebuilds bind groups per frame, would accumulate
unrecoverable GPU memory on either backend for the life of the process.

**What breaks:** No crash and no data corruption -- this is a resource-exhaustion defect, not a
correctness one. A long enough session (many resource-churning frames, or many scene loads) would
eventually exhaust GPU memory and start failing subsequent `*_create` calls with a driver-level
out-of-memory error, but no test in this crate's existing suite runs long enough, or churns enough
resources, to hit that ceiling -- see Why Not Caught.

**Consumer audit:** `gpu_hal` itself has no direct GPU-resource-churning call sites in its own
tests today (every existing test allocates a small, fixed resource set once per test process,
matching the "allocate once at startup" pattern this bug does *not* affect) -- the risk is
downstream, in `renderer`/`tilemap_renderer` consumers this crate does not control, which is
exactly why the fix adds the capability rather than trying to audit every current and future
consumer's own allocation pattern.

**Magnitude:** 8 resource types × 4 backends = 32 dispatch arms across 8 new `Device::*_destroy`
methods, plus 8 new backend-level Vulkan functions -- see Fix Location.

**Entity Scope:** None -- a code-level API-surface gap.

## How Discovered

Found during a repo-wide bug/UX-defect sweep of `gpu_hal` (not a dedicated resource-lifecycle
audit) -- prompted by checking whether a full resource-CREATE API had a matching destroy/free
counterpart on every backend. `grep -rn "impl Drop\|fn delete\|fn destroy\|fn free\|fn release"
module/helper/gpu_hal/src` returned zero matches; cross-checked against `vulkan.rs`'s own module
doc comment, which independently confirmed the Vulkan backend's leak was already known and
documented internally, but never surfaced to the public `device.rs` API or given an escape hatch.

## Minimum Reproducible Example

```rust
// module/helper/gpu_hal/tests/{native,vulkan}_backend_test.rs, resource_destroy_methods_do_not_panic
let ( device, .. ) = Device::new_vulkan( 8, 8 ).expect( ".." );
let buffer = device.buffer_create( 16, BufferUsage::UNIFORM ).expect( ".." );
// pre-fix: no method existed to free `buffer` early -- the only release path
// was dropping the whole `Device`, which also destroys every other resource
// still alive on it.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run -p gpu_hal --all-features -E 'test(resource_destroy_methods_do_not_panic)'
```

## Root Cause

The crate's v0 scope shipped resource creation before resource destruction ever became a concrete,
exercised need. Every existing test in this crate runs as an isolated, short-lived `cargo nextest`
process -- one process per test -- so a per-resource leak never accumulates across a run and
produces no observable failure; nothing in the existing suite ever needed to free a resource before
process exit, so nothing ever surfaced the gap.

## Why Not Caught

No test exercised resource teardown at all, only creation -- every existing test allocates a small,
fixed resource set once per test process and lets process exit reclaim everything, which is
indistinguishable from correct behavior under `cargo nextest`'s one-process-per-test isolation. The
Vulkan backend's own module doc comment disclosed the leak in prose, but no test asserted anything
about it, and the disclosure was never linked from the public `device.rs` surface a consumer would
actually read.

## Fix Location

`module/helper/gpu_hal/src/vulkan.rs`: added a new "Resource destruction" section (lines
1594-1721) with 8 `pub fn *_destroy` functions -- `buffer_destroy`, `texture_destroy`,
`texture_view_destroy`, `sampler_destroy`, `shader_module_destroy`, `bind_group_layout_destroy`,
`bind_group_destroy`, `render_pipeline_destroy` -- each issuing the real `vkDestroy*`/
`vkFreeMemory` call(s) for that type, exported via `mod_interface!`. Module doc comment (lines
26-37) extended to disclose the new API.

`module/helper/gpu_hal/src/device.rs`: added the matching 8 `Device::*_destroy` methods (lines
1154-1433), each dispatching per-backend -- a real call where the backend needs one (WebGL
`gl.delete_*`, Vulkan `vkDestroy*`/`vkFreeMemory`), and a documented, mismatch-checked no-op where
the backend's own GC/`Drop` already reclaims the resource (WebGPU always; native `wgpu` for every
type except `Buffer`/`Texture`, which also gained their own real `.destroy()` call). Every one of
the 8 `Device::*_create` methods got a new "Resource Lifetime" doc section documenting exactly
which backends leak/reclaim/no-op and pointing forward to its matching `*_destroy` method.

**Design decision:** `Device::*_destroy( &self, resource : T )` methods returning `()`, consuming
`resource` by value -- not `impl Drop` on the resource types themselves. Considered and rejected:
most Vulkan wrapper structs (`BufferVulkan`, the raw `Sampler`/`ShaderModule` handles,
`BindGroupLayoutVulkan`, `BindGroupVulkan`, `RenderPipelineVulkan`, `TextureViewVulkan`) and most
WebGL wrapper structs (`BufferWebGl`, `TextureWebGl`, the raw `WebGlSampler` handle,
`RenderPipelineWebGl`) carry no device/context handle of their own -- only `TextureVulkan` does. A
working `Drop` impl would need a device/context clone field added to roughly 10 structs across
`vulkan.rs`/`webgl.rs`, widening every one of them and touching every construction call site,
purely to support a destructor. Dispatching through `Device` instead needs zero struct changes --
`self` already holds the backend context every destroy call needs, exactly mirroring this crate's
own pre-existing `Queue::buffer_write`/`texture_write` dispatch idiom, and the Vulkan arms reuse the
same match-self-then-match-owned-resource-by-value pattern already established by `Queue::submit`/
`vulkan_queue_submit`. Every backend's destroy operation is also provably infallible per its own
spec (`vkDestroy*`/`vkFree*` are void-returning; `wgpu`'s and WebGPU's `.destroy()` return
`()`/`undefined`; WebGL's `gl.delete*` calls return `undefined`), so these methods return `()`
rather than `Result< (), Error >` -- no new `Error` variant needed. Consuming `resource` by value
is a deliberate safety margin beyond what any single backend strictly requires: since none of these
types carry a `Drop` impl, a caller holding onto a stale handle after an explicit destroy could
otherwise pass it to another HAL call and reach the driver with an already-freed handle -- taking
ownership here makes that a compile error instead of a runtime hazard.

**Per-type × per-backend cleanup matrix** (🔴 real leak pre-fix / now freed by `*_destroy`; ⚪ never
leaked -- GC/`Drop`-managed, `*_destroy` is a documented, mismatch-checked no-op):

| Type | WebGPU | WebGL | Native (`wgpu`) | Vulkan |
|------|--------|-------|------------------|--------|
| `Buffer` | ⚪ (`.destroy()`, now callable early too) | 🔴 `gl.delete_buffer` | ⚪ (`.destroy()`, now callable early too) | 🔴 `vkDestroyBuffer`+`vkFreeMemory` |
| `Texture` | ⚪ (`.destroy()`, now callable early too) | 🔴 `gl.delete_texture` | ⚪ (`.destroy()`, now callable early too) | 🔴 `vkDestroyImage`+`vkFreeMemory` |
| `TextureView` | ⚪ | ⚪ (alias/backbuffer, never independently deletable) | ⚪ | 🔴 `vkDestroyImageView` |
| `Sampler` | ⚪ | 🔴 `gl.delete_sampler` | ⚪ | 🔴 `vkDestroySampler` |
| `ShaderModule` | ⚪ | ⚪ (no GL object until pipeline link) | ⚪ | 🔴 `vkDestroyShaderModule` |
| `BindGroupLayout` | ⚪ | ⚪ (CPU-only entry list) | ⚪ | 🔴 `vkDestroyDescriptorSetLayout` |
| `BindGroup` | ⚪ | ⚪ (entries are clones owned elsewhere) | ⚪ | 🔴 `vkDestroyDescriptorPool` (implicitly frees its one set) |
| `RenderPipeline` | ⚪ | 🔴 `gl.delete_program`, conditional on `Rc::strong_count() == 1` (shared with any `RenderPass` currently bound to it) | ⚪ | 🔴 `vkDestroyPipeline`+`vkDestroyPipelineLayout` |

`wgpu` 30.0.0's own explicit-`.destroy()` surface was confirmed by direct source grep
(`~/.cargo/registry/.../wgpu-30.0.0/src/api/`, `pub fn destroy` matched only `buffer.rs`,
`texture.rs`, `external_texture.rs`, `query_set.rs`, `device.rs`) -- not assumed from memory.

## Prevention

Two new reproducer tests, `resource_destroy_methods_do_not_panic` -- one in
`tests/native_backend_test.rs`, one in `tests/vulkan_backend_test.rs` -- each creating one resource
of every type, destroying each through its new `Device::*_destroy` method, then running a full
render + submit + readback to confirm the `Device` is still completely usable afterward (proving no
destroy call corrupted internal state). The Vulkan variant is this crate's only test giving
`TextureView`/`Sampler`/`ShaderModule`/`BindGroupLayout`/`BindGroup`/`RenderPipeline` genuine
per-type teardown coverage, since all 8 of its destroy calls issue a real `vkDestroy*` -- the
native variant's calls are no-ops for 6 of the 8 types (`wgpu`'s own `Drop` already does the work),
so it mainly exercises the real `Buffer`/`Texture` `.destroy()` path plus confirms every method is
at least callable and panic-free for every type on that backend.

## Pitfall

An API surface that lets a caller allocate GPU resources but never free them looks complete from
the type signatures alone -- `Result< Buffer, Error >` gives no hint that the only way to release
the allocation was dropping the whole `Device`. A backend's own internal doc-comment disclosure
(the Vulkan module doc's leak note, already present pre-fix) is not a substitute for that
disclosure reaching the *public* API surface a consumer actually reads (`device.rs`'s own
`*_create` doc comments) -- and is not a substitute for an actual escape hatch existing at all. Any
future resource type added to `resource.rs` needs its own `Device::*_destroy` method added in the
same change, not as a follow-up; a `match` arm with an empty no-op body compiles and passes clippy
with no test at all, so the reproducer tests above deliberately exercise every type, including the
backends where destroying it is a pure no-op, rather than only the types with real work to do.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide `gpu_hal` bug/UX-defect sweep; no prior BUG-NNN reference, no destroy API of any kind existed. |
| 2026-08-20 | fixed | Added 8 `Device::*_destroy` methods (`device.rs`) dispatching to 8 new Vulkan-backend functions (`vulkan.rs`) plus WebGL/WebGPU/native per-backend handling; added "Resource Lifetime" doc sections to all 8 `*_create` methods; added 2 reproducer tests. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | API correctness & dispatch completeness | -- | 🟢 | Confirming pass: every one of the 8 types × 4 backends was checked against `resource.rs`'s actual enum/derive definitions (Copy vs. non-Copy, boxed vs. plain) before choosing deref-copy vs. by-value-move dispatch per type -- not assumed from the crate's general pattern. `cargo check -p gpu_hal --all-features` (native) and `cargo check -p gpu_hal --target wasm32-unknown-unknown --features webgl,webgpu` both clean; `cargo nextest run -p gpu_hal --all-features` 30/30 pass, including both new reproducer tests; `cargo doc --no-deps -p gpu_hal --all-features` with `RUSTDOCFLAGS="-D warnings"` clean (validates every new intra-doc link). Adversarial pass: found and fixed 3 real gaps before considering this done -- (a) 6 of 8 methods' no-op arms silently accepted a resource built by *any* backend instead of panicking on cross-backend mismatch like every other dispatch method in this file (`Queue::submit`, the `expect_*` accessor family) -- fixed by adding a `resource.expect_<backend>();` check to every previously-bare `{}` arm; (b) 5 methods that can panic on cross-backend mismatch were missing the `# Panics` doc section the workspace's `clippy::missing_panics_doc` pedantic lint (`Cargo.toml`: `pedantic = { level = "warn", priority = -1 }`) requires -- added to all 8; (c) `cargo clippy -p gpu_hal --all-features --all-targets -- -D warnings` could not be run to completion -- blocked 3 separate times across this session by a pre-existing, unrelated `missing_panics_doc` compile error in `ndarray_cg`, a dependency being actively edited by a concurrent sub-agent's own math-crate bug-fix sweep (confirmed out of this bug's scope: the failure is in `ndarray_cg`'s own source, not in anything this fix touches). | 6 no-op arms hardened to check backend match before returning; 8 `# Panics` doc sections added |
| D2 | Fix documentation compliance | -- | 🟢 | 3-field `Fix(BUG-430)`/`Root cause`/`Pitfall` source comment present at the top of the new section in both `vulkan.rs` (module doc, lines 26-37) and `device.rs` (lines 1103-1152, immediately before the 8 new methods). 5-section (`Root Cause`/`Why Not Caught`/`Fix Applied`/`Prevention`/`Pitfall`) test doc comment present on both new reproducer tests, each marked `// test_kind: bug_reproducer(BUG-430)` matching this repository's established convention -- cross-checked against 20+ existing examples (`grep -rn "bug_reproducer" --include="*.rs"`) before use, not invented. | -- |
| D3 | Scope containment | -- | 🟢 | `git status --short module/helper/gpu_hal/ task/` confirms this session's own edits are confined to exactly 4 files: `module/helper/gpu_hal/src/device.rs`, `module/helper/gpu_hal/src/vulkan.rs`, `module/helper/gpu_hal/tests/native_backend_test.rs`, `module/helper/gpu_hal/tests/vulkan_backend_test.rs`, plus this new bug report. The large number of additional `task/` modifications visible in that same `git status` output belong to concurrent sub-agents working other crates in this repository-wide sweep (separately-filed `BUG-419`..`BUG-469` reports, task state transitions) -- not caused by this fix; no other crate's source was edited, and no commit was made at any point. | -- |

**Reproduced:** YES -- `grep -rn "impl Drop\|fn delete\|fn destroy\|fn free\|fn release"
module/helper/gpu_hal/src` returned zero matches pre-fix, confirming the missing API surface
directly rather than inferring it; post-fix the same grep matches all 16 new functions (8 in
`device.rs`, 8 in `vulkan.rs`), and both new `resource_destroy_methods_do_not_panic` tests pass.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/vulkan.rs` | Module doc comment extended (lines 26-37) to disclose the new destroy API. New "Resource destruction" section (lines 1594-1721): `buffer_destroy`, `texture_destroy`, `texture_view_destroy`, `sampler_destroy`, `shader_module_destroy`, `bind_group_layout_destroy`, `bind_group_destroy`, `render_pipeline_destroy`, each issuing real `vkDestroy*`/`vkFreeMemory` calls; all 8 exported via `mod_interface!`. |
| `module/helper/gpu_hal/src/device.rs` | `Fix(BUG-430)`/`Root cause`/`Pitfall` source comment plus design-rationale block added (lines 1103-1152). 8 new `Device::*_destroy` methods added (lines 1154-1433) dispatching per-backend, each with a `# Panics` doc section. 8 existing `Device::*_create` methods (`buffer_create`, `buffer_init_create`, `texture_create`, `sampler_create`, `shader_module_create`, `bind_group_layout_create`, `bind_group_create`, `render_pipeline_create`) got new "Resource Lifetime" doc sections documenting per-backend leak/reclaim/no-op behavior and pointing to the matching `*_destroy` method. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/native_backend_test.rs` | Added `resource_destroy_methods_do_not_panic` (line 692, marked `bug_reproducer(BUG-430)`, line 656): creates one resource of every type, destroys each via its new `Device::*_destroy` method, then re-runs a full render + submit + readback to confirm the device remains usable. |
| `module/helper/gpu_hal/tests/vulkan_backend_test.rs` | Added `resource_destroy_methods_do_not_panic` (line 431, marked `bug_reproducer(BUG-430)`, line 386): identical structure to the native variant, but every one of the 8 destroy calls exercises a real `vkDestroy*`/`vkFreeMemory` call on this backend, giving the 6 GC-managed-elsewhere types their only real per-type teardown coverage in this crate. |
