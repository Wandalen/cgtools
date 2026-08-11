# Close workspace-wide test coverage gaps (needs decomposition at pickup)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

The audit identified three distinct test-coverage problems spanning many crates (P7 — test coverage
tier): (1) crates with zero test coverage at all; (2) crates whose tests violate the "all tests in
`tests/` directories" convention by testing public API inline within `src/`; (3) crates whose docs claim a
level of test coverage the actual test suite doesn't support. **This is explicitly a tracking/umbrella
Draft, not directly actionable as filed** — this session's context compaction lost the specific per-crate
breakdown originally produced by the audit subagents. At pickup: re-run a workspace-wide sweep (per-crate
`find tests/`, `grep -rn "#\[test\]" src/`, and a diff of each crate's coverage claims against its actual
tests) to rebuild the per-crate list, then decompose into one task per affected crate per Crate Scope
Unity (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, PROC17) rather than trying to
force this single Draft through full `File Task` as one multi-crate task.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P7 (test coverage)
  tier, Fix-in-place bucket. Flagged as needing decomposition — do not promote to full task as-is.
- **[2026-08-10]** `IMPLEMENTED` — Executed exactly as the Goal prescribed for an umbrella: rebuilt
  the lost per-crate breakdown, then decomposed. Census (30 module/ crates, per-crate `find tests/`
  + `#[ test ]` grep in both trees):

  **Dimension (3) — claims vs reality: DISSOLVED.** Zero readme coverage claims remain anywhere in
  module/ (grep for coverage-claim phrasings returned empty) — this session's earlier false-claims
  rewrites (tasks 024, 030 et al.) already retired the fiction the audit saw. Nothing to file.

  **Dimension (1) — zero-coverage crates:** 5 functional crates have no tests/ at all:
  embroidery_tools (8 inline), behaviour_tree (14 inline), canvas_renderer (1 inline), minwebgl
  (4 inline), minwgpu (21 inline). Exempt-documented: the 8 blank/ placeholders (4 with empty
  doc-stub test files, 4 with none — consistent with 038's blank-template disposition) and the 2
  alias crates, which both run donor suites by path-include (ndarray_tools' include was enabled by
  task 038 — 257 tests; browser_tools includes browser_log's suite).

  **Dimension (2) — inline tests in src/:** 11 crates carry 247 inline `#[ test ]` fns total:
  tilemap_renderer 80, tiles_tools 56, tilemap_scene 38, minwgpu 21, behaviour_tree 14, mingl 13,
  embroidery_tools 8, renderer 6, browser_input 6, minwebgl 4, canvas_renderer 1.

  **Key triage finding (changed the successor tasks' shape):** inline tests are inline largely
  BECAUSE they need private access — canvas_renderer's single inline test, inspected directly, is a
  fully documented 5-section bug reproducer importing `super::private::*` to test an internal
  function; blind relocation would force publishing internals. Every successor draft therefore
  mandates a per-test expose-or-exception DECISION, never mechanical moves, and never deleting or
  API-widening solely to satisfy the convention.

  **Decomposed into 12 per-crate drafts (066-077, per the Goal's own PROC17 mandate):** 066
  embroidery_tools, 067 behaviour_tree, 068 canvas_renderer (decision-shaped: the private-access
  reproducer), 069 minwebgl (runnability-story-first: wasm crate), 070 minwgpu, 071
  tilemap_renderer, 072 tiles_tools, 073 tilemap_scene (each coordinating with their 038-stream
  marker drafts where both touch the same crate), 074 mingl, 075 renderer (pre-existing
  working-tree mods flagged), 076 browser_input, 077 browser_log (absorbs the 2 `cover by test`
  markers task 038 routed into this stream; carries the browser_tools include coupling).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) the confirming pass nominated canvas_renderer's 1 inline test as a
  trivial relocation instance to execute immediately; the adversarial pass read the test and found
  it requires private access by design — executing the "trivial" move would have published an
  internal or broken a documented bug reproducer, so the instance became the decision-shaped draft
  068 instead; (2) the census's raw marker counts initially treated alias/blank stub test files as
  zero-coverage findings — inspection showed browser_tools' stub is a live path-include of
  browser_log's suite and the blank stubs are placeholder scaffolds, both excluded with rationale.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's umbrella contract executed as written: census rebuilt, PROC17 decomposition, no forced single-task promotion | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | Claims dimension dissolved with evidence (empty grep), not filed as busy-work; blank/alias crates exempted with rationale | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | "Trivial instance" plan (relocate canvas_renderer's 1 test) was wrong — test needs private access by design | Instance converted to decision-shaped draft 068; uniform procedure gained the expose-or-exception mandate |
| D5 | Execution Scope | 🟢 | 🟢 | Zero code edits — census reads + task/ writes only | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | One draft per crate, 12 crates, per the Goal's own PROC17 citation | — |
| D7 | Crate Locality | 🟢 | 🟢 | Each draft's work lands inside its own crate; cross-crate couplings (browser_tools include, 038-stream overlaps) named explicitly | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | All-tests-in-tests/ convention applied as a decision framework, not a blind rule — documented-exception path preserved | — |
| B2 | Test-First | 🟢 | 🟢 | Census is empirical (find + grep per crate, both trees); stub files read before classification | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Counts on record per crate: 5 zero-coverage functional crates, 247 inline tests across 11 crates | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | Blind relocation would break private-access tests or force API widening | Per-test expose-or-exception decision mandated in every successor draft |
| B5 | Fix Verification | 🟢 | 🟢 | Every successor carries its own longrun test-verification step; nothing claimed fixed here | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Full census + triage rationale in History; per-crate specifics embedded in each draft | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No code touched; no temp files left unhyphenated | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved | 2/2 |
