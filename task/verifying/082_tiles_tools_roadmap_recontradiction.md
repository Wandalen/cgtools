# Re-fix tiles_tools/roadmap.md's 4 remaining ECS-movement self-contradiction sites

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **started_at:** 2026-08-11 17:37:12
- **expires_at:** 2026-08-11 19:37:12
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-11 17:35:44
- **unverified_by:** unknown
- **in_motion:** true
- **verifying_at:** 2026-08-11 17:37:12
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/

## Goal

`module/helper/tiles_tools/roadmap.md` is internally self-contradictory about ECS-movement gap
status. Task 063 implemented `World::request_movement` (retiring `docs/pitfall/002`, the "ECS
movement is a no-op" pitfall) and correctly updated the Known Gaps table (lines 77-85, confirmed
fresh this session: lists only 3 gaps — `pitfall/001`, `003`, `004` — `pitfall/002` absent) and the
Phase 3 detail paragraph (line 208, confirmed fresh: "implemented by task 063; formerly the
layer's one no-op call, tracked as `pitfall/002` until retired"). But 4 other locations in the same
file still describe ECS movement as an open gap, confirmed fresh this session:

- Line 5 (`**Status:**`): "Phase 3 substantially complete (**1 known gap**)" — should read 0 gaps
  for Phase 3 now that ECS movement is fixed (Flow Fields, the other gap, belongs to a different
  phase per the Known Gaps table).
- Line 19 (`**Current Priority:**`): "the two documented functional gaps (Flow Fields, **ECS
  movement resolution**)" — should name only Flow Fields.
- Line 24 (Ready-to-Code item 3): "Read Known Gaps before touching Flow Fields or **ECS
  movement** — both already have a documented pitfall" — should name only Flow Fields.
- Line 249 (Next Priority Actions item 2): "**Close `docs/pitfall/002`**" as an open action item —
  `pitfall/002` is already deleted (confirmed: absent from `docs/pitfall/`), this action item is
  fully stale and should be removed or replaced.

**Root cause:** not a mechanical/generated-content refactor — confirmed fresh this session via
`git show 5f33be66 --stat` and `git show cd98503d --stat`, both zero hits for `roadmap.md`. Traces
instead to task 063's own edit not being propagated to every mention it claimed to update (task
063's own § Verification C7: "PARTIALLY — ... 4 other roadmap.md locations still describe ECS
movement/`pitfall/002` as an open gap"; task 025's § Verification C1 independently confirmed and
enumerated the same 4 exact locations with the same line numbers).

**Related Tasks:** `025` (`task/completed/025_tiles_tools_roadmap_rewrite.md`) — its own C1
verification finding first enumerated these exact 4 locations but left them unfixed (Verification
findings document drift, they don't self-remediate). `063`
(`task/completed/063_tiles_tools_marker_resolution.md`) — the task whose incomplete propagation
caused the contradiction; its own C7 finding independently confirms the same 4 locations.

## In Scope

-   `module/helper/tiles_tools/roadmap.md` — rewording every site that still described ECS movement
    or `docs/pitfall/002` as an open/unresolved gap, to match the file's own already-correct Known
    Gaps table (lines 77-85) and Phase 3 detail paragraph (line 208): originally-enumerated lines 5,
    19, 24, 249, plus any further same-class site an exhaustive `grep` sweep for the stale patterns
    (`pitfall/002`, `ECS movement`, `two documented`, `Four functional`, `[Ss]ubstantially
    [Cc]omplete`) turns up before editing

## Out of Scope

-   `docs/pitfall/002` itself — already deleted by task 063; this task only removes stale
    documentation *references* to it, never touches `docs/pitfall/`
-   Any other crate's `roadmap.md`/`readme.md` — single-file, single-crate scope
-   Any behavioral/source-code change — this is a documentation-text-only correction; no `src/`,
    `tests/`, or `Cargo.toml` file is touched
-   Lines whose "substantially complete" / `pitfall/002` mention is accurate and unrelated to the
    ECS-movement contradiction (e.g. Phase 4's own genuinely-partial status, Phase 6's own header,
    the dated Revision History entry, the forward-looking Target statement) — these are not
    self-contradictory and must not be reworded just because a blunt grep pattern also matches them

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Doc-only correction — no test code applies; Test Matrix is not populated (non-code task)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution` before
    task state is updated to ✅
-   Task state updated to ✅ only upon verification pass; file moved to `task/completed/`

## Acceptance Criteria

-   `module/helper/tiles_tools/roadmap.md` line 5 (`**Status:**`) no longer lists ECS movement as a
    counted gap for Phase 3
-   `module/helper/tiles_tools/roadmap.md` line 19 (`**Current Priority:**`) names only Flow Fields
    as the documented functional gap, not "the two documented functional gaps"
-   `module/helper/tiles_tools/roadmap.md` line 24 (Ready-to-Code item 3) names only Flow Fields,
    not "Flow Fields or ECS movement"
-   `module/helper/tiles_tools/roadmap.md`'s Next Priority Actions list (originally around line 249)
    contains no "Close `docs/pitfall/002`" item
-   `module/helper/tiles_tools/roadmap.md`'s Known Gaps section states "Three functional gaps," not
    "Four"
-   `module/helper/tiles_tools/roadmap.md`'s component-status table shows the ECS Integration row as
    unqualified "✅ Complete" with no dangling `docs/pitfall/002` link
-   `git diff --stat -- module/helper/tiles_tools/` touches only `roadmap.md` — no `src/`, `tests/`,
    or `docs/pitfall/` change

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**roadmap.md content**
- [ ] C1 — Is line 5's Status line free of any ECS-movement/`pitfall/002` gap count?
- [ ] C2 — Does line 19's Current Priority name only Flow Fields as the functional gap?
- [ ] C3 — Does line 24's Ready-to-Code item 3 name only Flow Fields?
- [ ] C4 — Is the "Close `docs/pitfall/002`" Next Priority Actions item absent?
- [ ] C5 — Does the Known Gaps section say "Three functional gaps"?
- [ ] C6 — Does the component-status table's ECS Integration row read unqualified "✅ Complete"
      with no `pitfall/002` link?

**Out of Scope confirmation**
- [ ] C7 — Is `docs/pitfall/002` itself still absent from `docs/pitfall/` (untouched by this task)?
- [ ] C8 — Does `git diff --stat -- module/helper/tiles_tools/` touch only `roadmap.md`?
- [ ] C9 — Are the deliberately-unchanged legitimate mentions (Phase 3 retrospective, Phase 6
      header, Revision History entry, Phase 4's own partial-status line, the Target statement)
      still present and unreworded — confirming this task didn't over-correct into deleting
      accurate content?

### Measurements

- [ ] M1 — `grep -n 'pitfall/002\|ECS movement\|two documented\|Four functional\|[Ss]ubstantially
      [Cc]omplete' module/helper/tiles_tools/roadmap.md` — every returned line is independently
      confirmed non-contradictory (was: 4+ lines described ECS movement/`pitfall/002` as an open
      gap; matches remaining after the fix are all legitimate — cite each by line number and why)

### Invariants

- [ ] I1 — `git diff --stat -- module/helper/tiles_tools/roadmap.md` (against the commit that
      introduced the fix) shows a real, non-empty diff touching only this file
- [ ] I2 — No other file under `module/helper/tiles_tools/` is touched by that same diff

### Anti-faking checks

- [ ] AF1 — The fix isn't achieved by deleting the Known Gaps table or Phase 3 paragraph outright
      (which would trivially stop them "contradicting" anything) — both sections must still exist
      and still correctly describe Flow Fields as the one remaining Phase-4-adjacent functional gap
- [ ] AF2 — M1's remaining grep matches are individually justified in the Measurement's own citation,
      not waved off with a bare "false positives" claim

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 17:35:44 | unknown | SUBMIT | structural completeness gate passed |

## History

- **[2026-08-11]** `EXECUTED` — All 4 enumerated sites fixed, plus 3 more same-class sites found by
  an exhaustive stale-pattern sweep (`grep -n 'pitfall/002\|ECS movement\|two documented\|Four
  functional\|[Ss]ubstantially [Cc]omplete' roadmap.md`) before editing:
  - **Enumerated:** line 5 Status → "Phases 1, 2, 3, 6 complete" (Phase 3 parenthetical dropped);
    line 19 → "the documented functional gap (Flow Fields)"; line 24 → "before touching Flow
    Fields — it already has a documented pitfall"; line 249 item 2 ("Close `docs/pitfall/002`")
    deleted, items 3-5 renumbered 2-4.
  - **Additional same-class sites:** line 63 component-status table row still read "⚠️
    Substantially complete — movement resolution is a no-op" and linked the *deleted*
    `docs/pitfall/002` → now "✅ Complete" with the dangling doc link dropped; line 79 "Four
    functional gaps" → "Three" (table has 3 rows); line 202 Phase 3 header "⚠️ Substantially
    Complete" → "✅ Complete"; line 301 Current State → "Phases 1, 2, 3, and 6 complete; Phase 4
    substantially complete with documented gaps".
  - **Deliberately untouched:** line 208 (correct retrospective: "tracked as `pitfall/002` until
    retired"); line 295 Revision History (dated 2026-08-10 entry — accurate at its date,
    append-only record); line 302 Target statement (end-state goal vocabulary, not an open-gap
    claim); line 234 Phase 6 header (different phase, unrelated).
  - **Verification:** re-grep of all stale patterns leaves exactly the 3 deliberate keeps (208,
    295, 302); both status summaries now carry the "1, 2, 3, 6 complete" form (2 matches). Doc-only
    change — no compile involvement. Awaits independent verification/promotion per the task
    lifecycle.
- **[2026-08-11]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) during this session's TA106
  out-of-scope-findings triage. Classified via `tsk.rulebook.md § Task File : Deduplication Search`
  as Case E (closed tasks 025 and 063 both name this exact regression in their own Verification
  sections, but neither's scope covers actually applying the fix; this task's scope — editing the 4
  stale lines — is a distinct, not-yet-tracked unit of work). Cross-linked to both 025 and 063 via
  `**Related Tasks:**`. Confirmed via `grep -rl "roadmap.md" task/draft/ task/bug/` that no other
  tracker already covers this.
| 2026-08-11 17:37:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | CLAIM_VERIFY | verification claimed |

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by
user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | Deliverable was already applied and committed (`96bb2aef`) before this task's own Draft filing; this task's remaining value is formal Readiness+Acceptance verification/closure of already-landed work, not new execution — a legitimate, previously-established pattern this session (tasks 085/087/090), not a YAGNI violation. Flagged for the eventual Acceptance Verifier: confirm execution-actor ≠ verification-actor per `§ Acceptance Verification : Procedure - Execution`'s independence requirement. | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 non-blocking | — |

Confirming pass (per dimension): D1 — In Scope/Out of Scope both non-empty, meaningful observable
outcome (reword N self-contradictory doc sites), Scope Sizing Gate passes. D2 — Motivated (misleading
docs steer contributors to re-investigate an already-resolved gap or misjudge Current
Priority/Ready-to-Code), Observable (exact file+line+text named), Scoped (one file), Testable (exact
grep command + git diff-stat check stated). D3 — Null Hypothesis answered (docs stay contradictory,
risk of wasted contributor effort); concrete already-manifested defect, not speculative. D4 —
Delivery Requirements concrete for a doc-only task; Test Matrix correctly omitted (non-code); all 7
Acceptance Criteria bullets name exact file+line+expected content. D5 — every path
(`module/helper/tiles_tools/roadmap.md`) resolves inside this repo; `repo_identity: self` correct.
D6 — every deliverable path resolves inside the single `tiles_tools` crate. D7 — `roadmap.md` is
already crate-local documentation in its own leaf crate; no aggregator-crate concern. D8 — the fix
only corrects existing status text, grafts no new concern onto the crate's one-sentence
responsibility.

Adversarial pass (per dimension, genuine attempt to disprove — not a restatement): D1 — checked
whether the "any further same-class site an exhaustive grep sweep... turns up" In-Scope clause makes
scope unbounded; the grep patterns are tied to one already-known defect class in one file, and remaining
matches were independently re-confirmed (below) to be exactly the 5 expected legitimate lines — bounded
in practice, not open-ended. D2 — checked whether "why this matters now" is genuine or cosmetic;
Current Priority/Ready-to-Code directly drive contributor next-actions, so misleading text there has
real behavioral consequence, not mere prose grooming. D3 — the sharpest attack: since the fix is
already committed, does this task have zero remaining value? Rejected — the task-lifecycle's own
Delivery Requirements mandate independent verification before ✅; an applied-but-unverified fix is
exactly the gap this task closes, which is real (not speculative) value; recorded as a Non-Blocking
Issue for the next gate rather than silently dropped. D4 — did not trust the task's own prose;
independently re-ran the cited checks against the live file this round:
`grep -n 'functional gap\|ECS Integration\|✅ Complete\|Known Gaps' module/helper/tiles_tools/roadmap.md`
confirmed AC bullets 2/3/5/6 verbatim (line 19 names only Flow Fields; line 24 names only Flow
Fields; line 79 "Three functional gaps"; line 63 ECS Integration row "✅ Complete" with
`docs/api/001`/`docs/type/002` links, no `pitfall/002`); `sed -n '1,6p'` confirmed AC bullet 1 (line 5
Status has no ECS-movement gap count); `sed` over the Next Priority Actions block confirmed AC bullet
4 (no "Close `docs/pitfall/002`" item — list is `pitfall/001`, Phase 5, region analysis,
`pitfall/003`+`004`); `git diff --stat -- module/helper/tiles_tools/` returned empty, confirming no
stray uncommitted change beyond the already-committed fix. No discrepancy found between the task's
claims and the live file. D5/D6 — re-scanned Goal/In Scope/Out of Scope/Acceptance Criteria for any
foreign-repo or foreign-crate path; the only path outside `tiles_tools` proper
(`docs/pitfall/002`) appears solely in Out of Scope as an explicitly-excluded, untouched reference,
not a deliverable path — does not break D5/D6. D7/D8 — checked whether `tiles_tools` could be an
aggregator crate whose docs should live elsewhere; it is a concrete leaf ECS-engine-helper crate
throughout this session's own prior work (scene_script/tiles_tools examples reorg), so no relocation
concern.
