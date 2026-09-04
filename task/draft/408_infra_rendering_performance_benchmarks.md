# 408: infra — rendering performance benchmarks

## Execution State

- **id:** 408
- **title:** infra — rendering performance benchmarks
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

**Tracking placeholder — needs scoping before becoming claimable.** `roadmap.md`'s infra section lists
performance benchmarks as remaining work: no `criterion` (or equivalent) benchmark harness exists for
hot rendering paths (batch submission, pass orchestration, tessellation once path rendering lands) —
performance regressions in these paths would go undetected until someone notices a subjective slowdown.
Too large for one-pass implementation: needs a benchmark-target selection (which hot paths matter most)
and a baseline/regression-threshold policy before any harness lands.

## In Scope

- Select a bounded initial set of hot rendering paths worth benchmarking (e.g. `tilemap_renderer`'s
  batch submission, `renderer`'s pass orchestration).
- Establish a `criterion` (or equivalent) benchmark harness for that initial set.

## Out of Scope

- Comprehensive benchmark coverage of every rendering path in one pass — start with the highest-value
  subset.
- CI-gated performance regression enforcement — a natural follow-up once baselines exist, not part of
  establishing the harness itself.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped benchmark-target selection at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — infra remaining-work section

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: claim (no criterion/equivalent harness exists for hot **rendering** paths — `tilemap_renderer`'s batch submission, `renderer`'s pass orchestration) confirmed accurate — neither crate has any bench-named file/directory or a `criterion` Cargo.toml dependency. **Adversarial check**: `module/helper/tiles_tools` *does* have a real criterion setup (`Cargo.toml:101`, `benches/pathfinding_benchmarks.rs`, `benches/coordinate_benchmarks.rs`) — but that crate is pathfinding/coordinate-math, not a rendering path, so it does not contradict this task's specifically-scoped claim. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
