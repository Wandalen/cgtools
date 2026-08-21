# Legacy webgl-path frame-orchestration attachment-selection test coverage

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 00:46:28
- **expires_at:** 2026-08-19 02:46:28
- **round:** 1
- **state:** 🔎 (Accepting)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** system
- **verification_date:** null
- **blocked_by:** null
- **executing_at:** 2026-08-19 00:46:28
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** true
- **accepting_at:** 2026-08-19 00:46:28
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-19 00:40:43

## Goal

Give the legacy `renderer::webgl::Renderer::render()`'s drawbuffers
attachment-selection logic (the `has_transparent`/`has_emissive` 4-way
branch selecting which of `[0]` / `[0,1]` / `[0,2,3]` / `[0,1,2,3]` color
attachments to enable per frame) natively-testable, zero-GL-context unit
coverage — the one piece of this method's frame-shape logic that is a pure
function of two booleans, extractable without touching any
`WebGl2RenderingContext` call, unlike the canonical webgpu path which
already gets end-to-end coverage via `opaque_path_renders_lit_quad`. Matters
now because `docs/layer/003_l2_frame_orchestration.md`'s Embedded Instances
Today section documents this exact ordering/attachment logic as fact with
zero test citation, unlike its sibling canonical-path bullet which cites a
real passing test. Bounded to one pure-function extraction plus one new
native test file in this one crate. Testable: `cargo test -p renderer --test
webgl_frame_orchestration_test` exits 0 with all 4 branch cases passing.

## In Scope

- `module/helper/renderer/src/webgl/renderer.rs`: extract the
  attachment-selection branch — currently inline in `Renderer::render()`,
  the `if has_transparent && has_emissive {...} else if has_transparent
  {...} else if has_emissive {...} else {...}` block that feeds
  `gl::drawbuffers::drawbuffers` — into a pure function, e.g. `fn
  frame_attachments( has_transparent : bool, has_emissive : bool ) -> &'static
  [ u32 ]`, called from `render()` in place of the inline branch, immediately
  followed by the same `gl::drawbuffers::drawbuffers( gl, ... )` call
  `render()` already makes.
- New `module/helper/renderer/tests/webgl_frame_orchestration_test.rs`
  (native, no GL context, no feature gate beyond the crate's default), 4
  cases — one per boolean combination — pinning the exact attachment array
  each combination returns.

## Out of Scope

- Pixel-level / live-`WebGl2RenderingContext` rendering verification for the
  legacy path — this workspace has no native/offscreen WebGL2 provider
  (confirmed absent: no swiftshader/osmesa/surfman/glutin dependency
  anywhere in `minwebgl`/`mingl`/`renderer`'s `Cargo.toml`), so unlike the
  canonical webgpu path's native-wgpu-plus-lavapipe route, there is no way
  to construct a real `WebGl2RenderingContext` outside an actual browser.
  Closing that is a workspace-wide browser-test-infrastructure decision
  (already flagged as an accepted gap elsewhere — see
  `tilemap_renderer/tests/webgpu_backend_test.rs`'s own doc comment), not
  achievable inside this leaf-crate task.
- Any other part of `render()`'s GL-calling sequence (`nodes_collect`,
  `opaque_draw`, `transparent_draw`, `composite`) — these are not pure
  functions (every step calls `gl` directly) and extracting them is a much
  larger refactor than this task's bounded scope.
- The canonical `src/webgpu/` path — already covered by
  `native_render_test.rs`.
- Migrating the legacy path onto `gpu_hal` (tracked separately per
  `docs/layer/004_l3_stack_engine.md`'s strangulation note).

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Extraction lands with zero behavior change — `render()`'s call sequence
    still invokes `gl::drawbuffers::drawbuffers` with the same arrays it did
    before, just via the new function
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
| T01 | has_transparent=false, has_emissive=false | `frame_attachments` | returns `[0]` |
| T02 | has_transparent=false, has_emissive=true | `frame_attachments` | returns `[0,1]` |
| T03 | has_transparent=true, has_emissive=false | `frame_attachments` | returns `[0,2,3]` |
| T04 | has_transparent=true, has_emissive=true | `frame_attachments` | returns `[0,1,2,3]` |

## Acceptance Criteria

-   A pure attachment-selection function exists in `src/webgl/renderer.rs`
    (no `&self`, no `gl` parameter)
-   `render()` calls it in place of the previous inline branch, passing the
    same two booleans it already computes
-   `tests/webgl_frame_orchestration_test.rs` exists with all 4 Test Matrix
    cases passing
-   No pre-existing test in `renderer`'s suite regresses
-   Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Extraction**
- [ ] C1 — Does a pure attachment-selection function exist, taking exactly `( bool, bool )` and returning the attachment index list, with no `&self`/`gl` parameter?
- [ ] C2 — Does `render()`'s body call it instead of the previous inline 4-arm branch, immediately followed by the same `gl::drawbuffers::drawbuffers( gl, ... )` call it already had?

**Tests**
- [ ] C3 — Does `tests/webgl_frame_orchestration_test.rs` exist with 4 tests, one per Test Matrix row?

**Out of Scope confirmation**
- [ ] C4 — Is the new test file free of any `WebGl2RenderingContext`/`gl::` construction call?
- [ ] C5 — Do `nodes_collect`/`opaque_draw`/`transparent_draw`/`composite` remain unmodified (`git diff` shows no edits to those functions' bodies beyond the one branch touched)?
- [ ] C6 — Does `src/webgpu/` remain unmodified (`git diff` shows no edits under `src/webgpu/`)?
- [ ] C7 — Does the legacy `src/webgl/` path remain on its existing direct-GL call surface, with no new `gpu_hal` dependency introduced?

### Measurements

- [ ] M1 — new test count: `cargo test -p renderer --test webgl_frame_orchestration_test 2>&1 | grep -c "test result: ok"` → 1 (was: file did not exist)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p renderer --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — the 4 test cases assert 4 genuinely different return values, not the same array asserted 4 times: `grep -c "assert_eq" tests/webgl_frame_orchestration_test.rs` → ≥4, and the 4 expected arrays are pairwise distinct (checked by reading the file, not merely counted)

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

## Outcomes

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ (independent acceptance-verification session)
- **Date:** 2026-08-16
- **Verdict:** PASS

**B1 separation-of-concerns disclosure:** this verifying session's own visible context never implemented `frame_attachments`/`webgl_frame_orchestration_test.rs` — the work was executed by an earlier session (Journal `CLAIM_EXEC`/`EXEC_COMPLETE` entries, `executing_by` recorded as `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/`). This verifying session's own resolved identity (`scope get::id` → `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/`) collides with that `executing_by` value under the `user@host`-only granularity `tsk .acceptance_pass`'s BUG-197 guard (`same_session()`, `sys/module/scope_task/src/commands/lifecycle.rs`) compares against — both resolve to `user1@w002`. Per `tsk_verify §B1` this is disclosed rather than treated as a silent blocker; the two sessions are nonetheless genuinely distinct (this session had no memory of the diff under review — it was discovered fresh via `git diff` during this walk). Flagged upfront before starting the walk: `tsk .acceptance_pass` is expected to mechanically refuse regardless of verdict.

#### Checklist

- C1 — PASS — `pub fn frame_attachments( has_transparent : bool, has_emissive : bool ) -> &'static [ u32 ]` added to `module/helper/renderer/src/webgl/renderer.rs` (new hunk, `git diff`) — exactly `(bool, bool)` in, `&'static [u32]` out, no `&self`, no `gl` parameter.
- C2 — PASS — `render()`'s body now reads `gl::drawbuffers::drawbuffers( gl, frame_attachments( has_transparent, has_emissive ) );` in place of the previous 4-arm `if/else` block (`git diff` hunk 1) — the same `gl::drawbuffers::drawbuffers( gl, ... )` call `render()` already made, immediately fed by the new function's return value.
- C3 — PASS — `module/helper/renderer/tests/webgl_frame_orchestration_test.rs` exists with exactly 4 `#[ test ]` functions, one per Test Matrix row: `no_transparent_no_emissive_yields_main_color_only` (T01), `no_transparent_with_emissive_yields_main_and_emission` (T02), `transparent_no_emissive_yields_main_and_accumulate_revealage` (T03), `transparent_and_emissive_yields_all_four_attachments` (T04).
- C4 — PASS — read the full test file: zero occurrences of `WebGl2RenderingContext` or `gl::`; sole import is `use renderer::webgl::frame_attachments;`.
- C5 — PASS — `git diff -- module/helper/renderer/src/webgl/renderer.rs` contains exactly 3 hunks (branch→call replacement, new `frame_attachments` fn, `mod_interface!` export addition); `nodes_collect`/`opaque_draw`/`transparent_draw`/`composite` function bodies (lines 726/918/1017/1102) show no diff.
- C6 — PASS — `git diff --stat -- module/helper/renderer/src/webgpu/` → empty output.
- C7 — PASS — `git diff -- module/helper/renderer/Cargo.toml` → empty output; no `gpu_hal` dependency added; legacy path still calls `gl::drawbuffers::drawbuffers` directly.

#### Measurements

- M1 — PASS — `cargo test -p renderer --test webgl_frame_orchestration_test 2>&1 | grep -c "test result: ok"` → `1` (expected 1; file did not exist before).

#### Invariants

- I1 — PASS — `verb/test` via mandatory `longrun .launch` detached pattern → exit 0, elapsed 125s (`-0046_longrun.log`, 2502 lines, repo root). Full log content swept: `grep -n "FAIL"` (excluding the benign "0 failed" substring), `warning:`, `error:`/`error[`, `panicked`, `TIMEOUT` → zero hits anywhere in the file. Native nextest: "1852 tests run: 1852 passed, 0 skipped" (incl. all 4 `renderer::webgl_frame_orchestration_test` cases and unaffected `renderer::native_render_test opaque_path_renders_lit_quad`). Native doc-tests: all `test result: ok` with 0 failed. Native clippy (`--all-targets --all-features --workspace`): clean finish, 0 warning lines anywhere in log. wasm32 check: "52 example(s) checked, 0 failed". wasm32 test: "3 crate(s) tested, 0 failed" (the new test file correctly shows "no tests to run!" under wasm32 — it has no `#[wasm_bindgen_test]` tests by design, the same benign pattern as the crate's other native-only test files, e.g. `native_render_test.rs`).
- I2 — PASS — `RUSTFLAGS="--cfg web_sys_unstable_apis -D warnings" cargo check -p renderer --all-features` → exit 0; zero `warning:` lines in output.

#### Anti-faking checks

- AF1 — PASS — `grep -c "assert_eq" module/helper/renderer/tests/webgl_frame_orchestration_test.rs` → `4` (≥4 required). Read the file directly: the 4 expected arrays are `&[ 0 ]`, `&[ 0, 1 ]`, `&[ 0, 2, 3 ]`, `&[ 0, 1, 2, 3 ]` — pairwise distinct, not the same array repeated.

**Adversarial pass (dedicated, beyond the per-item checks above):** actively attempted to disprove each PASS above: (1) checked whether C2's fused one-line call (`frame_attachments` passed directly as `gl::drawbuffers::drawbuffers`'s second argument, rather than an intermediate `let` binding) violates the "immediately followed by" wording — resolved as satisfying it; the function call sits in place of the branch and its result flows straight into the same `gl::drawbuffers::drawbuffers` call, zero behavior change; (2) checked whether the wasm32 harness's "no tests to run!" for `webgl_frame_orchestration_test.rs` (log line 2433) was a silent failure — resolved as benign: every other native-only test file in the crate (`blender_tests.rs`, `native_render_test.rs`, etc.) shows the identical message, and the crate's own native nextest run (not wasm32) is where these 4 tests actually execute and pass; (3) checked for scope creep beyond `renderer.rs` — `git diff --stat` for the renderer crate also shows `gltf.rs`/`tests/readme.md` touched, but `git diff` on `gltf.rs` confirms it belongs to the concurrently in-flight task 118 (`light_list_get` promoted to `pub`, unrelated to drawbuffers), and `tests/readme.md`'s 1-line addition is this task's own Responsibility Table registration for the new test file; (4) checked AF1's arrays for a copy-paste duplicate — confirmed all 4 pairwise distinct by direct read, not merely by count. No blocking finding surfaced.

**BUG-197 mechanical guard (upfront disclosure):** per the B1 disclosure above, `tsk .acceptance_pass` is expected to refuse this transition (exit 1, "self-verification forbidden (actor matches executing_by)") since this verifying session's `scope get::id` shares the `user@host` prefix (`user1@w002`) with the task's own `executing_by` field, despite being a procedurally distinct session. No user-directed override was requested or authorized for this task — the CLI's actual exit code and message are reported verbatim in the Journal below; no Execution State field was hand-edited to force closure.

### Post-Hoc Drift Reconfirmation (2026-08-19)

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 1/1

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | No drift since 2026-08-16 walk invalidates C2/C5/C6/C7/M1/I1 | 🟢 | 🟢 | — | — |

Confirming: `frame_attachments`/`webgl_frame_orchestration_test.rs` unchanged since the walk (`git log --oneline -S"frame_attachments" -- .../renderer.rs` → only `fbd3f206`, this task's own commit); test file still present, still 4 `#[ test ]` functions, still counted in this window's shared full-workspace `verb/test` evidence (`-0001_longrun.log`, exit 0, 2352/2352 native tests). Adversarial: 3 later commits (`1df2f9d8`, `297ec46f`, `1b3f87ae`/`612445c4`/`bc9ffea6`) do touch `renderer.rs`/`src/webgpu/`/`Cargo.toml` after `fbd3f206` — actively checked whether any invalidates C5/C6/C7. `1df2f9d8`'s only `renderer.rs` hunk near `frame_attachments`'s line range (`@@ -1174,6 +1182,27@@`) is a brand-new unrelated function `program_needs_recompile` (BUG-258, material-cache invalidation) inserted after it, not a modification to `frame_attachments`/`nodes_collect`/`opaque_draw`/`transparent_draw`/`composite`; current file confirms all 4 named functions and `frame_attachments` still exist as distinct, unmerged definitions. `1df2f9d8`/`297ec46f`'s `webgpu/` touches and `1b3f87ae`'s `Cargo.toml` comment (mentioning `gpu_hal` only in prose, not adding it as a webgl-path dependency) are later, unrelated commits — not part of this task's own `fbd3f206` diff, and the current Cargo.toml's `gpu_hal` optional-dep entries remain gated behind `webgpu`/`native` features only, never the legacy `webgl` path this task covers. No blocking finding. Re-attempting `tsk .acceptance_pass`.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 05:58:41 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 06:07:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 06:07:58 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 115` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with task 202/206 precedent; not forced/spoofed, left at 🔎 Accepting per standing project convention |
| 2026-08-17 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | RENUMBERED | 115 → 247 — resolved a bug/task ID collision with `BUG-115` (`task/bug/completed/115_query_markdown_width_truncation_overridden_by_auto_wrap.md`), both filed independently under the shared tsk ID namespace. File and Tasks Index row renamed; external citations in `docs/layer/003_l2_frame_orchestration.md` and `task/verifying/221_*.md` updated to 247. |
| 2026-08-19 00:40:43 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 00:46:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-19 00:46:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-19 00:46:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-20 10:18:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | ACCEPTANCE_CHECK | Round 7 re-confirmation: `executing_by`/`accepting_by`/current actor all share `user1@w002` — no independent verifier available this round; per B1 separation-of-concerns, no acceptance walk performed, task left as-is at 🔎 Accepting |
| 2026-08-19 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 247` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with prior 2026-08-17 attempt and this sweep's 202/246/192/118 precedent; not forced/spoofed, left at 🔎 Accepting with PASS verdict (drift-reconfirmed) documented in `### Acceptance Results` above per standing project convention |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-15]** `FILED` — Task filed via `/doc_tsk` Phase 2 (docs/layer gap audit): add legacy webgl-path frame-orchestration attachment-selection test coverage to `renderer`.

## Related Documentation

- `docs/layer/003_l2_frame_orchestration.md` — Embedded Instances Today section's legacy-path bullet this task backs with tests
- `docs/layer/004_l3_stack_engine.md` — `renderer`'s L3 engine table entry (legacy vs canonical downward seam)
- `module/helper/renderer/tests/native_render_test.rs` — the canonical-path precedent this task partially mirrors, at the orchestration level rather than the pixel level
