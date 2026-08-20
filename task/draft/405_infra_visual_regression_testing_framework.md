# 405: infra — visual regression testing framework

## Execution State

- **id:** 405
- **title:** infra — visual regression testing framework
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 22:51:09
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** repository
- **unit:** lib/yrd_gamedev/cgtools
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

**Tracking placeholder — needs scoping before becoming claimable.** `module/helper/tilemap_renderer/
roadmap.md`'s infra section lists visual regression testing as remaining work: verification of rendered
output today happens via ad hoc `browsee` pixel-spot-checks scoped to whichever task is active (e.g.
tasks 191, 218, 539), never as a repeatable golden-image regression suite that would catch an
unintentional pixel-output change in an already-working example/adapter. Too large for one-pass
implementation: needs a golden-image storage strategy (where baseline images live, how they're
regenerated intentionally vs. flagged as a regression), a diff-tolerance policy (exact match is
infeasible across GPU/driver variance), and CI wiring once the golden-image mechanism exists.

## In Scope

- Design a golden-image capture/comparison mechanism (baseline storage location, diff-tolerance policy,
  intentional-update workflow) reusable across the example gallery and adapter test suites.
- Wire at least one pilot crate/example through it end-to-end before generalizing further.

## Out of Scope

- Full-gallery golden-image coverage in one pass — start with a bounded pilot, expand later.
- CI integration — separate sibling draft task (CI feature-matrix coverage); sequence after the
  mechanism itself exists.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped golden-image/diff-tolerance design at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — infra remaining-work section
- `gpu_hal/tests/manual/readme.md` — existing manual browser-verification procedure this would formalize

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: core claim (verification today is ad hoc `browsee` pixel-spot-checks, never a repeatable golden-image suite) is independently plausible and `browsee` itself confirmed to genuinely exist in this environment (`/home/user1/pro/lib/yrd_core/family_dev/default/bin/browsee`, not on default PATH). Of the three cited precedent tasks, 191 and 218 confirmed to exist (`task/verified/191_...`, `task/verifying/218_...`) — **but 539 does not exist** (`tsk .get 539` returns nothing), a citation-accuracy defect matching the pattern found in sibling drafts 404/406. `gpu_hal/tests/manual/readme.md` cross-reference confirmed to exist with real manual-verification content. Flagging the "539" citation for whoever next scopes this task. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
