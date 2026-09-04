# BUG-164: `adapter_request`/`preferred_format` panic at the FFI boundary when the browser has no WebGPU support at all

- **Severity:** High (a raw, uncaught JS `TypeError` at the wasm-bindgen FFI boundary -- not
  even a Rust panic -- reachable in any browser lacking WebGPU support entirely, which as of
  this crate's own target date is still not universal; upstream of BUG-162's own fix)
- **state:** Completed
- **Affects:** `context::adapter_request`, `context::preferred_format` -- any caller running in a
  browser where `navigator.gpu` itself is `undefined` (WebGPU unsupported, not merely
  unavailable-on-this-system)
- **Component:** `module/min/minwebgpu` (`src/context.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered as a direct side effect of running BUG-162's own regression test
  for the first time -- the test browser's `navigator.gpu` is itself `undefined`, hitting this
  bug's failure mode one call before BUG-162's own `is_null()`/`map_err` logic was ever reached.
  Shares no root cause with BUG-162 (that bug is about interpreting a *present* `Gpu` object's
  responses; this bug is about the `Gpu` object itself never being validated as present), but
  shares this bug's new `gpu_or_unsupported` helper as their common fix point.

## Symptom

```rust
// pre-fix
pub async fn adapter_request() -> Result< web_sys::GpuAdapter, WebGPUError >
{
  let navigator = navigator();
  let gpu = navigator.gpu(); // returns JS `undefined`, typed as `Gpu`, if unsupported
  let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap();
  // ^ throws at the FFI boundary before this line ever resolves: "can't access property
  //   'requestAdapter', arg0 is undefined" -- not a Rust panic, an uncaught JS TypeError
  ...
}
```

## Impact

**Who is affected:** Any caller of `adapter_request` or `preferred_format` running in a browser
with no WebGPU support at all (`navigator.gpu` is `undefined`, not a `Gpu` object) -- as of this
crate's target date, WebGPU support is not universal across deployed browsers (varies by
browser, version, and platform), so this is a realistically reachable production condition, not
only a test-environment artifact.

**What breaks:** The process crashes with a raw, uncaught JS `TypeError` at the wasm-bindgen FFI
boundary -- a strictly worse failure mode than a Rust panic (no Rust stack unwinding, no
`# Panics` doc contract, an error message naming an internal generated shim function rather than
anything in this crate).

**Magnitude:** `adapter_request` is the first async call in every real call site's setup
sequence; `preferred_format` is independently reachable out of that sequence too (the WebGPU
spec defines `getPreferredCanvasFormat()` as a standalone capability query, not dependent on a
live adapter/device) -- both are one-call-deep, unconditional crashes with no existing guard.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, not speculative: surfaced by BUG-162's own regression test
(`adapter_request_returns_result_never_panics_test`) on its first real run via geckodriver. The
failure was a raw FFI error naming `web_sys::features::gen_Gpu::Gpu::request_adapter`, invoked
from `minwebgpu::context::private::adapter_request` -- one call *before* BUG-162's own
`is_null()` check could ever run, meaning the receiver of `.request_adapter()` (the `gpu`
variable, i.e. `navigator.gpu()`'s return value) was itself `undefined`, not a `Gpu` object.
Confirmed by reading the full JS stack trace: `"can't access property 'requestAdapter', arg0 is
undefined"` names `arg0` (the call's receiver) as undefined, not the call itself throwing.

## Minimum Reproducible Example

```bash
cd module/min/minwebgpu && cargo test --target wasm32-unknown-unknown --all-features --test webgpu_unsupported_tests 2>&1 | tail -6
```

**Expected** (post-fix):
```
test tests::preferred_format_returns_result_never_panics_test ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out
```

**Actual** (pre-fix -- confirmed via real geckodriver execution, this project's actual test
browser having no WebGPU support at all):
```
imported JS function that was not marked as `catch` threw an error: can't access property
"requestAdapter", arg0 is undefined
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgpu && cargo test --target wasm32-unknown-unknown --all-features --test webgpu_unsupported_tests
# 3 "ok" = fixed; a raw FFI "arg0 is undefined" error = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `navigator.gpu()`'s return value is itself JS `undefined` in a WebGPU-unsupported browser, and no code anywhere validates this before calling a method on it. | ✅ Root Cause | Read the full JS stack trace: `arg0` (the receiver of `.request_adapter()`) is named as undefined, not the property access `navigator.gpu` itself throwing -- confirms `navigator.gpu()` executed and returned `undefined`, typed unsafely as `Gpu` by the `web_sys` binding. | E1 |
| H2 | This is purely a test-environment artifact (this specific headless Firefox/geckodriver setup), not a realistically reachable production condition. | ❌ Falsified (as sole explanation) | WebGPU support varies across real, currently-deployed browsers/versions/platforms as of this crate's target date -- the same `undefined`-property condition is reachable in any such browser, not only this test harness; treated as a real, fixable defect rather than a test-only quirk. | — (design judgment, not evidence-falsifiable) |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/min/minwebgpu/-0002_longrun.log` (real geckodriver run, captured before this fix) | Full JS stack trace: `"can't access property \"requestAdapter\", arg0 is undefined"`, originating in `web_sys::features::gen_Gpu::Gpu::request_adapter`, called from `minwebgpu::context::private::adapter_request::{closure#0}`. | H1 ✅ |
| E2 | `tests/webgpu_unsupported_tests.rs::preferred_format_returns_result_never_panics_test` (real geckodriver run, post-fix) | Confirms `preferred_format` returns `Err(ContextError::WebGpuUnsupported)` cleanly against the same WebGPU-less test browser, no FFI panic. | H1 ✅ |

## Root Cause

```rust
// before -- both functions call navigator.gpu() unconditionally and immediately invoke a
// method on the result, with no check that navigator.gpu is actually present
pub async fn adapter_request() -> Result< web_sys::GpuAdapter, WebGPUError >
{
  let navigator = navigator();
  let gpu = navigator.gpu(); // JS `undefined`, typed as `Gpu`, if browser has no WebGPU support
  let adapter = JsFuture::from( gpu.request_adapter() ).await.unwrap(); // FFI panic here
  ...
}

pub fn preferred_format() -> GpuTextureFormat
{
  let navigator = navigator();
  navigator.gpu().get_preferred_canvas_format() // identical unchecked-receiver panic risk
}
```

`web_sys::Navigator::gpu()` is a raw, unchecked property getter -- it returns whatever JS value
is actually present, even `undefined`, typed as `Gpu` regardless of whether the browser supports
the feature at all. Nothing in this crate validated the getter's result before using it.

## Why Not Caught

No existing test called `adapter_request` or `preferred_format` in a WebGPU-less browser before
this session's own first-ever live geckodriver execution of `minwebgpu`'s test suite (writing
BUG-162's regression test was what first exercised this path for real).

## Fix Location

`module/min/minwebgpu/src/context.rs`, `src/error.rs`.

```rust
// after -- shared helper checks navigator.gpu()'s result via JsValue::is_undefined before use
fn gpu_or_unsupported() -> Result< web_sys::Gpu, WebGPUError >
{
  let gpu = navigator().gpu();
  if AsRef::< wasm_bindgen::JsValue >::as_ref( &gpu ).is_undefined()
  {
    return Err( crate::error::ContextError::WebGpuUnsupported.into() );
  }
  Ok( gpu )
}
```

`adapter_request` now calls `gpu_or_unsupported()?` instead of `navigator().gpu()` directly.
`preferred_format` converted to `Result< GpuTextureFormat, WebGPUError >`, calling the same
helper -- it had the identical unconditional-`navigator.gpu()` panic risk, independently
reachable since callers may query the preferred format before ever calling `adapter_request`
(the WebGPU spec defines `getPreferredCanvasFormat()` as a standalone capability query). `setup()`
and all 5 real call sites updated to propagate `preferred_format()`'s new `?`.

## Prevention

Added `tests/webgpu_unsupported_tests.rs` (new file, `bug_reproducer(BUG-164)`): a live test
calling `preferred_format` for real against the test browser (which genuinely has no WebGPU
support, making this the most direct, single-cause reproducer available), plus 2 pure
`Display`/conversion tests for the new `ContextError::WebGpuUnsupported` variant. BUG-162's own
existing live test additionally widened to accept `WebGpuUnsupported` as a valid outcome, since
that is what this test browser now genuinely, cleanly returns.

## Pitfall

`web_sys` types a property getter like `Navigator::gpu()` as non-`Option` even when the
underlying browser feature is experimental/optional -- the binding itself won't tell you the
feature is absent, it just returns `undefined` typed as if it were the real object. Callers must
feature-detect explicitly (`JsValue::is_undefined`) before use; don't assume a `web_sys` getter's
non-`Option` return type implies the underlying JS property is guaranteed present.

## Generalized Version

**Broken assumption:** "a `web_sys` binding's non-`Option` return type means the underlying
browser feature is always present at runtime."

**Confirmed general rule:** for any experimental/optional Web API surfaced by `web_sys` as a
directly-typed (non-`Option`) getter, explicitly feature-detect via `JsValue::is_undefined` (or
equivalent) before invoking any method on the result -- the binding's Rust type signature carries
no runtime guarantee the browser actually implements the feature.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered as a side effect of BUG-162's own regression test's first real geckodriver run; root-caused by reading the full JS FFI stack trace, confirming `navigator.gpu()`'s return value (not the property access itself) was the undefined receiver. |
| 2026-08-16 | fixed | Added shared `gpu_or_unsupported` helper and `ContextError::WebGpuUnsupported`; `adapter_request` and `preferred_format` (converted to `Result`-returning) both route through it; `setup()` and 5 real call sites propagate via `?`. |
| 2026-08-16 | verified | Added `tests/webgpu_unsupported_tests.rs` (3 tests); widened BUG-162's existing live test to accept the new outcome. Real geckodriver execution against the genuinely WebGPU-less test browser confirmed no panic (17/17 tests passing workspace-wide in `minwebgpu`). Scoped wasm32 clippy (6 crates) clean; full-workspace `verb/test` (native + wasm32 Stage 1/2) clean, 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote a live test against the fixed `preferred_format`; adversarial pass re-read the original raw FFI stack trace (captured pre-fix, in `-0002_longrun.log`) line by line to confirm `arg0` (not the `.gpu()` access itself) was the undefined value, ruling out a misattributed root cause before committing to this fix's shape. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Explicitly cross-referenced against BUG-162 in both directions (shared `gpu_or_unsupported` helper, BUG-162's test widened to accept this bug's outcome) -- no silent, undocumented coupling. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct JS stack-trace evidence (`arg0 is undefined`) plus a deliberate rejection of the "test-environment-only artifact" alternative explanation (H2), reasoned through explicitly rather than assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | `adapter_request`'s signature unchanged (already `Result` from BUG-162); `preferred_format` widened to `Result` as the minimum correct fix, not a larger refactor. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `minwebgpu` src + 2 test files + bug file touched, plus the same 5 downstream call sites BUG-162 already required (mechanical `?` addition only). | — |
| D7 | Crate Locality | 🟢 | 🟢 | All 5 call sites for `preferred_format` were already identified and touched by BUG-162's own fix pass; no new call sites discovered. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | `gpu_or_unsupported` is a single-purpose private helper (feature-detection only), not exposed via `mod_interface!`. | — |

**Reproduced:** YES -- pre-fix, `adapter_request`/`preferred_format` both crashed with a raw,
uncaught JS `TypeError` ("arg0 is undefined") when run for real via geckodriver against this
project's actual WebGPU-less test browser; captured verbatim in `-0002_longrun.log`. Post-fix,
both return `Err(ContextError::WebGpuUnsupported)` cleanly against the same browser -- confirmed
via `preferred_format_returns_result_never_panics_test` and the widened
`adapter_request_returns_result_never_panics_test`. Scoped wasm32 clippy + full-workspace
`verb/test` (native + wasm32 Stage 1/2) clean, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgpu/src/error.rs` | `ContextError` gained `WebGpuUnsupported`. |
| `module/min/minwebgpu/src/context.rs` | New private `gpu_or_unsupported` helper (full `Fix(BUG-164)` comment); `adapter_request` and `preferred_format` (now `Result`-returning) both route through it; `setup()` propagates via `?`. |
| `examples/minwebgpu/hello_triangle/src/main.rs` | `preferred_format()` call site updated to `?`. |
| `examples/orrery/webgpu/src/main.rs` | `preferred_format()` call site updated to `?`. |
| `examples/minwebgpu/deffered_rendering/src/main.rs` | `preferred_format()` call site updated to `?`. |
| `module/shader/shader_chunks_preview_web/src/main.rs` | `preferred_format()` call site updated to `?`. |
| `module/helper/gpu_hal/src/device.rs` | `preferred_format()` call site updated to `?`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgpu/tests/webgpu_unsupported_tests.rs` | New file: 3 tests (live `bug_reproducer(BUG-164)` + 2 pure `Display`/conversion tests). |
| `module/min/minwebgpu/tests/context_adapter_device_request_tests.rs` | BUG-162's live test's match arms widened to also accept `WebGpuUnsupported` as a valid, non-panicking outcome. |
| `module/min/minwebgpu/tests/readme.md` | New file: Responsibility Table for all 5 test files in this directory (crossed the 3-file threshold). |
