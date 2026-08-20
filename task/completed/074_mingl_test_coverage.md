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
- **unit_type:** module
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

## In Scope

- `module/min/mingl`, `src/web/file.rs`: triage of 13 inline `#[test]` functions, all kept
  inline as one documented exception pinning private helpers `resolve_url` (9 tests) and
  `data_url_base64_payload` (4 tests)

## Out of Scope

- Exposing `resolve_url`/`data_url_base64_payload` to the public API solely for test
  placement — rejected (zero non-test callers; native testing through `load` is impossible,
  it's wasm-only)
- The `data_type.rs:84` "verify" marker test — already landed by task 061, nothing left to
  land here

## Verification

### Checklist

- [x] C1 — Do all 13 inline `#[ test ]` functions in `module/min/mingl/src/` sit in exactly one file, `web/file.rs`, as the census claimed? `grep -rln "#\[ *test *\]" src/` → only `src/web/file.rs`; `grep -c "#\[ *test *\]" src/web/file.rs` → `13`.
- [x] C2 — Are `resolve_url` and `data_url_base64_payload` (the two private helpers the kept-inline tests pin) absent from `web/file.rs`'s `mod_interface!` export block — confirming the expose-or-exception decision was "keep private", not "widen API"? Confirmed: the block only exports `load`, `is_self_contained_url`, `Error`.
- [x] C3 — Does `web/file.rs` carry the documented exception-rationale comment naming this task and both rejected alternatives? Confirmed at lines 196-204: `// Exception ( task 074 ) : ...` ending `// Rejected alternatives : exposing the helpers ( zero non-test callers ), or testing through `load` itself ( impossible natively -- browser-only APIs ).`
- [x] C4 — Does `tests/readme.md` list all 5 `tests/tests/*.rs` files with a Responsibility Table row each? Confirmed: 5 rows — `bounding_box.rs`, `bounding_sphere.rs`, `camera_orbit_controls.rs`, `data_type_test.rs`, `nd_test.rs`.
- [x] C5 — Was the draft's coordination item — the `data_type.rs:84` "verify" marker's test — actually already satisfied by task 061 before this task needed to land it? Confirmed: `tests/tests/data_type_test.rs` exists with 4 tests (independently re-verified under task 061's own Verification section); this task added no separate test file.
- [x] C6 — Do the 13 kept-inline tests actually execute under the workspace's canonical (native) verification command, proving they are live coverage rather than dormant wasm-only code — the load-bearing fact for the keep-inline exception? `cargo nextest run -p mingl --all-features` → all 13 `web::file::private::tests::*` entries run and PASS natively (see I1).

### Measurements

- [x] M1 — Inline `#[ test ]` count in `src/web/file.rs`: current `13` (was: `13`, `git show 25ceae76:module/min/mingl/src/web/file.rs | grep -c "#\[ *test *\]"`) — unchanged, confirming the claimed "zero relocations".
- [x] M2 — `"Exception ( task 074 )"` rationale-comment occurrences in `src/web/file.rs`: current `1` (was: `0`, `git show 25ceae76:module/min/mingl/src/web/file.rs | grep -c "Exception ( task 074 )"`).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p mingl --all-features` → exit 0, 51/51 passed (13 inline + 38 integration — matches this task's own claimed 13/38 split exactly).
- [ ] I2 — Compiler/lints clean (crate-scoped): `cargo clippy -p mingl --all-targets --all-features -- -D warnings` → exit 101, NOT clean. Root cause fully isolated to a different, workspace-local crate: `module/helper/browser_log/src/panic.rs:82`'s `#[ allow( clippy::exhaustive_structs ) ]` lacks a `reason = ".."`, tripping the workspace's `allow_attributes_without_reason = "warn"` lint (escalated to a hard error by `-D warnings`). `browser_log` is pulled in only transitively, via mingl's optional `web_log` feature; the build aborts there before mingl's own source is ever clippy-checked. `git log -1 --format="%h %ad %s" --date=iso -- module/helper/browser_log/src/panic.rs` → commit `5f33be66`, dated 2026-08-11 (today) — lands after this task's 2026-08-10 completion and is unrelated to the test-placement work this task performed (a single comment block added to `web/file.rs`). (Independently corroborated: a concurrent sibling verification of the unrelated `primitive_generation` crate hit the identical `browser_log:82` failure in the same session.)

### Anti-faking checks

- [x] AF1 — Guards against a future contributor "fixing" the convention by relocating these tests and silently widening the API to make relocation possible: re-running C2's `mod_interface!` export-list check must keep showing `resolve_url`/`data_url_base64_payload` absent; if a future change exports either, the exception comment (C3) must be removed/updated in the same change, not left stale and contradicted.
- [x] AF2 — Guards against the kept-inline exception being cited as blanket precedent for other, non-exceptional inline tests elsewhere in the crate: C1's `grep -rln "#\[ *test *\]" src/` must keep returning only `src/web/file.rs` — any second file appearing there is a new instance requiring its own independent expose-or-exception decision, not an automatic pass under this task's exception.

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
