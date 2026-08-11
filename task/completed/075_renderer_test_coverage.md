# Restore test-directory convention in renderer (decomposed from task 035)

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
- **unit:** module/helper/renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **16 tests/ files with 93 test markers; 6 inline #[test] in src/**. 6 inline tests in the crate with the workspace's biggest tests/ suite (incl. native pixel tests). Small, likely quick expose-or-exception pass. NOTE: renderer has pre-existing uncommitted working-tree modifications (native backend work) — inspect git diff before editing any file, touch only test-placement concerns.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception. Never delete a test to
   satisfy the rule; never widen an API solely to satisfy it either.
2. Consolidate any inline test that duplicates existing tests/ coverage instead of relocating a
   second copy.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p renderer --all-features` —
   all green before and after each relocation batch.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17).
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup: all 6 inline tests sit in ONE
  module — `src/webgl/loaders/gltf.rs` — and every one pins the PRIVATE pure helper
  `resolve_asset_uri` (URI-resolution table: folder join, blob/data/absolute/origin-absolute
  pass-through, empty-folder collapse). The helper is not in the file's `mod_interface` block
  (only `GLTF` and `load` are exported) and is the natively-testable logic extracted from the
  browser-bound glTF load path; it delegates origin-side rules to mingl's exported
  `is_self_contained_url` (the same pattern task 074 documented in mingl itself).
  Expose-or-exception decision: **all 6 KEPT INLINE as one documented exception** — exporting
  the helper widens the public API solely for test placement (zero non-test callers), and
  testing through `load` needs a browser context. Exception rationale comment added naming
  task 075 + both rejected alternatives. Zero relocations, zero consolidations, no API
  widened, no test deleted. Working-tree caution from the draft honoured: `git status` read
  first — the crate carries the user's uncommitted native-backend work, and the only files
  touched by this task are `gltf.rs` (NOT among the modified files) and the task/health
  records; `tests/readme.md` (user-modified) deliberately left untouched since no files were
  added.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0045` (`cargo test -p renderer
  --all-features`) exit 0, 22s, compiled against the working tree INCLUDING the user's
  uncommitted modifications: unit suite 6/6 passed natively (the kept-inline tests are live
  canonical coverage), all 13 tests/ suites green (animation_graph 8, blender 19,
  color_grading 2, mirror 4, native_render 1, scaler 8, shader_validation 2, skeleton 6,
  harness 23, three 0-test files), doc-tests 3/3. 0 relocated + 6 kept + 0 consolidated = 6 —
  every inline test accounted for.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Draft's working-tree warning honoured: git status read before editing; gltf.rs not among the user's modified files | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Census matched at pickup (6/6, single module) | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | No speculative exports; user-modified tests/readme.md left untouched (no rows needed) | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Disposition benefited directly from 074's finding — same private-pure-helper shape, confirmed against the mod_interface block before deciding | — |
| D5 | Execution Scope | 🟢 | 🟢 | One src comment + task/ + health.md; zero contact with the user's uncommitted work | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Adversarial pass caught a near-stale health row: the crate's tests/ fn count regenerated to 75 (dashboard said 93) — the delta comes from the user's uncommitted work, so blindly keeping the old number would have been silently wrong | health.md row updated with regenerated values and an in-progress note |
| B2 | Test-First | 🟢 | 🟢 | Comment-only change to src — behavior identical | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — single launch green first try | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Exception comment names task 075 + both rejected alternatives | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0045` exit 0: unit 6/6 native, 13 tests/ suites green incl. the user's in-progress native_render_test 1/1, doc 3/3 | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Rationale comment records the mingl delegation link (is_self_contained_url) and why inline is correct | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Minimal diff: one comment block; nothing created or deleted | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 15/15 |
