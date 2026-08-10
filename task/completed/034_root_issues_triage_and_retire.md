# Triage root issues.md's remaining markers then retire the file

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

Root `issues.md` self-describes its own purpose as "This document lists task comments found in the
codebase, formatted for creating GitHub issues" (confirmed by direct read this session) but catalogs only
8 items — stale against the workspace's real marker backlog (task 038 tracks the ~86 `xxx:`/`qqq:`/
`aaa:`/`TODO:` markers actually present). P6 (retire bucket): for each of the 8 already-catalogued items,
confirm whether it's already resolved, still live, or superseded by one of this triage plan's own new
tasks — file any still-live, not-yet-covered item as its own proper task, then delete `issues.md`
entirely once its content has a traceable destination (this task itself is a Content-Preserving Edit in
spirit — every one of the 8 existing entries needs an accounted-for outcome before the file is removed,
not a silent drop).

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P6 (retire
  issues.md) tier, Delete-candidate bucket (root file, not crate-scoped).

- **[2026-08-10]** `INVESTIGATED_AND_RESOLVED` — Root `issues.md` no longer exists in the working tree;
  confirmed via direct `find`/`git log --oneline -- issues.md`, which shows it was already deleted by
  commit `9b71cf39` ("feat: add scene script support and comprehensive testing across examples", commit
  body: "Remove legacy issues.md and todo.md in favor of task system") — a prior, unrelated commit, not
  an action of this task. This task's own Goal text claims "8" catalogued items; the actual pre-deletion
  content, recovered via `git show 9b71cf39^:issues.md`, contains **16** items — the "8" figure in the
  original filing was itself stale/wrong. Triaged all 16 against current source by grepping each cited
  file/line for its original marker text:

  **8 items RESOLVED** (marker and/or code gone, re-confirmed by direct grep this session):
  1. `module/math/ndarray_cg/tests/inc/d2_test/mod.rs:10` (`// xxx`) — file's marker gone.
  2. `module/math/ndarray_cg/tests/inc/d2_test/access_test/mod.rs:12` (`// qqq : fix tests, please`) —
     marker gone.
  3. `module/math/ndarray_cg/tests/inc/d2_test/access_test/indexing_test/iter_test.rs:3`
     (`use test_tools::hset; // xxx : remove it later`) — import fully removed; file re-read in full
     this session (10 lines), no `hset` reference remains.
  4. `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/add_test.rs:47` (`// xxx`) — marker gone.
  5. `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/mul_test.rs:70` (`// xxx : uncomment`) —
     marker gone.
  6. `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/mul_test.rs:78` (`// // qqq : implement
     try build test throwing error`) — marker gone; **traced to TASK-014**, whose fix replaced the old
     commented-out attempt with a working `should_panic` test — confirmed via a 5-section fix-doc comment
     at the test's current location (`sed -n '55,95p'` this session) explicitly stating "This test
     replaces the old commented-out attempt (`qqq : implement try build test throwing error`)."
  7. `module/min/minwebgl/src/shader.rs:214` (`// xxx : clean`, commented-out `ProgramInterface::draw`) —
     resolved; `grep -n "fn draw"` against the current file (420 lines total) returns zero matches.
  8. `module/min/minwebgl/src/shader.rs:373` (`// xxx : clean`, commented-out `Program::draw`) — same
     resolution as #7, confirmed by the same zero-match grep.

  **8 items STILL LIVE** (marker and code both still present, confirmed by direct grep this session,
  line numbers shifted from the original citations since surrounding code has changed):
  9. `module/min/mingl/Cargo.toml:68` (was line 55) — `# bytemuck = { workspace = true, features =
     [ "derive" ] } # xxx : replace`.
  10. `module/min/mingl/src/data_type.rs:50` (was 46) — `// xxx : usize?`.
  11. `module/min/mingl/src/data_type.rs:70` (was 67) — `// xxx : usize?`.
  12. `module/min/mingl/src/data_type.rs:84` (was 80) — `// xxx : qqq : verify`.
  13. `module/min/mingl/src/derive.rs:12` (unchanged) — `exposed use ::former; // xxx : make it
      unncecessary`.
  14. `module/min/minwebgl/Cargo.toml:77` (was 65) — `# bytemuck = { workspace = true, optional = true,
      features = [ "derive" ] } # xxx : replace`.
  15. `module/min/minwebgl/src/browser.rs:10` (unchanged) — `// xxx : investigate`.
  16. `module/min/minwebgl/src/geometry.rs:79` (was 53) — `// qqq : xxx : move out switch and make it
      working for all types`.

  **Disposition of the 8 still-live items:** not filed as new one-off tasks. Re-confirmed via direct grep
  this session that `task/draft/038_workspace_marker_backlog_cleanup.md` (the umbrella task tracking
  ~86 workspace-wide `xxx:`/`qqq:`/`aaa:`/`TODO:` markers) already states explicitly, in its own filed
  text: "the 8 items already catalogued there [issues.md] are a subset of this marker backlog; reconcile
  the two rather than double-filing the same markers as separate tasks from each." All 8 still-live items
  are exactly this subset — task 038 is their correct, already-anticipated disposition; filing separate
  tasks for them here would violate the No Code Duplication / avoid-double-filing principle and
  contradict task 038's own explicit reconciliation instruction.

  **Content-Preserving obligation satisfied:** all 16 original items now have a traceable destination —
  8 resolved (with #6 traced to a specific fixing task, TASK-014), 8 accounted for under task 038's
  already-planned scope. No item silently dropped. Since `issues.md` itself was already deleted by a
  prior, unrelated commit, no further file deletion is required by this task — its remaining obligation
  (retroactively accounting for the file's content before treating its retirement as final) is discharged
  by this History entry and the Verification Record below.

- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`). Confirming pass re-verified all 16 dispositions above by re-running every
  cited grep fresh in this pass (not reused from the investigation entry above) and got identical results.
  Adversarial pass raised 2 points: (1) is task 038 actually still a live, available destination for the
  8 still-live items, or a dead end? — checked `task/draft/038_workspace_marker_backlog_cleanup.md`'s own
  `state:` field directly: still 📝 Draft, not cancelled/superseded, so the disposition is real, not
  aspirational. (2) this task's own `## Goal` paragraph still asserts "8 items," now known wrong (16) —
  considered editing it, but rejected in favor of leaving the original filing-time text untouched and
  relying on the explicit correction already stated in the `INVESTIGATED_AND_RESOLVED` entry above,
  matching this project's established convention (e.g. task 023) of preserving Goal as a record of intent
  at filing time rather than silently rewriting it once later investigation supersedes it. No Blocking
  Findings surfaced; both adversarial points resolved without requiring a file change beyond what was
  already written. All 8 dimensions PASS; state → ✅ Completed.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass checked whether task 038 (the stated destination for 8 still-live items) is actually still open — confirmed 📝 Draft, not cancelled/superseded | — |
| D2 | MOST Goal Quality | — | 🟢 | Goal text still says "8 items" (now known to be 16) — left as-is, matching this project's convention of preserving Goal as filing-time intent; the correction is explicit in History | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skipped → an orphaned Draft task with a self-contradictory premise (claims 8, actually 16) stays open indefinitely with no accounting; closing it after full retroactive accounting is the concrete value | — |
| D4 | Implementation Readiness | — | 🟢 | No formal Test Matrix (Draft-format, non-code task) — grep-based re-verification of all 16 items serves as the equivalent evidence, executed and cited per item | — |
| D5 | Execution Scope | — | 🟢 | All 16 cited paths plus this task's own file resolve inside this repo | — |
| D6 | Crate Scope Unity | — | 🟢 | Zero code deliverables in any crate — pure read-only cross-crate triage plus a write to this task's own file; `unit_type: workspace` by original design, not a D6 violation | — |
| D7 | Crate Locality | — | 🟢 | N/A — no code/test/doc artifact added to any crate | — |
| D8 | Crate Single Responsibility | — | 🟢 | N/A — no crate modified; workspace-root administrative task | — |
| **Total** | | — | 🟢 | — | — |

**Aggregate verdict:** PASS — zero Blocking Findings on either pass; the adversarial pass's 2 points (task 038's liveness, Goal-text staleness) both resolved without requiring further file changes. D1–D8 are the Readiness Verification Gate dimensions (`tsk.rulebook.md § Task File : Readiness Verification Gate`), reused at completion per this session's established precedent (e.g. task 011, task 023); this is a triage/accounting task with no code changes, so the Bug-Fixing Task Quality Requirements (B1–B7) do not apply.
