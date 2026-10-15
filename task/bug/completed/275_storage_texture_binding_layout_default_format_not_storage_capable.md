# BUG-275: `StorageTextureBindingLayout::new()`'s default `format` (`Rgba8unormSrgb`) does not support `STORAGE_BINDING` usage, copy-pasted from a sibling texture descriptor's default

- **Severity:** High (unconditional, 100%-reproducible failure of the affected code path, reachable
  simply by following the type's own pre-fix doc comment, which presented the default as safe to
  rely on)
- **state:** Completed
- **Affects:** `minwebgpu`'s `StorageTextureBindingLayout::new()` (also reachable via
  `binding_type::storage_texture_type()`) — any `BindGroupLayoutEntry`/`BindGroupLayoutDescriptor`
  built with a storage-texture binding whose `.format(..)` is never called
- **Component:** `module/min/minwebgpu` (`src/binding_type/storage_texture.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`StorageTextureBindingLayout::new()` defaults its `format` field to
`GpuTextureFormat::Rgba8unormSrgb`, and the struct's own field doc comment documented this as the
intended default ("Defaults to `Rgba8unormSrgb`"). Per the WebGPU spec's texture format capability
table, no `-srgb` texture format supports `STORAGE_BINDING` usage — storage texture reads/writes
operate on raw texel values, with no sRGB transfer function applied, so the spec excludes every
sRGB format from the storage-capable format set. Any caller who builds a storage-texture binding
via `StorageTextureBindingLayout::new()` (or the equivalent `binding_type::storage_texture_type()`)
without also explicitly calling `.format(..)` therefore produces a `GPUBindGroupLayoutEntry` whose
`storageTexture.format` is `"rgba8unorm-srgb"` — a format `GPUDevice.createBindGroupLayout` rejects
with a `GPUValidationError` the moment the layout is actually used to create a real bind group
layout on a real device.

## Impact

**Who is affected:** any `minwebgpu` consumer that constructs a storage-texture binding layout and
relies on the default `format` instead of calling `.format(..)` explicitly — a usage pattern the
type's own pre-fix doc comment actively encouraged by presenting `Rgba8unormSrgb` as a normal,
intended default rather than flagging it as mandatory-to-override.

**What breaks:** the resulting `web_sys::GpuStorageTextureBindingLayout` is well-formed Rust-side
(no panic, no compile error) and survives every conversion in `binding_type.rs` /
`descriptor/bind_group_layout_entry.rs` / `descriptor/bind_group_layout.rs` without incident — the
defect is invisible until the layout reaches a real `GPUDevice.createBindGroupLayout` call, which
then fails with a `GPUValidationError` (browser/WebGPU-implementation-level, not a Rust-level
error this crate can catch or report through its own `Result` types). Every existing test in this
crate that exercises `StorageTextureBindingLayout` explicitly calls `.format(..)` first (see `Why
Not Caught`), so the invalid default has no prior observed failure in this repository's own test
suite — this is an as-yet-unexercised latent defect in real usage, not a currently-failing code
path elsewhere in the workspace.

**Entity Scope:** `None` — source-level default-value defect, not entity directory instances.

## How Discovered

Assigned, as one of 14 parallel bug-scouting forks sweeping `module/min`'s 5 crates for latent
defects, the `minwebgpu` binding-type + descriptor-start file group (12 files, including all 5
parallel `binding_type/*.rs` variant files: `buffer.rs`, `external_texture.rs`, `sampler.rs`,
`storage_texture.rs`, `texture.rs`). The assignment's own domain hint specifically flagged this
"N parallel files for N enum variants" shape as where a copy-pasted field default is most likely to
hide, naming `storage_texture.rs`'s `access`/`format` default as a concrete example to check.
Reading `storage_texture.rs` in full surfaced the `Rgba8unormSrgb` default; independently confirmed
via web search that the WebGPU spec's texture format capability table excludes `rgba8unorm-srgb`
from `STORAGE_BINDING` support (while `rgba8unorm` supports it), then confirmed the copy-paste
origin by grepping the crate for other `Rgba8unormSrgb` defaults — found in `descriptor/texture.rs`
(general-purpose sampled/render texture descriptor) and `state/color_target.rs` (render pipeline
color-attachment state), both of which are legitimate uses of that default (an sRGB color format is
a sensible default for a sampled or render-attachment texture; `RENDER_ATTACHMENT`/`TEXTURE_BINDING`
usage does support `-srgb` formats), confirming the value was carried into `storage_texture.rs`
without re-checking it against storage-texture-specific format constraints.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgpu --target wasm32-unknown-unknown --all-features --test storage_texture_binding_layout_tests
```
**Expected** (fixed): compiles, real headless-Firefox run, 1 passed / 0 failed.
**Actual** (pre-fix, confirmed via temporary `git stash` revert of only the source fix, real run):
```
test tests::default_format_supports_storage_binding_test ... FAIL
---- tests::default_format_supports_storage_binding_test output ----
    error output:
        panicked at module/min/minwebgpu/tests/storage_texture_binding_layout_tests.rs:55:5:
        assertion `left != right` failed: StorageTextureBindingLayout::new()'s default format
        must not be an sRGB format — sRGB formats never support STORAGE_BINDING usage per the
        WebGPU spec's texture format capability table
          left: Rgba8unormSrgb
         right: Rgba8unormSrgb
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 filtered out
```

## Root Cause

`src/binding_type/storage_texture.rs` (pre-fix):
```rust
pub fn new() -> Self
{
  let format = GpuTextureFormat::Rgba8unormSrgb;
  let access = None;
  let view_dimension = None;
  // ...
}
```
compared with the legitimate uses of the identical default value elsewhere in the crate:
```rust
// src/descriptor/texture.rs — general-purpose sampled/render texture, sRGB is a valid default
let format = web_sys::GpuTextureFormat::Rgba8unormSrgb;

// src/state/color_target.rs — render pipeline color-attachment state, sRGB is a valid default
let format = GpuTextureFormat::Rgba8unormSrgb;
```
`format` is a *required* field in the WebGPU spec's `GPUStorageTextureBindingLayout` dictionary
(unlike `access`/`view_dimension`, which have real spec-level defaults and are correctly modeled as
`Option<..>` in this same struct, left unset to let the browser apply its own default). Since
`format` has no spec-level default, the crate needed to supply *some* concrete value when a caller
never calls `.format(..)`. The value chosen (`Rgba8unormSrgb`) is valid for `TEXTURE_BINDING` and
`RENDER_ATTACHMENT` usage — the two other files' contexts — but the WebGPU spec's texture format
capability table excludes every `-srgb` format from `STORAGE_BINDING` usage entirely, a constraint
specific to storage textures that the general-purpose default never had to satisfy.

## Why Not Caught

Every existing call site and test that constructs a `StorageTextureBindingLayout` calls `.format(..)`
explicitly before converting it (the only existing production call site pattern in this crate always
supplies a concrete format), so the default value's own validity was never read back or asserted
against by any prior test. The defect is also invisible to the crate's native (`cargo test -p
minwebgpu`) test invocation regardless: `minwebgpu`'s real functionality — including all of
`binding_type/`, `descriptor/`, and this struct — is entirely `#[cfg(target_arch = "wasm32")]`-gated
(see `src/lib.rs`); native builds only compile a small `stub` module. Exercising this code at all
requires `--target wasm32-unknown-unknown`, run for real through geckodriver
(`.cargo/config.toml`'s `[target.wasm32-unknown-unknown]` runner, also documented in
`module/min/minwebgpu/tests/readme.md`) — a bare `cargo test -p minwebgpu --all-features` (no
`--target`) silently compiles none of the affected code and would report a false-clean pass.

## Fix Applied (2026-08-17)

**`src/binding_type/storage_texture.rs`:** changed `StorageTextureBindingLayout::new()`'s default
from `GpuTextureFormat::Rgba8unormSrgb` to `GpuTextureFormat::Rgba8unorm` — the non-sRGB
counterpart, confirmed by the WebGPU spec's texture format capability table to support
`STORAGE_BINDING` usage, and the closest same-channel-layout format to the original (evidently
copy-pasted) intent. Updated the field's own doc comment to match. Source comment
(`Fix(BUG-275)`/`Root cause`/`Pitfall`) added immediately above `new()`.

Also corrected an adjacent, unrelated doc-comment copy-paste slip in the same file while it was
open: `write_only()`'s doc comment read "Sets the `access` property to `ReadOnly`" (copied verbatim
from `read_only()`'s comment, one method above); the method's own behavior was always correct
(`GpuStorageTextureAccess::WriteOnly`) — corrected the comment text only, no behavior change, not
filed as a separate bug since there is no incorrect runtime behavior to reproduce.

**`tests/storage_texture_binding_layout_tests.rs`** (new file): `default_format_supports_storage_binding_test`
constructs `binding_type::storage_texture_type()` without calling `.format(..)`, converts it into
the real `web_sys::GpuStorageTextureBindingLayout` via `.into()`, and reads back the actual
JS-object `format` property via the real generated `.get_format()` getter — asserting it is neither
`Rgba8unormSrgb` nor any other sRGB format, and specifically equals `Rgba8unorm`. No mocking: this
constructs and inspects the real `wasm-bindgen`-backed JS object; no live `GPUDevice` is required
since `GPUStorageTextureBindingLayout` is a plain descriptor dictionary, not a live GPU resource.

## Verification

`longrun`-detached, from repo root, using an isolated `CARGO_HOME`/`CARGO_TARGET_DIR` in scratchpad
(hardlink-cloned `~/.cargo`) to work around this host's background repo-root temp sweeper, which was
observed mid-session wiping `~/.cargo/registry/src` and caused one unrelated transient build failure
before the workaround was applied:
- `cargo test -p minwebgpu --target wasm32-unknown-unknown --all-features --test storage_texture_binding_layout_tests`
  — pre-fix (temporary `git stash push -- module/min/minwebgpu/src/binding_type/storage_texture.rs`,
  reverting only the source fix while leaving the new test live): fails exactly as shown in
  `Minimum Reproducible Example` above (real headless-Firefox run, real assertion failure showing
  both sides as `Rgba8unormSrgb`). Post-fix (`git stash pop`, restoring the fix): 1 passed / 0
  failed.
- `cargo test -p minwebgpu --target wasm32-unknown-unknown --all-features` (full crate suite, real
  headless Firefox via geckodriver): 19 passed / 0 failed across all 6 test binaries
  (`bind_group_layout_entry_tests` 6, `context_adapter_device_request_tests` 4,
  `shader_compilation_diagnostics_tests` 3, `storage_texture_binding_layout_tests` 1,
  `vertex_attribute_tests` 2, `webgpu_unsupported_tests` 3), 0 doctests — full regression-free
  confirmation.
- `cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings`
  (forced fresh via `touch` on the changed file to rule out stale-cache reuse): clean, exit 0.
- `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` (native, as literally
  specified by this session's verification template): also clean, exit 0 — expected, since this
  invocation never compiles the affected wasm32-gated code at all (see `Why Not Caught`); recorded
  here only for completeness against the template command, not as meaningful coverage of the fix.

## Generalized Version

**Broken assumption:** a builder field's "default value" is safe to copy from a sibling type that
happens to share the same field name and Rust type, without re-checking the *new* type's own
domain-specific validity constraints. `format : GpuTextureFormat` appears in three different
`minwebgpu` structs (`descriptor::texture`, `state::color_target`, `binding_type::storage_texture`),
but each corresponds to a different WebGPU texture-usage class (`TEXTURE_BINDING`/
`RENDER_ATTACHMENT`/`RENDER_ATTACHMENT`/`STORAGE_BINDING` respectively), and the WebGPU spec's
format capability table scopes valid formats *per usage class*, not globally. A shared field name
and shared Rust type across sibling "N parallel files for N variants" structs is exactly the
pattern where a default value copy-pastes cleanly at the source level while silently violating the
new context's own constraints — undetectable by the type system, the compiler, or any test that
(like every pre-existing test here) always overrides the field explicitly before use.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found while reading all 12 assigned files of the `minwebgpu` binding-type + descriptor-start fork group in full, per this session's domain hint calling out `storage_texture.rs`'s `access`/`format` default as a specific area to check. Root cause: `StorageTextureBindingLayout::new()`'s `format` default (`Rgba8unormSrgb`) was copy-pasted from `descriptor/texture.rs`'s and `state/color_target.rs`'s legitimate sRGB defaults, but the WebGPU spec excludes all `-srgb` formats from `STORAGE_BINDING` usage. Fixed by changing the default to `Rgba8unorm`. Verified via 1 new wasm32/browser unit test against real headless-Firefox-executed `web_sys` JS objects (confirmed fail pre-fix / pass post-fix via temporary `git stash` revert-and-rerun) plus the crate's full wasm32 suite (19/19) and clean clippy on both the wasm32 target (the one that actually compiles the affected code) and the native target (as literally specified by the verification template). Filed as BUG-275 not BUG-273 after a fresh on-disk scan, run immediately before updating `task/bug/readme.md`, found two concurrent session actors had already claimed 273 (`273_report_obj_model_num_faces_zero_for_triangulated_meshes.md`) and 274 (`274_minwebgl_diagnostics_feature_missing_future_file_dependency.md`) in the window since the pre-write scan (which had found 272 as the highest existing ID with no collision) — a third concurrent actor had also since claimed 276 (`276_render_target_2d_zero_size_panic.md`), confirmed still-free via one further immediate rescan before this file was written. |
