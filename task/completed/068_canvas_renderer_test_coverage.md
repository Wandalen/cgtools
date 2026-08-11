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

## Verification

### Checklist

- [x] C1 — Is the census still accurate: `0` `tests/` files and exactly `1` inline `#[test]` in `src/`? `ls module/helper/canvas_renderer/tests` → no such directory; `grep -rn "#\[ *test *\]" src/` → exactly 1 hit (`src/renderer.rs:448`).
- [x] C2 — Is the documented-exception rationale comment still present immediately above `mod tests`, naming task 068 and both rejected alternatives? `src/renderer.rs:379-386` — present, names "task 068" explicitly, explains why exposing `resolve_mesh_colors` (option a) and testing through `render`'s public surface (option c) both lose.
- [x] C3 — Is `resolve_mesh_colors` still genuinely unreachable from outside the crate, confirming option (a) was never silently done later? Same evidence as TASK-016/C3: `grep -n resolve_mesh_colors src/lib.rs` → `0` hits; the crate-root `mod_interface!` block exposes only `CanvasRenderer`.
- [x] C4 — Does every public `CanvasRenderer` method still require a live `&GL`, confirming option (c) is still genuinely impossible natively? `grep -n "pub fn" src/renderer.rs` → `new`, `upload_node`, `render`, `set_texture` all take `gl : &GL` directly; `get_texture` takes none but requires an already-constructed `Self`, which itself required `&GL` via `new` — there is no GL-free construction path.
- [x] C5 — Was a public-surface `tests/` suite deliberately never created, and does that remain the case? `module/helper/canvas_renderer/tests/` still does not exist (see C1); the crate's only public type (`CanvasRenderer`) still has no GL-free method to exercise from an integration test.

### Measurements

- [x] M1 — Inline `#[test]` count in `canvas_renderer/src/`: `1`, unchanged since TASK-016 added it and TASK-068's own census recorded it — TASK-068 was a placement decision, not a code change, so no delta is expected or claimed here.

### Invariants

- [x] I1 — Test suite (crate-scoped, `longrun`-launched, same run reused from TASK-016): `cargo nextest run -p canvas_renderer --all-features` → exit `0`, "1 test run: 1 passed, 0 skipped" (`-0138_longrun.log`).
- [ ] I2 — Compiler/lints (crate-scoped, `longrun`-launched, same run reused from TASK-016): `cargo clippy -p canvas_renderer --all-targets --all-features -- -D warnings` → exit `101` (FAIL), root-caused entirely to the unrelated, pre-existing `browser_log` dependency issue described in TASK-016's I2 — `canvas_renderer`'s own `src/` carries zero `#[allow(...)]` attributes and is never reached by this run (`-0138_longrun.log`).

### Anti-faking checks

- [x] AF1 — Guards against a future contributor "fixing" the test-placement convention deviation by blindly moving the test to `tests/` (which would fail to compile, since integration tests can't see `super::private::*`): the exception comment (`renderer.rs:379-386`) is the guard — re-check it is still present and still explains why relocation is impossible, not just asserting "keep for now" without reasoning.
- [x] AF2 — Guards against a second, unrelated inline test accreting in `src/` under cover of "there's already precedent for an inline test here": re-running C1's `grep -rn "#\[ *test *\]" src/` must still return exactly `1` hit; any second inline test needs its own fresh Crate Locality decision (per this workspace's D7 gate), not silent accretion.

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
