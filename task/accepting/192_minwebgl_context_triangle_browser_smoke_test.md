# minwebgl core context + triangle-draw browser pixel smoke test

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 00:46:27
- **expires_at:** 2026-08-19 02:46:27
- **round:** 1
- **state:** 🔎 (Accepting)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** system
- **verification_date:** null
- **blocked_by:** null
- **executing_at:** 2026-08-19 00:46:27
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** true
- **accepting_at:** 2026-08-19 00:46:27
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-19 00:40:42

## Goal

`minwebgl`'s pure-logic layer is natively tested (`tests/readme.md`: 5 files, task
069), but "anything that touches a live GL context or the DOM (context creation,
shaders, VAOs, textures, uniforms, file/fetch) is not natively testable and has no
test runner yet" per the crate's own readme — zero automated or scripted coverage
exists for `context::from_canvas`/`from_canvas_with`, the crate's most foundational
entry point. Close the narrowest slice of this gap — NOT comprehensive coverage —
with one `browsee`-driven, pixel-verified smoke test proving `from_canvas` plus a
minimal shader/buffer/draw sequence actually paints a triangle of the expected
color in a real browser, mirroring the shape of proof `gpu_hal`'s
`triangle_render_readback` (task 191) gives its own backends. The crate readme
currently states this gap "waits on workspace-level `wasm-bindgen-test` runner
infrastructure" (`readme.md` line 164) — the first concrete step is confirming
whether `browsee` (already available and used elsewhere in this repo for
browser-side pixel verification, e.g. task 191) closes the gap without that
infrastructure; if so, correct the readme's Testing section accordingly as part of
this task's own delivery. Success is testable by a documented `browsee` command
sequence reporting the triangle's configured color at its expected pixel in a real
browser. **Conditional outcome:** if the investigation instead confirms `browsee`
is genuinely insufficient for this class of check, the task closes on that finding
alone — recorded in Outcomes with the specific limitation found, `readme.md` left
stating the real (confirmed, not assumed) blocker — without authoring the example
crate or standing up `wasm-bindgen-test` infrastructure; see Out of Scope.

## In Scope

- Confirm whether `browsee` (external, Bash-driven real-browser automation) is a
  sufficient mechanism for this smoke test, distinct from the `wasm-bindgen-test`
  in-process Rust-test runner the readme names as a blocker — record the finding
  either way before writing the example.
- A new minimal example crate under `examples/minwebgl/` (e.g.
  `examples/minwebgl/context_triangle_smoke/`) that calls `context::from_canvas`
  (or `from_canvas_with`), compiles+links a minimal vertex/fragment shader pair via
  `Program::new( gl, vertex_src, fragment_src )` (`src/shader.rs`), uploads a
  triangle via `buffer::create`/`buffer::upload` (`src/buffer.rs`), and issues one
  draw call — the smallest sequence that exercises `from_canvas` as a load-bearing
  step, not a comprehensive feature tour.
- Registering the new example crate in the root `Cargo.toml` workspace members and
  the gallery tracking files (`examples/readme.md`, `examples/index.md`,
  `examples/index.html`, `examples/demo_completeness.md`).
- Using `browsee` (`.launch` -> `.wait for::render` -> `.pixel`) to confirm the
  triangle's configured color appears at its expected on-canvas pixel, and the
  configured clear color appears outside it.
- A `tests/manual/readme.md` entry in `minwebgl` documenting this as a scripted
  browser-verification procedure (prerequisites, exact `browsee` commands,
  expected pixel readings).
- Updating `minwebgl/readme.md`'s Testing section (line 154-164) to reflect
  whatever the browser-testing-mechanism finding turns out to be: either
  correcting the "waits on workspace-level `wasm-bindgen-test` runner
  infrastructure" claim (if `browsee` is confirmed sufficient for this class of
  check) or leaving it as a distinct, still-open need (if the investigation finds
  `browsee` genuinely insufficient for what that sentence was pointing at) — either
  way, the sentence must state the CURRENT truth, not the pre-task claim
  unconditionally.
- Updating `minwebgl/tests/readme.md`'s intro sentence ("The GL-context/DOM layer
  has no runner yet") once this smoke test lands.

## Out of Scope

- Comprehensive browser-side coverage of `minwebgl`'s GL-context/DOM surface
  (shaders, VAOs, textures, uniforms, file/fetch beyond this one smoke path) — this
  task closes the narrowest foundational slice only; broader coverage is a
  separate, future task if and when a concrete need for it is committed.
- `gpu_hal`'s own webgpu/webgl2 backend browser tests (task 191) — a separate
  crate.
- Any change to `minwebgl`'s `src/` implementation. If the smoke test uncovers an
  actual defect in `context.rs`/`shader.rs`/`buffer.rs`, file it as a new
  `BUG-NNN` per `bugs/file.rulebook.md` rather than patching it inside this task.
- Standing up `wasm-bindgen-test` workspace-level runner infrastructure — this
  task's premise is that `browsee` may make that infrastructure unnecessary for
  this class of check; standing it up regardless is out of scope unless the
  in-task investigation finds `browsee` insufficient, in which case this task
  reports that finding rather than building the infrastructure itself.

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
-   Minimum example code to satisfy Test Matrix — one triangle draw, not a feature
    tour
-   `verb/test` passes with zero failures and zero warnings (native regression
    check — this task does not touch native code paths)
-   `cargo check -p minwebgl --target wasm32-unknown-unknown` (plus the new example
    crate) compiles clean — never env-prefix `RUSTFLAGS`/`RUSTDOCFLAGS` for this
    check, it clobbers `.cargo/config.toml`'s required `--cfg web_sys_unstable_apis`
    (this exact gotcha is called out in `minwebgl/readme.md` line 164 itself)
-   No function exceeds 50 lines; no duplication; public items have `///` doc
    comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Investigate `browsee` vs. the readme's stated `wasm-bindgen-test`-runner blocker | Mechanism confirmation | Finding recorded (sufficient / insufficient) before the example is authored |
| T02 | Load context-triangle example in a real browser via `browsee` | `context::from_canvas` + minimal shader/buffer/draw | `browsee .wait for::render` exits 0; `browsee .pixel region::center` reports the triangle's configured color |
| T03 | Sample a pixel outside the triangle's bounds | Same page | `browsee .pixel` reports the configured clear color |
| T04 | `cargo check -p minwebgl --target wasm32-unknown-unknown` (example crate included) | New example crate | Compiles clean, no `RUSTFLAGS` env override |
| T05 | `cargo test -p minwebgl --all-features` | Existing native pure-logic suite | Still passes — unaffected regression check |

## Acceptance Criteria

-   The `browsee`-vs-`wasm-bindgen-test` mechanism question is answered and
    recorded before the example crate is authored
-   **If `browsee` is confirmed sufficient (expected outcome):** pixel verification
    confirms the triangle's configured color at its expected location and the clear
    color outside it; `minwebgl/readme.md`'s Testing section and
    `minwebgl/tests/readme.md`'s "no runner yet" sentence both state the current,
    post-task truth; every Test Matrix row has a corresponding passing check
-   **If `browsee` is confirmed insufficient (fallback outcome):** the specific
    limitation is recorded in Outcomes with concrete evidence (what was tried, what
    failed and why); `readme.md` is left stating the real, confirmed blocker rather
    than the untested pre-task assumption; T02-T05 and the example crate are not
    authored (Out of Scope); the task still closes — this is not a stalled or
    ambiguous state

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Mechanism investigation**
- [ ] C1 — Is the `browsee`-vs-`wasm-bindgen-test` finding recorded in the task's Outcomes or in a doc update, not left implicit?

**Example crate**
- [ ] C2 — Does `examples/minwebgl/context_triangle_smoke/` (or equivalent) exist, registered in root `Cargo.toml` workspace members?
- [ ] C3 — Is it registered in all 4 gallery tracking files?

**Browser verification**
- [ ] C4 — Does `minwebgl/tests/manual/readme.md` document the exact `browsee` command sequence and expected pixel readings?
- [ ] C5 — Does `minwebgl/readme.md`'s Testing section state the current truth (not the stale pre-task claim)?

**Out of Scope confirmation**
- [ ] C6 — Is `module/min/minwebgl/src/` untouched (zero diff)?
- [ ] C7 — Is `gpu_hal` untouched by this task?

### Measurements

- [ ] M1 — center pixel: `browsee .pixel region::center` → matches the example's configured triangle color (was: no example existed)
- [ ] M2 — corner pixel: `browsee .pixel region::<corner>` → matches the configured clear color

### Invariants

- [ ] I1 — native pure-logic suite: `cargo test -p minwebgl --all-features` → 0 failures (unaffected by this task)
- [ ] I2 — wasm32 compiles clean: `cargo check -p minwebgl --target wasm32-unknown-unknown` → 0 errors, no `RUSTFLAGS` env override used

### Anti-faking checks

- [ ] AF1 — real paint, not a stale/blank canvas: `browsee .wait for::render timeout::60` exits 0 before any `.pixel` call is trusted
- [ ] AF2 — bounded draw, not a full-canvas clear: T03's outside-triangle pixel check reads the clear color, not the triangle color
- [ ] AF3 — scope discipline: the example exercises `from_canvas` plus the minimum shader/buffer/draw sequence to paint one triangle — not an expanded feature tour smuggled in under this task's narrow goal

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, two Fix-and-Recheck iterations across sessions, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🔴 | 🟢 | (Prior session) Goal/Acceptance Criteria only cleanly covered the case where `browsee` is confirmed sufficient, leaving the negative-finding branch's closure path ambiguous | (Prior session) Added a conditional-outcome branch to Goal and split Acceptance Criteria into explicit sufficient/insufficient branches |
| D3 | Value / YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | (This session) In Scope cited `shader::make`/`compile_vertex`/`compile_fragment`/`buffer::make`/`upload_f32` — none of these exist in `src/shader.rs`/`src/buffer.rs`; live grep found the real surface is `Program::new(gl, vertex_src, fragment_src)` and `buffer::create`/`buffer::upload` | (This session) Rewrote the In Scope bullet to cite the real, grep-confirmed API |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | `unit` path (`module/min/minwebgl`, not `module/helper/`) live-confirmed against actual directory + `Cargo.toml` package name | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 open | 2 fixes |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 12:11:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 12:33:59 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 12:33:59 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 192` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with task 202/206 precedent; not forced/spoofed, left at 🔎 Accepting per standing project convention |
| 2026-08-19 00:40:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk`-methodology gap-closure round (docs/layer follow-up): add a bounded browser-side pixel-verified smoke test for `minwebgl`'s core context-creation path.
- **[2026-08-16]** `EXECUTED` — Mechanism finding (T01/C1): `browsee` confirmed sufficient, no `wasm-bindgen-test` infrastructure needed — recorded in `minwebgl/readme.md`'s Testing section. Created `examples/minwebgl/context_triangle_smoke/` (`context::from_canvas` + `Program::new` + `buffer::create`/`upload` + `BufferDescriptor` + one `draw_arrays`), registered in root `Cargo.toml` workspace members (glob-matched, no manual edit needed) and all 4 gallery tracking files. `browsee` session against a real Firefox render: `.wait for::render` exited 0; chrome-corrected pixel probes (re-derived from this session's own screenshot, not assumed portable from the task-191 precedent) read `region::40x40x306,120` → `rgb 0 0 0` (clear) and `region::40x40x306,260` → `rgb 255 0 0` (triangle) — M1/M2/T02/T03/AF1/AF2 satisfied. Documented the full command sequence and pixel readings in new `minwebgl/tests/manual/readme.md` (C4); updated `minwebgl/tests/readme.md`'s intro sentence and Responsibility Table for the new `manual/` entry. Native regression (`cargo test -p minwebgl --all-features`): 13 unit/integration tests + 1 doc test, 0 failures (T05/I1). wasm32 check (`cargo check -p minwebgl_context_triangle_smoke --target wasm32-unknown-unknown`, no `RUSTFLAGS` override): clean (T04/I2). `module/min/minwebgl/src/` untouched by this task (C6); `gpu_hal` diffs present in the working tree are task 191's own prior deliverables, not touched by this task (C7).

## Related Documentation

- `module/min/minwebgl/readme.md` — Testing section (lines 154-164) this task investigates and updates
- `module/min/minwebgl/tests/readme.md` — "no runner yet" claim this task's coverage addresses
- `module/helper/gpu_hal/tests/native_backend_test.rs` — the `triangle_render_readback` precedent this task's pixel-verification shape mirrors
- Task 191 (`191_gpu_hal_browser_pixel_verification.md`) — sibling browser-pixel-verification task, same round, distinct crate
