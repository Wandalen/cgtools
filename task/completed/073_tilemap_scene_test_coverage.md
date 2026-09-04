# Restore test-directory convention in tilemap_scene (decomposed from task 035)

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
- **unit:** module/helper/tilemap_scene
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **10 tests/ files with 133 test markers; 38 inline #[test] in src/**. Same shape as tiles_tools: healthy tests/ suite + 38 inline tests needing per-test disposition.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception. Never delete a test to
   satisfy the rule; never widen an API solely to satisfy it either.
2. Consolidate any inline test that duplicates existing tests/ coverage instead of relocating a
   second copy.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p tilemap_scene --all-features` —
   all green before and after each relocation batch.

## In Scope

- `module/helper/tilemap_scene`: triage of 38 inline `#[test]` functions across 9 src modules
  (`hash`, `compile/ids`, `compile/conditions`, `compile/camera`, `compile/edges`,
  `compile/vertex`, `compile/viewport`, `compile/animation`, `compile/coords`)
- All 38 relocated — zero kept inline, zero consolidated — into `tests/hash_test.rs` (2) and
  `tests/compile_units_test.rs` (36), since every tested item was externally reachable via the
  crate's `mod_interface` root export
- New `tests/readme.md` (unit/integration two-level structure + 12-row Responsibility Table)

## Out of Scope

- Consolidating `edge_rotation_flat_top_table`/phase-separation tests onto existing
  integration tests — examined and rejected; both kept as testing different observation levels
- Per-source-module `tests/` file layout (one file per src module) — rejected in favor of the
  crate's existing domain-file convention

## Verification

### Checklist

- [x] C1 — Is `src/` still free of inline `#[test]` functions (all 38 relocated, none reintroduced)? `grep -rc "#\[ *test *\]" src/` summed across every file → `0`.
- [x] C2 — Do `tests/hash_test.rs` and `tests/compile_units_test.rs` still hold exactly `2` and `36` tests respectively (the claimed 38-test split)? `grep -c "#\[ *test *\]"` → `2` and `36` respectively.
- [x] C3 — Is a relocated item still genuinely reachable via a crate-root import (confirming "zero exceptions needed", i.e. nothing was silently made a documented visibility exception)? `tests/hash_test.rs:8` — `use tilemap_scene::{ hash_coord, hash_str };`; `tests/compile_units_test.rs:16` — same crate-root `use tilemap_scene::` pattern.
- [x] C4 — Does `tests/readme.md` still exist with the claimed 12-row Responsibility Table and two-level (unit/integration) structure? Confirmed: 44 lines, 12-row table, "Unit level" / "Integration level" split present.
- [x] C5 — Does the `scene_state_test.rs` 19-vs-17 anomaly still hold exactly as explained (pre-existing `cfg(not(debug_assertions))` split, not a TASK-073 regression)? `grep -c "#\[ *test *\]" tests/scene_state_test.rs` → `19`; `grep -c "cfg( *not( *debug_assertions *) *)"` → `2` (19 − 2 = 17 run in a debug-profile invocation).

### Measurements

- [x] M1 — Inline `#[test]` count in `src/compile/edges.rs`: `0` (was: `2`, per `git show 4469eafb^:module/helper/tilemap_scene/src/compile/edges.rs`) — one of the 9 relocated modules, spot-checked exactly against the filed census.
- [x] M2 — `tests/compile_units_test.rs`: exists now with `36` tests (was: did not exist — `git show 4469eafb^:module/helper/tilemap_scene/tests/compile_units_test.rs` resolves to no such path).

### Invariants

- [x] I1 — Test suite (crate-scoped, `longrun`-launched, same run reused from TASK-028): `cargo nextest run -p tilemap_scene --all-features` → exit `0`, "169 tests run: 169 passed, 0 skipped" (`-0141_longrun.log`) — matches the 169-total this task's own History cites (36 + 2 relocated + all pre-existing suites, unchanged).
- [ ] I2 — Compiler/lints (crate-scoped, `longrun`-launched, same run reused from TASK-028): `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings` → exit `101` (FAIL), root-caused to the same pre-existing `browser_log` / `tilemap_renderer` dependency drift described in TASK-028's I2, plus the same 7 pre-existing `reason=`-less `#[allow(...)]` sites in `tilemap_scene`'s own `src/` — none of which were touched by TASK-073's purely mechanical test relocation (`-0141_longrun.log`).

### Anti-faking checks

- [x] AF1 — Guards against a new inline `#[cfg(test)] mod tests` block being silently reintroduced in `src/`, bypassing the crate's now-established all-tests-in-`tests/` convention: re-running C1's `grep -rc "#\[ *test *\]" src/` must stay `0` for every future change, unless a genuinely private-only item forces a fresh, documented exception per `tests/readme.md`'s own "Adding tests" procedure.
- [x] AF2 — Guards against silent test loss or duplication during any future relocation-style refactor: the `169`-test total (I1) is the trip-wire — any relocation that drops or duplicates a test changes this independently-reproducible number (`cargo nextest run -p tilemap_scene --all-features`), rather than relying on any prose claim of "N relocated."

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17).
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup: 38 inline tests across 9 src
  modules (hash 2, compile/ids 2, compile/conditions 5, compile/camera 3, compile/edges 2,
  compile/vertex 5, compile/viewport 6, compile/animation 9, compile/coords 4) — matches the
  filed census. Structural finding that shaped the disposition: the crate is a `mod_interface`
  crate whose lib.rs `layer` chains propagate every `exposed` item to the crate root, and the
  full export map showed EVERY tested item externally reachable (functions, `Camera`,
  `NeighborState`, `TriBlendPattern`, `EdgePosition`, resource types — all `exposed` with pub
  fields; `Transform` and `ResourceId` come from the tilemap_renderer dependency). So, unlike
  the sibling crates: **all 38 RELOCATED, zero exceptions, zero consolidations**. The two
  duplication candidates examined and rejected as duplicates: `scene_model_compile_test.rs`'s
  `edge_rotation_matches_direction` tests rotation through the compile_frame/SceneSnapshot
  integration surface while the inline `edge_rotation_flat_top_table` pins the unit formula
  directly (different observation levels — both kept, distinction documented in
  tests/readme.md); `scene_events_test.rs`'s phase-separation test uses `hash_coord`
  behaviorally, not as a determinism pin. Placement follows the crate's existing domain-file
  convention (not file-per-src-module): `tests/hash_test.rs` (2 — normative SPEC §13
  known-answer pins) and `tests/compile_units_test.rs` (36 — unit contracts of the
  compile-layer primitives, sectioned per source module, deliberately separate from
  scene_model_compile_test.rs's integration-level responsibility). Bodies moved verbatim
  (dedented one level) with only named mechanical transforms: crate-qualified root imports
  (`use tilemap_scene::{ … }` — the same pattern the existing suite uses); fully-qualified
  `crate::resource::TimedFrame` in the irregular-timing test rewritten to the imported root
  re-export; viewport module's `#[ allow( clippy::float_cmp ) ]` + rationale comment carried
  to file level; coords module's reference-facts comment carried into its section. All 9
  inline modules deleted whole by an assertion-guarded partition script (per-file cfg/
  mod_interface!/close-brace boundary asserts + per-module and total test-count asserts,
  38/38 verified by the script). `tests/readme.md` created (unit-vs-integration two-level
  structure + Responsibility Table for all 12 entries + adding-tests procedure — the
  directory previously had none).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0043` (`cargo test -p tilemap_scene
  --all-features`) exit 0, 14s: src unit suite now 0 tests (inline modules gone),
  compile_units_test 36/36, hash_test 2/2, and every pre-existing suite untouched and green
  (catalog 6, hex_config 3, renderer_cache 18, renderer 6, scene_events 16,
  scene_model_compile 42, scene_model 16, scene_state 17, sorted_batching 7, doc-tests 0
  passed / 2 ignored as before). 38 relocated + 0 kept + 0 consolidated = 38 — every inline
  test accounted for. One run-count anomaly investigated and explained honestly:
  scene_state_test.rs greps 19 `#[ test ]` fns but runs 17 in a debug-profile run — two are
  `cfg( not( debug_assertions ) )`-gated release-only tests (pre-existing profile split, not
  a 073 regression). In-loop catch: the initial per-source-module file plan (9 new files, the
  072 pattern) was rejected at the One-Second Test against this crate's own domain-file
  tests/ convention — consolidated to 2 domain files instead.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Census matched at pickup (38/38) | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Both duplication candidates adversarially examined and kept — unit vs integration observation levels, not copies | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Initial placement plan copied 072's file-per-source-module pattern (9 new files); adversarial pass against THIS crate's tests/ tree found its convention is domain files — 9 stub files would have violated the One-Second Test against scene_model_compile_test.rs's domain | Consolidated to 2 domain files: hash_test.rs (normative pins) + compile_units_test.rs (compile-layer unit contracts, sectioned per module) |
| D5 | Execution Scope | 🟢 | 🟢 | All edits within tilemap_scene + task/ + health.md | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | mod_interface export map verified per item BEFORE classifying — all 38 reachable at root, so zero exceptions needed; no test deleted, no API widened, no mocks | — |
| B2 | Test-First | 🟢 | 🟢 | Relocation task — green census before, green run after | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — single launch green first try | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Bodies verbatim; only named mechanical transforms (root imports, TimedFrame qualification, carried allow + comments, dedent) | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0043` exit 0: unit 0 (emptied), 36+2 relocated run, all 10 pre-existing suites green; 19-vs-17 scene_state anomaly investigated — pre-existing cfg(not(debug_assertions)) release-only pair, not a regression | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | tests/readme.md created (previously absent): two-level structure, 12-entry Responsibility Table, adding-tests procedure; both-levels edge_rotation distinction documented | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Partition script boundary-asserted every deletion (38/38); src/ now 0 inline tests; junctions left exactly one blank line before each mod_interface! block | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 15/15 |
