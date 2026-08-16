# WebGL2 adapter test coverage and cross-backend command-consistency check

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Give `tilemap_renderer`'s WebGL2 adapter (`src/adapters/webgl.rs`) the same
compile-and-construct-level test coverage every sibling adapter already has
(`none_backend_test.rs`, `svg_backend_test.rs`, `native_backend_test.rs`, and
`webgpu_backend_test.rs` all exist; no `webgl_backend_test.rs` does), and add
a cross-backend command-consistency test proving every backend constructible
without a live device honors its own `capabilities()` claim against the same
fixed `RenderCommand` fixture set. Matters now because
`docs/layer/003_l2_frame_orchestration.md`'s Embedded Instances Today section
documents the WebGL2 adapter's per-batch VAO lifecycle as established fact
with zero test citation backing it, and the 2026-08-15 docs/layer gap audit
flagged this as the last adapter in the crate without any dedicated test
file. Bounded to one pure-function extraction (`WebGlBackend::capabilities()`'s
body, parameterized on `max_texture_size`) plus two new test files in this
one crate. Testable: `cargo test -p tilemap_renderer --features
adapter-webgl,adapter-none,adapter-svg,adapter-native` exits 0 with the new
tests present and passing.

## In Scope

- `module/helper/tilemap_renderer/src/adapters/webgl.rs`: extract
  `capabilities()`'s body into a new pure associated function
  `WebGlBackend::declared_capabilities( max_texture_size : u32 ) ->
  Capabilities`; `capabilities( &self )` becomes a one-line delegate:
  `Self::declared_capabilities( self.max_texture_size )`.
- New `module/helper/tilemap_renderer/tests/webgl_backend_test.rs`,
  feature-gated `#![ cfg( feature = "adapter-webgl" ) ]` only — no
  `target_arch = "wasm32"` / `wasm_bindgen_test` needed, since the extracted
  function touches no `web_sys`/`wasm_bindgen` types — mirroring
  `webgpu_backend_test.rs`'s two-test shape:
  - honest-subset pin: `meshes`/`sprites`/`batches` true;
    `paths`/`text`/`gradients`/`patterns`/`clip_masks`/`effects`/
    `blend_modes`/`text_on_path` false; `supported_blend_modes` equals
    `[ Normal, Add, Multiply, Screen ]`.
  - anti-hardcoding pin: two different `max_texture_size` inputs produce two
    different `Capabilities.max_texture_size` outputs.
- New `module/helper/tilemap_renderer/tests/command_consistency_test.rs`: a
  shared fixed `RenderCommand` fixture set (one `Sprite`, one command from an
  unsupported family) submitted through the `none`/`svg`/`native` backends
  (each already constructible without a live external device per their own
  existing test files), asserting for each backend that every command family
  its own `capabilities()` marks `true` is accepted by `submit()` without
  `Err`, and every family marked `false` is handled per that backend's own
  documented policy (reject-with-`Err` or graceful no-op) — never a panic.

## Out of Scope

- Browser-runtime / live-`WebGl2RenderingContext` pixel verification — this
  workspace has no native/offscreen WebGL2 provider (confirmed: no
  swiftshader/osmesa/surfman/glutin dependency anywhere in
  `minwebgl`/`mingl`/`tilemap_renderer`'s `Cargo.toml`), so this remains the
  same accepted, already-documented gap `webgpu_backend_test.rs`'s own doc
  comment names for WebGPU. Closing it is a workspace-wide test-infrastructure
  decision, not a leaf-crate task.
- Making all backends' `capabilities()` report identical flags — they
  legitimately differ by design (e.g. `WebGpuBackend` reports `meshes:
  false`, `WebGlBackend` reports `meshes: true`); this task tests each
  backend's own self-consistency against its own claim, not cross-backend
  uniformity.
- `adapter-terminal`; `adapter-webgpu`'s own instance-level `submit()`
  (already covered by its own test file, needs `wasm32`).
- Any change to `RenderError`, `Capabilities`, or `RenderCommand` type
  definitions.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `capabilities()` extraction lands with zero behavior change — all
    pre-existing `tilemap_renderer` tests stay green
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its
    implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///`
    doc comments
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Fresh `Capabilities` from `declared_capabilities( 4096 )` | `WebGlBackend::declared_capabilities` | meshes/sprites/batches=true; paths/text/gradients/patterns/clip_masks/effects/blend_modes/text_on_path=false; supported_blend_modes=[Normal,Add,Multiply,Screen] |
| T02 | `declared_capabilities( 2048 )` vs `declared_capabilities( 8192 )` | same fn, two inputs | `max_texture_size` differs between calls and equals the respective input each time |
| T03 | `none`/`svg`/`native` backends each submit one `Sprite` command | `Backend::submit` | Returns `Ok` for every backend (all three declare `sprites: true`) |
| T04 | Same 3 backends each submit one command from a family their own `capabilities()` marks `false` (e.g. `BeginPath`) | `Backend::submit` | Returns `Err` (reject) or `Ok` with no panic and no state corruption (graceful skip) — never a panic |

## Acceptance Criteria

-   `tests/webgl_backend_test.rs` exists and both its tests pass
-   `tests/command_consistency_test.rs` exists and its tests pass for
    `none`/`svg`/`native`
-   `WebGlBackend::capabilities( &self )` delegates to the new pure
    `declared_capabilities` fn (verified by reading the diff, not just by
    passing tests)
-   No pre-existing test in `tilemap_renderer`'s suite regresses
-   Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Adapter (webgl.rs)**
- [ ] C1 — Does `WebGlBackend::declared_capabilities( max_texture_size : u32 ) -> Capabilities` exist as a pure associated function (no `&self`)?
- [ ] C2 — Does `capabilities( &self )` delegate to it with `self.max_texture_size`?

**Tests**
- [ ] C3 — Does `tests/webgl_backend_test.rs` exist, gated `#![ cfg( feature = "adapter-webgl" ) ]` only (no wasm32/wasm_bindgen_test)?
- [ ] C4 — Does `tests/command_consistency_test.rs` exist covering `none`/`svg`/`native`?

**Out of Scope confirmation**
- [ ] C5 — Is any live-`WebGl2RenderingContext`-constructing call absent from both new test files?
- [ ] C6 — Do `RenderError`, `Capabilities`, `RenderCommand` type definitions remain unchanged (`git diff` shows no edits to `types.rs`/`commands.rs`/`backend.rs` type defs)?
- [ ] C7 — Do `NoneBackend`/`SvgBackend`/`NativeBackend`/`WebGpuBackend`'s own `capabilities()` outputs remain distinct from `WebGlBackend`'s (not homogenized to a shared value by this change)?
- [ ] C8 — Do `adapter-terminal` and `WebGpuBackend`'s own instance-level `submit()` remain untouched (`git diff` shows no edits to `adapters/terminal.rs` or `WebGpuBackend::submit`)?

### Measurements

- [ ] M1 — new test count: `cargo test -p tilemap_renderer --features adapter-webgl,adapter-none,adapter-svg,adapter-native --no-run 2>&1 | grep -c "webgl_backend_test\|command_consistency_test"` → ≥2 binaries built (was: 0)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p tilemap_renderer --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — T02's two `max_texture_size` assertions use different literal input values (not the same value asserted twice) — checked by reading `tests/webgl_backend_test.rs`, not merely by the test passing
- [ ] AF2 — T04's "unsupported family" test submits a command whose family is genuinely `false` in that backend's own `capabilities()` output (not a family that happens to be `true`) — cross-checked against each backend's own `capabilities()` body by reading the test, not assumed

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

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-15]** `FILED` — Task filed via `/doc_tsk` Phase 2 (docs/layer gap audit): add WebGL2 adapter test coverage + cross-backend command-consistency check to `tilemap_renderer`.

## Related Documentation

- `docs/layer/003_l2_frame_orchestration.md` — Embedded Instances Today section documents the WebGL2 adapter's per-batch VAO lifecycle claim this task backs with tests
- `docs/layer/004_l3_stack_engine.md` — `tilemap_renderer`'s L3 engine table entry
- `module/helper/tilemap_renderer/tests/webgpu_backend_test.rs` — the compile-and-construct-level precedent pattern this task's new tests mirror
- `module/helper/tilemap_renderer/src/backend.rs` — `Backend` trait and `Capabilities` struct definitions
