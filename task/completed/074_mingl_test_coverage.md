# Restore test-directory convention in mingl (decomposed from task 035)

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
- **unit:** module/min/mingl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **5 tests/ files with 34 test markers; 13 inline #[test] in src/**. 13 inline tests in an otherwise convention-following crate. Coordinate with draft 061 (marker resolution in the same crate); data_type.rs:84's 'verify' marker wants a test — land it in tests/ as part of this work if 061 hasn't yet.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception. Never delete a test to
   satisfy the rule; never widen an API solely to satisfy it either.
2. Consolidate any inline test that duplicates existing tests/ coverage instead of relocating a
   second copy.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p mingl --all-features` —
   all green before and after each relocation batch.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17).
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup: all 13 inline tests sit in ONE
  module — `src/web/file.rs` — and every one targets a PRIVATE function: `resolve_url` (9 tests
  pinning the URL-resolution table of `load`'s contract) and `data_url_base64_payload` (4 tests
  pinning data-URL validation). Neither is in the file's `mod_interface` block (only `load`,
  `is_self_contained_url`, and `Error` are exported), and the module structure is deliberate:
  these two helpers ARE the natively-testable pure logic extracted from the wasm-only async
  `load` (browser `window`/fetch/atob). Expose-or-exception decision: **all 13 KEPT INLINE as
  one documented exception** — exporting the helpers would widen the public API solely for test
  placement (zero non-test callers; `load`'s doc contract already documents the rules the tests
  pin), and testing through `load` natively is impossible. Exception rationale comment added to
  the module naming task 074 + both rejected alternatives. Zero relocations, zero
  consolidations, no API widened, no test deleted. Draft's conditional side-item verified
  already satisfied: the `data_type.rs` 'verify' marker is gone and `tests/tests/data_type_test.rs`
  (4 tests) landed via task 061 — nothing left to land here.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0044` (`cargo test -p mingl
  --all-features`) exit 0, 2s: unit suite 13/13 PASSED NATIVELY — the load-bearing fact for the
  exception: the `web` feature compiles on native (wasm-bindgen/web-sys are type-only there)
  and the pinned helpers are pure, so the kept-inline tests remain live coverage under the
  workspace's canonical verification, not dormant wasm-only code; tests/ harness 38/38
  (bounding_box 9, bounding_sphere 2, camera_orbit_controls 22, data_type 4, nd 1); doc-tests
  0 passed / 10 ignored (unchanged). 0 relocated + 13 kept + 0 consolidated = 13 — every
  inline test accounted for. tests/readme.md already exists and needed no new rows (no files
  added).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Draft's 061-coordination item verified moot — marker resolved and data_type test landed by 061 | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Census matched at pickup (13/13, single module) | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | No speculative exports — helpers stay private with zero non-test callers | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Sibling-crate pattern-matching would have relocated all 13 (the 073 outcome); adversarial pass on the mod_interface block found BOTH tested fns private and the extraction deliberate (pure logic split out of wasm-only `load`) | Disposition flipped to keep-inline-as-exception before any file was created |
| D5 | Execution Scope | 🟢 | 🟢 | One src comment + task/ + health.md | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Expose-or-exception decided per procedure; no test deleted, no API widened, no mocks | — |
| B2 | Test-First | 🟢 | 🟢 | Comment-only change to test module surroundings — behavior identical | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — single launch green first try | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Exception comment names task 074 + both rejected alternatives (export-for-tests; native testing through `load`) | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0044` exit 0: unit 13/13 runs NATIVELY under --all-features (proves kept tests are live canonical coverage, not wasm-dormant), tests/ 38/38, doc 0+10 ignored | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Rationale comment documents WHY inline placement is correct here; History records the native-run proof | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No files created, nothing deleted — minimal diff (one comment block) | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 15/15 |
