# New minwebgpu example: live browser preview of a composed shader_chunks set with UI-tunable parameters

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
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgpu/shader_chunk_preview
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-14 21:24:02
- **blocked_by:** null
- **priority:** 0
- **executing_at:** 2026-08-14
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false
- **accepting_at:** 2026-08-14 21:00:27
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **completed_at:** 2026-08-14 21:24:02
- **completed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

Answer the direct question "what command opens a window with one shader chunk rendered, tunable by UI" — which this workspace had no answer to (`shader_chunks`'s CLI `tunables`/`compose` commands are text-output only; zero `winit`/`egui`/`eframe`/`imgui`/`iced`/windowing dependency exists anywhere in the workspace) — by adding `examples/minwebgpu/shader_chunk_preview/`, a WebGPU browser example that composes a `shader_chunks_core` chunk set (the 4 bundled chunks `hash21`/`value_noise`/`fbm3`/`fullscreen_triangle` plus one local `//@ param:`-annotated fragment chunk) and wires its 3 declared tunable uniforms to a live slider panel, launched via the existing `action/run shader_chunk_preview` mechanism (no new CLI verb needed). Motivated by the user's direct request, now answered rather than left unimplemented. Observable: `action/run shader_chunk_preview` opens a browser tab rendering the composed shader with a visibly responsive 3-slider control panel — drag a slider, the shader redraws that frame with the new value, no rebuild. Scoped: one new example crate plus documentation cross-references; zero behavior change to `shader_chunks_core`/`shader_chunks_params`/`shader_chunks`. Testable: `cargo nextest run -p minwebgpu_shader_chunk_preview` (6/6 passing) plus `cargo check --target wasm32-unknown-unknown` (clean) for the new crate.

## In Scope

- `examples/minwebgpu/shader_chunk_preview/` — new example crate: `Cargo.toml`, `index.html`, `style.css`, `controls.js`, `verb/run`, `src/main.rs`, `src/lib.rs`, `src/shader_source.rs`, `src/uniforms.rs`, `src/controls.rs`, `shader/preview_fragment.wgsl`, `tests/shader_source_test.rs`, `readme.md`
- `examples/minwebgpu/readme.md` — Examples table row for the new crate
- `examples/readme.md` — WebGPU Examples gallery row + Responsibility Table demo count (4→5)
- `examples/demo_completeness.md` — status row (`shader_chunk_preview (webgpu)`)
- `module/shader/shader_chunks/docs/cli/command_group/04_parameters.md` — cross-reference from the `.tunables` command's Purpose/Typical Patterns to this crate as the live UI consumer
- `examples/index.md` / `examples/index.html` — regenerated via `action/gallery` (never hand-edited directly)

## Out of Scope

- Annotating any *bundled* `shader_chunks_core` chunk with `//@ param:` lines — ruled out of scope by decision Q-03; tunables live on this crate's own local `preview_fragment` chunk only
- Any change to `shader_chunks_core`, `shader_chunks_params`, or `shader_chunks`'s own source, tests, or docs — all three are consumed as-is
- A `showcase.webp` screenshot — documented sandbox limitation shared with `orrery/webgpu`: this environment's headless Chromium cannot back a WebGPU swap-chain texture with a real shared image
- A new CLI verb or `action/run` mechanism change — the existing case-insensitive partial-path match already resolves `action/run shader_chunk_preview` uniquely

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a real test (native `naga`-validated WGSL assembly, no mocked GPU)
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo nextest run -p minwebgpu_shader_chunk_preview` passes with zero failures and zero warnings
-   `cargo check --target wasm32-unknown-unknown` (crate-scoped) exits clean
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution` (reserved for an independent verifier — not self-administered)
-   Task state updated to ✅ on verification pass; file moved to `task/completed/` (final)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Composed shader from 4 bundled chunks + 1 local chunk | `assemble()` | Each of `hash21`/`value_noise`/`fbm3`/`vs_main`/`VertexOutput`/`fs_main` declared exactly once |
| T02 | Local `preview_fragment.wgsl` body | `fragment_body_redeclares_no_chunk_symbol_and_consumes_them` | No chunk symbol redeclared locally; `VertexOutput` and `fbm3(` are genuinely consumed |
| T03 | Assembled WGSL text | `naga::front::wgsl::parse_str` + `Validator` | Parses and validates with zero errors — real front-end validation, not a structural approximation |
| T04 | `PREVIEW_FRAGMENT` descriptor vs. its own `//@` manifest | `shader_chunks_core::manifest_mismatches` | Zero mismatches — descriptor and manifest never drift |
| T05 | Assembled shader symbol offsets | `assembled_shader_orders_dependencies_before_dependents` | `hash21` precedes `value_noise` precedes `fbm3` precedes `fs_main` |
| T06 | `shader_chunks_params::chunk_discover(&PREVIEW_FRAGMENT)` vs. `Params` uniform struct fields | `discovered_tunable_parameters_match_params_uniform_fields` | Exactly `[noise_scale, warp_strength, brightness]`, each `Uniform`/`F32`/ranged, each name matches a real `f32` field declared in the WGSL — a tunable in the manifest with no matching struct field would move a slider that changes nothing |

## Acceptance Criteria

-   `action/run shader_chunk_preview` resolves uniquely and launches the example (confirmed via `action/run list` showing the crate discoverable with `runtime:browser`/`api:webgpu` tags)
-   `cargo nextest run -p minwebgpu_shader_chunk_preview` passes 6/6
-   `cargo check --target wasm32-unknown-unknown` (crate-scoped) exits 0
-   Every Test Matrix row has a corresponding passing test
-   `readme.md` documents the answer to the original question, the local-chunk-not-bundled rationale (Q-03), and the no-showcase limitation
-   `examples/index.md` and `examples/index.html` (regenerated via `action/gallery`) list the new example with correct title/description/tags/link

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Crate structure**
- [x] C1 — Does `Cargo.toml` declare package name `minwebgpu_shader_chunk_preview` with `[package.metadata.action] tags = ["runtime:browser", "api:webgpu"]`? Confirmed via direct read: `name = "minwebgpu_shader_chunk_preview"`, `[package.metadata.action] tags = ["runtime:browser", "api:webgpu"]`.
- [x] C2 — Does `src/shader_source.rs`'s `PREVIEW_CHUNKS` select exactly the 4 bundled chunks (`hash21`, `value_noise`, `fbm3`, `fullscreen_triangle`) plus the local `PREVIEW_FRAGMENT`? Confirmed via direct read of `src/shader_source.rs`.
- [x] C3 — Does `shader/preview_fragment.wgsl` declare exactly 3 `//@ param:` lines (`noise_scale`, `warp_strength`, `brightness`), each `uniform`/`f32`? Confirmed via `discovered_tunable_parameters_match_params_uniform_fields` (T06), fresh pass this session.

**Discoverability**
- [x] C4 — Does `action/run list` show `shader_chunk_preview` as a discoverable browser example? Confirmed: `action/run list` row 53 — `examples/minwebgpu/shader_chunk_preview  browser  runtime:browser·api:webgpu`.

**Out of Scope confirmation**
- [x] C5 — Is `shader/preview_fragment.wgsl` absent from `shader_chunks_core`'s bundled `shader/` chunk directories (confirms tunables were NOT added to a bundled chunk, per Q-03)? Confirmed: `find module/shader/shader_chunks_core -iname "preview_fragment*"` → zero matches.
- [x] C6 — Did this task's own work modify any file under `module/shader/shader_chunks_core/`, `module/shader/shader_chunks_params/`, or `module/shader/shader_chunks/`? Confirmed NO — this task's own In Scope/Related Documentation never names a path under those three crates for edits; `shader_chunk_preview` only *consumes* their public API (`shader_chunks_core::{chunk, ChunkDescriptor, set_compose, dependency_closed}`). Caveat: `git status --short` on those three crates currently shows unrelated modifications (docs/cli/, src/cli.rs, shader_chunks_params/src/lib.rs, etc.) — attributed to a different, concurrently-running task-system actor (consistent with this session's independently-confirmed API-rename activity tracked separately from task 112), not to this task's own work.
- [x] C7 — Is a `showcase.webp` absent from `examples/minwebgpu/shader_chunk_preview/`? Confirmed: `find examples/minwebgpu/shader_chunk_preview -iname "*showcase*"` → zero matches.
- [x] C8 — Are `action/run` and `action/gallery` themselves unmodified (confirms no new CLI verb or dispatch-mechanism change)? Confirmed: `git status --short action/run action/gallery` → zero output (clean).

**Documentation content**
- [x] C9 — Does `readme.md` document the original question answered, the Q-03 local-chunk rationale, and the no-showcase limitation? Confirmed via direct grep: line 5 states the question and the `action/run shader_chunk_preview` answer; line 9 states the no-`showcase.webp` limitation; line 13 states the Q-03 rationale for a local (not bundled) chunk.
- [x] C10 — Do `examples/index.md` and `examples/index.html` list the new example with correct title/description/tags/link? Confirmed via direct grep: `index.md` has 2 matching rows (ToC + gallery table) with title "Shader Chunk Preview (WebGPU)" and matching description; `index.html` has 2 matching `demo-card` articles with `data-tags="runtime:browser,api:webgpu"` and a working `View Details` link to `./minwebgpu/shader_chunk_preview/`.

### Measurements

- [x] M1 — native test count: `cd examples/minwebgpu/shader_chunk_preview && cargo nextest run` → `6 tests run: 6 passed` (was: crate did not exist). Re-run fresh this session (pid 3010613, exit 0, elapsed 50s) after the concurrent actor's API rename — confirms the result reflects current source, not a stale pre-rename pass.

### Invariants

- [x] I1 — test suite: `cd examples/minwebgpu/shader_chunk_preview && cargo nextest run` → 0 failures. Confirmed fresh this session (6/6 passed, 0 skipped).
- [x] I2 — wasm32 compiler clean: `cd examples/minwebgpu/shader_chunk_preview && cargo check --target wasm32-unknown-unknown` → 0 errors. Confirmed fresh this session (pid 1466903, exit 0, elapsed 257s).

### Anti-faking checks

- [x] AF1 — no disabled tests: `grep -rn "#\[ignore\]" examples/minwebgpu/shader_chunk_preview/tests/` → zero matches. Confirmed (grep exit 1 = no match).
- [x] AF2 — no mocked GPU/stubbed rendering: `grep -rniE "\bmock\b|\bstub\b" examples/minwebgpu/shader_chunk_preview/src/` → zero matches (real `minwebgpu`/`web-sys` calls only). One textual hit found and inspected: `src/main.rs:175` `// Stub main for native targets` — a `#[cfg(not(target_arch = "wasm32"))]` placeholder `main()` that only prints "run with wasm32 target" and returns; it does not stub any GPU call or rendering logic. The real `#[cfg(target_arch = "wasm32")]` `main()` (line 169-172) calls genuine `gl::spawn_local`/`app_run()`. Non-blocking — the match is a benign native-target placeholder, not faked rendering.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope lists 13 concrete crate files (all 13 confirmed present on disk this session) + 5 doc/index cross-references; Out of Scope lists 4 specific exclusions with rationale | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (direct user request this session, confirmed no prior capability existed); Observable (`action/run` + slider response — the literal browser-visible portion isn't mechanically tested, an inherent sandbox limitation already disclosed in Out of Scope and consistent with `orrery/webgpu`'s own accepted precedent); Scoped (one crate, zero behavior change to shader_chunks_core/params/shader_chunks); Testable (nextest + wasm32 check, both fresh-verified this session) | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis explicitly answered (2 Explore passes found zero windowing/UI dependency anywhere in the workspace); real user request this session; scope kept to 1 local chunk + 3 params, no new CLI verb | — |
| G4 | Implementation Readiness | — | 🟢 | Test Matrix present (6 rows), each mapped 1:1 to a real test function in `tests/shader_source_test.rs`; fresh native run this session: 6/6 passed (pid 3010613, exit 0); fresh wasm32 check this session: exit 0 (pid 1466903) | — |
| G5 | Execution Scope | — | 🟢 | Every listed path is relative and repo-internal (`examples/`, `module/`, `task/`) | — |
| G6 | Crate Scope Unity | — | 🟢 | Primary deliverables confined to `examples/minwebgpu/shader_chunk_preview/`; the 5 external touches are registration/cross-reference doc edits only (no src/test code), the same pattern used by every prior "new example crate" task in this history (099, 044-049) | — |
| G7 | Crate Locality | — | 🟢 | All code/test/shader/asset files live in the leaf crate itself, mirroring `examples/orrery/webgpu`'s own self-contained structure — no logic pushed up into `examples/minwebgpu/` or `examples/` | — |
| G8 | Crate Single Responsibility | — | 🟢 | One sentence: "a browser example that renders a live-tunable, UI-controlled preview of a composed shader_chunks fragment shader" — rendering and UI control are one fused capability (the user's literal request), not two bolted-together concerns | — |
| **Total** | | — | 🟢 | 0 blocking, 2 non-blocking | — |

Adversarial pass (summary): attempted to falsify G1 by checking every one of the 13 claimed crate
files against disk — all present, none missing (`Cargo.toml`, `index.html`, `style.css`,
`controls.js`, `verb/run`, `src/main.rs`, `src/lib.rs`, `src/shader_source.rs`, `src/uniforms.rs`,
`src/controls.rs`, `shader/preview_fragment.wgsl`, `tests/shader_source_test.rs`, `readme.md`).
Attempted to falsify G4 by re-running the native test suite and the wasm32 compile check fresh in
this session rather than trusting an earlier window's result, given a concurrent actor had already
once silently renamed the exact `shader_chunks_core`/`shader_chunks_params` APIs this crate depends
on (`compose_set`→`set_compose`, `discover_chunk`→`chunk_discover`, etc.) — both came back clean
against current source (6/6 tests, wasm32 exit 0). Attempted to falsify AF2 by inspecting the one
`mock|stub` grep hit directly rather than accepting the zero-match claim or rejecting on the hit
alone — confirmed benign (native-target placeholder `main()`, not a faked render path). Flagged two
Non-Blocking findings, both recorded in the Gate Table above (G2's browser-visible claim isn't
mechanically tested — sandbox limitation, disclosed, precedented; G6's external doc touches are
registration-only, matching every prior "add new example" task). Neither blocks PASS.

## Related Documentation

- `examples/minwebgpu/shader_chunk_preview/readme.md` — this crate's own readme (created)
- `examples/minwebgpu/readme.md` — Examples table row (updated)
- `examples/readme.md` — WebGPU Examples gallery row + Responsibility Table count (updated)
- `examples/demo_completeness.md` — status row (updated)
- `module/shader/shader_chunks/docs/cli/command_group/04_parameters.md` — `.tunables` Purpose/Typical Patterns cross-reference to this crate (updated)
- `examples/index.md`, `examples/index.html` — regenerated via `action/gallery` (auto-generated, never hand-edited)
- `task/decisions.md` — Q-03 (already ✅ Decided; this task builds on it but does not close it, since Q-03's own decision text explicitly deferred "any live GPU-backed parameter-preview UI" as future work — this task is that future work)
- No doc_pln plan file — raw-description input path, not doc_pln
- No related/blocking task — Deduplication Search (below) found no existing task covering this scope

## Outcomes

### Acceptance Results — Round 1

**Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
**Date:** 2026-08-14
**Verdict:** PASS

**Separation of concerns:** the Checklist/Measurements/Invariants/Anti-faking boxes above were already checked with "Confirmed via..." evidence text at task-filing time (`SELF_CORRECTION` history event), before this task ever entered execution. That population happened under `doc_tsk`'s Readiness Verification Gate (a pre-execution task-quality check, itself self-administered by design per `tsk.rulebook.md`) — not during a genuine post-execution acceptance walk by an independent verifier. This Round 1 section is that independent walk: every claim below was re-derived fresh this session by me as acceptance verifier, not copied from the pre-existing checkmarks.

**Checklist:** C1 🟢 re-confirmed via direct `grep` on `Cargo.toml` this session (`name = "minwebgpu_shader_chunk_preview"`, `tags = ["runtime:browser", "api:webgpu"]`). C2/C3 unchanged (T06's fresh pass this session covers C3's 3-param claim; C2 verified by direct read). C4 🟢 upgraded from a static `action/run list` grep to a genuine behavioral confirmation — `action/run shader_chunk_preview` actually resolved and launched this session (real dispatch, not just a discoverability listing). C5 unchanged (structural, low-risk). C6 🟢 re-confirmed fresh: `git status --short` on the 3 excluded crates now shows a *different* (evolved) set of concurrent-actor edits than the task file's own C6 note describes (now `shader_chunks_core/src/lib.rs` + `.../tests/shader_chunks_core_test.rs`, not the docs/cli.rs/params set previously recorded) — expected drift from the same disclosed concurrent actor, not a regression; critically, `git status --short examples/minwebgpu/shader_chunk_preview` returned zero output, confirming this task's own deliverable remains untouched regardless of how the excluded crates continue to move. C7/C8 🟢 re-confirmed fresh via `find`/`git status` this session, unchanged results. C9/C10 unchanged (static content, low-risk, no evidence of drift).

**Measurements/Invariants:** M1/I1/I2 🟢 re-confirmed fresh this session via `longrun`-detached `cargo nextest run && cargo check --target wasm32-unknown-unknown`, run from a clean shell, independent of the executor's own earlier run — 6/6 tests passed, wasm32 check exited 0. Cross-checked the 6 actual test names against the Test Matrix's 6 rows (T01-T06) by name, not just count: `assembled_shader_declares_every_symbol_exactly_once`↔T01, `fragment_body_redeclares_no_chunk_symbol_and_consumes_them`↔T02, `assembled_wgsl_parses_and_validates`↔T03, `preview_fragment_descriptor_matches_its_manifest`↔T04, `assembled_shader_orders_dependencies_before_dependents`↔T05, `discovered_tunable_parameters_match_params_uniform_fields`↔T06 — genuine 1:1 coverage, not just a matching count.

**Anti-faking:** AF1 unchanged (grep, deterministic). AF2 🟢 superseded by direct empirical evidence (see below) — a live, parameter-responsive render is strictly stronger proof of "no mocked/stubbed rendering" than a static grep for the words mock/stub.

**Live WebGPU render check (the one claim never mechanically tested before now):** launched `action/run shader_chunk_preview` via `longrun` (trunk serve on a fresh port, release build from the current, git-clean working tree). Dual-engine test, mirroring the rigor applied to task 097's now-discredited "cannot verify" WebGL claim:
- **Firefox** (session `minwebgpu_shader_chunk_preview`, auto-launched by `action/browser_serve`): `.wait for::render` → non-blank; screenshot shows a genuine, richly-detailed procedural fbm/noise texture (not blank, not a solid-color fallback) with the 3-slider panel correctly labeled (Noise scale=4, Warp strength=0.6, Brightness=1.2). Clicked the Noise scale slider track (value 4→17.2) — the rendered pattern visibly changed from smooth large-scale blobs to fine-grained high-frequency texture, a causally-consistent response to increasing a noise-scale uniform, captured in a before/after screenshot pair. This directly confirms the MOST Goal's Observable claim ("drag a slider, the shader redraws that frame with the new value, no rebuild") — a claim the pre-filled checklist never actually tested (flagged as G2's own Non-Blocking gap in the Verification Record above).
- **Chromium** (session `shader_chunk_preview_chromium`, `features::webgpu`): `.wait for::render` reported non-blank (rgb ≈243,244,244), but the screenshot shows a blank/white canvas under the same correctly-rendered slider panel. `.gpu` confirmed the adapter is `swiftshader` (software rasterizer) with the explicit caveat "adapter availability does NOT guarantee frames can be presented on-screen." Console showed zero WebGPU errors (one unrelated, benign `integrity`-attribute preload notice). This is **consistent with, not contradictory to**, the task's own Out of Scope claim — which names "headless Chromium" specifically, not the environment broadly, unlike task 097's now-discredited blanket claim.
- **New finding (Non-Blocking, disclosed not fixed):** the task's Out of Scope section frames the missing `showcase.webp` as an environment-wide limitation shared with `orrery/webgpu`; this session's evidence shows Firefox can in fact present real frames for this example. This doesn't unblock a screenshot within this acceptance pass (out of scope for a verification walk) but is worth the filer's awareness for future example/screenshot work in this workspace.
- Cleanup: both browsee sessions killed and purged (`purge::1`, confirmed via `.list`); the launched `action/browser_serve`/`trunk serve` process group (distinct pgid from the longrun supervisor) was signal-killed and confirmed gone via `ps`, without touching unrelated pre-existing dev servers found running on other ports.

**Adversarial pass:** attempted to falsify the render claim by checking whether the "before/after" slider comparison could be explained by time-based animation alone rather than genuine parameter response — ruled out because the slider's own displayed value changed (4→17.2) in lockstep with the visual frequency change, and the change (finer-grained noise) is the causally correct direction for an increased noise_scale uniform, not an arbitrary drift. Attempted to falsify C6 by re-running `git status` fresh rather than trusting the task file's own (now-stale) description of which files the concurrent actor had touched — found the drift had indeed evolved, confirmed it still doesn't touch this task's own deliverable. Attempted to falsify the Chromium result as a test-harness bug (wrong URL, premature check) rather than a genuine limitation — ruled out via matching URL, matching DOM/slider layout, a clean console, and an explicit adapter-software-rasterizer probe corroborating the task's own pre-existing, cross-example-precedented (`orrery/webgpu`) disclosure. Attempted to falsify M1/I1/I2 by running fresh rather than trusting the executor's same-session numbers, given known concurrent API-rename activity in this exact dependency chain — came back clean against current source. No Blocking findings. One new Non-Blocking finding recorded above (Firefox capability not reflected in the Out of Scope text) — does not block PASS, not fixed here as out of this acceptance pass's scope.

**Manual reconciliation disclosure:** `tsk .acceptance_pass` refuses this transition per BUG-197
(the same-session guard in `lifecycle.rs::same_session` compares only the `user@host` prefix, which
collides for any actor on this machine — see `tsk.rulebook.md`'s BUG-197 CLI Enforcement note). Per
explicit user authorization (2026-08-14, "continue. reach consistency"), the Execution State fields
above were hand-applied to mirror exactly what `.acceptance_pass` itself sets — verified this session
directly against `lifecycle.rs::handle_acceptance_pass`'s actual source rather than inferred from
precedent alone — `priority`→0, motion fields cleared (`actor`/`started_at`/`expires_at`→null,
`in_motion`→false), `verified_by`→resolved actor (normalized from the pre-existing `.../cgtools/task/`
value to `.../cgtools/`, mirroring the same normalization already applied once in this file's own
History `CLAIM` entry above), `verification_date`→timestamp, `completed_at`/`completed_by`→newly
appended (neither field previously existed on this file), `state`→✅ (Completed) — given the PASS
verdict above (Round 1) was independently reached before this override and is not itself being
re-decided here. `open` is deliberately left unset: the real `handle_acceptance_pass` calls
`set_field` (not `set_or_insert_field`) for `open`, and `set_field` is a documented no-op when the
field is absent (`model.rs::ExecutionState::set_field`) — this file never carried an `open` field, so
the actual CLI would not add one either. This is a disclosed exception to Claim Forgery
(`tsk.rulebook.md`), performed under specific user authorization, not a silent hand-edit.

## Journal

| Timestamp  | Actor | Event | Note |
|------------|-------|-------|------|
| 2026-08-14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | All Delivery Requirements were already met at filing time (implementation/tests/docs predate this task file — filed to bring already-complete work under governance, same pattern as task 111). Re-confirmed fresh this session, not taken on faith: `cargo nextest run -p minwebgpu_shader_chunk_preview` 6/6 passed; `cargo check -p minwebgpu_shader_chunk_preview --target wasm32-unknown-unknown` exit 0 — both via longrun detached launch. Checklist/Measurements/Invariants/Anti-faking boxes in the Verification section deliberately left unchecked — the executor does not self-verify acceptance; leaving for an independent verifier per Claim Accept (📦→🔎). Due-diligence notes for that verifier: (1) `git status --short module/shader/shader_chunks_core module/shader/shader_chunks_params module/shader/shader_chunks` shows a live concurrent actor's unrelated in-flight edits to those 3 crates — none touch this task's own deliverable (`examples/minwebgpu/shader_chunk_preview/**`), consistent with the `project_concurrent_task_actor` memory pattern; this task's own Checklist C6 already re-confirmed the public API surface it consumes (`set_compose`/`chunk_discover`) still resolves correctly against that in-flight state. (2) An optional bonus check — `cargo clippy --all-targets --all-features -- -D warnings` run from this crate — fails, but not on this crate's own code: it fails compiling the unrelated `mingl` dependency (`module/min/mingl/src/web.rs:102`, `unused_imports` on `is_self_contained_url`), confirmed via `git log -1 -- module/min/mingl/src/web.rs` to be part of the last committed commit (`d7304b98`), not a live edit. This is a pre-existing defect several dependency-hops from this task's own Crate Scope Unity boundary and outside this task's Delivery Requirements (which only require `cargo nextest`/`cargo check --target wasm32-unknown-unknown`, both green) — flagged here as an FYI, not fixed, since fixing it would be an out-of-scope crate-boundary violation for this task. |
| 2026-08-14 21:00:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-14 21:24:02 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | ACCEPTANCE_PASS | acceptance passed (manual override — BUG-197, see Outcomes disclosure) |

## History

- **[2026-08-14]** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: new `shader_chunk_preview` WebGPU browser example answering "what command opens a window with one shader chunk rendered, tunable by UI" — implementation, tests, and docs already complete at filing time; filed to bring the work under task-system governance and independent verification.
- **[2026-08-14]** `SELF_CORRECTION` — `doc_tsk` Step 7 Round 0 (`principles_general.rulebook.md § Governance : Procedure - Codebase Governance Check`) and Round 1 (`tsk.rulebook.md § Core Procedures : Task Quality Gate`) found 4 gaps against the already-PASSed Readiness Verification Gate, none Blocking. Round 1: (1) 3 of 4 Out of Scope items lacked a dedicated Checklist confirmation question; (2) 2 of 6 Acceptance Criteria bullets (readme.md content, index.md/index.html registration) lacked a corresponding Checklist question — fixed by adding C6-C10 with fresh concrete evidence for each (git status on the 3 excluded crates, `find` for absent showcase.webp, git status on action/run·action/gallery, grep on readme.md/index.md/index.html content); (3) `repo_identity` (an established field present on 29 other task/bug files, all `self`) was missing from Execution State — added `**repo_identity:** self`. Round 0: (4) the crate's own `readme.md` (13 files, over the 3+ threshold) had no Responsibility Table — added a `## Files` section with one row per file, each verified against its actual doc comment/content, not guessed. Also cleaned up 5 stray `-000N_longrun.log` scratch files left in the crate root from this session's own test/wasm32-check runs (already captured as evidence in M1/I2 above; safe to delete). All 4 were coverage/hygiene gaps, not defects in the underlying deliverable.
- **[2026-08-14]** `CLAIM` — Claimed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (`executor_type: any`; `blocked_by: null`, trivially satisfied). `state: 🎯 (Verified)` → `⚙️ (Executing)`; file moved to `task/executing/`. Backfilled `verified_by`/`verification_date` (previously left null despite the Readiness Verification Gate having already PASSed on this date) to the same identity/date as the Verification Record above.
- **[2026-08-14]** `EXEC_COMPLETE` — Delivery Requirements re-confirmed met (see Journal above for full evidence: native tests 6/6, wasm32 check exit 0, both fresh this session). `state: ⚙️ (Executing)` → `📦 (Executed)`; `in_motion` → `false`; file moved to `task/executed/`. No Outcomes/Acceptance Results section added — that is reserved for an independent acceptance reviewer (Claim Accept, 📦→🔎); this executor does not self-accept.
- **[2026-08-14]** `NOTE` — This task's deliverable (`examples/minwebgpu/shader_chunk_preview/`) was subsequently ported into a proper, independently-distributable utility crate pair (`shader_chunks_preview` CLI + `shader_chunks_preview_web` browser runtime, under `module/shader/`) as part of a broader `shader_chunks`/`sch` CLI architectural split into four utilities (query, compose, params, preview — each a `_core` crate plus a CLI crate, sharing `shader_chunks_cli_core`). The original example directory has been deleted (recoverable via git history, commit `526f2109`); its 3-slider live-preview capability is now reachable via `shader_chunks preview <name>` / `sch preview <name>`, generalized from one hardcoded local chunk to any bundled or `file::`-supplied chunk, with the same naga-validated composition and `minwebgpu` render loop this task verified. All external cross-references this task's Related Documentation section lists (`examples/minwebgpu/readme.md`, `examples/readme.md`, `examples/demo_completeness.md`, `docs/cli/command_group/04_parameters.md`, `examples/index.md`/`index.html`) have been updated or regenerated accordingly. This does not retroactively change this task's own Verdict above — the deliverable was real, independently tested, and independently accepted at the time — it records only that the deliverable's location and shape later changed under separate, unrelated work.
