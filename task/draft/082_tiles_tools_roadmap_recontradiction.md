# Re-fix tiles_tools/roadmap.md's 4 remaining ECS-movement self-contradiction sites

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

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
