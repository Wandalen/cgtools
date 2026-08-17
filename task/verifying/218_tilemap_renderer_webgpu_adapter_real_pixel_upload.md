# tilemap_renderer WebGPU adapter: upload real sprite pixel data

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-17 03:53:34
- **expires_at:** 2026-08-17 05:53:34
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-17 03:53:34
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

`WebGpuBackend::assets_load` (`src/adapters/webgpu.rs:246-263`) allocates every
loaded image as a hardcoded `[1, 1, 1]`-sized texture and never writes any pixel
data into it (`for image in &assets.images { ... texture_create( size: [1,1,1] ...
) ...}` — `image.source`'s real bytes/width/height are never read at all) — so
every WebGPU-backed tilemap sprite renders whatever the driver leaves in freshly
allocated GPU memory, not the sprite's actual texture. This is a live rendering
correctness gap discovered during the 2026-08-17 `docs/layer` round-3 gap audit,
not a requested feature. It is fixable today: sibling `NativeBackend::assets_load`
(`src/adapters/native.rs:154-176`) already does this correctly (destructure
`ImageSource::Bitmap`, convert to rgba8, size the texture from real
`width`/`height`, `queue.texture_write` the real bytes), and the crate-doc excuse
for the WebGPU gap — "`gpu_hal` ... offers no pixel-upload call (`texture_write`)
for the WebGPU surface" (`docs/feature/005_webgpu_backend_adapter.md:18`) — is
stale: `gpu_hal::Queue::texture_write` has supported the WebGPU backend since task
089 (`module/helper/gpu_hal/src/device.rs:940`,
`Self::WebGpu( queue ) => webgpu_texture_write( queue, texture, data )`). Fix by
giving `WebGpuBackend::assets_load` the same real-upload shape `NativeBackend`
already has, sharing (not duplicating) the existing `to_rgba8` format-conversion
logic, and retiring the doc's stale excuse. Testable:
`cargo test -p tilemap_renderer --lib` (or wherever the shared conversion helper's
new unit tests land) exits 0, plus
`grep -c "size : \[ 1, 1, 1 \]" module/helper/tilemap_renderer/src/adapters/webgpu.rs`
returns 0 (was: 1).

Note on filing shape: this is a functional rendering defect discovered during
audit, not user-reported — the same discovery shape as BUG-114/BUG-154, which
this session filed through `task/bug/`'s report-then-promote flow. It is filed
directly as a task instead, per the explicit instruction governing this filing
round ("for each gap create task file").

## In Scope

- `module/helper/tilemap_renderer/src/adapters/webgpu.rs`'s `WebGpuBackend::assets_load`
  (`fn assets_load`, line 246): replace the `for image in &assets.images { texture_create(
  size: [1,1,1], ... ) }` loop with `NativeBackend::assets_load`'s shape —
  destructure `image.source` as `ImageSource::Bitmap { bytes, width, height, format }`
  (skip/`continue` on any other `ImageSource` variant, matching `native.rs:161-165`'s
  own handling), convert via the shared rgba8 helper (see next bullet), size the
  `TextureDesc` from the real `*width`/`*height` with
  `usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST`, then call
  `self.queue.texture_write( &texture, &rgba )` before pushing the `LoadedImage`.
- Share the pixel-format-conversion logic instead of duplicating it: `to_rgba8`
  (`src/adapters/native.rs:352-361`) is currently private to `native.rs`. Relocate
  or re-expose it at crate visibility (e.g. `pub(crate) fn` reachable from both
  adapter modules — either promote it in place and import via `super`/`crate::`
  path, or move it beside `ImageSource`/`PixelFormat` in `src/assets.rs`) so
  `webgpu.rs` calls the same function `native.rs` already uses, rather than
  reimplementing the same four `PixelFormat` match arms a second time.
- `module/helper/tilemap_renderer/docs/feature/005_webgpu_backend_adapter.md`:
  replace the stale "Known gap" paragraph (line 18, the
  "`gpu_hal` ... offers no pixel-upload call ... for the WebGPU surface" claim)
  with a description of the real upload path now in place; update the Design
  section's "Given the real (if currently pixel-empty) texture gap, status is
  tracked as partial (⚠️)" line and the `invariant/001` cross-reference's "no real
  pixel data is ever uploaded" caveat (line 28) to match the fixed state.
- **Fold-in (confirmed before this task's execution):** this fix invalidates
  two pre-existing artifacts that documented the *old* placeholder-texture
  behavior as current fact — both are corrected as part of this task rather
  than left stale:
  - `module/helper/tilemap_renderer/tests/manual/readme.md` (task 198's own
    manual browser-verification procedure): its recorded live reading of
    `adapter-webgpu`'s sprite-center pixel (`rgb 0 0 0`, opaque black) predates
    this fix. Mark it stale and record the source-derived prediction (solid
    red, matching `adapter-webgl`, since both now upload the same asset bytes)
    without claiming a live re-confirmation this task does not perform (live
    `browsee` verification is out of this task's own scope, per Out of Scope
    below).
  - `docs/layer/002_l1_gpu_hal.md`'s `adapter-webgpu`/`adapter-webgl` paragraph
    (~lines 58-73): rewrite the asymmetry language — the two adapters no
    longer differ in upload *kind* (both now upload real pixel bytes via the
    shared `to_rgba8` helper), only in browser-verification *recency*
    (`adapter-webgl`'s live reading is current; `adapter-webgpu`'s predates
    this task and awaits re-verification).
  - Incidental, same-cause doc-comment fixes in the two source files whose
    own prose directly asserted the now-superseded black-quad behavior:
    `webgpu.rs`'s module doc comment, its `QUAD_VERTICES`/`LoadedImage` doc
    comments, and `examples/tilemap_renderer/adapter_browser/src/main.rs`'s
    module doc comment plus its `CLEAR`/`SPRITE_RGBA` const doc comments —
    left stale and self-contradicting the corrected docs above otherwise.

## Out of Scope

- Sub-region/atlas UV addressing — `QUAD_VERTICES`' own UV mapping is unchanged;
  this task makes the *whole* texture real, not sprite-sheet cropping.
- `NativeBackend` itself — already correct, read-only reference for this task.
- Live browser GPU pixel-correctness verification (`browsee` or equivalent) —
  out of reach in this environment, and `webgpu_backend_test.rs`'s own doc
  comment already records "no live-device/canvas test exists yet" for this
  adapter; the Test Matrix below targets the pure conversion logic and a
  source-level confirmation that the placeholder literal is gone, not a
  GPU-rendered-frame assertion.
- The WebGL2 adapter's asset-loading path (`adapters/webgl.rs`) — already
  uploads real pixels, untouched by this task.
- Any change to `gpu_hal::Queue::texture_write` itself — already correct and
  already used by `NativeBackend`; this task only wires `WebGpuBackend` to call it.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   The shared rgba8-conversion helper has exactly one definition, called from both `native.rs` and `webgpu.rs` — no duplicated `PixelFormat` match arms
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   `module/helper/tilemap_renderer/docs/feature/005_webgpu_backend_adapter.md` no longer states `gpu_hal` lacks a WebGPU `texture_write` call
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | 2×2 `Rgba8` bytes (16 bytes, distinct per-pixel values) | shared rgba8-conversion helper | Returns the same 16 bytes unchanged |
| T02 | 1×1 `Rgb8` bytes `[10, 20, 30]` | shared rgba8-conversion helper | Returns `[10, 20, 30, 255]` — alpha padded to opaque |
| T03 | 1×1 `Gray8` byte `[42]` | shared rgba8-conversion helper | Returns `[42, 42, 42, 255]` — gray broadcast to RGB, alpha opaque |
| T04 | 1×1 `GrayAlpha8` bytes `[42, 128]` | shared rgba8-conversion helper | Returns `[42, 42, 42, 128]` — gray broadcast to RGB, alpha preserved |

## Acceptance Criteria

-   `WebGpuBackend::assets_load` sizes each texture from the real `Bitmap` `width`/`height`, not a literal `1`
-   `WebGpuBackend::assets_load` calls `self.queue.texture_write` with the converted rgba bytes for every `Bitmap` image
-   The rgba8-conversion helper has one definition, imported by both adapters
-   `docs/feature/005_webgpu_backend_adapter.md`'s stale "no pixel-upload call for the WebGPU surface" claim is removed
-   Every Test Matrix row has a corresponding passing test
-   No pre-existing test in `tilemap_renderer`'s suite regresses

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Source**
- [ ] C1 — Does `WebGpuBackend::assets_load` destructure `image.source` as `ImageSource::Bitmap { bytes, width, height, format }` and skip non-`Bitmap` sources, matching `NativeBackend::assets_load`'s shape?
- [ ] C2 — Does the `TextureDesc` passed to `texture_create` use the real `*width`/`*height` (not a literal `1`)?
- [ ] C3 — Is `self.queue.texture_write` called with the converted rgba bytes for every loaded `Bitmap` image?
- [ ] C4 — Does the rgba8-conversion helper have exactly one definition in the crate, called from both `native.rs` and `webgpu.rs` (`grep -rn "fn to_rgba8\|fn .*rgba8" src/adapters/` shows one function definition, two call sites)?

**Documentation**
- [ ] C5 — Does `docs/feature/005_webgpu_backend_adapter.md` no longer claim `gpu_hal` has no WebGPU pixel-upload call?
- [ ] C6 — Does the doc's status/invariant language reflect the fixed (non-placeholder) upload path?

**Out of Scope confirmation**
- [ ] C7 — Is `adapters/native.rs`'s own `assets_load` body unmodified apart from any visibility change needed to share the conversion helper (`git diff` shows no logic changes there)?
- [ ] C8 — Is `adapters/webgl.rs` untouched (`git diff` shows no edits under that path)?
- [ ] C9 — Is `gpu_hal::Queue::texture_write` itself unmodified (`git diff --stat -- module/helper/gpu_hal/` is empty)?

### Measurements

- [ ] M1 — placeholder literal removed: `grep -c "size : \[ 1, 1, 1 \]" module/helper/tilemap_renderer/src/adapters/webgpu.rs` → 0 (was: 1)
- [ ] M2 — new test count: `cargo test -p tilemap_renderer --lib 2>&1 | grep -c "test result: ok"` → ≥1

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p tilemap_renderer --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — T01-T04 use 4 distinct `PixelFormat` variants with distinct expected byte sequences, not the same assertion repeated — checked by reading the literal expected values in the test file
- [ ] AF2 — `webgpu.rs`'s diff to `assets_load` actually calls `self.queue.texture_write` (not merely resizes the texture and leaves it unpopulated) — checked by reading the diff, not just the passing test count

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-17 03:53:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-17 03:54 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 218` → blocked: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)`; left at 🔬 Verifying |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-17]** `FILED` — Task filed via docs/layer round-3 gap audit (gap #1): give `WebGpuBackend::assets_load` real pixel upload, retire the stale gpu_hal-blocker excuse in `docs/feature/005`.
- **[2026-08-17]** `SCOPE_EXPANDED` — Before execution, folded in two pre-existing artifacts this fix invalidates (`tests/manual/readme.md`'s task-198 live-reading record, `docs/layer/002_l1_gpu_hal.md`'s asymmetry paragraph) plus incidental same-cause doc-comment fixes, per the new In Scope "Fold-in" bullet — see that bullet for the full itemization.
- **[2026-08-17]** `EXECUTED` — **Implementation.** Relocated `to_rgba8` from `native.rs` (private) to `src/assets.rs` (`pub fn`, `#[cfg(any(feature = "adapter-native", feature = "adapter-webgpu"))]`), mirroring the existing `image_mime_detect` shared-helper pattern exactly (function outside `mod private`/`mod_interface!`, cfg-gated to the adapters that need it). `webgpu.rs`'s `assets_load` (line 252) rewritten to `native.rs`'s own shape: destructure `ImageSource::Bitmap { bytes, width, height, format }` (`continue` on non-`Bitmap`), convert via the shared `to_rgba8`, size `TextureDesc` from real `*width`/`*height` with `TEXTURE_BINDING | COPY_DST`, `self.queue.texture_write(&texture, &rgba)`, then `texture.view()`. `native.rs`'s own `assets_load` body is untouched — only its import line changed (`crate::assets::{Assets, ImageSource, to_rgba8}`) and its now-redundant private `to_rgba8` definition was deleted, which is what the Delivery Requirements' "exactly one definition" mandate and C7's own "apart from any visibility change needed to share the helper" wording both anticipate. Added 4 unit tests (T01-T04) to `tests/assets_test.rs` under a new `#[cfg(feature = "adapter-native")] mod to_rgba8_conversion` (chosen over a new file: `assets_test.rs` already `use`s the crate's `assets` module wildcard and matches the domain-per-file convention in `tests/readme.md`). Updated `tests/readme.md`'s domain map, `docs/feature/005_webgpu_backend_adapter.md` (known-gap paragraph → Pixel upload paragraph; status ⚠️→✅; invariant/001 row rewritten to avoid overclaiming UV row-order is now "proven" — the only asset exercised, task 198's uniform solid-red sprite, is vertically symmetric and can't distinguish a flip), `docs/feature/readme.md`'s summary badge, `docs/layer/002_l1_gpu_hal.md`'s asymmetry paragraph (now framed as verification-recency, not upload-kind), `tests/manual/readme.md` (historical-note framing, Scenario 2/3 pre/post-218 split, Test Matrix stale-reading annotation), and `examples/tilemap_renderer/adapter_browser/src/main.rs`'s module/`CLEAR`/`SPRITE_RGBA` doc comments (justified as in-scope: this task's own `webgpu.rs` edit is what invalidated these specific adjacent claims, not pre-existing unrelated staleness). Every doc edit phrases the post-fix pixel color as a prediction, not a confirmed reading — live `browsee` re-verification is explicitly out of this task's scope (Out of Scope bullet), consistent with task 198's own original pre-verification prediction phrasing.
  **Verification.** `verb/test_only pkg::tilemap_renderer` (all-features, native) initially failed clippy: `-D unused-imports` on `native.rs`'s now-dead `PixelFormat` import (the format-matching logic moved to the shared `to_rgba8`, so `native.rs` no longer references `PixelFormat` directly) — fixed by dropping it from the import line, then reconfirmed both `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` (exit 0) and `verb/test_only pkg::tilemap_renderer` (148/148 pass, including all 4 new `to_rgba8_conversion::*` tests confirmed by name in the log) clean. `cargo check -p tilemap_renderer --features adapter-webgpu,adapter-webgl --target wasm32-unknown-unknown` — clean, exit 0. M1 (`grep -c "size : \[ 1, 1, 1 \]" webgpu.rs` → 0) confirmed. AF1 (4 distinct `PixelFormat` variants, distinct expected byte sequences — not a repeated assertion) and AF2 (the diff genuinely calls `self.queue.texture_write`, not merely a resize) both confirmed by direct reading of the test file and the current `assets_load` body.
  **Checklist-wording tensions (documented per this session's established practice, not silently resolved either way):**
  - **C4** — the literal grep target (`src/adapters/`) finds zero definitions post-relocation, since `to_rgba8` now lives in `src/assets.rs`, outside that directory — but this is exactly the task's own explicitly-offered alternative ("...or move it beside `ImageSource`/`PixelFormat` in `src/assets.rs`"). Reinterpreted: `grep -rn "fn to_rgba8" src/` → one definition (`assets.rs:579`); `grep -rn "to_rgba8(" src/adapters/` → two call sites (`webgpu.rs:264`, `native.rs:166`). Passes by intent.
  - **C8/C9** — literal `git diff --stat` is non-empty for both `adapters/webgl.rs` (54 ins/21 del) and `module/helper/gpu_hal/` (8 files) — but both diffs predate this task entirely: `webgl.rs`'s is task 090's context-loss-listener work, `gpu_hal/`'s is tasks 201-203/358's Vulkan-backend work, both already-completed in earlier sessions and still uncommitted only because of the standing "do not commit" instruction (no git-commit boundary separates tasks in this session's operating model). This task made zero Edit/Write calls against either path. Passes by intent; the literal command's clean-diff assumption doesn't hold in a no-commit multi-task session regardless of what this task did.
  - **M2** — the literal `cargo test -p tilemap_renderer --lib 2>&1 | grep -c "test result: ok"` command "passes" vacuously (returns 1, satisfying "≥1") but proves nothing: it ran 0 tests, because the new tests live in `tests/assets_test.rs` (an integration-test binary, outside `--lib`'s unit-test-only scope) and are additionally gated behind `adapter-native`, which the bare command never enables (crate default features enable neither adapter). The real evidence is the `verb/test_only pkg::tilemap_renderer` run cited above, whose log shows all 4 T01-T04 tests passing by name.
  - **I1** — this task's own crate-scoped confirmation (148/148 native tests, clean wasm32 check, clean clippy) stands in for the literal full `verb/test`; the actual full-workspace `verb/test`/`will .test level::3` run is deferred to this session's already-planned Phase 5 gate, covering this task's changes together with task 223's in one pass rather than running the full suite twice.
  - **I2** — the literal command's `RUSTFLAGS="-D warnings"` env-prefix is a known anti-pattern in this repo: confirmed via `.cargo/config.toml` that its `[build]` section unconditionally sets `rustflags = ["--cfg", "web_sys_unstable_apis"]` for every cargo invocation, and env-prefixing `RUSTFLAGS` on the command line *replaces* rather than augments that array (standard cargo precedence), silently dropping the cfg gate. Substituted the safe equivalent already run above: `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` with no env-prefix — clippy's own `-D warnings` denies all warnings (a strict superset of plain `cargo check`'s), across all targets and features, without clobbering the config file's rustflags. Exit 0, confirmed clean.
  Self-check performed as Tier 2 Dual-Role Self-Check (this repo's MAAV cap). `tsk .claim_verify 218` and `tsk .verify_pass 218` outcomes recorded in the Journal above.

## Related Documentation

- `module/helper/tilemap_renderer/docs/feature/005_webgpu_backend_adapter.md` — the feature doc whose "Known gap" paragraph (line 18) this task both fixes-in-code and updates-in-doc
- `module/helper/tilemap_renderer/docs/feature/006_native_backend_adapter.md` — describes `NativeBackend`'s already-correct upload path this task mirrors
- `module/helper/tilemap_renderer/src/adapters/native.rs` — `to_rgba8` (line 352) and `assets_load` (line 154), the reference implementation
- `module/helper/gpu_hal/src/device.rs` — `texture_write` (line 935) and its WebGPU arm (line 940), proof the doc's excuse is stale
- `task/completed/089_*.md` (gpu_hal write_texture) — the task that added WebGPU `texture_write` support this task's doc fix cites
