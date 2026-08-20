# Add `write_texture` queue primitive to `minwebgpu`

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** module/min/minwebgpu
- **verified_by:** independent verifier (general-purpose Agent, blind dispatch)
- **verification_date:** 2026-08-11
- **blocked_by:** null
- **executing_at:** 2026-08-11 13:47:14
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **priority:** 0

## Goal

Add a `write_texture` function to `module/min/minwebgpu/src/queue.rs`, mirroring the existing
`write_buffer` function (`queue.rs:19-30`) in shape and error handling, wrapping
`web_sys::GpuQueue`'s WebGPU-spec `writeTexture()` binding so pixel data can be uploaded into a
`web_sys::GpuTexture` after creation — closing the gap where minwebgpu's `queue` module supports
buffer writes (`write_buffer`) and texture creation (`texture::create`, `texture.rs:20-30`) but no
texture *write* path exists anywhere in the crate. Motivated directly by `gpu_hal`'s own
`Queue::write_texture` gap (task 089, this task's sole planned consumer), which needs this
primitive to implement its WebGPU backend arm — `gpu_hal`'s existing `Queue::write_buffer`
(`gpu_hal/src/device.rs:876-910`) already depends on this same `minwebgpu::queue::write_buffer`
precedent for its own WebGPU arm, establishing the pattern this task extends one primitive further.
Testable: `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` exits 0 with the
new function present and exported via `mod_interface`.

## In Scope

- New `pub fn write_texture` in `module/min/minwebgpu/src/queue.rs`'s `mod private` block, matching
  `write_buffer`'s existing signature shape: takes `&web_sys::GpuQueue`, a destination
  (`&web_sys::GpuTexture` at minimum — WebGPU's `writeTexture` also accepts an origin/mip-level via
  `GPUImageCopyTexture`, out of scope here per the "v0: whole-texture, base mip only" boundary
  below), a `&[u8]` data slice, a data layout (bytes-per-row, rows-per-image), and a size
  (width/height/depth) — calling the corresponding `web_sys::GpuQueue` write-texture method
  (`web_sys`'s generated binding name for `GPUQueue.writeTexture()`, confirmed present because
  `web-sys`'s `Gpu*` binding family already exposes the sibling surface
  `write_buffer_with_f64_and_u8_slice` comes from)
- New `web-sys` feature entries in `module/min/minwebgpu/Cargo.toml`'s
  `[dependencies.web-sys] features = [...]` list for whichever WebGPU dictionary types the chosen
  `write_texture` overload requires (e.g. `GpuImageCopyTexture`, `GpuImageDataLayout`,
  `GpuExtent3dDict` — exact names confirmed against `web-sys`'s own generated docs at implementation
  time, not invented here)
- New `TextureError` variant (e.g. `FailedWriteToTexture(String)`) in
  `module/min/minwebgpu/src/error.rs`, alongside the existing `TextureError::FailedToCreateTexture`
  (`error.rs:96`) and mirroring `BufferError::FailedWriteToBuffer`'s (`error.rs:77`) shape
- Export `write_texture` via the `crate::mod_interface!` block at the bottom of `queue.rs`, alongside
  the existing `submit`/`write_buffer` exports

## Out of Scope

- **Partial-region writes** (non-zero origin, sub-rect updates) — v0 is whole-texture-at-base-mip-
  level only, matching `gpu_hal::TextureDesc`'s own documented "v0 surface: 2d, one mip, one sample"
  boundary (`gpu_hal/src/types.rs:302`)
- **`gpu_hal::Queue::write_texture`** — separate task (089), `blocked_by` this task; consumes this
  primitive for its WebGPU arm only, implements WebGL/Native arms independently
- **WebGL and Native texture-write paths** — those go through `web_sys::WebGl2RenderingContext` and
  `wgpu::Queue` directly, neither touches `minwebgpu` at all; entirely task 089's scope
- **Mipmap generation** — no mip-level parameter is added; `gpu_hal::TextureDesc` has no
  `mip_level_count` field today, so nothing above this primitive could drive multi-mip writes yet
  regardless
- Modifying `texture.rs`'s existing `create`/`desc`/`view`/`view_with_descriptor` functions — this
  task only adds a new function to `queue.rs` and a new error variant; texture creation itself is
  unchanged

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` passes with zero
    errors; `RUSTFLAGS="-D warnings" cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-features -- -D warnings`
    exits 0
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` after adding `write_texture` | `minwebgpu`, wasm32 target, `full` feature set | Exit 0 — `write_texture` compiles and its symbol is present in the crate's public API |
| T02 | `cargo doc -p minwebgpu --target wasm32-unknown-unknown --no-deps` | Same | Exit 0 — new `///` doc comment on `write_texture` renders without broken intra-doc links |
| T03 | `cargo check -p minwebgpu --target x86_64-unknown-linux-gnu` (native target, `web-sys`/`wasm-bindgen` symbols absent) | Default (non-wasm) target | Exit 0 — `queue.rs` remains behind the same feature gating as `write_buffer` (no new unconditional-on-native compile dependency introduced) |
| T04 | Repo-wide diff of `queue.rs` after the change | — | `write_buffer`'s existing function body is byte-identical to pre-task (no accidental shared-code regression) |

## Acceptance Criteria

-   `module/min/minwebgpu/src/queue.rs` exports a new `pub fn write_texture` alongside
    `submit`/`write_buffer` in the `crate::mod_interface!` block
-   `module/min/minwebgpu/src/error.rs` contains a new `TextureError` variant for a failed write,
    distinct from `FailedToCreateTexture`
-   `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` exits 0
-   `RUSTFLAGS="-D warnings" cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-features -- -D warnings`
    exits 0
-   Every row T01–T04 in `## Test Matrix` passes
-   `git diff -- module/min/minwebgpu/src/queue.rs` shows only additions — no modified lines inside
    the existing `write_buffer` function body

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**`queue.rs` — new primitive**
- [ ] C1 — Does `write_texture` follow `write_buffer`'s exact error-handling shape
      (`.map_err(|e| ...Error::FailedWriteToTexture(format!("{e:?}")))?`)?
- [ ] C2 — Is `write_texture` exported via `crate::mod_interface!` alongside the existing exports?

**Scope discipline**
- [ ] C3 — Is `texture.rs` unchanged (this task adds to `queue.rs`/`error.rs` only)?
- [ ] C4 — Does the new function accept no origin/mip-level parameter beyond whole-texture-base-
      mip-level (confirms the v0 boundary held)?

**Feature/target gating**
- [ ] C5 — Is the new `web-sys` feature list addition scoped to the same
      `[dependencies.web-sys] features = [...]` array `write_buffer`'s own `GpuQueue`/`GpuBuffer`
      entries already live in (no new top-level Cargo.toml section)?

**Out of Scope confirmation**
- [ ] C6 — Does `git diff --stat` show zero changes to `module/helper/gpu_hal/` (task 089's own
      scope, untouched by this task)?
- [ ] C7 — Is `write_buffer`'s function body byte-identical to its pre-task state?

### Measurements

- [ ] M1 — `write_texture` line count: `wc -l module/min/minwebgpu/src/queue.rs`
      (before: 41 lines; after: expected ~55-70)
- [ ] M2 — `git diff --stat -- module/helper/`: expected empty (confirms no cross-crate leak into
      task 089's territory)

### Invariants

- [ ] I1 — `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` → exit 0
- [ ] I2 — `RUSTFLAGS="-D warnings" cargo clippy -p minwebgpu --target wasm32-unknown-unknown --all-features -- -D warnings` → 0 warnings
- [ ] I3 — `cargo check -p minwebgpu --target x86_64-unknown-linux-gnu` → exit 0 (native target
      unaffected)

### Anti-faking checks

- [ ] AF1 — `write_texture` actually calls a `web_sys::GpuQueue` method that forwards to the real
      `GPUQueue.writeTexture()` DOM binding — not a stub that silently drops `data` and returns
      `Ok(())` unconditionally (cross-reference against `web_sys`'s generated method signature, same
      bar `write_buffer` meets today)
- [ ] AF2 — The new `TextureError` variant is actually returned on the WebGPU-throw path (mirrors
      `create`'s existing `.map_err` pattern) — not swallowed by an `.unwrap()` or ignored `Result`

## Related Documentation

- `docs/adr/003_d2_stack_hal_adoption.md` — Decision #1 (adapter-webgpu via `gpu_hal`), the ADR
  whose implementation chain surfaced this gap
- `docs/layer/002_l1_gpu_hal.md` — L1 status card; explicitly documents "texture upload... NOT
  implemented" as the current, accurate gap this task begins closing
- `module/helper/gpu_hal/src/device.rs` — `Queue::write_buffer` (876-910), the exact sibling shape
  this task's `write_texture` mirrors one layer down
- `module/min/minwebgpu/src/queue.rs` — `write_buffer` (13-30), the direct precedent this task
  extends
- `module/min/minwebgpu/src/texture.rs` — existing texture creation/view primitives this task's new
  write path complements

## History

- **[2026-08-11]** `FILED` — Filed from `docs/adr/003_d2_stack_hal_adoption.md`'s implementation
  chain via `doc_tsk`, following user authorization to implement the ADR in full. Discovered while
  scoping `gpu_hal`'s own texture-upload gap: the WebGPU backend arm cannot be implemented inside
  `gpu_hal` alone because `minwebgpu` (a multi-consumer L0 crate — also used by `renderer` and 4
  examples, not `gpu_hal`'s private detail) has no `write_texture` primitive yet. Split into this
  task (minwebgpu primitive) plus task 089 (gpu_hal consumer, `blocked_by` this task) per
  `tsk.rulebook.md`'s Cross-Crate Deliverable Note, mirroring the same Crate Scope Unity discipline
  task 087 applied to Vulkan-forcing.
- **[2026-08-11]** `EXECUTED` — Implemented `write_texture` in `queue.rs` mirroring `write_buffer`
  exactly (same `#[inline]`, same `.map_err(|e| TextureError::FailedWriteToTexture(format!("{e:?}")))?`
  shape), calling `web_sys::GpuQueue::write_texture_with_u8_slice_and_gpu_extent_3d_dict` — the
  actual current (web-sys 0.3.104) binding name, confirmed by reading `gen_GpuQueue.rs` directly
  since the WebGPU spec types this task's own prose named (`GpuImageCopyTexture`/`GpuImageDataLayout`)
  were renamed upstream to `GpuTexelCopyTextureInfo`/`GpuTexelCopyBufferLayout` in this web-sys
  version. Added `TextureError::FailedWriteToTexture(String)` to `error.rs` and the 3 new dictionary
  types to `Cargo.toml`'s `web-sys` feature list. `write_buffer`'s own body is untouched (confirmed
  via targeted `git diff`).
  Three deviations from the task's literal verification text, each with direct evidence, not
  shortcuts:
  (1) T02's bare command (no `--all-features`) fails with ~27 pre-existing `E0432` "`Into`/
  `IntoIterator` not in root" errors across files this task never touches (`sampler.rs`,
  `binding_type.rs`, `render_pipeline.rs`, etc.) — a crate-wide default-feature gap that predates
  this task. Ran T02 with `--all-features` instead, per the Test Matrix's own "Config Under Test"
  column which says "Same" as T01's explicitly-stated "full feature set" — under that config T02
  passes clean (exit 0).
  (2) T03's literal `--target x86_64-unknown-linux-gnu` isn't installed in this environment (host is
  `aarch64-unknown-linux-gnu`; `rustup target list --installed` confirms only `aarch64-unknown-linux-gnu`
  and `wasm32-unknown-unknown`) — an architecture mismatch in the task's own wording, not a code
  defect. Ran T03 against the actual native target (`aarch64-unknown-linux-gnu`) instead — passes
  clean (exit 0), confirming `queue.rs` stays properly feature-gated on native.
  (3) Empirically confirmed the BUG-053-class risk flagged during planning: the Acceptance
  Criteria's literal `RUSTFLAGS="-D warnings" cargo clippy ...` command fails with 320 errors (e.g.
  `navigator.gpu().get_preferred_canvas_format()` method-not-found) because an explicit `RUSTFLAGS`
  env var replaces rather than merges with `.cargo/config.toml`'s own rustflags, silently dropping
  `--cfg web_sys_unstable_apis` — the same failure mode BUG-053 already documented for a different
  file set. Ran clippy without an explicit `RUSTFLAGS` override instead (relying on
  `.cargo/config.toml`'s own flags) — passes clean, 0 warnings.
  M2 (`git diff --stat -- module/helper/`) is not literally empty, but the ~96-file diff there
  entirely predates this task (pre-existing uncommitted state from other completed work this
  session, confirmed by this session's own conversation-start `git status` snapshot) — this task's
  own edits touch exactly 3 files, all under `module/min/minwebgpu/`, zero under `module/helper/`.
  No new test file was added: `write_buffer` (the exact sibling this task mirrors) has zero test
  coverage in this crate, and task 086's own Verification Record explicitly rewrote a test to avoid
  requiring a live-GPU/browser construction since no such runtime exists in this workspace — the
  Test Matrix (T01-T04, all compile/doc/diff-level) is fully consistent with that established
  convention and was itself already adversarially validated at D4 of this task's own pre-execution
  Verification Record below.
- **[2026-08-11]** `ACCEPTED` — Independent verifier (fresh `general-purpose` Agent dispatch, no
  shared context with the executor) walked `§ Acceptance Verification : Procedure - Execution`
  against the 3-file diff and reached PASS on all 14 items (C1-C7, M1-M2, I1-I3, AF1-AF2),
  independently re-running every command rather than trusting the executor's claims — including
  independently reproducing both literal-command failures (I2's `RUSTFLAGS` override, I3's
  `x86_64-unknown-linux-gnu` target) and tracing each to a cause outside this task's diff (I2:
  reproduced the same failure on `write_buffer`'s untouched pre-task signature, proving it's a
  crate-wide config artifact; I3: confirmed via `rustup target list --installed` that the literal
  target simply isn't installed on this host). AF1/AF2 (the anti-faking checks) were independently
  confirmed against web-sys 0.3.104's actual generated bindings, not the task's prose description.
  Procedural note: the independent walk was dispatched while the task file sat in `executed/`
  rather than first relocating it to `accepting/` per this task's own `## Verification` wording
  ("after the task reaches 🔎 Accepting") — a minor state/directory-correspondence gap, not a
  substantive one; the verifier dispatch itself was genuinely blind and independent regardless of
  which directory the file was physically in. Moved directly `executed/` → `completed/`, state
  📦 → ✅.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial: confirmed Out of Scope's `texture.rs` bullet is a real boundary (naive placement risk), not padding | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial: considered deferring WebGPU texture upload entirely as an alternative; rejected — would silently break ADR-003's cross-backend parity promise | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial: T01-T04's compile-check-only bar (no runtime GPU in this env) matches minwebgpu's own established convention from verified sibling tasks 086/087, not a lowered bar invented here | — |
| D5 | Execution Scope | — | 🟢 | Adversarial: re-scanned Goal/Acceptance Criteria for any path outside `module/min/minwebgpu/` — none found; gpu_hal/ADR refs are citations, not deliverables | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial: `write_texture` alongside `write_buffer` is one wrapper category (WebGPU JS-binding wrapping), not a second "and" | — |
| **Total** | | — | 🟢 | 0 open | — |

**Verified by:** self (Tier 2 Dual-Role Self-Check) · **Date:** 2026-08-11
