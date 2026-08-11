# Restore test-directory convention in browser_input (decomposed from task 035)

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
- **unit:** module/helper/browser_input
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **1 tests/ file with 7 test markers; 6 inline #[test] in src/**. Split suite: 7 tests in tests/, 6 inline. Unify per the uniform procedure. wasm-leaning crate — confirm native runnability of the inline 6 before relocating.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception. Never delete a test to
   satisfy the rule; never widen an API solely to satisfy it either.
2. Consolidate any inline test that duplicates existing tests/ coverage instead of relocating a
   second copy.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p browser_input --all-features` —
   all green before and after each relocation batch.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17).
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup: all 6 inline tests sit in ONE
  file-level module in `src/input.rs`, pinning `PointerType::from_dom_str` (mouse/touch/pen,
  empty and unrecognised → Unknown) plus the `Unknown` default. Unlike the 074/075 siblings,
  the tested surface is fully public: `PointerType` is a pub enum, `from_dom_str` a pub fn, and
  lib.rs re-exports `input::*` — so **all 6 RELOCATED, zero exceptions, zero consolidations**.
  Placement: new `tests/pointer_type_test.rs` — DOM-string parsing is a distinct domain from
  `active_pointers_test.rs`'s pointer-tracking responsibility (One-Second Test passed). Bodies
  verbatim, dedented one level; sole import transform `use super::PointerType` →
  `use browser_input::PointerType`. Inline module deleted to EOF by an assertion-guarded
  script (cfg-uniqueness, close-brace, 6/6 test-count, blank-separator asserts). Two
  knowledge-accuracy follow-throughs: the `last_pointer_type` doc comment that referenced the
  old "unit tests" updated to name `tests/pointer_type_test.rs`; and `tests/readme.md` created
  (the directory reached 3 entries and had no Responsibility Table — rows for both test files
  + manual/, plus a native-vs-manual routing note).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0046` (`cargo test -p browser_input
  --all-features`) exit 0, 3s: unit suite now 0 tests (module gone), pointer_type_test 6/6
  passed natively (the draft's wasm-runnability concern resolved affirmatively — the pins are
  pure string→enum logic), active_pointers_test 7/7 untouched, doc-tests 0. 6 relocated + 0
  kept + 0 consolidated = 6 — every inline test accounted for.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Census matched at pickup (6/6); draft's native-runnability question answered by the run itself | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | New file justified by distinct domain, not created reflexively — One-Second Test against active_pointers_test.rs passed | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Visibility verified item-by-item (pub enum + pub fn + `pub use input::*`) before classifying — no pattern-matching from the 074/075 keep-inline outcomes | — |
| D5 | Execution Scope | 🟢 | 🟢 | All edits within browser_input + task/ + health.md | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Adversarial pass found tests/ had NO readme.md while gaining its 3rd entry — creating the file without registering responsibilities would have violated the directory protocol | tests/readme.md created with rows for both test files + manual/ |
| B2 | Test-First | 🟢 | 🟢 | Green census before, green run after | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — single launch green first try | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Bodies verbatim; sole transform is the one-line import rewrite | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0046` exit 0: unit 0 (emptied), pointer_type 6/6 native, active_pointers 7/7, doc 0 | — |
| B6 | Knowledge Preservation | 🟡 | 🟢 | Adversarial sweep for references to the old inline tests found the `last_pointer_type` doc comment pointing at "unit tests" — stale after relocation | Doc comment updated to name tests/pointer_type_test.rs |
| B7 | Code Cleanliness | 🟢 | 🟢 | Module deleted to EOF with boundary asserts; no dead imports left (module's `use super::PointerType` went with it) | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
