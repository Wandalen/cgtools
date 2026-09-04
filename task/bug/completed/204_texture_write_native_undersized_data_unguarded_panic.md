# BUG-204: `Queue::texture_write`'s native arm performs no validation on undersized `data`

- **Severity:** High (panics the whole process on entirely ordinary caller input)
- **state:** Completed
- **Affects:** Every caller of `gpu_hal::Queue::texture_write` on the native backend --
  concretely `tilemap_renderer`'s native adapter (`adapters/native.rs:174`, `assets_load`, every
  bitmap asset load) and the crate's own test fixtures.
- **Component:** `module/helper/gpu_hal` (`src/device.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Fourth occurrence of the same unguarded-native-panic-on-bad-input defect
  class in this crate/its sibling: BUG-165 (`minwgpu::Surface::configure`, sibling crate),
  BUG-176 (`gpu_hal::Device::texture_create`, this file) and BUG-199
  (`gpu_hal::Device::new_native`, this file) all fixed earlier this session. `texture_write` is a
  fourth, independent call site into a fallible-but-`()`-returning `wgpu` API, discovered as
  Finding 3 of task #121's `gpu_hal` review (deferred pending investigation, since the original
  review had only Low-Medium confidence pending confirmation of the actual panic-vs-silent-noop
  behavior).

## Symptom

```rust
// pre-fix -- gpu_hal/src/device.rs, Queue::texture_write, native arm
Self::Native( queue ) =>
{
  let raw = texture.expect_native();
  let width = raw.width();
  let height = raw.height();
  let bytes_per_row = width * raw.format().block_copy_size( None )
  .ok_or_else( || Error::Unsupported( format!( "{:?} has no portable CPU-side texel layout", raw.format() ) ) )?;

  queue.write_texture
  (
    wgpu::TexelCopyTextureInfo { texture : raw, mip_level : 0, origin : wgpu::Origin3d::ZERO, aspect : wgpu::TextureAspect::All },
    data,
    wgpu::TexelCopyBufferLayout { offset : 0, bytes_per_row : Some( bytes_per_row ), rows_per_image : Some( height ) },
    wgpu::Extent3d { width, height, depth_or_array_layers : raw.depth_or_array_layers() }
  );
  // `data`'s length reaches `write_texture` with zero validation
  Ok( () )
}
```

`data`'s length is never checked against the destination region's required byte count before
reaching `wgpu::Queue::write_texture` -- a plain `&[u8]` with no compile-time link to the
texture it's written into.

## Impact

**Who is affected:** Every native-backend caller of `texture_write` -- most directly
`tilemap_renderer`'s native adapter (`module/helper/tilemap_renderer/src/adapters/native.rs:174`,
`assets_load`), which converts caller-supplied image bytes via `to_rgba8(bytes, *format)` and
writes the result into a texture sized from the same asset's declared `width`/`height`. This call
site already does everything right -- `assets_load` returns `Result<(), RenderError>` and both
the preceding `texture_create` and this `texture_write` are `.map_err(...)?`-propagated -- but a
malformed or truncated asset (corrupt bitmap bytes, a format/dimensions mismatch from a hand-edited
scene file, or any latent bug in `to_rgba8` producing fewer than `width * height * 4` bytes) still
reached an unguarded panic underneath a `Result`-returning call chain.

**What breaks:** `wgpu::Queue::write_texture` is documented (`wgpu-30.0.0/src/api/queue.rs`) to
"fail... if `data` is too short", but the method's signature returns `()`, not `Result` -- the
failure has nowhere to go except wgpu's own internal error sink. `gpu_hal` never installs a custom
`on_uncaptured_error` handler anywhere, so the failure reaches wgpu-core's `default_error_handler`
(`wgpu-core-30.0.0/src/backend/wgpu_core.rs`), which panics unconditionally: "Handling wgpu errors
as fatal by default" -- taking down the entire process, exactly like BUG-165/176/199's `Extent3d`
panics, just via a different validation path inside wgpu-core (`validate_linear_texture_data`
instead of the zero-size early-out).

**Magnitude:** Single chokepoint -- every native-backend `texture_write` call site was exposed
identically.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Surfaced as Finding 3 of task #121's dedicated review pass of `gpu_hal` (this session, Low-Medium
confidence pending confirmation): "`Queue::texture_write` has no validation that `data`'s length
matches the texture's expected byte size." Investigated after BUG-199/BUG-200 (the review's other
two findings) were both closed. Confirmed as a real defect -- specifically a panic, not a silent
no-op like BUG-200 -- by reading `wgpu`'s and `wgpu-core`'s own source directly: `Queue::write_texture`'s
doc comment states it fails on short `data`; its backend dispatch (`wgpu_core.rs::write_texture`)
routes any `Err` from wgpu-core's `queue_write_texture` through `handle_error_nolabel`, which --
absent a custom `uncaptured_handler` (none is installed anywhere in this crate) -- falls through to
`default_error_handler`'s unconditional `panic!`.

## Minimum Reproducible Example

```rust
// module/helper/gpu_hal/tests/native_backend_test.rs -- pre-fix, this panics the test process
let ( device, queue, _surface ) = Device::new_native( 64, 64 ).unwrap();
let texture = device.texture_create( &TextureDesc { size : [ 2, 2, 1 ], format : TextureFormat::Rgba8Unorm, usage : TextureUsage::COPY_DST } ).unwrap();
let _ = queue.texture_write( &texture, &[ 0u8, 0, 0, 255 ] ); // 4 bytes into a 16-byte region
// panics inside wgpu-core's default_error_handler:
// "wgpu error: Validation Error ... Copy at offset 0 for 16 bytes would end up
//  overrunning the bounds of the Source buffer of size 4"
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_hal && cargo nextest run --all-features --test native_backend_test -E 'test(texture_write_rejects_undersized_data)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `texture_write`'s native arm never validates `data.len()` against the destination region's required byte count, and wgpu's default error sink panics on this exact validation failure since no custom handler is installed. | ✅ Root Cause | Confirmed by reading `wgpu::Queue::write_texture`'s doc comment plus `wgpu-core`'s `write_texture`/`handle_error_or_return_handler`/`default_error_handler` source directly, then reproducing the exact predicted panic message empirically. | E1, E2, E3 |
| H2 | The WebGPU and WebGL arms of `texture_write` have the same gap and need the same fix. | ❌ Falsified (WebGL) / Unproven, not fixed (WebGPU) | WebGL's `tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array` binding is `#[wasm_bindgen(catch)]`-wrapped (unlike BUG-200's `bufferSubData`, which had no catch mechanism at all) and already `.map_err(...)?`-propagated in this same function -- an undersized-data JS exception, if thrown, already surfaces as `Err`. WebGPU's arm already forwards `minwebgpu`'s own `Result` via `?`; no concrete evidence of an unguarded panic or silent drop was found for either browser backend, so neither was touched -- scope kept to the one backend with proven evidence. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `wgpu-30.0.0/src/api/queue.rs`, `Queue::write_texture` doc comment | "This method fails if `size` overruns the size of `texture`, or if `data` is too short" -- yet the method returns `()`. | H1 ✅ |
| E2 | `wgpu-30.0.0/src/backend/wgpu_core.rs:633-696`, `ErrorSinkRaw::handle_error_or_return_handler` / `default_error_handler` | No custom `uncaptured_handler` is ever installed by this crate, so any validation `Err` from `queue_write_texture` falls through to `default_error_handler`, which unconditionally `panic!`s: `"wgpu error: {err}"`. | H1 ✅ |
| E3 | This session's own empirical revert-and-rerun of `texture_write_rejects_undersized_data` | With the guard temporarily disabled, the test panics with the exact predicted message: `"wgpu error: Validation Error ... In Queue::write_texture ... Copy at offset 0 for 16 bytes would end up overrunning the bounds of the Source buffer of size 4"`. | H1 ✅ |
| E4 | `web-sys-0.3.104/.../gen_WebGl2RenderingContext.rs`, `tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array` | Declared `#[wasm_bindgen(catch, ...)]` -- any JS exception the browser throws for this call is already converted to `Err(JsValue)`, and `texture_write`'s WebGL arm already `.map_err(...)?`-propagates it (pre-existing code, confirmed unchanged). Structurally different from BUG-200's `buffer_sub_data_with_i32_and_u8_array`, which returns bare `()` with no catch mechanism to propagate anything through. | H2 (WebGL) ❌ |

## Root Cause

```rust
// before -- data.len() reaches wgpu::Queue::write_texture with zero validation
Self::Native( queue ) =>
{
  let raw = texture.expect_native();
  let width = raw.width();
  let height = raw.height();
  let bytes_per_row = width * raw.format().block_copy_size( None ) /* ... */ ?;
  queue.write_texture( /* ... */, data, /* ... */, wgpu::Extent3d { width, height, depth_or_array_layers : raw.depth_or_array_layers() } );
  Ok( () )
}
```

`wgpu::Queue::write_texture` is fallible by its own documentation but `()`-returning by its own
signature -- the same "infallible-looking wrapper around a fallible call" shape as BUG-200's
`bufferSubData`, except here the failure surfaces as an unconditional panic (wgpu-core's default
error sink) rather than a silent no-op (WebGL's unread `getError()` queue). No validation existed
between the caller's `data` and this call to close that gap.

## Why Not Caught

`texture_write_readback` (the crate's one existing `texture_write` test) only ever wrote
exactly-sized data (`64 * 64 * 4` bytes for a `64×64` `Rgba8Unorm` texture) -- no existing test
exercised an undersized write on any backend.

## Fix Location

`module/helper/gpu_hal/src/device.rs`, `Queue::texture_write`'s native arm: computes the
destination region's required byte count from the same `bytes_per_row` / `height` /
`depth_or_array_layers` the arm already derives for the write call itself, and rejects a shorter
`data` with `Error::InvalidInput` (BUG-176's existing variant, reused) before calling
`wgpu::Queue::write_texture`.

This is a purely additive, native-arm-only fix. The WebGPU and WebGL arms were investigated (see
Hypothesis H2 / Evidence E4) and found to already propagate their own underlying errors correctly,
or to have no concrete evidence of a gap -- neither was touched, keeping the fix scoped to the one
backend with proven evidence, consistent with BUG-200's own precedent of a backend-specific fix
where only one backend was actually broken.

## Prevention

1 new test added, `module/helper/gpu_hal/tests/native_backend_test.rs`:
`texture_write_rejects_undersized_data` (asserts `Err(Error::InvalidInput(_))` when writing 4
bytes into a `2×2` `Rgba8Unorm` texture that requires 16).

## Pitfall

A `wgpu` API can be fallible by its own documentation while still returning `()` at the type
level -- the `Result` never existed to propagate in the first place, so `?`/`.map_err(...)?` at
every real call site (as `tilemap_renderer`'s `assets_load` already correctly does) is powerless
against a defect living one layer further down, inside `gpu_hal` itself. A `Result`-returning
caller signature is only as safe as the narrowest `()`-returning call it wraps.

## Generalized Version

**Broken assumption:** "this backend's write calls either return `Result` or can't fail."

**Confirmed general rule:** When a downstream API's own documentation admits a call "fails" under
some condition but the binding's Rust signature returns `()`, that condition is not actually
unreachable -- it is a silent failure mode (panic via the runtime's default error sink, or a
silently-dropped operation) waiting for validation to be added at the call site that wraps it, the
same way BUG-200 added it for `bufferSubData` and this bug adds it for `write_texture`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced as task #121's Finding 3 (deferred, Low-Medium confidence); investigated and confirmed as a real unguarded native panic via direct `wgpu`/`wgpu-core` source inspection, reserved as BUG-204 (IDs 201-203 already claimed by a concurrent actor's unrelated Vulkan-backend task files). |
| 2026-08-16 | fixed | Added a required-byte-count guard to `texture_write`'s native arm, reusing BUG-176's `Error::InvalidInput` variant; 1 new regression test added; WebGPU/WebGL arms investigated and left unchanged (no proven gap). |
| 2026-08-16 | verified | Empirical fail-then-pass: guard temporarily disabled (`if false && ...`), `cargo nextest run -p gpu_hal --all-features -E 'test(texture_write_rejects_undersized_data)'` confirmed the test FAILS, process panicking with the exact predicted wgpu validation message; guard restored, full scoped suite (`cargo nextest run -p gpu_hal --all-features`) confirmed 10/10 passed. `cargo clippy -p gpu_hal --all-targets --features native -- -D warnings`: clean. `cargo check --target wasm32-unknown-unknown --no-default-features --features "enabled,webgpu,webgl" -p gpu_hal`: clean. `cargo clippy --target wasm32-unknown-unknown -p gpu_hal --all-targets --no-default-features --features "enabled,webgl" -- -D warnings`: clean. `cargo clippy --target wasm32-unknown-unknown -p gpu_hal --all-targets --no-default-features --features "enabled,webgpu" -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the fail-then-pass empirical check (guard disabled -> test panics with the exact predicted wgpu message; guard restored -> 10/10 pass). Adversarial pass specifically distrusted the "panics = bug" claim before writing any fix -- read `wgpu`'s and `wgpu-core`'s own source to confirm the panic path is real and unconditional (no `uncaptured_handler` installed anywhere in this crate) rather than assuming from the doc comment's "fails" wording alone. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Correctly identified as a fourth occurrence of the BUG-165/176/199 unguarded-native-panic class at an independent call site, not a duplicate; correctly distinguished from BUG-200's silent-no-op class (confirmed via direct evidence, not assumed) since the WebGL arm here already has a working catch/propagate path BUG-200's `bufferSubData` never had. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct inspection of `wgpu-30.0.0` and `wgpu-core-30.0.0` source (not just the panic message), corroborated by an empirical reproduction matching the predicted error text exactly. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is one validation guard in one match arm, reusing an existing `Error` variant; WebGPU/WebGL arms deliberately left untouched after investigation found no evidence of a gap there — no speculative defensive changes. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix lives entirely in `gpu_hal`; the one real downstream call site (`tilemap_renderer`'s `assets_load`) audited and confirmed to need no changes — already correctly `.map_err(...)?`-propagates. | — |

**Reproduced:** YES -- pre-fix (guard temporarily disabled), `texture_write_rejects_undersized_data`
panics the test process with wgpu's own validation message; post-fix (guard restored), the same
test passes cleanly with `Err(Error::InvalidInput(_))`. 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/device.rs` | `texture_write`'s native arm: added a required-byte-count guard (`bytes_per_row * height * depth_or_array_layers` vs. `data.len()`) returning `Error::InvalidInput` before calling `wgpu::Queue::write_texture` (full `Fix(BUG-204)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/tests/native_backend_test.rs` | Appended 1 test: `texture_write_rejects_undersized_data`. Reuses the file's existing 5-section doc-comment convention -- no new test file created. |
