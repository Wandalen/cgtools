# BUG-300: `TextureDescriptor::new()`'s default `format` (`Rgba8unormSrgb`) is incompatible with `.storage_binding()` usage, silently producing a texture `GPUDevice.createTexture` rejects

- **Severity:** Medium (zero reachable call sites for `.storage_binding()` anywhere in the
  workspace currently -- a latent defect, not an active regression; would be High if any caller
  combined `.storage_binding()` with the unset default, since the resulting failure is silent at
  the Rust level -- `texture::create()` returns `Ok` -- and only surfaces later, disconnected from
  the call site, via an async WebGPU device error-scope event)
- **state:** Verified
- **Affects:** `minwebgpu`'s `TextureDescriptor::new()` (also reachable via `texture::desc()`) --
  any texture built by chaining `.storage_binding()` without also calling `.format(..)`
- **Component:** `module/min/minwebgpu` (`src/descriptor/texture.rs`, `src/texture.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18

## Symptom

`TextureDescriptor::new()` defaults its `format` field to `GpuTextureFormat::Rgba8unormSrgb`, a
valid default for this builder's `TEXTURE_BINDING`/`RENDER_ATTACHMENT`/`COPY_SRC`/`COPY_DST` usage
flags. But `TextureDescriptor` also exposes `.storage_binding()`, which ORs `STORAGE_BINDING` into
`usage` with no cross-check against `format`. Per the WebGPU spec's texture format capability
table, no `-srgb` format supports `STORAGE_BINDING` usage. A caller who chains
`.storage_binding()` without also calling `.format(..)` produces a `GPUTextureDescriptor` whose
`format` is `"rgba8unorm-srgb"` and whose `usage` includes `STORAGE_BINDING` -- a combination a
real `GPUDevice.createTexture` call rejects with a `GPUValidationError`.

## Impact

**Who is affected:** any `minwebgpu` consumer that builds a storage-capable texture via
`TextureDescriptor::new().storage_binding()` (directly or through `texture::desc()`) and relies on
the default `format` instead of calling `.format(..)` explicitly.

**What breaks:** `TextureDescriptor::create()` (`src/descriptor/texture.rs:180-188`) forwards to
`texture::create()` (`src/texture.rs:20-30`), whose only error path is
`device.create_texture( descriptor ).map_err( |e| DeviceError::FailedToCreateTexture(...) )?` --
this catches only a *synchronous* JS throw. `GPUDevice.createTexture()` does not throw
synchronously for an invalid format/usage combination; the validation error surfaces
asynchronously via the device's error-scope / `uncapturederror` mechanism. So `create()` returns
`Ok( GPUTexture )` for a texture the browser has already rejected -- the caller has no Rust-level
signal anything is wrong, and any later use of that texture (bind, write, read) fails against a
resource that was never actually valid, with no error pointing back to the real cause.

**Entity Scope:** `None` -- source-level default-value defect, not entity directory instances.

## How Discovered

Found during this session's workspace-wide bug-hunt pass, `module/math`/`module/min` review stage,
while cross-checking `descriptor/texture.rs` against the fix and Generalized Version of the
already-completed `BUG-275` (`binding_type/storage_texture.rs`'s `StorageTextureBindingLayout`,
same session, same crate). `BUG-275`'s own Generalized Version section names `descriptor::texture`
explicitly as one of three sibling `minwebgpu` structs sharing a `format : GpuTextureFormat` field
with the same `Rgba8unormSrgb` default -- but scopes `descriptor/texture.rs`'s use of that default
as *legitimate* for its own (non-storage) usage flags, distinguishing it from
`storage_texture.rs`'s unconditionally-wrong case. Reading `descriptor/texture.rs` in full
confirmed it exposes `.storage_binding()` alongside `.texture_binding()`/`.render_attachment()`/
`.copy_src()`/`.copy_dst()` through the same shared `format` field with no per-usage-flag
cross-check -- the same defect *pattern* BUG-275 generalized about, in the one usage flag
(`STORAGE_BINDING`) the shared default does not actually cover. Confirmed via workspace-wide grep
that `.storage_binding()` has zero call sites anywhere in this repo (latent, not an active
regression) and that `TextureDescriptor` itself is otherwise unused outside `minwebgpu`'s own
`texture::desc()` wrapper.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgpu --target wasm32-unknown-unknown --all-features --test texture_descriptor_tests
```
**Expected** (fixed): compiles, real headless-Firefox run, 1 passed / 0 failed.

**Actual** (pre-fix, confirmed via temporary revert-and-rerun of only the source fix, real run):
```
test tests::default_format_supports_storage_binding_test ... FAIL
---- tests::default_format_supports_storage_binding_test output ----
    error output:
        panicked at module/min/minwebgpu/tests/texture_descriptor_tests.rs:50:5:
        assertion `left != right` failed: TextureDescriptor::new()'s default format must not be
        an sRGB format — sRGB formats never support STORAGE_BINDING usage per the WebGPU spec's
        texture format capability table
          left: Rgba8unormSrgb
         right: Rgba8unormSrgb
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 filtered out
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `TextureDescriptor::new()`'s default `format` (`Rgba8unormSrgb`) is invalid for `STORAGE_BINDING` usage per the WebGPU spec's capability table | ✅ Root Cause | `descriptor/texture.rs:44` (pre-fix) sets the default to `Rgba8unormSrgb`; `storage_binding()` (lines 156-163) never cross-checks `format` before ORing in `STORAGE_BINDING` | E1, E2, E5 |
| H2 | That same default is legitimately valid for this builder's other usage flags (`TEXTURE_BINDING`/`RENDER_ATTACHMENT`/`COPY_SRC`/`COPY_DST`), so the fix must not treat sRGB as universally wrong here | ✅ Verified | BUG-275's own root-cause comment explicitly names `descriptor/texture.rs`'s sRGB default as valid for its general-purpose role; `Rgba8unorm` (the replacement) is valid for every usage flag this builder can produce, so switching the default loses nothing for any of them | E4 |
| H3 | `texture::create()` cannot catch the resulting spec violation because the WebGPU validation error surfaces asynchronously, not as a synchronous JS throw | ✅ Verified | `src/texture.rs:20-30`: the only error path is a `.map_err(..)` on `device.create_texture(..)`, which wraps a synchronous throw only | E3 |
| H4 | The defect is currently unreachable in production because no caller in this workspace combines `.storage_binding()` with the unset default | ✅ Verified | Workspace-wide grep for `.storage_binding()` returns zero matches anywhere, including `examples/` | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/descriptor/texture.rs:44` (pre-fix) | `let format = web_sys::GpuTextureFormat::Rgba8unormSrgb;` | H1 |
| E2 | `src/descriptor/texture.rs:156-163` | `storage_binding()` ORs `STORAGE_BINDING` into `usage` with no read or check of `self.format` | H1 |
| E3 | `src/texture.rs:20-30` | `texture::create()`'s only error path is `.map_err(..)` on a synchronous `create_texture` throw | H3 |
| E4 | `src/binding_type/storage_texture.rs:39-41` (BUG-275's fix comment) | States outright that `descriptor/texture.rs`'s sRGB default "is valid" for its own (non-storage) use cases | H2 |
| E5 | Terminal output (MRE section above) | Pre-fix assertion failure: `left: Rgba8unormSrgb, right: Rgba8unormSrgb` | H1 (demonstrates) |
| E6 | Terminal output (How Discovered section above) | Workspace-wide grep for `.storage_binding()`: zero matches | H4 |

## Root Cause

```
TextureDescriptor::new()     -> format defaults to Rgba8unormSrgb        (descriptor/texture.rs:44)
  .texture_binding()         -> usage |= TEXTURE_BINDING                 -- sRGB valid, fine
  .render_attachment()       -> usage |= RENDER_ATTACHMENT               -- sRGB valid, fine
  .copy_src() / .copy_dst()  -> usage |= COPY_SRC / COPY_DST             -- sRGB valid, fine
  .storage_binding()         -> usage |= STORAGE_BINDING                 -- sRGB invalid  ✗
                                 (no cross-check against `format` anywhere)
  .create( device )          -> texture::create() -> device.create_texture(..).map_err(..)
                                 -- catches only a *synchronous* throw; the WebGPU spec surfaces
                                    an invalid format/usage combination asynchronously, via a
                                    device error-scope event, so `create()` returns `Ok` regardless
```
A single `format` default shared across every usage flag a builder can produce must be valid for
the *narrowest* usage class among them, not just the most common one. `Rgba8unormSrgb` is valid
for four of the five usage flags this builder exposes and invalid for the fifth
(`STORAGE_BINDING`) -- the default silently assumed the majority case covered every case.

## Why Not Caught

No existing test in this crate ever converted a `TextureDescriptor` without first calling
`.format(..)` explicitly, and (per H4/E6) nothing in this workspace calls `.storage_binding()` on
this builder at all yet, so the invalid default was never read back or asserted against, and never
exercised end-to-end against a real `GPUDevice`. The defect is also invisible to a native (`cargo
test -p minwebgpu`, no `--target`) invocation regardless: `minwebgpu`'s real functionality is
entirely `#[cfg(target_arch = "wasm32")]`-gated, so exercising this code at all requires `--target
wasm32-unknown-unknown` run for real through geckodriver (`.cargo/config.toml`'s
`[target.wasm32-unknown-unknown]` runner) -- a bare native test run would silently compile none of
the affected code.

## Fix Location

`src/descriptor/texture.rs:14, 39-51`:

```rust
// Before:
/// Texture's format. Default: Rgba8unormSrgb
...
pub fn new() -> Self
{
  let format = web_sys::GpuTextureFormat::Rgba8unormSrgb;
  ...
}

// After:
/// Texture's format. Default: Rgba8unorm
...
pub fn new() -> Self
{
  let format = web_sys::GpuTextureFormat::Rgba8unorm;
  ...
}
```
`Rgba8unorm` is valid for all five usage flags this builder can produce (`TEXTURE_BINDING`,
`RENDER_ATTACHMENT`, `COPY_SRC`, `COPY_DST`, `STORAGE_BINDING`) per the WebGPU spec's texture
format capability table, so this closes the defect for every usage combination without narrowing
any existing legitimate use. Source comment (`Fix(BUG-300)`/`Root cause`/`Pitfall`) added
immediately above `new()`.

**`tests/texture_descriptor_tests.rs`** (new file): `default_format_supports_storage_binding_test`
constructs `TextureDescriptor::new().storage_binding()` without calling `.format(..)`, converts it
into the real `web_sys::GpuTextureDescriptor` via `.into()`, and reads back the actual JS-object
`format` property via the real generated `.get_format()` getter -- asserting it is neither
`Rgba8unormSrgb` nor any other sRGB format, and specifically equals `Rgba8unorm`. No mocking: this
constructs and inspects the real `wasm-bindgen`-backed JS object; no live `GPUDevice` is required
since `GPUTextureDescriptor` is a plain descriptor dictionary, not a live GPU resource.

## Prevention

Add (done, see MRE) a default-value test on every builder in this crate that exposes more than one
`.<usage>()` method sharing a single default-valued field, asserting the default is valid across
*all* of that builder's exposed usage flags, not just the most common one. Detection command for
the general pattern (a shared `format` default reused across sibling descriptor/binding-layout
files without a per-file capability re-check):
```bash
grep -rn "GpuTextureFormat::Rgba8unormSrgb" module/min/minwebgpu/src/
```

**Pitfall:** a default value that is valid for a builder's most common usage flags is not
automatically valid for every usage flag the same builder exposes -- each additional `.<usage>()`
method widens the set of WebGPU spec constraints the shared default must satisfy, and a
format/usage combination that is invalid per spec can still convert cleanly at the Rust level and
fail only later, asynchronously, on a real device.

## Generalized Version

**Broken assumption:** a builder's single shared default value stays valid as new usage-flag
methods are added to that same builder, without re-checking the default against each new usage
flag's own spec constraints.

Fails for any `TextureDescriptor` built by chaining `.storage_binding()` when:
1. `.format(..)` is never called (the default `Rgba8unormSrgb` is left in place), AND
2. The resulting descriptor is actually passed to a real `GPUDevice.createTexture` call

**Detection invariant:**
```
for all usage flags U a builder exposes: default_format must be in capability_table( U )
```
Second confirmed instance of the same underlying defect pattern this session, after BUG-275
(`binding_type/storage_texture.rs`) -- BUG-275's own Generalized Version section named
`descriptor::texture` as a sibling sharing the same `format` field and default, and this is that
prediction materializing in the one usage flag (`STORAGE_BINDING`) the shared default does not
cover.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during this session's workspace-wide bug-hunt task, while cross-checking `descriptor/texture.rs` against BUG-275's fix and Generalized Version in the same crate |
| 2026-08-18 | fix_applied | `src/descriptor/texture.rs:44`: `TextureDescriptor::new()`'s default `format` changed from `Rgba8unormSrgb` to `Rgba8unorm` |
| 2026-08-18 | verified | `default_format_supports_storage_binding_test` (bug_reproducer) passes; full `minwebgpu` wasm32 suite (20 passed across 8 binaries) and clippy (wasm32 + native, `-D warnings`) clean |

## Refs: src/

- `src/descriptor/texture.rs` — `TextureDescriptor::new()`'s default `format` changed to `Rgba8unorm`

## Refs: tests/

- `tests/texture_descriptor_tests.rs` — added `default_format_supports_storage_binding_test` (bug_reproducer)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE uses an in-repo `cargo test` command, not a synthetic `/tmp/mreNNN/` fixture, and doesn't spell out the exact revert command used to capture the pre-fix failure — matches this crate's own BUG-275 precedent's MRE shape exactly, not an oversight | — |
| D3 | Cross-Reference Integrity | — | 🟢 | `## Refs:` sections + FI027 backreference comments added proactively in both `src/` and `tests/`, before this gate ran (learned from BUG-298's self-caught gap this session); `grep -rn 'BUG-300'` confirms both directions resolve | — |
| D4 | Root Cause Quality | — | 🟢 | E4's cited line range (`storage_texture.rs:39-41`) independently re-confirmed against the file's actual content, not trusted from memory; Hypothesis↔Evidence cross-references checked bidirectionally, consistent both ways | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | `git status --porcelain -- module/min/minwebgpu/` re-checked immediately before this gate: only this fix's own 3 files touched, no concurrent drift | — |
| D8 | Crate Single Responsibility | — | 🟢 | Unlike BUG-275 (which also fixed an adjacent unrelated doc-comment typo "while the file was open"), this fix stayed scoped to exactly the one reported defect | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — exit 0 (`default_format_supports_storage_binding_test` ... ok), 2026-08-18, real headless-Firefox run via geckodriver. Full `minwebgpu` wasm32 suite (20 passed / 0 failed across 8 test binaries) and `cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings` plus native `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` (both clean, exit 0) also re-confirmed post-fix.
