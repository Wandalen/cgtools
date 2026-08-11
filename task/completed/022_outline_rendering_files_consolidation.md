# Consolidate near-identical outline-rendering files (crate TBD — needs re-scoping)

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

The workspace audit flagged a pair (or set) of near-identical outline-rendering source files as
consolidation candidates (P3, dead-code/hygiene bucket, Fix-in-place) — likely somewhere under
`module/helper/line_tools` or `module/helper/renderer` given those crates own outline/line-rendering
responsibility, but **the exact files and crate were not preserved precisely through this session's
context compaction and must be re-derived from scratch at pickup** (re-run a workspace-wide search for
near-duplicate rendering/outline source files, e.g. by comparing file sizes and diffing candidates, rather
than trusting this citation). Note: `module/helper/line_tools/src/d2/line.rs` and `d3/line.rs` show
uncommitted local modifications as of this session's start (per initial `git status`) — check whether
those in-flight edits already address this finding before starting new work here.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code /
  consolidation) tier, Fix-in-place bucket. Flagged low-confidence: citation not preserved through
  compaction, needs full re-derivation at pickup.
- **[2026-08-10]** `IMPLEMENTED` — Re-derivation swept every outline-named source file workspace-wide
  and diffed all candidate pairs. Rejected with evidence: example `outline/outline.frag` vs
  `narrow_outline/outline.frag` (248 normalized diff lines — different algorithms: JFA+equirect vs
  depth/normal+ID-list), renderer `narrow_outline.rs` vs `normal_depth_outline.rs` (100 diff lines),
  `line_tools` `d2/line.rs` vs `d3/line.rs` (811 diff lines; the draft's in-flight-edit concern also
  cleared — git status on line_tools is now clean). The real near-identical set: the JFA algorithm
  shaders duplicated between the renderer's productized wide-outline pass and the self-contained outline
  example — `jfa_init.frag` (20 diff lines) and `jfa_step.frag` (56), naming-only differences (`vUv` vs
  `v_tex_coord` etc.); the third file (`outline.frag` compositing pass, 181 diff lines) diverged
  legitimately when the example gained skybox/reflections. Consolidation options rejected by name:
  cross-crate `include_str!` (brittle workspace-relative path, violates crate locality), deleting the
  example (contradicts task 024's settled triage; unique pedagogical content), shared shader-resource
  infrastructure (YAGNI for two small files). Chosen fix per the Intentional Duplication Only rule:
  mirrored cross-reference notes in all four files declaring the duplication intentional (example =
  self-contained minwebgl JFA walkthrough; renderer = production integration) and instructing JFA-core
  bug fixes be mirrored. Notes placed AFTER `#version 300 es` (WebGL2 requires the version directive on
  line 1). Verified: all four files still start with `#version 300 es`; `cargo check -p renderer` exit 0.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) the first candidate pair's opening diff hunk was comment-only, suggesting
  near-identity — full normalized diff proved them different algorithms, preventing misidentification;
  (2) the naive fix (header comment at top of file) would have silently broken all four shaders at
  runtime — WebGL2 rejects `#version` not on line 1 — caught before editing, notes placed after the
  directive with a post-edit first-line assertion.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's full re-derivation done: all outline candidates diffed, real pair isolated with line counts | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | Shared-resource infra rejected as YAGNI; smallest fix that makes the duplication declared and navigable | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | 4 shader files touched, comment-only edits; no functional change | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Cross-crate finding handled as filed (workspace unit) | — |
| D7 | Crate Locality | 🟢 | 🟢 | Rejected cross-crate include_str! precisely to preserve locality | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Intentional Duplication Only rule satisfied: aim declared, siblings cross-referenced | — |
| B2 | Test-First | 🟡 | 🟢 | Naive top-of-file comment would break WebGL2 (#version must be line 1) — constraint identified before editing | Notes inserted after #version; first-line assertion ran post-edit |
| B3 | Evidence of Failure | 🟢 | 🟢 | Duplication proven: jfa_init 20 / jfa_step 56 diff lines (naming-only) vs 100-811 for rejected pairs | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Consolidation alternatives rejected by name with reasons, not silently skipped | — |
| B5 | Fix Verification | 🟡 | 🟢 | First candidate pair looked near-identical from opening comment-only hunk | Normalized full diff (248 lines) disproved it; real pair confirmed instead |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Notes live in the shaders themselves — visible exactly where a future editor would introduce drift | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | cargo check -p renderer exit 0; all four files verified #version-first | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved | 2/2 |
