# gpu_hal browser pixel verification (webgpu + webgl2 backends)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 2
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/gpu_hal
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** null
- **blocked_by:** null
- **executing_at:** 2026-08-19 00:46:27
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false
- **accepting_at:** 2026-08-19 00:46:27
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-19 10:21:54

## Goal

`gpu_hal`'s `native` backend is pixel-verified end-to-end by `triangle_render_readback`
(`tests/native_backend_test.rs`), but its `webgpu` and `webgl` backends have zero
render-level coverage — the crate readme's own Directory Layout table states
"Integration tests (native backend only)," and `docs/layer/002_l1_gpu_hal.md` line 36
explicitly names this as an open gap ("browser-side runtime pixel tests still to
run"). Close it by adding a minimal wasm example that draws a triangle through
`Device::new_webgpu( canvas )` and `Device::new_webgl( canvas )`, and verifying the
actual painted pixels via a real browser using `browsee` (`.wait for::render` then
`.pixel`) — the same category of proof `triangle_render_readback` gives the native
backend, just through a browser instead of an offscreen wgpu readback. Success is
testable by a documented `browsee` command sequence reporting the triangle's
configured color at the expected pixel and the clear color elsewhere, for both
backends, plus `docs/layer/002_l1_gpu_hal.md` line 36 citing this task's completion
in place of the open-gap parenthetical (mirroring how `texture_write_readback`/task
089 is cited later in the same paragraph, once the native backend's own coverage is
described).

## In Scope

- A new minimal example crate under `examples/gpu_hal/` (e.g.
  `examples/gpu_hal/triangle_browser/`) that constructs a `Device` via
  `Device::new_webgpu( canvas )` and, behind a separate feature/build, via
  `Device::new_webgl( canvas )`, and issues one render pass drawing a single
  triangle of a known, fixed color — reusing `triangle_render_readback`'s existing
  WGSL vertex/fragment shaders and vertex data where they carry over directly.
- Registering the new example crate in the root `Cargo.toml` workspace members and
  the gallery tracking files (`examples/readme.md`, `examples/index.md`,
  `examples/index.html`, `examples/demo_completeness.md`), per this repo's
  established example-crate registration convention.
- Building the example for `wasm32-unknown-unknown` under each backend feature, and
  using `browsee` (`.launch` -> `.wait for::render` -> `.pixel`/`.shot`) to confirm:
  the triangle's configured color appears at its expected on-canvas pixel, and the
  configured clear color appears at a pixel outside the triangle's bounds — for
  both `webgpu` and `webgl` builds.
- A `tests/manual/readme.md` entry in `gpu_hal` documenting this as a scripted
  browser-verification procedure (prerequisites, exact `browsee` commands, expected
  pixel readings) — this is not `cargo test`-automatable, since it requires an
  actual browser.
- Updating `docs/layer/002_l1_gpu_hal.md` line 36 to replace the "(browser-side
  runtime pixel tests still to run)" parenthetical with a completed-state citation
  of this task, in the same style as the `texture_write_readback`/task 089 sentence
  later in the same paragraph.
- Updating `gpu_hal/readme.md`'s `## Verify` section to document the new browser
  verification command sequence alongside the existing native `cargo nextest`
  command.

## Out of Scope

- `renderer`'s own canonical opaque-path browser pixel test (`src/webgpu/renderer.rs`,
  gated on the same "browser-side runtime pixel tests still to run" language in
  `docs/layer/002_l1_gpu_hal.md`) — a distinct, not-yet-filed gap on a different
  crate, sharing this task's root cause but out of scope here.
- `tilemap_renderer`'s `adapter-webgpu` browser pixel test (same shared root cause,
  explicitly cross-referenced in `docs/layer/002_l1_gpu_hal.md` as "the same
  browser-side-pixel-test gap noted above for `renderer`") — a distinct crate, out
  of scope here.
- `minwebgl`'s own live-GL-context browser test — a separate crate-scoped task
  (task 192).
- Any change to `gpu_hal`'s `src/` implementation. If the browser verification
  uncovers an actual rendering defect, file it as a new `BUG-NNN` per
  `bugs/file.rulebook.md` rather than patching it inside this task.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before the example crate is authored
-   Every Test Matrix case backed by an actual `browsee` pixel reading, not an
    assumption from source inspection
-   Minimum example code to satisfy Test Matrix — no features beyond a single
    triangle draw per backend
-   `verb/test` passes with zero failures and zero warnings (native regression
    check — this task does not touch native code paths)
-   `cargo check -p gpu_hal --features webgpu,webgl --target wasm32-unknown-unknown`
    (plus the new example crate) compiles clean — never env-prefix `RUSTFLAGS`/
    `RUSTDOCFLAGS` for this check, it clobbers `.cargo/config.toml`'s required
    `--cfg web_sys_unstable_apis`
-   No function exceeds 50 lines; no duplication; public items have `///` doc
    comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Load triangle example in a real browser via `browsee`, `webgpu` feature build | `Device::new_webgpu( canvas )` + one render pass | `browsee .wait for::render` exits 0; `browsee .pixel region::center` reports the triangle's configured color |
| T02 | Load triangle example in a real browser via `browsee`, `webgl` feature build | `Device::new_webgl( canvas )` + one render pass | Same as T01, WebGL2 backend |
| T03 | Sample a pixel outside the triangle's bounds (e.g. a canvas corner) | Same page, either backend | `browsee .pixel` reports the configured clear color, confirming the draw call is bounded |
| T04 | `cargo check -p gpu_hal --features webgpu,webgl --target wasm32-unknown-unknown` (example crate included) | New example crate | Compiles clean, no `RUSTFLAGS` env override |
| T05 | `cargo nextest run -p gpu_hal --features native` | Existing native suite | Still passes — unaffected regression check |

## Acceptance Criteria

-   `browsee`-driven pixel verification confirms the triangle's configured color at
    its expected location for both `webgpu` and `webgl` backends
-   A pixel outside the triangle reads the clear color for both backends
-   `docs/layer/002_l1_gpu_hal.md` line 36 cites this task's completion in place of
    the open-gap parenthetical
-   `gpu_hal/tests/manual/readme.md` documents the exact reproduction commands
-   Every Test Matrix row has a corresponding passing check

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Example crate**
- [ ] C1 — Does `examples/gpu_hal/triangle_browser/` (or equivalent) exist, registered in root `Cargo.toml` workspace members?
- [ ] C2 — Is it registered in all 4 gallery tracking files (`examples/readme.md`, `index.md`, `index.html`, `demo_completeness.md`)?

**Browser verification**
- [ ] C3 — Does `gpu_hal/tests/manual/readme.md` document the exact `browsee` command sequence and expected pixel readings for both backends?
- [ ] C4 — Does `docs/layer/002_l1_gpu_hal.md` line 36 cite this task instead of the open-gap parenthetical?

**Out of Scope confirmation**
- [ ] C5 — Is `module/helper/gpu_hal/src/` untouched (zero diff)?
- [ ] C6 — Are `renderer` and `tilemap_renderer` untouched by this task?

### Measurements

- [ ] M1 — webgpu center pixel: `browsee .pixel region::center` on the `webgpu` build → matches the example's configured triangle color (was: no example existed)
- [ ] M2 — webgl center pixel: same command on the `webgl` build → matches the same configured color
- [ ] M3 — corner pixel, either backend: `browsee .pixel region::<corner>` → matches the configured clear color

### Invariants

- [ ] I1 — native test suite: `cargo nextest run -p gpu_hal --features native` → 0 failures (unaffected by this task)
- [ ] I2 — wasm32 compiles clean: `cargo check -p gpu_hal --features webgpu,webgl --target wasm32-unknown-unknown` → 0 errors, no `RUSTFLAGS` env override used

### Anti-faking checks

- [ ] AF1 — real paint, not a stale/blank canvas: `browsee .wait for::render timeout::60` exits 0 before any `.pixel` call is trusted (per the browsee skill's core rule — never trust launch exit code alone as proof a page painted)
- [ ] AF2 — bounded draw, not a full-canvas clear: T03's corner-pixel check reads the clear color, not the triangle color — guards against a test that would pass even if the draw call painted the whole canvas the triangle's color

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one Fix-and-Recheck iteration, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | Goal/In Scope claimed the `texture_write_readback`/task 089 citation sits "two lines below"/"immediately following" the gap sentence at `docs/layer/002_l1_gpu_hal.md` line 36 — live re-read of the file showed it's actually ~13 lines later, separated by a full paragraph about the native backend | Reworded both citations to "later in the same paragraph, once the native backend's own coverage is described" |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 open | 1 fix |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 11:30:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 12:10:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 12:10:38 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 191` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with task 202/206 precedent; not forced/spoofed, left at 🔎 Accepting per standing project convention |
| 2026-08-19 00:40:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-19 10:21:54 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ACCEPTANCE_FAIL | acceptance failed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk`-methodology gap-closure round (docs/layer follow-up): add browser-side pixel-verified render coverage for `gpu_hal`'s webgpu/webgl2 backends.

## Related Documentation

- `docs/layer/002_l1_gpu_hal.md` — the doc instance carrying the open-gap parenthetical this task resolves
- `module/helper/gpu_hal/readme.md` — crate readme's `## Verify` section and Directory Layout table this task updates
- `module/helper/gpu_hal/tests/native_backend_test.rs` — the `triangle_render_readback` precedent this task mirrors for the browser backends

## Outcomes

### Acceptance Results

**Verified by:** user1@w002 (Round 4 re-verification — Tier 2 Dual-Role Self-Check)
**Date:** 2026-08-19
**Verdict:** FAIL (2 Blocking issues)

**Separation of Concerns:** Original implementation executed 2026-08-16 by a prior session (Journal: CLAIM_EXEC 11:30:34, EXEC_COMPLETE 12:10:32) — distinct from this one. The acceptance-claim window then expired unused (TIMEOUT_2H, 2026-08-19 00:40:42) with no acceptance walk ever completed. This session performed only the mechanical re-claim bookkeeping chain (CLAIM_EXEC → EXEC_COMPLETE → CLAIM_ACCEPT, all 00:46:27) to restore 🔎 Accepting for verification — it did not re-execute or modify the implementation; this walk's findings are independently re-derived against current source. The re-claim chain stamped `executing_by`/`accepting_by` to this session's own `user@host`, so `tsk .acceptance_pass` would hard-block on the same-actor guard regardless of verdict — moot here since the verdict is FAIL.

#### Checklist

- C1 — PASS. `examples/gpu_hal/triangle_browser/Cargo.toml` exists; covered by root `Cargo.toml`'s `"examples/gpu_hal/*"` workspace-member glob (line 77) — no explicit per-crate listing needed.
- C2 — PASS. `triangle_browser` present in all 4 gallery files: `examples/readme.md`, `examples/index.md`, `examples/index.html`, `examples/demo_completeness.md`.
- C3 — PASS. `gpu_hal/tests/manual/readme.md` documents exact `trunk serve` + `browsee .launch/.wait for::render/.shot/.pixel` command sequences for both `webgpu` and `webgl` builds, including a window-chrome pixel-offset troubleshooting section.
- C4 — PASS (Non-Blocking note). `docs/layer/002_l1_gpu_hal.md` (current lines 60-62) correctly cites this task's completion. The task's own Goal section self-cites "line 36" — now stale (doc has grown since filing); content is correct, only the line-number self-reference drifted.
- C5 — **FAIL (Blocking)**. `module/helper/gpu_hal/src/` is NOT zero-diff. `git show 3843aef7 --stat -- module/helper/gpu_hal/src/` shows `device.rs` (+33/-4) and `error.rs` (+9/-2) modified in the same commit as this task's own deliverables. The diff is a self-documented `Fix(BUG-176)` (zero-sized-texture validation guard in `texture_create`, new `Error::InvalidInput` variant). `BUG-176` was properly filed (now `task/bug/completed/176_texture_create_zero_size_unguarded.md`) — but this task's own Out-of-Scope states: "Any change to gpu_hal's src/ implementation. If the browser verification uncovers an actual rendering defect, file it as a new BUG-NNN... rather than patching it inside this task." The fix was filed AND patched inline in the same commit — exactly the sequence the clause prohibits. (The fix's own content is well-reasoned and plausibly correct — a canvas can legitimately report zero size mid-load — the process violation is independent of the fix's quality.)
- C6 — PASS (observation). The same commit also touches 23 `renderer` test/shader files (~1556 insertions) — commit message ("Expand renderer test suite...", "Improve animation scaling...") and diff content (animation/blending/outline/shadow/geometry test expansion, nothing touching Device/canvas construction) indicate unrelated work bundled into the same large commit, consistent with this repo's documented multi-task-commit pattern (cf. task 246's Outcomes). `tilemap_renderer`: zero diff.
- C7 (derived from Delivery Requirements — not present in original Checklist, added during this walk per the Pre-Walk Gate's missing-content rule) — **FAIL (Blocking)**. Delivery Requirements require "No function exceeds 50 lines." `triangle_draw` (`examples/gpu_hal/triangle_browser/src/main.rs:58-151`) spans 94 lines — 88% over the ceiling.

#### Measurements

- M1, M2, M3 — **NOT RUN this walk.** Live `browsee` verification requires standing up a `trunk serve` dev server; deferred to avoid resource contention with the concurrently-running full-workspace background verification build (shared `target/`). Both C5 and C7 require source edits to the exact files these measurements exercise (`main.rs`'s `triangle_draw`, `gpu_hal/src/device.rs`) — any reading captured now would be invalidated by the required rework, so capture is deferred to the post-fix re-verification pass rather than wasted now.

#### Invariants

- I1 — **NOT INDEPENDENTLY CONFIRMED this walk.** Shared full-workspace `verb/test` run for this task batch was still in progress at time of this walk (Durable Log `-0001_longrun.log`, pid 1587932).
- I2 — **NOT CONFIRMED this walk.** `cargo check -p gpu_hal --features webgpu,webgl --target wasm32-unknown-unknown` attempted with a 25s bound, did not complete (likely lock contention with the concurrent full-workspace build on shared `target/`) — genuinely not run to completion, not treated as a failure.

#### Anti-faking checks

- AF1 — PASS (procedural). Documented sequence correctly places `browsee .wait for::render timeout::60` before every `.shot`/`.pixel` call (`tests/manual/readme.md` lines 54/73/106). Not independently re-confirmed via a live run this walk.
- AF2 — PASS (procedural). Documented procedure includes a distinct corner-pixel check (T03) separate from center-pixel checks (T01/T02), with troubleshooting guidance for window-chrome pixel-offset correction. Not independently re-confirmed via a live run this walk.
