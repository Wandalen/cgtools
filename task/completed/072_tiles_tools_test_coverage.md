# Restore test-directory convention in tiles_tools (decomposed from task 035)

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
- **unit:** module/helper/tiles_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **10 tests/ files with 203 test markers; 56 inline #[test] in src/**. Well-covered crate whose 56 inline tests coexist with a large tests/ suite — triage which inline tests duplicate tests/ coverage (consolidate per No Duplication) vs test true internals (expose-or-exception each). Coordinate with draft 063 (marker resolution in the same crate).

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception. Never delete a test to
   satisfy the rule; never widen an API solely to satisfy it either.
2. Consolidate any inline test that duplicates existing tests/ coverage instead of relocating a
   second copy.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p tiles_tools --all-features` —
   all green before and after each relocation batch.

## In Scope

- `module/helper/tiles_tools`: triage of 56 inline `#[test]` functions across 7 src modules
  (`game_systems`, `spatial`, `debug`, `events`, `serialization`, `field_of_view`, `flowfield`)
- 46 tests relocated into 7 new top-level `tests/*_test.rs` files, each feature-gated to match
  its source module's own `lib.rs` gate
- 5 `field_of_view` tests consolidated onto pre-existing near-verbatim integration-suite
  duplicates; 5 tests (debug's `GridRenderer` builder-state, field_of_view's
  calculator-defaults, flowfield's private dead-code fields) kept inline as documented
  exceptions

## Out of Scope

- The disabled `tests/integration/flowfield_tests.rs` module — filed separately as follow-up
  task 078 rather than fixed here
- Exposing any pinned private field/behavior to enable relocation — rejected as unwarranted
  API widening

## Verification

### Checklist

- [x] C1 — Are the 7 relocated top-level `tests/*_test.rs` files still present? `debug_test.rs`, `events_test.rs`, `field_of_view_test.rs`, `flowfield_test.rs`, `game_systems_test.rs`, `serialization_test.rs`, `spatial_test.rs` — 7/7, confirmed via directory listing.
- [x] C2 — Is the inline `#[test]` count in `src/` still exactly 5, across exactly the 3 claimed exception files? `grep -rlE "#\[ ?test ?\]" src/` → `src/flowfield.rs`, `src/field_of_view.rs`, `src/debug.rs` (3 files); names confirmed via the fresh nextest run: `debug::tests::test_grid_renderer_creation`, `debug::tests::test_grid_renderer_markers`, `flowfield::tests::test_flow_field_creation`, `flowfield::tests::test_dynamic_flow_field_dirty_marking`, `field_of_view::tests::test_fov_calculator_creation` (5/5, matches the claimed 2+2+1 split).
- [x] C3 — Does the crate still pass green under BOTH feature configurations (this task's central claim)? Freshly re-run, not cited from old logs: `cargo test -p tiles_tools --all-features` → 245/245 (I1); `cargo test -p tiles_tools` (default features) → exit 0 — the six `enabled`-gated top-level files ran 37 tests (debug 5, events 11, field_of_view 1, flowfield 3, game_systems 8, spatial 9) + 5 inline unit tests, `serialization_test.rs` and `integration_tests.rs` both compiled to "running 0 tests", 39 doc-tests ran — an exact match to this task's own claimed default-features breakdown (log `-0049_longrun.log`).
- [x] C4 — Does `tests/readme.md` still exist with its 9-row Responsibility Table? Read in full — 9 rows present exactly as claimed (`integration_tests.rs`, `integration/`, `debug_test.rs`, `events_test.rs`, `field_of_view_test.rs`, `flowfield_test.rs`, `game_systems_test.rs`, `serialization_test.rs`, `spatial_test.rs`).

### Measurements

- [x] M1 — Inline `#[test]` count in `src/`: `5` (was: `56`, this task's own re-derived census at pickup, matching the filing task's independent count).
- [x] M2 — Live crate-wide non-doc test total: `245` (this task's own end-state was `240` = 5 unit + 46 relocated + 189 integration, logs `-0041`/`-0042`; current total is higher because task 078 — filed as this task's own explicit follow-up — later added 5 previously-dead integration tests; see task 078's Measurements).

### Invariants

- [x] I1 — Test suite (crate-scoped, `--all-features`): `cargo nextest run -p tiles_tools --all-features` → exit 0, 245/245 passed (log `-0009_longrun.log`).
- [ ] I2 — Compiler/lints (crate-scoped, `--all-features`): `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` → exit 101 (log `-0015_longrun.log`). Root cause is an unrelated dependency (`browser_log`) — full trace in task 025's Verification I2. Unrelated to this task's test-relocation work.
- [x] I3 — Default-features build/test, this task's own central claim re-run fresh: `cargo test -p tiles_tools` → exit 0 (log `-0049_longrun.log`, elapsed 210s).

### Anti-faking checks

- [x] AF1 — Guards against a relocated test silently losing its feature gate: re-running C3's dual-config check after any `tests/*_test.rs` or `lib.rs` feature-gate edit must reproduce the same 0-vs-N pattern for `integration_tests.rs`/`serialization_test.rs`.
- [x] AF2 — Guards against a "documented exception" inline test being silently duplicated back into `tests/` (recreating the pre-072 duplication this task eliminated): re-running C2's file list must stay exactly `debug.rs`/`field_of_view.rs`/`flowfield.rs` with exactly 5 tests combined.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17).
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup: 56 inline tests across 7 src
  modules (game_systems 8, spatial 9, debug 7, events 11, serialization 9, field_of_view 7,
  flowfield 5) — matches the filed census. Structural findings that shaped the per-test
  disposition: the crate has NO ungated tests/ files — the entire existing suite is behind the
  opt-in `integration` feature (`integration = []`, not in default/full), so the 56 inline tests
  were the crate's only default-features coverage; and `tests/integration/flowfield_tests.rs` is
  disabled in `mod.rs` ("temporarily disabled until flowfield generic constraints are resolved"),
  so flowfield had NO live tests/ counterpart at all. Per-test classification of the 56:
  - **46 RELOCATED** into 7 new top-level `tests/*_test.rs` files, each gated by the same
    feature that gates its source module in lib.rs (`enabled` for six files, `serialization`
    for serialization_test.rs — that module is NOT in default features). Bodies moved verbatim
    (dedented one level) with only named mechanical transforms: crate-qualified imports;
    game_systems' two private-field reads losslessly rewritten onto EXISTING pub accessors
    (`completed_quests` field → `completed_quests()`, `participants.get( &1 )` →
    `current_participant()` — sole participant is current at that point); events' fn-local
    `use common_events::*;` rewritten crate-qualified; serialization's unused `use std::fs;`
    dropped. Each new file carries the integration suite's proven crate-level allow block
    (workspace lints are strict; those blocks are the suite's own convention).
  - **5 CONSOLIDATED** — field_of_view inline tests that duplicate the LIVE gated integration
    suite near-verbatim (only constants differ): visibility-state properties, basic
    shadowcasting calculate_fov, weak line-of-sight no-panic (superseded by 3 stronger LOS
    tests), light-source builder (same-name twin), lighting-calculator basic (superseded by
    single/multiple-source tests). Run-condition nuance recorded honestly: their coverage now
    lives behind the opt-in `integration` feature, which the workspace's canonical verification
    (`--all-features`) always exercises.
  - **5 KEPT INLINE as documented exceptions** (rationale comments on each module naming task
    072 + rejected alternatives): debug's 2 GridRenderer builder-state tests (private
    `width`/`height`/`style`/`markers`, no accessor; render-output inference would test a
    different behavior); field_of_view's calculator-defaults test (private `algorithm`/
    `include_viewer`; only pin that `new()` defaults to Shadowcasting); flowfield's 2 (private
    dead-code `width`/`height`; `dirty_positions` accumulation with no observable).
  - **Infrastructure:** `tests/readme.md` created (two-layer structure + Responsibility Table
    for all 9 entries + adding-tests procedure); no dev-dependency changes needed (tempfile and
    serde_json already present). Inline modules: 4 deleted whole, 3 rebuilt keeper-only, via an
    assertion-guarded partition script (boundary + per-module test-count asserts, 56 → 5
    verified by the script itself).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green under BOTH feature configurations:
  log `-0041` (`--all-features`) exit 0 — unit 5/5 (exceptions), relocated 46/46 (debug 5,
  events 11, field_of_view 1, flowfield 3, game_systems 8, serialization 9, spatial 9),
  integration suite 189/189 untouched, doc 40/40; log `-0042` (default features) exit 0 —
  proves the gates both ways: the six `enabled`-gated files RUN (37 tests + 5 unit),
  serialization_test and integration_tests correctly compile to zero, doc 39 (serialization
  doc-test gated off). 46 + 5 + 5 = 56 — every inline test accounted for; the 5 consolidated
  ones continue as their pre-existing integration twins. Follow-up work found (not scope-crept):
  the disabled flowfield integration module filed as draft 078 with both defect classes named
  (private-field reads + generic-constraint drift) and a dedup instruction against the new
  `tests/flowfield_test.rs`.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Draft's 063-coordination warning moot — 063 completed earlier | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Follow-up (disabled flowfield module) filed as draft 078 instead of scope-creeping into 072 | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Initial plan assumed live tests/ targets everywhere; adversarial survey found the ENTIRE existing suite opt-in-gated (`integration = []`) and flowfield's counterpart disabled in mod.rs — naive relocation would have moved default-features coverage behind an opt-in gate | Relocation targets redesigned: 7 new ungated top-level files, each mirroring its source module's own lib.rs feature gate |
| D5 | Execution Scope | 🟢 | 🟢 | All edits within tiles_tools + task/ + health.md | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Near-miss caught in-loop: fov/flowfield twins initially both slated for consolidation onto integration counterparts — flowfield's counterpart turned out to be DEAD (disabled module), and consolidating onto it would have silently deleted the crate's only live flowfield coverage | Flowfield trio relocated to a live file instead; only fov's 5 consolidated, onto genuinely LIVE near-verbatim twins; 46+5+5=56, no test deleted, no mocks |
| B2 | Test-First | 🟢 | 🟢 | Relocation task — green before (census) and after (logs) | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — both launches green first try | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Private-access rewrites only onto EXISTING pub accessors (`completed_quests()`, `current_participant()`) — no API widened; 5 exceptions documented rather than exposing fields for zero callers | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0041` (--all-features) exit 0: 5 unit + 46 relocated + 189 integration + 40 doc; log `-0042` (default) exit 0: gates proven both ways (37+5 run; serialization/integration compile to zero; 39 doc) | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Exception comments on all 3 modules name task + rejected alternatives; tests/readme.md documents the two-layer structure; draft 078 records both defect classes of the disabled module | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Moved bodies verbatim with named-transforms-only; dropped dead `use std::fs;` and the commented-out import in the rebuilt flowfield module | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
