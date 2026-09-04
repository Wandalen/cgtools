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
- **unit_type:** module
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

## In Scope

- `module/helper/renderer`, `src/webgl/loaders/gltf.rs`: triage of 6 inline `#[test]`
  functions, all kept inline as one documented exception pinning the private helper
  `resolve_asset_uri` (URI-resolution table)

## Out of Scope

- Exposing `resolve_asset_uri` to the public API solely for test placement — rejected (zero
  non-test callers; testing through `load` needs a browser context)
- Any file touched by the user's pre-existing uncommitted native-backend work — explicitly
  left untouched (only `gltf.rs` and task/health records modified)

## Verification

### Checklist

- [x] C1 — Are exactly the claimed 6 inline `#[ test ]` functions still present in `src/webgl/loaders/gltf.rs` (none relocated, none deleted)? `grep -c "#\[ test \]" src/webgl/loaders/gltf.rs` → `6` (lines 1134, 1144, 1154, 1164, 1174, 1184).
- [x] C2 — Is the private helper `resolve_asset_uri` still un-exported (the documented "keep inline" decision, not silently reversed)? `grep -n "fn resolve_asset_uri" src/webgl/loaders/gltf.rs` → private `fn` at line 416; the crate's `mod_interface!` block (lines 1199-1205) exports only `GLTF` and `load` — `resolve_asset_uri` is absent from it.
- [x] C3 — Is the documented exception rationale (naming this task) present at the test module? `src/webgl/loaders/gltf.rs:1120` → `// Exception ( task 075 ) : these tests stay inline because they pin the ...`.
- [x] C4 — Were zero relocations/consolidations performed, matching the "0 relocated + 6 kept + 0 consolidated = 6" claim? Confirmed by C1 (count unchanged at 6) and C2/C3 (no new export, exception comment present) — no test file was added or removed for this task.

### Measurements

- [x] M1 — "task 075" exception-comment occurrences in `gltf.rs`: `1` (was: `0`, cite `git show 4469eafb^:module/helper/renderer/src/webgl/loaders/gltf.rs` → `0` hits; `git show 4469eafb:...` → `1` hit).

### Invariants

- [x] I1 — Native test suite (shared with 013/020/047, package-scoped, `longrun`-detached): `cargo nextest run -p renderer --all-features` → exit 0, `79 tests run: 79 passed, 0 skipped`, including all 6 kept-inline tests (`renderer webgl::loaders::gltf::private::tests::passes_absolute_url_through`, `passes_data_uri_through`, `empty_folder_yields_origin_absolute_uri`, `joins_relative_uri_with_folder`, `passes_blob_uri_through`, `passes_origin_absolute_path_through` — all `PASS`).
- [x] I2 — Compiler/lints: `cargo clippy -p renderer --all-targets --all-features -- -D warnings` → exit 101, **fails**, same unrelated `browser_log` root cause documented in full under task 013's Verification (commit `5f33be66`, 2026-08-11, postdates this task). Isolated via the `--no-deps` variant → exit 0, clean — `renderer`'s own code, including `gltf.rs`, is unaffected.

### Anti-faking checks

- [x] AF1 — Guards against `resolve_asset_uri` being exported later without revisiting this task's decision: re-running C2's `mod_interface!` check must keep finding only `GLTF`/`load` exported — an export appearing there without a fresh task repeats the exact "widen the API solely for test placement" trade-off this task rejected.
- [x] AF2 — Guards against the 6 inline tests being silently deleted rather than genuinely exercised: I1's PASS list must keep showing all 6 `gltf::private::tests::*` names — a future full run showing fewer than 6 signals silent coverage loss, not a legitimate relocation (which this crate's own convention requires going through the same expose-or-exception decision, not a quiet deletion).

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
