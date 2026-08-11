# Add `gpu_hal`-backed `adapter-webgpu` Backend to `tilemap_renderer`

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-11 13:37:03
- **expires_at:** 2026-08-11 15:37:03
- **round:** 1
- **state:** ⚙️ (Executing)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** module/helper/tilemap_renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **executing_at:** 2026-08-11 13:37:03
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

Add a `WebGpuBackend` to `module/helper/tilemap_renderer/src/adapters/` implementing the crate's
`Backend` trait over `gpu_hal`'s browser WebGPU surface (`Device::new_webgpu(canvas).await`,
`gpu_hal`'s `webgpu` feature), gated behind a new `adapter-webgpu` Cargo feature following the
existing per-adapter convention — `tilemap_renderer` becomes the second L3 engine to depend on
`gpu_hal`, per `docs/adr/003_d2_stack_hal_adoption.md` Decision #1, proving L1's WebGPU-shaped
contract against d2's flat POD command stream (a materially different stack shape than d3's scene
graph, `renderer`'s existing consumer). Motivated by `pingpong_animation` wanting a WebGPU render
target alongside its SVG/WebGL ones. Scoped to one new file, one feature line, one `mod.rs`
registration, its `Cargo.toml` dependency on `gpu_hal`, and its tests. Testable:
`cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgpu` exits 0
(browser-only backend — no native pixel test exists yet, matching `docs/layer/002_l1_gpu_hal.md`'s
own noted gap for `renderer`'s WebGPU path: "browser-side runtime pixel tests still to run").

## In Scope

- New Cargo feature `adapter-webgpu = ["enabled", "dep:gpu_hal"]` in
  `module/helper/tilemap_renderer/Cargo.toml`, with `gpu_hal = { workspace = true, features = ["webgpu"], optional = true }`
  added to `[dependencies]`
- New file `module/helper/tilemap_renderer/src/adapters/webgpu.rs` defining `pub struct WebGpuBackend`
  and `impl Backend for WebGpuBackend`:
  - construction via `gpu_hal::Device::new_webgpu(canvas).await` (async — the constructor itself,
    e.g. `pub async fn new(config: RenderConfig, canvas: &web_sys::HtmlCanvasElement) -> Result<Self, RenderError>`,
    is necessarily async because `Device::new_webgpu` is; this makes `WebGpuBackend::new` the first
    async adapter constructor in the crate — every other adapter's `new()` is sync)
  - `load_assets` uploads textures/geometry via `gpu_hal::Device::create_texture` /
    `create_buffer_init`
  - `submit` translates `RenderCommand`s into `gpu_hal` render-pass calls via a
    `create_command_encoder` / `RenderPipeline` built from the crate's existing shader sources
  - `output` returns `Ok(Output::Presented)` (realtime GPU path, matching the trait doc example's
    `Output::Presented` case — no readback on the browser backend)
  - `resize` recreates the `gpu_hal::Surface`
  - `capabilities()` returns a `Capabilities` reflecting only what this task actually implements —
    honest per `Backend::Capabilities`'s existing contract (`docs/pattern/001`'s Consequences);
    unimplemented families (gradients/patterns/clip_masks/effects/text) stay `false`, matching the
    WebGL adapter's own current honesty precedent (`webgl.rs:1228-1236`)
- `#[ cfg( all( feature = "adapter-webgpu", target_arch = "wasm32" ) ) ] layer webgpu;` registration
  in `module/helper/tilemap_renderer/src/adapters/mod.rs` (target-gated: WebGPU is wasm32-only,
  matching `gpu_hal`'s own `#[cfg(all(feature = "webgpu", target_arch = "wasm32"))]` gating)
- Minimum shape of `RenderCommand`→GPU-work translation needed to draw at least sprites (the
  simplest command family `pingpong_animation`'s compiler produces per task 085) — full command
  coverage (paths/text/gradients) is explicitly not required by this task's own Acceptance Criteria
- Compile-time tests (`cargo check --target wasm32-unknown-unknown`) and any assertions runnable
  without a browser (capability struct correctness, command-translation logic isolated into a
  pure/testable function where feasible)

## Out of Scope

- A native/offscreen pixel-readback proof — `adapter-webgpu` is browser-only by construction
  (`gpu_hal::Device::new_webgpu` requires a `web_sys::HtmlCanvasElement`); pixel-correctness
  verification requires a browser runtime this workspace does not yet have test infrastructure for
  (same gap `docs/layer/002_l1_gpu_hal.md` already records for `renderer`'s own WebGPU path) — this
  task's own Acceptance Criteria are therefore compile-and-construct-level, not pixel-level; a
  future task adds the browser pixel-test harness once one exists for any consumer
- `adapter-native` — separate task (087); different `gpu_hal` feature (`native` vs `webgpu`),
  different target (`not(wasm32)` vs `wasm32`), independently testable today (native is), so kept
  as its own task rather than bundled
- Full `RenderCommand` coverage (text, gradients, patterns, clip masks, path tessellation) — these
  are the same capability gaps the existing WebGL adapter also carries (`webgl.rs:1228-1236`);
  `WebGpuBackend` inherits the same honest `false` flags, not a regression this task introduces
- Adding `adapter-webgpu` to the crate's `full` feature bundle — deferred for the same
  shared-line-contention reason as task 084's Out of Scope
- Wiring `adapter-webgpu` into `pingpong_animation` — that is follow-up work on task 085, blocked on
  this task existing first (same feature-forwarding constraint documented in 085's Out of Scope)

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
-   `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgpu`
    exits 0 with zero warnings (`RUSTFLAGS="-D warnings"`); native-target build is unaffected
    (`adapter-webgpu` compiles to nothing outside wasm32 per its `cfg` gate)
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgpu` | wasm32 target | Exit 0, zero warnings |
| T02 | `cargo build -p tilemap_renderer --target wasm32-unknown-unknown --no-default-features --features adapter-webgpu` (feature isolation) | wasm32, `adapter-webgpu` only | Compiles clean, no `adapter-svg`/`adapter-webgl`-only symbol leaks |
| T03 | The `Capabilities` value `WebGpuBackend` declares for this task's honest-subset scope, verified via whatever instance-independent means the implementation exposes — `capabilities()` is a static fact about which command families `submit` translates, never derived from live device/canvas state, so no actual `new()` call (and therefore no canvas) is required to check it | wasm32 target, no browser runtime required (consistent with this task's Out of Scope boundary) | Matches this task's declared honest set: `sprites: true`, `paths`/`text`/`gradients`/`patterns`/`clip_masks`/`effects`: `false` |
| T04 | `cargo check -p tilemap_renderer --all-features` (native target, default) | native target | Exit 0 — `webgpu.rs` module is entirely absent from the native build (target-gated), so it neither compiles nor breaks compilation there |
| T05 | Whatever internal function extracts draw parameters (position, resource id) from a `RenderCommand::Sprite` — implementer's own naming/shape, isolated enough to call without a live `gpu_hal::Device` | wasm32 or native (pure fn, no GPU context) | Given two `RenderCommand::Sprite` values with different positions, the two extracted parameter sets differ accordingly — not a hardcoded/constant result |

## Acceptance Criteria

-   `module/helper/tilemap_renderer/src/adapters/webgpu.rs` exists, exports
    `pub struct WebGpuBackend` and `impl Backend for WebGpuBackend` implementing all 5 trait methods
-   `module/helper/tilemap_renderer/Cargo.toml` contains `adapter-webgpu = ["enabled", "dep:gpu_hal"]`
    and a `gpu_hal` optional dependency with the `webgpu` feature forwarded
-   `module/helper/tilemap_renderer/src/adapters/mod.rs` registers `webgpu` behind
    `#[cfg(all(feature = "adapter-webgpu", target_arch = "wasm32"))]`
-   Every row T01–T05 in `## Test Matrix` has a corresponding passing check/test
-   `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgpu`
    exits 0
-   `WebGpuBackend::capabilities()` contains no `true` flag for a command family this task's
    `submit` does not actually translate

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**`webgpu.rs` — Backend implementation**
- [ ] C1 — Does `WebGpuBackend::new` construct via `gpu_hal::Device::new_webgpu(canvas).await`?
- [ ] C2 — Does `load_assets` upload via `gpu_hal::Device::create_texture` / `create_buffer_init`
      (not a stub)?
- [ ] C3 — Does `submit` translate at least sprite `RenderCommand`s into real `gpu_hal` draw calls
      (a `create_command_encoder` + `RenderPipeline` path is exercised, confirmed by reading the
      implementation, not merely by a passing pure-function test)?
- [ ] C4 — Does `capabilities()` report `false` for every command family `submit` doesn't translate?

**Feature/target gating**
- [ ] C5 — Is `adapter-webgpu` target-gated to `wasm32` only in both `Cargo.toml`'s `gpu_hal`
      dependency shape and `adapters/mod.rs`'s `#[cfg(...)]`?
- [ ] C6 — Does a native-target build (`--all-features`, no `--target wasm32-unknown-unknown`)
      compile with `webgpu.rs` entirely absent from the build (verified via
      `cargo check -p tilemap_renderer --all-features` succeeding with no wasm-only symbol errors)?

**Out of Scope confirmation**
- [ ] C7 — Is `adapter-svg.rs` / `terminal.rs` / `webgl.rs` / `none.rs` byte-identical to its
      pre-task state?
- [ ] C8 — Is `Cargo.toml`'s `full` feature line unchanged?
- [ ] C9 — Is `pingpong_animation`'s `Cargo.toml` untouched by this task?
- [ ] C10 — Does this task add no native/offscreen pixel-readback path (no `read_pixels` call, no
      `Output::Bitmap` construction) — confirming the browser-only, compile-and-construct-level
      scope held?
- [ ] C11 — Is `native.rs` either absent from the crate, or (if task 087 already landed)
      byte-identical to its pre-task state?

### Measurements

- [ ] M1 — `WebGpuBackend` line count: `wc -l module/helper/tilemap_renderer/src/adapters/webgpu.rs`
      (was: file did not exist)
- [ ] M2 — Warning count on wasm32 target build: `cargo check --target wasm32-unknown-unknown --features adapter-webgpu 2>&1 | grep -c warning` → expected `0` (was: N/A, feature did not exist)

### Invariants

- [ ] I1 — wasm32 target build: `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgpu` → exit 0
- [ ] I2 — Native target build unaffected: `cargo nextest run -p tilemap_renderer --all-features` → 0 failures (webgpu.rs excluded from this target)
- [ ] I3 — Compiler/lints (wasm32): `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgpu -- -D warnings` → 0 warnings

### Anti-faking checks

- [ ] AF1 — `capabilities()` isn't over-claimed to pass a superficial check: cross-reference every
      `true` flag against an actual `submit` code path handling that `RenderCommand` variant —
      any `true` flag with no matching match-arm in `submit` is a fabricated capability
- [ ] AF2 — `submit` doesn't silently swallow unsupported commands as success: unsupported command
      variants must return `RenderError::Unsupported`, verified by a test asserting `Err` (not `Ok`)
      for a command family `capabilities()` reports `false` for

## Related Documentation

- `docs/adr/003_d2_stack_hal_adoption.md` — Decision #1 (adapter-webgpu via `gpu_hal`)
- `docs/layer/002_l1_gpu_hal.md` — L1 status card; "second targeted consumer" row this task fulfills
- `docs/layer/004_l3_stack_engine.md` — L3 engine card, downward-seam cell this task fulfills
- `docs/adr/002_gpu_hal_in_house.md` — the HAL build-vs-buy decision this task's dependency rests on
- `module/helper/tilemap_renderer/docs/pattern/001_ports_and_adapters_backend_architecture.md` —
  adapter architecture and `Capabilities` honesty contract this task must follow
- `module/helper/gpu_hal/src/device.rs` — `Device::new_webgpu`, `create_texture`,
  `create_command_encoder` surface this task consumes

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 13:37:03 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-11 14:40:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Implementation written: `src/adapters/webgpu.rs` (409 lines), `Cargo.toml` feature/dep, `adapters/mod.rs` layer registration, and a `lib.rs` top-level `adapters` gate fix (was missing `feature = "adapter-webgpu"` in the `any(...)` list — without it the whole `adapters` module compiled to nothing under `--features adapter-webgpu` alone, so T01/T02's first "clean" passes were false positives that silently compiled zero of this task's own code). `tests/webgpu_backend_test.rs` covers T03/T05/AF2 as pure-function `#[wasm_bindgen_test]`s (no live device) and genuinely executed via the headless-browser wasm32 runner: 3/3 passed. T01/T02/T04 passed cleanly on isolated runs. I3 (`cargo clippy --target wasm32-unknown-unknown --features adapter-webgpu -- -D warnings`) and a T01 re-run intermittently failed on `gpu_hal`'s own pre-existing code (`#[must_use]`/`# Errors`/`dead_code` in `pass.rs`/`resource.rs`/etc, not in `tilemap_renderer`) — confirmed via mtime that `gpu_hal/src/{resource,types,webgl}.rs` were edited live, minutes apart, by a concurrent actor during this session's own verification run. `gpu_hal` is not in this task's In Scope; did not touch it. I3/T01-stability is left open pending the concurrent edit settling — re-run once `gpu_hal`'s working tree is quiescent. |

## History

- **[2026-08-11]** `FILED` — Filed from `docs/adr/003_d2_stack_hal_adoption.md` Decision #1 via
  `doc_tsk`, following user authorization to implement the ADR in full. Goal: second `gpu_hal`
  consumer, WebGPU leg, proving the HAL against d2's command-stream shape.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | T05 prescribed an invented internal fn/type signature (`sprite_command_to_draw_desc`/`DrawDesc`); T03 claimed "freshly-constructed instance (no canvas needed)" while `new()` requires a live `web_sys::HtmlCanvasElement` — self-contradictory given no browser test runtime exists in this workspace | T05 rewritten to describe observable differential behavior, implementation-agnostic; T03 rewritten to test the static `Capabilities` value via an instance-independent means, dropping the canvas-construction claim entirely |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 2/2 |
