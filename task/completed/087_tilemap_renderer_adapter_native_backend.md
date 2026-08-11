# Add `gpu_hal`-backed `adapter-native` Backend to `tilemap_renderer`

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
- **unit:** module/helper/tilemap_renderer
- **verified_by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-11 18:15:26
- **blocked_by:** null
- **executing_at:** 2026-08-11 16:09:11
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false
- **accepting_at:** 2026-08-11 17:59:22
- **accepting_by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **priority:** 0
- **completed_at:** 2026-08-11 18:15:26
- **completed_by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

Add a `NativeBackend` to `module/helper/tilemap_renderer/src/adapters/` implementing the crate's
`Backend` trait over `gpu_hal`'s native `wgpu` surface (`Device::new_native(width, height)`,
`gpu_hal`'s `native` feature — offscreen render into a texture, read back via
`gpu_hal::Surface::read_pixels`), gated behind a new `adapter-native` Cargo feature, per
`docs/adr/003_d2_stack_hal_adoption.md` Decision #1. Proof shape mirrors `gpu_hal`'s own
`triangle_render_readback` test (`module/helper/gpu_hal/tests/native_backend_test.rs`) and
`renderer`'s `opaque_path_renders_lit_quad` — an in-repo, no-browser-required pixel assertion,
unlike `adapter-webgpu` (task 086) which has no such native-side proof available. Motivated by
`pingpong_animation` wanting an offscreen/native render target alongside its browser-facing ones.
Testable: `cargo test -p tilemap_renderer --features adapter-native` exits 0 and includes at least
one test that reads back real pixel bytes and asserts on their content.

## In Scope

- New Cargo feature `adapter-native = ["enabled", "dep:gpu_hal"]` in
  `module/helper/tilemap_renderer/Cargo.toml`, with `gpu_hal = { workspace = true, features = ["native"], optional = true }`
  added to (or merged into, if task 086 landed first) the existing `[dependencies]` `gpu_hal` entry
- New file `module/helper/tilemap_renderer/src/adapters/native.rs` defining `pub struct NativeBackend`
  and `impl Backend for NativeBackend`:
  - construction via `gpu_hal::Device::new_native(width, height)` — sync, using the plain,
    unmodified constructor exactly as `gpu_hal` exposes it today (no backend-selection parameter —
    see Out of Scope)
  - `load_assets` uploads textures/geometry via `gpu_hal::Device::create_texture` /
    `create_buffer_init`, identical resource-creation path to `adapter-webgpu`'s (task 086) where
    the two overlap — both go through the same `gpu_hal::Device` API, differing only in how the
    `Device` was constructed
  - `submit` translates `RenderCommand`s into `gpu_hal` render-pass calls (same minimum sprite-level
    coverage as task 086, applied to this backend's own pipeline objects — not shared code between
    the two tasks, since they may land in either order)
  - `output` returns `Ok(Output::Bitmap(bitmap))` via `gpu_hal::Surface::read_pixels(&device, &queue)`,
    matching `Backend::Output`'s existing `Bitmap` variant contract (raw bytes + width + height +
    channels) — the offscreen/readback path `docs/adr/003`'s Decision #1 names
  - `resize` recreates the offscreen texture at the new dimensions
  - `capabilities()` — same honesty discipline as task 086: only flags what `submit` actually
    translates
- `#[ cfg( all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ] layer native;`
  registration in `module/helper/tilemap_renderer/src/adapters/mod.rs`, mirroring `gpu_hal`'s own
  native-backend target gate
- At least one pixel-readback test in the style of `triangle_render_readback` /
  `opaque_path_renders_lit_quad`: draws a known, simple scene (e.g. one solid-color sprite) through
  the full `load_assets`→`submit`→`output` path and asserts specific bytes in the returned `Bitmap`

## Out of Scope

- **Vulkan-forced construction** (a second `NativeBackend` constructor variant that forces `wgpu`
  onto its Vulkan backend, the way `examples/minwgpu/sun_grid_lines_vulkan` forces
  `wgpu::Backends::VULKAN` at the `minwgpu::Context::builder()` level). Confirmed by reading
  `gpu_hal/src/device.rs:198-222`: `Device::new_native` hardcodes
  `minwgpu::context::Context::builder().make_instance().request_adapter()?.finish_context()?` with
  no backend-selection parameter anywhere in its signature or body — there is no explicit-backend
  constructor path in `gpu_hal` today for `adapter-native` to call into. Per Crate Scope Unity, a
  task whose deliverable is a `tilemap_renderer` adapter cannot also modify `gpu_hal` (a different
  crate) to add that capability. Vulkan-forcing therefore remains deferred, exactly the "strangle
  when triggered" posture `docs/adr/003_d2_stack_hal_adoption.md` already accepts for
  `adapter-webgl`'s HAL migration — a future task, scoped to `gpu_hal` itself, would add an
  explicit-backend-selection constructor (e.g. `Device::new_native_with_backends`) before this
  adapter could offer a Vulkan-forced variant
- `adapter-webgpu` — separate task (086); different `gpu_hal` feature/target, bundled separately for
  the same reasons stated in 086's own Out of Scope
- Full `RenderCommand` coverage — same honest-subset posture as task 086
- Adding `adapter-native` to the crate's `full` feature bundle — deferred, same shared-line
  rationale as tasks 084/086
- Wiring `adapter-native` into `pingpong_animation` — follow-up on task 085, blocked on this task
  existing first

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
-   `cargo nextest run -p tilemap_renderer --features adapter-native` passes with zero failures and
    zero warnings (`RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --all-targets --features adapter-native -- -D warnings`
    exits 0)
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `NativeBackend::new(RenderConfig { width: 64, height: 64, .. })` then `load_assets`+`submit` a single solid-color sprite command, then `output()` | `adapter-native` feature enabled, native target | Returns `Ok(Output::Bitmap(bitmap))` with `bitmap.width == 64`, `bitmap.height == 64` |
| T02 | Same scene as T01, inspect `bitmap.bytes` at the sprite's known pixel location and at a corner outside it | `adapter-native` feature enabled | `assert_eq!` (exact byte match, mirroring `triangle_render_readback`'s own `at(50,50)`/`at(0,0)` exact-equality assertions — no tolerance needed for a flat-color sprite on a flat clear color): sprite pixel equals the configured sprite RGBA; corner pixel equals the clear color |
| T03 | `resize(128, 128)` called after construction, then repeat T01's flow | `adapter-native` feature enabled | Returned `Bitmap` reflects the new `128x128` dimensions |
| T04 | `cargo build -p tilemap_renderer --no-default-features --features adapter-native` (feature isolation) | `adapter-native` only, native target | Compiles clean, no `adapter-webgpu`/`adapter-webgl`-only symbol leaks |
| T05 | `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --all-features` | wasm32 target | Exit 0 — `native.rs` module is entirely absent from the wasm32 build (target-gated) |

## Acceptance Criteria

-   `module/helper/tilemap_renderer/src/adapters/native.rs` exists, exports
    `pub struct NativeBackend` and `impl Backend for NativeBackend` implementing all 5 trait methods
-   `module/helper/tilemap_renderer/Cargo.toml` contains `adapter-native = ["enabled", "dep:gpu_hal"]`
    forwarding `gpu_hal`'s `native` feature
-   `module/helper/tilemap_renderer/src/adapters/mod.rs` registers `native` behind
    `#[cfg(all(feature = "adapter-native", not(target_arch = "wasm32")))]`
-   Every row T01–T05 in `## Test Matrix` has a corresponding passing test
-   `cargo nextest run -p tilemap_renderer --features adapter-native` exits 0
-   At least one test performs a genuine pixel-content assertion on `Output::Bitmap` bytes (not
    only a dimension check)
-   No Vulkan-backend-selection API surface is added anywhere in this task's diff (confirms the
    Out of Scope boundary held)

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**`native.rs` — Backend implementation**
- [ ] C1 — Does `NativeBackend::new` construct via the unmodified
      `gpu_hal::Device::new_native(width, height)`?
- [ ] C2 — Does `output()` return `Ok(Output::Bitmap(..))` via `gpu_hal::Surface::read_pixels`?
- [ ] C3 — Does `resize` recreate the offscreen texture (not just update stored dimensions with no
      effect on the actual `gpu_hal::Surface`)?
- [ ] C4 — Does `capabilities()` report `false` for every command family `submit` doesn't translate?

**Pixel proof**
- [ ] C5 — Does at least one test assert specific byte content in the returned `Bitmap` (T02), not
      merely that `output()` returns `Ok`?

**Feature/target gating**
- [ ] C6 — Is `adapter-native` target-gated to `not(wasm32)` in both `Cargo.toml` and
      `adapters/mod.rs`?

**Out of Scope confirmation**
- [ ] C7 — Does a repo-wide search find zero new backend-selection parameters added to
      `gpu_hal::Device::new_native`'s signature (`git diff -- module/helper/gpu_hal/` is empty)?
- [ ] C8 — Is `adapter-svg.rs` / `terminal.rs` / `webgl.rs` / `none.rs` / `webgpu.rs` byte-identical
      to its pre-task state?
- [ ] C9 — Is `Cargo.toml`'s `full` feature line unchanged?
- [ ] C10 — Is `pingpong_animation`'s `Cargo.toml` untouched by this task?

### Measurements

- [ ] M1 — `NativeBackend` line count: `wc -l module/helper/tilemap_renderer/src/adapters/native.rs`
      (was: file did not exist)
- [ ] M2 — `git diff --stat -- module/helper/gpu_hal/` line count: expected `0` (no files changed) —
      confirms the Vulkan-forcing deferral held and no cross-crate leak occurred

### Invariants

- [ ] I1 — Crate test suite: `cargo nextest run -p tilemap_renderer --all-features` → 0 failures
- [ ] I2 — Compiler/lints: `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] I3 — wasm32 target unaffected: `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --all-features` → exit 0, `native.rs` absent from that build

### Anti-faking checks

- [ ] AF1 — The pixel-readback test doesn't accept an all-zero or all-identical buffer as a false
      pass: assert both that the sprite's pixel differs from the background clear color AND matches
      the configured sprite color — a backend that clears the texture and never actually draws would
      otherwise still pass a weaker "bytes are non-empty" check
- [ ] AF2 — `capabilities()` isn't over-claimed: cross-reference every `true` flag against an actual
      `submit` match-arm, same anti-faking bar as task 086's AF1

## Related Documentation

- `docs/adr/003_d2_stack_hal_adoption.md` — Decision #1 (adapter-native via `gpu_hal`), Decision #3
  (Vulkan as backend-selection detail — the reason this task's own Out of Scope excludes it)
- `docs/layer/002_l1_gpu_hal.md` — L1 status card; native backend proof shape
  (`triangle_render_readback`) this task mirrors
- `docs/layer/004_l3_stack_engine.md` — L3 engine card, downward-seam cell this task fulfills
- `module/helper/tilemap_renderer/docs/pattern/001_ports_and_adapters_backend_architecture.md` —
  adapter architecture and `Capabilities` honesty contract
- `module/helper/gpu_hal/tests/native_backend_test.rs` — `triangle_render_readback`, the proof
  shape this task's own pixel-readback test mirrors
- `module/helper/gpu_hal/src/device.rs` — `Device::new_native`, `Surface::read_pixels` surface this
  task consumes unmodified

## Outcomes

<!-- verified implementation deliverables -->


## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 17:59:22 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-11 18:15:26 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ACCEPTANCE_PASS | acceptance passed |

## History

- **[2026-08-11]** `FILED` — Filed from `docs/adr/003_d2_stack_hal_adoption.md` Decision #1 via
  `doc_tsk`, following user authorization to implement the ADR in full. Goal: second `gpu_hal`
  consumer, native leg, with an in-repo pixel-readback proof; Vulkan-forcing explicitly deferred
  pending a `gpu_hal`-side backend-selection capability that does not exist today.
- **[2026-08-11]** `EXECUTED` — Implemented `NativeBackend` in `src/adapters/native.rs` (375 lines):
  constructs via unmodified `gpu_hal::Device::new_native(width, height)`, renders sprites through a
  shared `GpuState` (device/queue/surface/sampler/index-buffer/bind-group-layout/pipeline) rebuilt
  wholesale by `build_gpu_state` from both `new()` and `resize()`, uploads `ImageSource::Bitmap`
  assets via a real `queue.write_texture()` (any `PixelFormat` converted to tight RGBA8 via a local
  `to_rgba8()` helper), and reads back through unmodified `gpu_hal::Surface::read_pixels`.
  `capabilities()` honestly reports `sprites: true` only; `submit()`'s match arms mirror that claim
  exactly (`RenderCommand::Sprite` succeeds, every other family returns `Err(Unsupported)`).
  Wired `adapter-native = ["enabled", "dep:gpu_hal"]` into `Cargo.toml` (merging into task 086's
  pre-existing `gpu_hal` dependency line, adding its `native` feature alongside the existing
  `webgpu` one) and `#[cfg(all(feature = "adapter-native", not(target_arch = "wasm32")))] layer
  native;` into `adapters/mod.rs`.
  **Fix during implementation:** `adapters/mod.rs`'s own inner gate was correct, but `lib.rs`'s
  separate, outer `#[cfg(any(...))]` gate on `layer adapters;` itself had never been given
  `feature = "adapter-native"` — the whole `adapters` module compiled out at the crate's top level
  even with `mod.rs`'s gate correct, producing `E0433: cannot find adapters in tilemap_renderer`.
  Added the missing `feature = "adapter-native",` line to `lib.rs`. This was only surfaced by
  actually compiling the tests, not by code review of `native.rs`/`mod.rs`/`Cargo.toml` alone.
  Authored `tests/native_backend_test.rs` (T01–T03, C4, AF1, AF2) against a real native
  `gpu_hal` device (a software Vulkan ICD such as lavapipe suffices), mirroring
  `gpu_hal/tests/native_backend_test.rs::triangle_render_readback`'s exact-byte-equality style: a
  centered sprite quad asserted equal to its configured RGBA at the viewport center, and equal to
  the clear color at a corner outside it (rules out an all-clear false pass). During this session's
  live, shared working tree, this test file was independently rewritten in place by concurrent
  activity to an equivalent 3-test version (same exact-equality proof shape, different helper/test
  names, an 8×8 solid-red texture instead of a 1×1 tinted one, an explicit leading `Clear` command
  instead of relying on background fill) — re-verified by full nextest re-run rather than assumed
  correct, and left in place rather than reverted (functionally equivalent coverage; re-editing a
  live-contended file adds collision risk with no verification benefit). One residual defect in
  that version — 3× `clippy::default_trait_access` (`Default::default()` for `filter`/`mipmap`/
  `wrap` instead of the concrete `SamplerFilter`/`MipmapMode`/`WrapMode` types) — was fixed directly
  (`task/verified/-0051_longrun.log` showed the failure; fix confirmed clean in
  `task/verified/-0055_longrun.log`). Verifier note: the current test file's C4/AF2 coverage
  (`capabilities()` vs. `submit()` cross-reference; rejection of an unsupported command family) is
  not exercised by a dedicated automated test in the current 3-test version — both were manually
  verified against `native.rs` source directly during implementation (capabilities() returns
  `sprites: true` only; submit()'s match has exactly one non-`Unsupported` arm, `RenderCommand::Sprite`).
  **Diff-contamination note (read before verifying):** all 3 modified files are clean — `git diff`
  on `Cargo.toml`, `adapters/mod.rs`, and `lib.rs` each show only this task's own additions, no
  unrelated churn mixed in.
  **Test Matrix results:**
  - T01/T02/T03 — exercised by `native_backend_test.rs`'s 3 tests, confirmed passing via I1's full
    nextest run below (`construct_load_submit_output_returns_matching_dimensions`,
    `sprite_and_corner_pixels_match_configured_colors`, `resize_then_output_reflects_new_dimensions`)
  - T04 — `cargo build -p tilemap_renderer --no-default-features --features adapter-native` → exit 0,
    16s (`task/verified/-0062_longrun.log`)
  - T05 — see I3 below (identical command)
  **Invariant results:**
  - I1 — `cargo nextest run -p tilemap_renderer --all-features` → exit 0, 131 tests run, 131 passed,
    0 skipped (`task/verified/-0060_longrun.log`)
  - I2 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings`
    currently **fails** (exit 101, `task/verified/-0056_longrun.log`) — but independently isolated
    (re-run with only `adapter-webgl` instead of `--all-features`, `task/verified/-0059_longrun.log`,
    identical failure with `adapter-native` entirely absent) to a pre-existing `minwebgl` defect
    unrelated to this task: `clippy::cast_lossless` on `src/texture/d2.rs:363` (`img_width as f64`/
    `img_height as f64`), reached only because `--all-features` pulls in `adapter-webgl` →
    `dep:minwebgl`. Git-blamed to commit `9b71cf39` (2026-08-10, predates this task and this
    session's GPU HAL work entirely). Filed as `BUG-091` (`task/bug/draft/091_...md`) rather than
    fixed inline — `module/min/minwebgl/src/` is outside this task's own declared scope, and the
    file has recent same-day concurrent activity (commit `96bb2aef`). The Delivery Requirement's own
    narrower clippy command (`--features adapter-native`, not `--all-features`) passes clean: exit 0,
    0 warnings (`task/verified/-0055_longrun.log`) — this task's own changes are lint-clean.
  - I3 — `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --all-features` → exit 0
    (`task/verified/-0061_longrun.log`); `native.rs` absence mechanically confirmed via the build's
    own dep-info file (`target/wasm32-unknown-unknown/debug/deps/tilemap_renderer-f4ded20becff482d.d`
    lists `mod.rs, none.rs, svg.rs, terminal.rs, webgl.rs, webgpu.rs` — `native.rs` absent — even
    though `adapter-native` is nominally enabled by `--all-features`, correctly excluded by
    `mod.rs`'s `not(target_arch = "wasm32")` gate)
  **Measurements:**
  - M1 — `wc -l src/adapters/native.rs` → 375 lines (was: file did not exist)
  - M2 — `git diff --stat -- module/helper/gpu_hal/` → 4 files changed, 28 insertions (not the
    spec's literal "expected 0"). Confirmed none of this is from this task: this task's own `git
    status` touches only `tilemap_renderer/{Cargo.toml, src/adapters/mod.rs, src/lib.rs,
    src/adapters/native.rs, tests/native_backend_test.rs}` — zero files under `module/helper/gpu_hal/`.
    The `gpu_hal` diff (`device.rs`, `pass.rs`, `resource.rs`, `types.rs`) corresponds to task 089's
    already-completed-and-verified `write_texture` work, left uncommitted per this session's
    no-autonomous-commit constraint — same class of pre-existing-uncommitted-diff situation task
    088/090's own History entries already documented. The measurement's actual intent (Vulkan-forcing
    deferral held, no cross-crate leak from *this* task) is satisfied: `git diff --
    module/helper/gpu_hal/src/device.rs | grep new_native` is empty — zero new backend-selection
    parameters, confirming C7's own underlying question directly.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | T02 said "within GPU-readback tolerance" but the real precedent (`triangle_render_readback`) asserts exact byte equality — no tolerance mechanism was ever specified | T02 rewritten to `assert_eq!` exact-match, mirroring the precedent's own `at(50,50)`/`at(0,0)` assertions, plus a corner-pixel check against the clear color |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 1/1 |
