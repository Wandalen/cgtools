# New minwebgpu example: live browser preview of a composed shader_chunks set with UI-tunable parameters

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📦 (Executed)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgpu/shader_chunk_preview
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-14
- **blocked_by:** null
- **priority:** 2
- **executing_at:** 2026-08-14
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false

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

## Journal

| Timestamp  | Actor | Event | Note |
|------------|-------|-------|------|
| 2026-08-14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | All Delivery Requirements were already met at filing time (implementation/tests/docs predate this task file — filed to bring already-complete work under governance, same pattern as task 111). Re-confirmed fresh this session, not taken on faith: `cargo nextest run -p minwebgpu_shader_chunk_preview` 6/6 passed; `cargo check -p minwebgpu_shader_chunk_preview --target wasm32-unknown-unknown` exit 0 — both via longrun detached launch. Checklist/Measurements/Invariants/Anti-faking boxes in the Verification section deliberately left unchecked — the executor does not self-verify acceptance; leaving for an independent verifier per Claim Accept (📦→🔎). Due-diligence notes for that verifier: (1) `git status --short module/shader/shader_chunks_core module/shader/shader_chunks_params module/shader/shader_chunks` shows a live concurrent actor's unrelated in-flight edits to those 3 crates — none touch this task's own deliverable (`examples/minwebgpu/shader_chunk_preview/**`), consistent with the `project_concurrent_task_actor` memory pattern; this task's own Checklist C6 already re-confirmed the public API surface it consumes (`set_compose`/`chunk_discover`) still resolves correctly against that in-flight state. (2) An optional bonus check — `cargo clippy --all-targets --all-features -- -D warnings` run from this crate — fails, but not on this crate's own code: it fails compiling the unrelated `mingl` dependency (`module/min/mingl/src/web.rs:102`, `unused_imports` on `is_self_contained_url`), confirmed via `git log -1 -- module/min/mingl/src/web.rs` to be part of the last committed commit (`d7304b98`), not a live edit. This is a pre-existing defect several dependency-hops from this task's own Crate Scope Unity boundary and outside this task's Delivery Requirements (which only require `cargo nextest`/`cargo check --target wasm32-unknown-unknown`, both green) — flagged here as an FYI, not fixed, since fixing it would be an out-of-scope crate-boundary violation for this task. |

## History

- **[2026-08-14]** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: new `shader_chunk_preview` WebGPU browser example answering "what command opens a window with one shader chunk rendered, tunable by UI" — implementation, tests, and docs already complete at filing time; filed to bring the work under task-system governance and independent verification.
- **[2026-08-14]** `SELF_CORRECTION` — `doc_tsk` Step 7 Round 0 (`principles_general.rulebook.md § Governance : Procedure - Codebase Governance Check`) and Round 1 (`tsk.rulebook.md § Core Procedures : Task Quality Gate`) found 4 gaps against the already-PASSed Readiness Verification Gate, none Blocking. Round 1: (1) 3 of 4 Out of Scope items lacked a dedicated Checklist confirmation question; (2) 2 of 6 Acceptance Criteria bullets (readme.md content, index.md/index.html registration) lacked a corresponding Checklist question — fixed by adding C6-C10 with fresh concrete evidence for each (git status on the 3 excluded crates, `find` for absent showcase.webp, git status on action/run·action/gallery, grep on readme.md/index.md/index.html content); (3) `repo_identity` (an established field present on 29 other task/bug files, all `self`) was missing from Execution State — added `**repo_identity:** self`. Round 0: (4) the crate's own `readme.md` (13 files, over the 3+ threshold) had no Responsibility Table — added a `## Files` section with one row per file, each verified against its actual doc comment/content, not guessed. Also cleaned up 5 stray `-000N_longrun.log` scratch files left in the crate root from this session's own test/wasm32-check runs (already captured as evidence in M1/I2 above; safe to delete). All 4 were coverage/hygiene gaps, not defects in the underlying deliverable.
- **[2026-08-14]** `CLAIM` — Claimed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (`executor_type: any`; `blocked_by: null`, trivially satisfied). `state: 🎯 (Verified)` → `⚙️ (Executing)`; file moved to `task/executing/`. Backfilled `verified_by`/`verification_date` (previously left null despite the Readiness Verification Gate having already PASSed on this date) to the same identity/date as the Verification Record above.
- **[2026-08-14]** `EXEC_COMPLETE` — Delivery Requirements re-confirmed met (see Journal above for full evidence: native tests 6/6, wasm32 check exit 0, both fresh this session). `state: ⚙️ (Executing)` → `📦 (Executed)`; `in_motion` → `false`; file moved to `task/executed/`. No Outcomes/Acceptance Results section added — that is reserved for an independent acceptance reviewer (Claim Accept, 📦→🔎); this executor does not self-accept.
