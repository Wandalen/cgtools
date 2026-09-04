# BUG-162: `adapter_request`/`device_request` panic on ordinary, spec-defined outcomes instead of returning `Result`

- **Severity:** High (violates this crate's own written invariant --
  `docs/invariant/001_result_based_error_handling.md` -- on two reachable, non-exceptional
  outcomes; every real call site is a top-level `app_run`/`setup` entry point, so the panic is
  the whole application crashing, not a recoverable internal state)
- **state:** Completed
- **Affects:** `context::adapter_request`, `context::device_request` -- any caller running in a
  browser with no compatible `GPUAdapter`, or whose device request is rejected
- **Component:** `module/min/minwebgpu` (`src/context.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered in the same review pass as BUG-163 (task #94, `minwebgpu` code
  review), different file, no shared root cause. BUG-164 was discovered as a direct side effect
  of writing this bug's own regression test (the test browser hit an earlier, distinct failure
  one call before `adapter_request`'s own logic was ever reached) -- filed and fixed separately.

## Symptom

```rust
// pre-fix
pub async fn adapter_request() -> web_sys::GpuAdapter
{
  let navigator = navigator();
  let gpu = navigator.gpu();
  let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap(); // panics: never rejects
  adapter.dyn_into().unwrap() // panics if adapter is JS `null` ("no adapter" -- ordinary outcome)
}
```

## Impact

**Who is affected:** Any caller of `adapter_request`/`device_request` (directly, or via the
`setup()` convenience) running in a browser that has WebGPU infrastructure but returns no
compatible adapter (`requestAdapter()` resolves to `null` -- a normal, spec-defined outcome, not
an exception), or whose `requestDevice()` promise is rejected.

**What breaks:** The process panics with an unattributed `Option::unwrap() on a None value` /
cast-failure message, with no indication the crate has a purpose-built error type
(`error::ContextError`) that this exact condition should have routed through.

**Magnitude:** Every real call site in this repo (`hello_triangle`, `orrery/webgpu`,
`deffered_rendering`, `shader_chunks_preview_web`, `gpu_hal::device::new_webgpu`) calls
`adapter_request`/`device_request` as one of the first two lines of its entry point -- there is
no fallback path, so this panic is the entire application failing to start.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Direct source review of `module/min/minwebgpu/src/context.rs` during task #94 (systematic
per-crate bug review). Both functions' bodies end in a bare `.unwrap()` on values the WebGPU
spec documents as having a normal failure outcome (`null` resolution, promise rejection) --
flagged against this crate's own `docs/invariant/001_result_based_error_handling.md`, which
requires exactly this class of outcome to surface as `Result::Err`.

## Minimum Reproducible Example

```bash
cd module/min/minwebgpu && cargo test --target wasm32-unknown-unknown --all-features --test context_adapter_device_request_tests 2>&1 | tail -8
```

**Expected** (post-fix):
```
test tests::adapter_request_returns_result_never_panics_test ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 filtered out
```

**Actual** (pre-fix -- confirmed via real geckodriver execution against this exact test browser,
which has no compatible `GPUAdapter`):
```
panicked at module/min/minwebgpu/src/context.rs:15:36:
called `Option::unwrap()` on a `None` value
Error: some tests failed
```
(This specific pre-fix run surfaced BUG-164's failure first, one call earlier in the same
function chain -- see BUG-164's own MRE for the isolated, single-cause reproduction of
`adapter_request`'s own `null`-adapter panic path.)

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgpu && cargo test --target wasm32-unknown-unknown --all-features --test context_adapter_device_request_tests
# 4 "ok" = fixed; a raw panic/unwrap message = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `adapter_request`'s `.dyn_into().unwrap()` panics when `requestAdapter()` resolves to `null` (no compatible adapter), an ordinary spec-defined outcome. | ✅ Root Cause | Read `src/context.rs`: no `is_null()` check existed before the cast; WebGPU spec documents `requestAdapter()` as resolving (never rejecting) with `null` on "no adapter". | E1 |
| H2 | `device_request`'s outer `.await.unwrap()` panics when `requestDevice()`'s promise is rejected. | ✅ Root Cause | Read `src/context.rs`: the `JsFuture`'s `Result` was unconditionally `.unwrap()`ed with no `map_err`; WebGPU spec documents `requestDevice()` as rejecting (not resolving with a sentinel) on failure. | E1 |
| H3 | Both functions' failure modes share the same shape (a rejected promise), so one shared fix pattern applies to both. | ❌ Falsified | `request_adapter`'s failure is a *resolved* `null`; `request_device`'s failure is a *rejected* promise -- different shapes need different checks (`is_null()` vs `map_err` on the `Result`), not one uniform pattern. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/context.rs` (pre-fix, unedited) | `adapter_request`: `JsFuture::from(gpu.request_adapter()).await.unwrap()` then `adapter.dyn_into().unwrap()` -- no `is_null()` check. `device_request`: `JsFuture::from(adapter.request_device()).await.unwrap()` -- no `map_err`. | H1 ✅, H2 ✅, H3 ❌ |
| E2 | `tests/context_adapter_device_request_tests.rs::adapter_request_returns_result_never_panics_test` (real geckodriver run) | Confirms the post-fix function returns `Result` and completes without panicking against a real, adapter-less test browser. | H1 ✅ |

## Root Cause

```rust
// before
pub async fn adapter_request() -> web_sys::GpuAdapter
{
  let navigator = navigator();
  let gpu = navigator.gpu();
  let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap();
  adapter.dyn_into().unwrap()
}

pub async fn device_request( adapter : &web_sys::GpuAdapter ) -> web_sys::GpuDevice
{
  let device = JsFuture::from( adapter.request_device() ).await.unwrap();
  device.dyn_into().unwrap()
}
```

Neither function distinguished its one ordinary failure mode from the happy path: `requestAdapter()`
communicates "no adapter" through a *resolved* `null`, while `requestDevice()` communicates
rejection through the promise itself -- a uniform `.unwrap()` on the outer `Result` handles
neither correctly.

## Why Not Caught

No existing test called either function at all; every real call site assumed a live, working
WebGPU adapter/device would always be available in the browsers this crate was exercised in
during manual development.

## Fix Location

`module/min/minwebgpu/src/context.rs`, `src/error.rs`.

```rust
// after
pub async fn adapter_request() -> Result< web_sys::GpuAdapter, WebGPUError >
{
  let gpu = gpu_or_unsupported()?; // BUG-164's shared helper
  let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap();
  if adapter.is_null()
  {
    return Err( crate::error::ContextError::NoAdapterAvailable.into() );
  }
  Ok( adapter.dyn_into().unwrap() )
}

pub async fn device_request( adapter : &web_sys::GpuAdapter ) -> Result< web_sys::GpuDevice, WebGPUError >
{
  let device = JsFuture::from( adapter.request_device() )
  .await
  .map_err( | e | crate::error::ContextError::DeviceRequestRejected( format!( "{e:?}" ) ) )?;
  Ok( device.dyn_into().unwrap() )
}
```

Two new `ContextError` variants (`NoAdapterAvailable`, `DeviceRequestRejected`) route each
function's real failure mode as `Result::Err`. `setup()` and all 5 real call sites
(`hello_triangle`, `orrery/webgpu`, `deffered_rendering`, `shader_chunks_preview_web`,
`gpu_hal::device::new_webgpu`) updated to propagate via `?`. The remaining `.unwrap()` on each
function's `dyn_into()` cast stays a documented, unreachable-per-spec panic (the WebGPU spec
guarantees the non-null/non-rejected resolution value casts cleanly).

## Prevention

Added `tests/context_adapter_device_request_tests.rs` (new file): a live test calling
`adapter_request`/`device_request` for real against the test browser (`bug_reproducer(BUG-162)`),
plus 3 pure `Display`/conversion tests for the two new `ContextError` variants.

## Pitfall

A Promise's resolve/reject shape doesn't map 1:1 onto "success/failure" -- `request_adapter`
communicates its one failure mode through a *resolved* `null`, while `request_device`
communicates its failure mode through *rejection*. Each needed its own check matching its actual
signature, not a uniform `.unwrap()` on the outer `Result`.

## Generalized Version

**Broken assumption:** "every WebGPU promise-returning method fails the same way (rejection), so
one `.unwrap()`/`map_err` pattern covers all of them."

**Confirmed general rule:** before writing error handling for any Web/JS API, check its actual
spec-defined resolve/reject contract per method -- some communicate failure via a sentinel
resolved value (`null`), others via rejection. Don't assume uniformity across a family of
similarly-shaped async methods.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via direct source review of `context.rs` during task #94's systematic `minwebgpu` crate review, cross-checked against `docs/invariant/001_result_based_error_handling.md`. |
| 2026-08-16 | fixed | Added `ContextError::NoAdapterAvailable`/`DeviceRequestRejected`; both functions now `Result`-returning; `setup()` and 5 real call sites propagate via `?`. |
| 2026-08-16 | verified | Added `tests/context_adapter_device_request_tests.rs` (4 tests). Real geckodriver execution against a live, adapter-less test browser confirmed no panic. Scoped wasm32 clippy (6 crates) clean; full-workspace `verb/test` (native nextest+doctest+clippy, wasm32 Stage 1+2) clean, 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against the actual `Result`-returning functions; adversarial pass ran the suite for real via geckodriver against a live, WebGPU-adapter-less browser, capturing the genuine pre-fix panic before the fix landed. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-163 (same review batch, different file). Shares `gpu_or_unsupported` with BUG-164 by design (same underlying `navigator.gpu()` access pattern) -- cross-referenced explicitly in both reports, not silently duplicated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading plus the WebGPU spec's own documented resolve/reject contract per method. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Only `adapter_request`/`device_request`'s error handling touched; their happy-path return types (`GpuAdapter`/`GpuDevice`) unchanged. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `minwebgpu` src + test + bug file touched, plus the 5 downstream call sites required to keep the workspace compiling (mechanical `?` addition, no logic change). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix widens two function signatures to `Result`; all call sites are internal to this workspace and were updated in the same change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface beyond the 2 new `ContextError` variants, which belong to the existing error enum's own responsibility. | — |

**Reproduced:** YES -- `adapter_request_returns_result_never_panics_test` was run for real via
geckodriver against this project's actual test browser (confirmed to have no compatible WebGPU
adapter); pre-fix, the equivalent unguarded code panicked with `Option::unwrap() on a None
value` at `context.rs:15:36`. Post-fix, the same live call returns `Err` cleanly. Scoped wasm32
clippy (6 crates) + full-workspace `verb/test` (native + wasm32 Stage 1/2) clean, 0 failures,
2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgpu/src/error.rs` | `ContextError` gained `NoAdapterAvailable`, `DeviceRequestRejected` (and, for BUG-164, `WebGpuUnsupported`). |
| `module/min/minwebgpu/src/context.rs` | `adapter_request`/`device_request` converted to `Result`-returning (full `Fix(BUG-162)` comment); `setup()` propagates via `?`. |
| `examples/minwebgpu/hello_triangle/src/main.rs` | Call site updated to `.await?`. |
| `examples/orrery/webgpu/src/main.rs` | Call site updated to `.await?`. |
| `examples/minwebgpu/deffered_rendering/src/main.rs` | Call site updated to `.await?`. |
| `module/shader/shader_chunks_preview_web/src/main.rs` | Call site updated to `.await?`; also gained an unrelated pre-existing `clippy::too_many_lines` allow, surfaced by this crate's first-ever wasm32 clippy sweep (not part of this bug's root cause). |
| `module/helper/gpu_hal/src/device.rs` | Call site updated to `.await?`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgpu/tests/context_adapter_device_request_tests.rs` | New file: 4 tests (live `bug_reproducer(BUG-162)` + 3 pure `Display`/conversion tests). Live test's match arms additionally accept BUG-164's `WebGpuUnsupported` outcome (this test browser has no WebGPU support at all). |
| `module/min/minwebgpu/tests/readme.md` | New file: Responsibility Table for all 5 test files in this directory (crossed the 3-file threshold). |
