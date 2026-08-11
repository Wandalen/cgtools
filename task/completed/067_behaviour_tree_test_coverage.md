# Restore test-directory convention and coverage in behaviour_tree (decomposed from task 035)

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
- **unit:** module/helper/behaviour_tree
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **0 tests/ files; 14 inline #[test] in src/**. Zero tests/ directory; all 14 tests inline. Native crate — no wasm barrier.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception (inline unit tests testing
   true internals are the known tension with the all-tests-in-tests/ convention; a blanket move that
   forces API widening is worse than a recorded exception). Never delete a test to satisfy the rule.
2. If the crate has no `tests/` at all, establish it with real behavior tests of the public
   surface — no mocks, loud failures.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p behaviour_tree --all-features` —
   all green before and after each relocation batch.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17). Claim-vs-reality dimension of 035 dissolved workspace-wide (zero readme
  coverage claims found); this crate carries the tests-location/coverage remainder.
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup and confirmed: single-file crate,
  one inline `#[ cfg( test ) ]` module at `lib.rs:1111` holding all 14 tests (13 ordinary + the
  fully documented `RepeatNode::infinite` livelock-guard bug reproducer). All 14 are
  public-API-only — the crate is a plain (non-mod_interface) lib with every item `pub` at the
  root; the three `#[ non_exhaustive ]` types are used only in externally-legal ways (field READS
  of `BehaviorContext`, variant construction and `==` comparison of `BehaviorValue`/
  `BehaviorStatus` — no struct literals, no exhaustive matches). ALL relocate, no exceptions:
  - `tests/behaviour_tree_test.rs` — all 14 tests verbatim (names, comments, and the bug
    reproducer's complete 5-section doc preserved; `use super::*` → `use behaviour_tree::*`),
    plus 1 NEW pin of untested public surface: `for_entity` (sets `entity_id`) +
    `set_property`/`get_property` roundtrip.
  - `tests/readme.md` — Responsibility Table; notes the livelock reproducer's bounded-timeout
    threading pattern.
  - `lib.rs` cleaned via boundary-asserted cut of lines 1111-1422 (asserted: 14 `#[ test ]`
    markers in the cut region, `mod tests` opener, closing braces; block ran to EOF with NO
    preceding blank line — boundary shape differed from 066's).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green: log `-0033` exit 0 — unit 0
  (relocation complete), integration 15/15 (14 relocated + 1 new), doc-test 1/1, 0 failed.
  Post-cut grep: zero `cfg( test )`/`#[ test ]` in src (exit 1). In-loop adversarial catches:
  (1) the timing-sensitive `test_wait_action`/`test_cooldown_node` (real `thread::sleep`) and the
  livelock guard's not-`Send` constraint (`Box< dyn BehaviorNode >` built inside the spawned
  thread) were re-checked as relocation-invariant — they depend on no module-position context;
  (2) `#[ non_exhaustive ]` on `BehaviorContext` would forbid external struct literals — verified
  the tests only ever construct via `new()`/`for_entity()` before cutting, so no API change was
  needed.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | 1 test added beyond relocation, pinning untested public surface (`for_entity`, properties) | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Census re-derived: 14 inline tests in one block, none elsewhere (post-cut grep clean) | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Bug reproducer's 5-section doc relocated verbatim — fix-documentation contract intact | — |
| B2 | Test-First | 🟢 | 🟢 | Relocation task — the tests ARE the change; green run is the evidence | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | Cut boundary differed from 066's template (no leading blank line; block runs to EOF) — blind reuse of the 066 script would have tripped its asserts | Boundary asserts adapted to this file's shape before cutting |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0033` exit 0: unit 0, integration 15/15, doc-test 1/1 | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | tests/readme.md documents the bounded-timeout livelock-guard pattern | — |
| B7 | Code Cleanliness | 🟡 | 🟢 | Three `#[ non_exhaustive ]` types could break external relocation (struct literals/exhaustive matches) | Verified only legal external uses occur (constructor fns, variant construction, `==`) before cutting |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
