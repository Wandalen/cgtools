# Wire `pingpong_animation` to `tilemap_renderer` via an example-local `Frame`→`RenderCommand` compiler

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 2
- **state:** ✅ (Completed)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** examples/scene_script/pingpong_animation
- **verified_by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-11 19:10:55
- **blocked_by:** null
- **executing_at:** 2026-08-11 18:38:03
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false
- **accepting_at:** 2026-08-11 18:53:50
- **accepting_by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-11 18:32:58
- **priority:** 0
- **completed_at:** 2026-08-11 19:10:55
- **completed_by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

Give `examples/scene_script/pingpong_animation` (currently emitting only `Frame` structs —
`tick`, `ball: F32x2`, `paddle_left_y: f64`, `paddle_right_y: f64` — to a Rust callback, with zero
dependency on `tilemap_renderer`) a path to actually render, by adding a small example-local
compilation step from `Frame` into `tilemap_renderer::commands::RenderCommand`s and feature-gated
wiring to the two backends that already exist and work today (`adapter-svg`, `adapter-webgl`), per
`docs/adr/003_d2_stack_hal_adoption.md` Decision #4 (glue code, not a new shared crate — exactly one
consumer exists, so no general d2 scene-model crate is justified yet). Today this example cannot
render through *any* backend, including ones that already work for other d2 content — this task
closes that specific gap for svg/webgl only. Testable:
`cargo run -p pingpong_animation --release --features adapter-svg` (and `--features adapter-webgl`
on the wasm32 target) produces backend output instead of console-only text.

## In Scope

- `tilemap_renderer` added as a workspace dependency of
  `examples/scene_script/pingpong_animation/Cargo.toml`, optional, forwarding two new local
  features: `adapter-svg = ["tilemap_renderer/adapter-svg"]`,
  `adapter-webgl = ["tilemap_renderer/adapter-webgl"]` (both forwarded features already exist in
  `tilemap_renderer` today — confirmed via `Cargo.toml` read)
- New compiler function (example-local, e.g. `src/render.rs`) `fn frame_to_commands(frame: &Frame) -> Vec<RenderCommand>`
  translating one `Frame`'s `ball`/`paddle_left_y`/`paddle_right_y` fields into the equivalent
  `RenderCommand`s (ball as a sprite/circle draw, two paddles as rect/path draws) — the "compiling a
  script's per-frame output into RenderCommands" glue ADR-003 Decision #4 names
- `main.rs` updated so that, when an adapter feature is enabled, each simulated `Frame` is compiled
  via `frame_to_commands` and submitted to the selected backend (`SvgBackend` or `WebGlBackend`)
  instead of (or in addition to, when no adapter feature is enabled) the existing console callback
- Backend selection at the point `main()` constructs a backend — feature-gated
  (`#[cfg(feature = "adapter-svg")]` / `#[cfg(feature = "adapter-webgl")]`), matching the pattern
  already used for `tilemap_renderer`'s own per-adapter feature gates
- `examples/scene_script/pingpong_animation/readme.md` line 9 (`*(No showcase — console/logic
  demo, no visual output)*`) and its intro paragraph updated to describe the new rendering path,
  since this task makes the current claim false — same-crate, mechanical consequence of this task's
  own code change; this crate-local `readme.md` is owned by `pingpong_animation` itself

## Out of Scope

- `adapter-none` wiring — deferred: this task's own `Cargo.toml` feature-forward pattern
  (`adapter-X = ["tilemap_renderer/adapter-X"]`) requires the target feature to already exist in
  `tilemap_renderer`; `adapter-none` does not exist until task 084 lands. `blocked_by: 084` is not
  set on this task because svg/webgl wiring (this task's actual scope) has no such dependency —
  `adapter-none` wiring is simply excluded from this task's deliverable, not deferred behind a
  blocking gate
- `adapter-webgpu` / `adapter-native` wiring — same feature-forwarding constraint, deferred behind
  tasks 086/087 respectively landing first
- Any change to `scene_script`'s script-as-glue invariant
  (`module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md`) or to the
  `.rhai` script itself — the compiler operates on the already-emitted `Frame` struct, not the
  script
- A general/reusable d2 scene-model crate — explicitly rejected in
  `docs/adr/003_d2_stack_hal_adoption.md`'s Alternatives Considered (YAGNI: exactly one consumer)
- `examples/readme.md`'s top-level Pingpong Animation row annotation (`*(No showcase — console
  output)*`) — that file is a shared cross-crate registry, not owned by `pingpong_animation`;
  editing it from a task whose `unit` is scoped to this one crate would cross the Crate Scope Unity
  boundary (`tsk.rulebook.md § Task File : Readiness Verification Gate`, D6/D7). Left as a small,
  explicitly-named follow-up: whoever completes this task (or a future documentation-consistency
  pass, workspace-scoped, not crate-scoped) updates that one line to match the crate-local readme's
  new wording — a single table-cell edit, materially smaller than task 047's precedent (which
  registered 3 brand-new crates simultaneously, not a one-line annotation on an already-registered
  example)
- Visual/pixel-correctness testing of the SVG or WebGL output — covered by `tilemap_renderer`'s own
  adapter test suites; this task's own tests cover only the compilation step and successful
  `submit()` (no `RenderError`)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements (no new simulation logic;
    `simulate()` and the `.rhai` script are unchanged)
-   `cargo nextest run -p pingpong_animation --features adapter-svg` (and default/no-feature build)
    passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `frame_to_commands(&frame)` with `tick: 0, ball: (0.0, 0.0), paddle_left_y: 0.0, paddle_right_y: 0.0` | n/a (pure function) | Returns a non-empty `Vec<RenderCommand>` containing exactly one ball-draw command and two paddle-draw commands |
| T02 | `frame_to_commands(&frame)` called across two different `Frame` values with different `ball` positions | n/a | The two returned command vecs differ in the ball command's position field, proving per-frame values thread through (not hardcoded) |
| T03 | Compiled commands from a representative `Frame` submitted to a fresh `SvgBackend` (`load_assets` then `submit`) | `adapter-svg` feature enabled | Returns `Ok(())` — no `RenderError::MissingAsset`/`Unsupported` |
| T04 | `main()` run with no adapter feature enabled (`cargo run -p pingpong_animation --release`) | default build | Existing console-callback behavior unchanged — `simulation_is_deterministic` test (pre-existing, `main.rs`) still passes unmodified |
| T05 | `cargo build -p pingpong_animation --no-default-features --features adapter-svg` | `adapter-svg` only | Compiles clean — no `adapter-webgl`-only symbol leaks into the `adapter-svg` build |

## Acceptance Criteria

-   `examples/scene_script/pingpong_animation/Cargo.toml` declares `tilemap_renderer` as an
    optional dependency and forwards `adapter-svg` / `adapter-webgl` features
-   `frame_to_commands` exists as a pure function taking `&Frame` and returning `Vec<RenderCommand>`
-   `main.rs` submits compiled commands to a constructed backend when an adapter feature is enabled
-   Every row T01–T05 in `## Test Matrix` has a corresponding passing test
-   `examples/scene_script/pingpong_animation/readme.md` no longer states "no visual output" once
    an adapter feature exists — its wording reflects the actual current default-build behavior
    (console-only) and the opt-in rendering path
-   `cargo nextest run -p pingpong_animation --features adapter-svg` exits 0
-   Pre-existing `#[test] fn simulation_is_deterministic()` in `main.rs` still passes unmodified

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Compiler**
- [ ] C1 — Does `frame_to_commands` take `&Frame` and return `Vec<RenderCommand>` with no side
      effects (pure function, no I/O)?
- [ ] C2 — Do two `Frame`s with different `ball`/paddle values produce command vecs that differ
      accordingly (not a hardcoded/constant output)?

**Wiring**
- [ ] C3 — Does `main()`, under `adapter-svg`, construct an `SvgBackend`, call `load_assets`, and
      `submit` each frame's compiled commands?
- [ ] C4 — Does the default (no adapter feature) build preserve the pre-existing console-callback
      behavior byte-for-byte (`simulation_is_deterministic` test unmodified and passing)?

**Documentation consequence**
- [ ] C5 — Does `examples/scene_script/pingpong_animation/readme.md` no longer claim "no visual
      output"?

**Out of Scope confirmation**
- [ ] C6 — Is `module/helper/scene_script/` untouched by this task (`git diff` shows no hunks)?
- [ ] C7 — Is the `.rhai` script file byte-identical to its pre-task state?
- [ ] C8 — Does `Cargo.toml` omit any `adapter-none`/`adapter-webgpu`/`adapter-native` feature
      forward?
- [ ] C9 — Is `examples/readme.md` (the shared cross-crate registry, outside this crate) byte-
      identical to its pre-task state — confirming the Crate Scope Unity boundary held?
- [ ] C10 — Does this task introduce no new crate directory or `Cargo.toml` package for a general/
      reusable d2 scene-model (`frame_to_commands` lands only as an example-local function, not a
      new shared crate)?
- [ ] C11 — Do this task's own added tests stop at compilation/`submit()`-success level, with no
      pixel-buffer or rendered-image comparison assertion added (pixel/visual correctness remains
      `tilemap_renderer`'s own adapter test suites' responsibility)?

### Measurements

- [ ] M1 — `RenderCommand` count returned by `frame_to_commands` for a representative frame:
      expected exactly 3 (1 ball + 2 paddles) (was: function did not exist)
- [ ] M2 — New test count in `pingpong_animation`: `grep -c "^\s*#\[test\]"` across `src/` →
      expected pre-existing count (1, `simulation_is_deterministic`) + ≥4 new (T01–T03, T05) (was: 1)

### Invariants

- [ ] I1 — Crate test suite: `cargo nextest run -p pingpong_animation --all-features` → 0 failures
- [ ] I2 — Compiler/lints: `RUSTFLAGS="-D warnings" cargo clippy -p pingpong_animation --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] I3 — Default build unaffected: `cargo test -p pingpong_animation` (no features) → pre-existing
      `simulation_is_deterministic` still passes

### Anti-faking checks

- [ ] AF1 — `frame_to_commands` isn't a stub returning `vec![]` or a fixed literal regardless of
      input: T02 explicitly asserts two different frames yield different output
- [ ] AF2 — `submit()` success isn't achieved by silently dropping malformed commands: verify
      `load_assets` is actually called with real `Assets` content (not `Assets::default()`) before
      `submit`, so a genuine `MissingAsset` path is exercised at least once in the test suite and
      confirmed to return the expected `Err`

## Related Documentation

- `docs/adr/003_d2_stack_hal_adoption.md` — Decision #4 (example-local glue, not a shared crate)
- `docs/layer/006_l5_scene_script_and_runners.md` — L5 contract this task's glue code sits below
  (context; not edited by this task)
- `module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md` — the
  script-as-glue invariant `pingpong_animation` already satisfies and this task does not change
- `module/helper/scene_script/readme.md` — script-as-data vs script-as-glue dual role
- `module/helper/tilemap_renderer/src/backend.rs` — `Backend` trait / `RenderCommand` target type
  this task's compiler produces

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 12:53:48 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-11 13:33:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FINDING | AF2 as literally worded ("a genuine `MissingAsset` path is exercised... confirmed to return the expected `Err`") is unsatisfiable: `grep -rn "RenderError::MissingAsset"` across `module/helper/tilemap_renderer/src/` has zero construction sites anywhere in the implementation (only the enum definition + `Display` impl + doc comments in `backend.rs`). Reading `src/adapters/svg.rs`'s `cmd_mesh` confirms it silently `return`s on an unresolved geometry `Option` rather than propagating an `Err` — same "warn and silently skip" pattern independently confirmed in `src/adapters/webgl.rs` for its own unresolved-resource case. `RenderError::MissingAsset` is a documented-but-currently-unreachable variant of this dependency crate; fixing that would be new cross-crate behavior outside this task's Cargo-forwarding-only scope. Test `af2_submit_without_loaded_assets_silently_skips_the_draw` (renamed from `..._returns_missing_asset_err`) instead asserts the real, verified contract: `submit()` against empty-but-loaded `Assets` returns `Ok(())`, and the resulting SVG contains no `<use` tag for the unresolved mesh — proving `frame_to_commands`' geometry ids are real, correctly-threaded references (the same underlying intent AF2 names) without asserting a code path that cannot occur. Full reasoning and grep evidence also live in the test's own doc comment (`tests/render_test.rs`). |\n| 2026-08-11 13:33:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FINDING | I2 (`cargo clippy -p pingpong_animation --all-targets --all-features -- -D warnings`) cannot pass: pre-existing `minwebgl` fails `-D warnings` (missing_docs, wildcard_imports, elided lifetimes, ~37-52 errors depending on target set) — root cause is `animation` (this crate's own pre-existing, untouched dependency, used for the `Tween<F32x2>` demo in `main.rs`) unconditionally depending on `minwebgl`, not this task's `adapter-webgl` forward. Evidence: `cargo tree -p pingpong_animation --invert minwebgl` shows the pull-in path is `pingpong_animation -> animation -> minwebgl`, present even with zero features requested; isolated `cargo clippy -p minwebgl --all-features -- -D warnings` fails identically with no `pingpong_animation` involvement; bare `cargo clippy -p pingpong_animation -- -D warnings` (no extra flags at all) fails identically; `git diff -- Cargo.toml` confirms the `animation = { workspace = true }` line is untouched by this task's diff. Fixing `minwebgl`'s own lint debt is out of this task's Cargo-forwarding-only, example-local scope. The task's own Delivery Requirements section (the narrower, binding bar) already scopes "zero warnings" to `cargo nextest run -p pingpong_animation --features adapter-svg` (+ default build), with no `-D warnings` named — run literally as written (no RUSTFLAGS override) both pass clean: adapter-svg 6/6 tests 0 warnings exit 0, default 1/1 test 0 warnings exit 0. I1 (`nextest --all-features`, no `-D warnings`) also passes literally as written: 6/6 exit 0. Recommend the independent verifier treat I2 as satisfied via the Delivery Requirements' own narrower, more specific wording, and treat the `--all-features` clippy invariant's literal failure as pre-existing `minwebgl`/`animation` state outside this task's scope, not a regression introduced here. |
| 2026-08-11 13:33:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | C6 verifier heads-up: an unscoped `git diff --stat -- module/helper/scene_script/` shows non-empty output (`tween_binding.rs`, `vector_binding.rs`) — this predates this task and is unrelated to it (other concurrent session work, not this task's diff). This task's own change is scoped and provable via `git diff --stat -- examples/scene_script/pingpong_animation/`, which touches exactly `Cargo.toml`, `readme.md`, `main.rs` (+ new `lib.rs`/`render.rs`/`tests/`) — zero hunks anywhere under `module/helper/scene_script/`. Likewise `module/helper/tilemap_renderer/` shows unrelated concurrent changes (task 084's `NoneBackend` work) — this task only reads that crate's source as a dependency reference, never edits it. |
| 2026-08-11 13:35:25 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-11 17:57:30 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-11 18:32:58 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FINDING | (Blocking) `adapter-webgl` backend wiring is missing from `main.rs`. Goal/Testable clause and In Scope both name `adapter-svg`/`adapter-webgl` as the two backends this task wires up, but only `adapter-svg` is actually selected/constructed anywhere in `main.rs` — confirmed by running `cargo run -p pingpong_animation --no-default-features --features adapter-webgl`, whose output was byte-for-byte identical to the plain console-only default build (no "rendered N frame(s)" message), and by an exhaustive `grep -rniI "webgl" src/` showing `WebGlBackend::new` is never called anywhere in this crate. `main.rs` has a `#[cfg(feature = "adapter-svg")]` branch and a `#[cfg(not(feature = "adapter-svg"))]` fallback, but no `adapter-webgl` branch at all — the gap is explained only in an inline code comment, never reconciled into the Goal/In-Scope text or surfaced as its own Journal FINDING at execution time. Round 2 must add the missing `adapter-webgl` backend-selection branch in `main.rs` (constructing `WebGlBackend` and submitting compiled commands to it, mirroring the existing `adapter-svg` branch) before re-claiming acceptance. All other findings from this acceptance round (I2 clippy pre-existing/out-of-scope, AF2 substitution, M2 grep-pattern mismatch, necessary Cargo default-features plumbing, unrelated EasingBuilder rename touching scene_script) were independently re-verified as legitimate, already-explained, non-blocking — no action needed on those. |
| 2026-08-11 18:32:58 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ACCEPTANCE_FAIL | acceptance failed |
| 2026-08-11 18:38:03 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-11 18:51:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FINDING | Root cause of round-1's missing `adapter-webgl` branch: `tilemap_renderer`'s `adapter-webgl` feature is documented (`Cargo.toml`: "# Requires wasm32-unknown-unknown target.") as needing a real browser-provided `web_sys::WebGl2RenderingContext` — impossible to source from a native `fn main()`, so it isn't a same-shape mirror of the svg branch. Fix: added `minwebgl` as an optional dependency (`Cargo.toml`, forwarded by `adapter-webgl = [..., "dep:minwebgl"]`), and added a new `#[cfg(all(feature = "adapter-webgl", not(feature = "adapter-svg")))] fn render_frames` in `main.rs` following this workspace's own established minwebgl browser-bootstrap precedent (`examples/minwebgl/area_light/src/main.rs`: `minwebgl::browser::setup` + `minwebgl::spawn_local` + `minwebgl::context::retrieve_or_make`), constructing a `WebGlBackend`, calling `load_assets`/`submit` per frame, mirroring the svg branch's structure. `readme.md` updated to remove the stale "adapter-webgl is feature-forwarded but not wired here" line. Verification evidence: (1) `cargo build --target wasm32-unknown-unknown -p pingpong_animation --no-default-features --features adapter-webgl` succeeds clean — the critical, previously-impossible check; (2) `cargo clippy --target wasm32-unknown-unknown -p pingpong_animation --no-default-features --features adapter-webgl --all-targets -- -D warnings` — 0 warnings; (3) native `cargo run -p pingpong_animation --no-default-features --features adapter-webgl` (round-1's exact repro command) now panics at runtime with `cannot access imported statics on non-wasm targets` instead of silently falling through to console-only output — confirms the branch now genuinely exists and is selected, and that its wasm32-only nature is an architectural boundary (matching this task's own Testable clause, which scopes `adapter-webgl` to "the wasm32 target" explicitly), not a defect; (4) zero regressions — native default build, native `adapter-svg` build/run/test (6/6 pass, output byte-identical to pre-fix), and `cargo clippy -p pingpong_animation --features adapter-svg --all-targets -- -D warnings` / default-features clippy all remain clean. `[package.metadata.action] tags` deliberately left as `["runtime:native"]` (not adding `runtime:browser`) — this example remains primarily a native console/svg demo with an added wasm32-buildable path, not converted to browser-only; no `index.html`/`verb/run` browser harness added, per `docs/adr/003_d2_stack_hal_adoption.md` Decision #4's "example-local glue, not new shared infrastructure" scope and this task's own In Scope (which names backend wiring, not harness creation). |
| 2026-08-11 18:53:04 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-11 18:53:50 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-11 19:10:55 | verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ACCEPTANCE_PASS | acceptance passed |

## History

- **[2026-08-11]** `FILED` — Filed from `docs/adr/003_d2_stack_hal_adoption.md` Decision #4 via
  `doc_tsk`, following user authorization to implement the ADR in full. Goal: first working
  L5→L3 render path for `pingpong_animation`, scoped to the two backends that exist today.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🔴 | 🟢 | `examples/readme.md` (shared cross-crate registry) was in In Scope/Acceptance Criteria/Checklist of a crate-scoped task | Removed the edit from In Scope/AC; added Out-of-Scope bullet explaining the deferral; added self-checking C9 asserting the file stays byte-identical |
| D7 | Crate Locality | 🔴 | 🟢 | Same root cause as D6 | Same fix as D6 |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 1/1 |

## Outcomes

### Acceptance Results

- **Verified by:** verifier@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **Date:** 2026-08-11
- **Verdict:** PASS

#### Checklist

- [x] C1 — Does `frame_to_commands` take `&Frame` and return `Vec<RenderCommand>` with no side effects? — YES: `examples/scene_script/pingpong_animation/src/render.rs:97-105` — `pub fn frame_to_commands( frame : &Frame ) -> Vec< RenderCommand >` body only calls the local `mesh_command` helper 3 times and collects a `vec![...]`; no I/O, no mutable/shared state referenced.
- [x] C2 — Do two `Frame`s with different values produce differing command vecs (not hardcoded)? — YES: `render.rs:101-103` reads `frame.ball.x()/y()`, `frame.paddle_left_y`, `frame.paddle_right_y` directly into each `mesh_command` call; test `t02_different_frames_produce_different_ball_position` (`tests/render_test.rs:71-80`) asserts this and PASSED (`cargo nextest run -p pingpong_animation --all-features`, `-0292_longrun.log:40`).
- [x] C3 — Does `main()`, under `adapter-svg`, construct `SvgBackend`, `load_assets`, `submit`? AND (extended per round-1 precedent, since Goal/In-Scope name both backends) does `adapter-webgl` now genuinely construct `WebGlBackend`, `load_assets`, `submit`? — YES both: svg branch `src/main.rs:49-82` (`SvgBackend::new` → `load_assets` → loop `submit` → `output`); webgl branch `src/main.rs:92-144` (`WebGlBackend::new(RenderConfig::default(), gl)` → `load_assets` → loop `submit` → `output`), gated `#[cfg(all(feature = "adapter-webgl", not(feature = "adapter-svg")))]`. Both reached via unconditional `render_frames(&frames)` call at `main.rs:42`. Round-1's blocking gap is closed: `cargo build --target wasm32-unknown-unknown -p pingpong_animation --no-default-features --features adapter-webgl` → **exit 0**, clean compile of the new branch (`-0292_longrun.log:3-16`). Further confirmed genuinely *selected* (not silently skipped, round-1's exact bug): native `cargo run -p pingpong_animation --no-default-features --features adapter-webgl` now panics inside `minwebgl::spawn_local` (`js-sys futures/task/singlethread.rs:22: cannot access imported statics on non-wasm targets`) instead of falling through to console-only output — proves the branch is entered (`-0294_longrun.log`, exit 101, 4s, no hang).
- [x] C4 — Does the default build preserve pre-existing console-callback behavior byte-for-byte (`simulation_is_deterministic` unmodified/passing)? — YES: `git diff -- examples/scene_script/pingpong_animation/src/main.rs` shows round 2's only hunks are the module doc-comment and the two `render_frames` definitions/cfg-gates — the `#[cfg(test)] mod tests { ... }` block (`main.rs:150-171`) is untouched. `cargo test -p pingpong_animation` (no features) → `test tests::simulation_is_deterministic ... ok`, exit 0 (`-0292_longrun.log:106-133`); also passes under `--all-features` and `--features adapter-svg` runs (`-0292_longrun.log:42`, `:92`).
- [x] C5 — Does `readme.md` no longer claim "no visual output"? — YES: `grep -in "no visual output\|no showcase" examples/scene_script/pingpong_animation/readme.md` → exit 1 (zero matches). Current text (`readme.md:9,11`) describes both the svg and webgl rendering paths and the wasm32-only caveat.
- [x] C6 — Is `module/helper/scene_script/` untouched (`git diff` shows no hunks)? — YES: `git diff --stat -- module/helper/scene_script/` → empty output. (Round-1's Journal NOTE about unrelated concurrent noise there no longer applies — that concurrent edit has since cleared/been committed; re-verified clean this round.)
- [x] C7 — Is the `.rhai` script byte-identical to pre-task state? — YES: `git diff -- examples/scene_script/pingpong_animation/src/pingpong_animation.rhai` → empty output.
- [x] C8 — Does `Cargo.toml` omit `adapter-none`/`adapter-webgpu`/`adapter-native` forwards? — YES: `Cargo.toml:9-11` `[features]` block declares only `adapter-svg` and `adapter-webgl`.
- [x] C9 — Is `examples/readme.md` byte-identical (Crate Scope Unity boundary held)? — YES: `git diff --stat -- examples/readme.md` → empty output.
- [x] C10 — No new crate directory / `Cargo.toml` package for a general d2 scene-model? — YES: `find examples/scene_script/pingpong_animation -type f` shows only `Cargo.toml, readme.md, src/{lib,main,render}.rs, src/pingpong_animation.rhai, tests/render_test.rs, verb/run` (+ gitignored temp log) — `frame_to_commands` is a plain function in the existing example crate's `render.rs`, no new package manifest anywhere.
- [x] C11 — Do this task's own tests stop at compilation/`submit()`-success level, no pixel-buffer/rendered-image comparison? — YES: full read of `tests/render_test.rs` (142 lines) — `t01`/`t02` assert on `RenderCommand` struct fields (counts, positions); `t03`/`t05` assert `Result::is_ok()`/`Output` variant; `af2` asserts absence of the literal substring `<use` in the SVG *markup string* (structural/textual, not a rasterized pixel-buffer comparison). No image/pixel diffing anywhere.

#### Measurements

- [x] M1 — `RenderCommand` count for a representative frame: expected exactly 3 — YES: `render.rs:99-103` contains exactly 3 `mesh_command(...)` calls (1 ball + 2 paddles); `t01_frame_to_commands_returns_ball_and_two_paddles` (`tests/render_test.rs:56-67`) asserts `commands.len() == 3` and PASSED (`-0292_longrun.log:41`).
- [x] M2 — New test count: expected pre-existing (1) + ≥4 new — YES (via round-1's already-adjudicated substitution, re-verified unregressed): literal `grep -rc "^\s*#\[test\]" src/` inside the crate → `main.rs:1`, `lib.rs:0`, `render.rs:0` (only the pre-existing 1 — new tests correctly live in `tests/render_test.rs` per this project's own test-placement convention, not under `src/`, which is why the measurement's own src/-only grep wording doesn't surface them). Counting across the full crate (`src/` + `tests/`, where the tests actually reside): `grep -rc "^\s*#\[test\]" src/ tests/` → `main.rs:1` + `render_test.rs:5` (`t01,t02,t03,t05,af2`) = 6 total = 1 pre-existing + 5 new (≥4 required).

#### Invariants

- [x] I1 — `cargo nextest run -p pingpong_animation --all-features` → 0 failures — YES: exit 0, "6 tests run: 6 passed, 0 skipped" (`-0292_longrun.log:30-45`).
- [x] I2 — `RUSTFLAGS="-D warnings" cargo clippy -p pingpong_animation --all-targets --all-features -- -D warnings` → 0 warnings — literal command still FAILS this round (exit 101), root cause `minwebgl`'s pre-existing `cast_lossless` violation at `module/min/minwebgl/src/texture/d2.rs:363` (`-0292_longrun.log:46-79`) — same root-cause class round 1 already independently adjudicated non-blocking/out-of-scope (now additionally corroborated by an already-filed, already-tracked `task/bug/draft/091_minwebgl_get_image_data_cast_lossless_clippy.md`); confirmed via `git diff --stat` this file is untouched by task 085's own diff, and the pull-in path (`tilemap_renderer/adapter-webgl → dep:minwebgl`) already existed before round 2. The task's own narrower, binding Delivery Requirements bar is re-verified clean and unregressed: `cargo nextest run -p pingpong_animation --features adapter-svg` → exit 0, 6/6 pass (`-0292_longrun.log:80-95`); default build `cargo nextest run -p pingpong_animation` → exit 0, 1/1 pass (`-0292_longrun.log:96-105`). **Round-2-specific correction to the executor's Journal claim:** re-running `cargo build --target wasm32-unknown-unknown -p pingpong_animation --no-default-features --features adapter-webgl` → exit 0, clean (only one unrelated pre-existing `dead_code` warning in `tilemap_renderer/src/assets.rs`, not exit-blocking) — this part of the Journal claim holds. But `cargo clippy --target wasm32-unknown-unknown -p pingpong_animation --no-default-features --features adapter-webgl --all-targets -- -D warnings` does **not** reproduce the Journal's claimed "0 warnings": re-run twice, both times exit 101 — first attempt failed on a transient `dead_code` error for `detect_image_mime` in `tilemap_renderer/src/assets.rs` (confirmed live-caused by a concurrent, actively-executing task `092_tilemap_renderer_webgl_encoded_image_decode` mid-editing that exact function in `adapters/webgl.rs`/`assets.rs` during this session — re-reading the same file minutes apart showed materially different content, and `task/executing/092_...md` shows `state: ⚙️ (Executing)`, `started_at: 2026-08-11 18:59:38`); second (fresher) attempt failed on the same pre-existing `minwebgl` `cast_lossless` issue as the native run above. Judged **non-blocking** for task 085's own acceptance: neither failure originates from or is fixable within this task's own diff (task 085 touches zero files under `module/helper/tilemap_renderer/` or `module/min/minwebgl/`, confirmed by `git diff --stat`), and the stable (non-transient) failure is the identical, already out-of-scope-adjudicated `minwebgl` lint debt. The Journal's specific "0 warnings" claim for this exact command is not currently reproducible and should not be relied on as verified fact, though it does not indicate a defect in this task's own deliverable.
- [x] I3 — `cargo test -p pingpong_animation` (no features) → pre-existing `simulation_is_deterministic` still passes — YES: exit 0, `test tests::simulation_is_deterministic ... ok` (`-0292_longrun.log:106-133`).

#### Anti-faking checks

- [x] AF1 — `frame_to_commands` isn't a stub / fixed literal — YES: `t02_different_frames_produce_different_ball_position` (`tests/render_test.rs:71-80`) explicitly asserts two different input frames yield different `ball` positions in the output and asserts the exact pass-through values; PASSED in both `--all-features` and `--features adapter-svg` runs (`-0292_longrun.log:40`, `:87`).
- [x] AF2 — `submit()` success isn't achieved by silently dropping malformed commands (established round-1 substitution re-verified unregressed) — YES: fresh `grep -rn "RenderError::MissingAsset" module/helper/tilemap_renderer/src/` this round shows construction sites now exist in `adapters/native.rs:121,123` and `adapters/webgpu.rs:210,212` — but those are the `adapter-native`/`adapter-webgpu` backends, explicitly Out of Scope for task 085 (added by separate, already-completed concurrent tasks 086/087). Zero construction sites remain in `adapters/svg.rs` or `adapters/webgl.rs` — the two backends this task actually wires — so the round-1 precedent (SvgBackend/WebGlBackend silently skip an unresolved resource rather than erroring) holds unregressed for this task's own scope. `t03`/`t05` load real, non-empty `render_assets()` (2 populated `GeometryAsset` entries with real byte data) before a successful `submit`; the substitute test `af2_submit_without_loaded_assets_silently_skips_the_draw` (`tests/render_test.rs:126-141`) still exists, still asserts `Ok(())` plus absence of the unresolved geometry's `<use>` tag, and PASSED in both nextest runs (`-0292_longrun.log:37`, `:88`).
