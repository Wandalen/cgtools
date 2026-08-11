# Decide test placement for canvas_renderer's private-access bug reproducer (decomposed from task 035)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/helper/canvas_renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **0 tests/ files; 1 inline `#[ test ]` in
`src/renderer.rs:440`**. The single inline test is NOT ordinary cleanup material: it is a fully
documented 5-section bug reproducer (`resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings`,
color-counter desync across non-mesh scene nodes) that imports `super::private::*` — it tests
`resolve_mesh_colors`, an internal function not on the public surface. Blind relocation to `tests/`
would force publishing an internal, which is a worse outcome than the convention deviation.

Decide explicitly: (a) expose `resolve_mesh_colors` (only if the API genuinely warrants a public
mesh-color resolution entry point), (b) keep the test inline as a documented exception (record the
rationale next to the test), or (c) restructure so the behavior is observable through `render`'s
public contract and test THAT from `tests/`. Also decide whether the crate needs any public-surface
tests/ suite at all beyond this reproducer (it currently has none). wasm-target caveat: the crate's
docs.rs default target is wasm32 — verify which parts are natively testable before promising a
native `tests/` suite.

Verify with `longrun .launch dir::<workspace root> -- cargo test -p canvas_renderer --all-features`
(or the wasm-appropriate equivalent established at pickup) — the reproducer must stay green and
keep its 5-section documentation intact wherever it lands.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17). Flagged during 035's triage as the case that disproved "relocation is
  mechanical": the crate's only test requires private access by design.
- **[2026-08-10]** `IMPLEMENTED` — DECISION: **(b) keep the test inline as a documented
  exception**; rationale recorded next to the test as a comment block on `mod tests`
  (`src/renderer.rs`). Why the alternatives lose:
  - **(a) expose `resolve_mesh_colors`:** the function is deliberately internal — its own doc
    states it exists so the mesh-to-color correspondence can be verified "independent of a live
    WebGL context". No caller needs a public mesh-color-resolution entry point; publishing an
    internal solely for test placement is API widening for zero users (YAGNI).
  - **(c) test through `render`'s public contract from `tests/`:** impossible natively — every
    `CanvasRenderer` method (including `render`) takes `&GL`, a live WebGL2 context; that is
    exactly why `resolve_mesh_colors` was extracted in the first place. Browser-side execution
    waits on the workspace's missing wasm test-runner infrastructure (same gap recorded by
    task 064 for tilemap_renderer's Encoded decoder).
  - **No public-surface `tests/` suite created** — decided explicitly: the crate's entire
    public surface is `CanvasRenderer`, whose every method requires `&GL`, so a native `tests/`
    directory would have nothing real to exercise (mocking a GL context is banned). The census's
    wasm caveat is settled the other way around: the crate COMPILES and its reproducer RUNS
    natively (minwebgl/web-sys compile everywhere); only live-context behavior is
    browser-bound.
  The reproducer itself (`resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings`) and its
  complete 5-section documentation are untouched.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green: log `-0034` exit 0 — the
  reproducer 1/1 natively + doc-test 1/1, 0 failed, proving native testability alongside the
  exception rationale. In-loop adversarial catch: the draft's option (c) initially looked
  attractive ("restructure so the behavior is observable through `render`"), but reading
  `render` showed it ALREADY consumes `resolve_mesh_colors`'s output in lockstep — the
  correspondence logic is the extracted function; re-testing it through `render` would only
  re-add the GL-context dependency the extraction removed. The exception is the fix-preserving
  outcome, not a convention dodge.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Option (a) rejected precisely on YAGNI: no caller for a public resolve_mesh_colors | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Census re-derived: 1 inline test at renderer.rs (was :440, now shifted by the exception comment), nothing else | — |
| D5 | Execution Scope | 🟢 | 🟢 | Decision task — only artifact is the exception comment + this record | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | The convention's own escape hatch used as designed: recorded exception beats forced API widening | — |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | Option (c) initially looked viable — reading `render` showed it already walks `resolve_mesh_colors`'s output in lockstep; re-testing via `render` would only re-add the GL dependency the extraction removed | Exception chosen with the reasoning recorded in the comment |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0034` exit 0: reproducer 1/1 natively + doc-test 1/1 — wasm caveat settled (crate tests natively) | — |
| B6 | Knowledge Preservation | 🟡 | 🟢 | A decision living only in the task record would be invisible at the code site where the next reader trips over the convention deviation | Rationale comment placed directly on `mod tests`, naming the task and both rejected options |
| B7 | Code Cleanliness | 🟢 | 🟢 | Reproducer + 5-section doc untouched | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
