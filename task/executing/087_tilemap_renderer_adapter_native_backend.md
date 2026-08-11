# Add `gpu_hal`-backed `adapter-native` Backend to `tilemap_renderer`

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-11 16:09:11
- **expires_at:** 2026-08-11 18:09:11
- **round:** 1
- **state:** ⚙️ (Executing)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** module/helper/tilemap_renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

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

## History

- **[2026-08-11]** `FILED` — Filed from `docs/adr/003_d2_stack_hal_adoption.md` Decision #1 via
  `doc_tsk`, following user authorization to implement the ADR in full. Goal: second `gpu_hal`
  consumer, native leg, with an in-repo pixel-readback proof; Vulkan-forcing explicitly deferred
  pending a `gpu_hal`-side backend-selection capability that does not exist today.

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
