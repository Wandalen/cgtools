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

## In Scope

- Root `issues.md`: recover the pre-deletion content via `git show` (16 items, not the stale "8"
  claimed in the original filing) and triage each against current source
- Confirm a traceable destination for all 16 items: 8 already resolved (re-confirmed by grep), 8
  correctly covered by existing umbrella task 038 (no new tasks filed for them)

## Out of Scope

- Filing separate one-off tasks for the 8 still-live marker items — explicitly rejected; task 038
  already claims them as its own subset, and duplicating would violate the no-double-filing principle
- Deleting `issues.md` itself — already done by an unrelated prior commit (`9b71cf39`) before this task
  began; not an action this task performed

## Verification

### Checklist

- [x] C1 — Is root `issues.md` still absent from the working tree? `test -f issues.md` → absent, confirmed.
- [x] C2 — Is the deletion still correctly attributed to the prior, unrelated commit (not this task)? `git log --oneline -- issues.md` → single entry, `9b71cf39` ("feat: add scene script support and comprehensive testing across examples"; commit body explicitly states "Remove legacy issues.md and todo.md in favor of task system").
- [x] C3 — Does the recovered pre-deletion content still contain exactly 16 catalogued items (not the Goal text's stale "8")? `git show 9b71cf39^:issues.md | grep -c "^## Issue:"` → `16`.
- [x] C4 — Are all 8 claimed-resolved items (1-8) still resolved today? Re-ran each item's original grep fresh this session (not reused from the file): markers 1, 2, 4, 5 gone; the `hset` import (3) gone entirely from `iter_test.rs`; `fn draw` (7, 8) — zero matches in `shader.rs`. Item 6's marker is gone as a live comment; it now exists solely as a *quoted* citation inside a `///` fix-doc comment at `mul_test.rs:79,88` ("previously commented out (`qqq : implement try build test throwing error`)..." / "This test replaces the old commented-out attempt...") — matching the file's own description exactly.
- [x] C5 — Are the 8 claimed-still-live items (9-16) still traceable to a real, non-dead-end destination? **Drift found, reported honestly:** as of today all 8 are themselves fully resolved, not merely traceable — re-grepping each of the 6 files (`mingl/{Cargo.toml,src/data_type.rs,src/derive.rs}`, `minwebgl/{Cargo.toml,src/browser.rs,src/geometry.rs}`) for the original marker text returns 0 hits on all 8. Traced the chain: this task named task 038 as the disposition → task 038 (now ✅ Completed) filed the 8 items into per-crate drafts **061** `mingl_marker_resolution` (5 items) and **062** `minwebgl_marker_resolution` (3 items) rather than resolving them itself → **061** and **062** are both now ✅ Completed and have since resolved all 8. This confirms this task's disposition call was correct and has since been fulfilled exactly as anticipated — expected forward progress, not a defect in this task's own 2026-08-10-dated claims.
- [x] C6 — Is task 038 (this task's named destination) still a real, non-cancelled task? At this task's own gate check it was 📝 Draft; per C5 it is now ✅ Completed — never a dead end at any point in between.

### Measurements

- [x] M1 — Catalogued items in the recovered pre-deletion snapshot: `16` (was: `8`, per this task's own Goal-text citation — which this task's History entry itself already identified as stale at filing time).
- [x] M2 — Of the original 16 items, currently-still-live markers: `0` (was: `8` live at this task's 2026-08-10 investigation) — all 8 resolved since, via the chain traced in C5 (034 → 038 → 061/062, all now ✅ Completed).

### Invariants

- [x] I1 — Re-derivation of issues.md's pre-deletion content and item count: `git show 9b71cf39^:issues.md | grep -c "^## Issue:"` → `16`.
- [x] I2 — Chain-of-custody re-check for the 8 still-live-at-filing items' disposition: `grep -H "state:" task/completed/038_workspace_marker_backlog_cleanup.md task/completed/061_mingl_marker_resolution.md task/completed/062_minwebgl_marker_resolution.md` → all 3 show `✅ (Completed)`.

### Anti-faking checks

- [x] AF1 — Guards against a future `issues.md` reappearing with new, unaccounted content being waved through as already-triaged: re-running C1 (`test -f issues.md`) is the trigger — if it ever returns present, this task's "content-preserving obligation discharged" claim no longer covers the new content and a fresh triage is required.
- [x] AF2 — Guards against citing a destination task that turns out to be cancelled/dead with no successor: re-running C5/C6's `state:` grep on the cited destination(s) must never show a cancelled/superseded state with no further successor named — the live chain (038 → 061/062) must always terminate in either an open task or a genuinely resolved one.

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
