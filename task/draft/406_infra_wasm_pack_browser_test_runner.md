# 406: infra — wasm-pack browser test runner

## Execution State

- **id:** 406
- **title:** infra — wasm-pack browser test runner
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

**Tracking placeholder — needs scoping before becoming claimable.** `roadmap.md`'s infra section lists a
wasm-pack (or equivalent headless-browser) test runner as remaining work: browser-side verification today
relies on manual/task-scoped `browsee` sessions (a real-browser pixel-readback tool used per-task, e.g.
tasks 191, 337, 539), not an automated `wasm_bindgen_test`-driven headless suite that could run
unattended in CI. gpu_hal's own docs/layer/002 gap (no automated wasm test suite of any kind for the
WebGL backend) is one concrete beneficiary. Too large for one-pass implementation: needs a
headless-browser driver decision (`wasm-pack test --headless --chrome`/`--firefox` or equivalent) and
per-crate wiring once the runner itself is established.

## In Scope

- Establish a `wasm-pack test --headless` (or equivalent) runner wired into the workspace, proven on at
  least one pilot crate.
- Document the invocation convention so individual crates can adopt it incrementally.

## Out of Scope

- Migrating every crate's browser verification to the new runner in one pass — pilot first, expand later.
- CI wiring — separate sibling draft task (CI feature-matrix coverage), sequence after the runner exists.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped headless-driver approach at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — infra remaining-work section
- `docs/layer/002_l1_gpu_hal.md` — gpu_hal's own "no automated wasm test suite" gap this would help close

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: `docs/layer/002_l1_gpu_hal.md:88` confirmed to literally state gpu_hal "has no automated wasm test suite of any kind" — the concrete-beneficiary claim holds. Of the three cited precedent tasks, 191 confirmed to exist (`task/verified/191_...`) — **but 337 and 539 do not exist** (`tsk .get` on both returns nothing), two citation-accuracy defects in one task, matching the pattern found in sibling drafts 404/405. Flagging both for whoever next scopes this task. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
