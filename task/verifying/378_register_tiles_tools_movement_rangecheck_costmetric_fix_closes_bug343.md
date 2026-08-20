# 378: Register tiles_tools movement range-check cost-metric fix (closes BUG-343)

## Execution State

- **id:** 378
- **title:** Register tiles_tools movement range-check cost-metric fix (closes BUG-343)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 19:47:49
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-343
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:33
- **expires_at:** 2026-08-20 00:45:33
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system
- **verifying_at:** 2026-08-19 22:45:33
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-343 (`task/bug/verified/343_movement_calculate_raw_distance_ignores_cost_policy.md`, Medium
severity, 🎯 Verified) found `module/helper/tiles_tools/src/ecs/systems.rs`'s
`MovementSystem::movement_calculate` gating `movable.range` with two different metrics: a
pre-pathfind check using raw grid distance (`current.distance(target)`), and a post-pathfind
check using the pathfinder's own weighted `cost`. Since `astar` never receives `range` as a
search budget, the raw-distance pre-check was an independent, disagreeing gate that could reject
a target the authoritative cost-based check would have accepted — any caller `cost` policy
cheaper than raw distance (e.g. free/discounted terrain) hit this. The fix — removing the
raw-distance pre-check entirely, leaving reachability decided solely by the pre-existing,
already-correct `cost <= movable.range` post-pathfind check, with a `Fix(BUG-343)`/`Root cause`/
`Pitfall` 3-field source comment in its place — is already applied and independently confirmed
via a new reproducer test (`test_movement_uses_cost_not_raw_distance_for_range_check`,
`tests/integration/ecs_tests.rs`) proving a target with raw distance 10 > range 2 but a `cost`
policy of `|_| 0` now succeeds — the bug file's own VERIFY Gate, 8/8 PASS, 2026-08-18, plus a
full-suite re-run (272/272 tests). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-343.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/ecs/systems.rs` — the already-applied removal of the
  raw-distance pre-check in `MovementSystem::movement_calculate` and its `Fix(BUG-343)`/
  `Root cause`/`Pitfall` source comment — verify present; no further edit expected.
- The already-applied `tests/integration/ecs_tests.rs::test_movement_uses_cost_not_raw_distance_for_range_check`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/343_movement_calculate_raw_distance_ignores_cost_policy.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate.
- Re-running BUG-343's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- Any other function pairing a cheap pre-filter heuristic against a different authoritative
  post-check metric — BUG-343's own Generalized Version section confirmed via a grep sweep of
  `tiles_tools/src/**/*.rs` that `movement_calculate` is the only function with this shape;
  re-confirmed during this task's own filing (see Verification Record).
- `MovementResult::OutOfRange`'s continued existence as a public enum variant — it remains part
  of the crate's API surface (no longer constructed anywhere, but not removed); removing it is
  a separate, unrequested API-breaking change out of scope for this registration task.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix failure (`OutOfRange { requested_distance: 10, maximum_range: 2 }` instead of
  `Success`) via temporary revert-and-rerun — this task does not re-derive that evidence.
- Fix already applied: `systems.rs`'s `movement_calculate` no longer contains a
  `current.distance(target) > movable.range` pre-check; reachability is decided solely by the
  `cost <= movable.range` post-pathfind check, with the required 3-field source comment in
  place of the removed code.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~51s).
- No refactor needed — the fix is a deletion plus a comment, no structural churn.
- Fix documentation already complete at the bug level: BUG-343 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-343`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | `grep -n "distance( target ) > movable\|distance(target) > movable" module/helper/tiles_tools/src/ecs/systems.rs` | Whole-file scan for the removed raw-distance-vs-range conditional (not just the bare `distance(target)` term, which the fix's own explanatory comment still mentions in backticks) | empty (pre-check conditional fully removed) |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `movement_calculate` with `current=(0,0)`, `target=(10,0)`, `range=2`, `cost=\|_\| 0` | fixed reachability gate | `MovementResult::Success`, not `OutOfRange` |

## Acceptance Criteria

- `module/helper/tiles_tools/src/ecs/systems.rs`'s `movement_calculate` contains no
  raw-grid-distance pre-check against `movable.range`; reachability is decided solely by
  `cost <= movable.range`
- The fix's source comment carries all 3 required fields: `Fix(BUG-343)`, `Root cause`,
  `Pitfall`
- `ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check` exists and passes
- No other function in `tiles_tools/src/**/*.rs` pairs a `Distance`-based pre-check against a
  different authoritative post-check metric for the same budget
- `task/bug/verified/343_movement_calculate_raw_distance_ignores_cost_policy.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `movement_calculate` in `systems.rs` no longer contain a
  `current.distance(target) > movable.range` pre-check?
- [ ] C2 — Does the replacement source comment carry `Fix(BUG-343)`, `Root cause`, and
  `Pitfall` fields?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -n "distance( target ) > movable\|distance(target) > movable"
  module/helper/tiles_tools/src/ecs/systems.rs` return empty (the removed pre-check's full
  conditional, not merely the bare `distance(target)` term the fix's own comment still
  mentions in backticks)?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-343`?
- [ ] C7 — Does BUG-343's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/ecs/systems.rs` (the fix content matches what BUG-343's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "OutOfRange {" module/helper/tiles_tools/src/ecs/systems.rs` → 1 (the
  enum variant's own declaration only — it is never constructed/returned anywhere in the file;
  confirmed by reading the single hit's line, which is the `enum MovementResult` definition)
- [ ] M2 — `grep -c "cost <= movable.range" module/helper/tiles_tools/src/ecs/systems.rs` → 2
  (1 real code check at the post-pathfind gate, 1 backticked mention inside the fix's own
  explanatory comment)

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually invokes the real `MovementSystem::movement_calculate`
  with a genuine `cost = |_| 0` closure (not a hardcoded `MovementResult::Success` literal
  standing in for the call) — checked by reading the test body itself, not just its pass/fail
  result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | Adversarial pass re-ran every Verification-section grep live and found 3 defects: (1) T02/C5's `current.distance(target)` search isn't empty — the fix's own explanatory comment still mentions the bare term in backticks — false-positiving against prose, not code; (2) M1's `OutOfRange {` count is 1, not 0 (matches the enum variant's own declaration, which correctly still exists); (3) M2's `cost <= movable.range` count is 2, not 1 (one real code check, one backticked mention in the comment) | Reworded T02/C5 to search for the full removed conditional (`distance(target) > movable`), confirmed empty; corrected M1 to expect 1 (declaration only, with an explanation) and M2 to expect 2 (code + comment mention) |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`module/helper/tiles_tools`); the BUG-343 link-back edit touches a tracking file outside `unit_type: module`'s crate boundary — same disposition as every other bug-promotion cross-link in this repo | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 3 fixed | 3/3 |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0,
~51s. `cargo check -p tiles_tools --all-features` → exit 0, 0 errors, ~24s.
`grep -n "distance( target ) > movable\|distance(target) > movable" src/ecs/systems.rs` →
empty. `grep -c "OutOfRange {"` → 1 (enum declaration, line 157); `grep -c "cost <=
movable.range"` → 2 (code at line 124, comment at line 102).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 19:47:49 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 378 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:33 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:33 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 378` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-343's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/ecs/systems.rs`'s
  `movement_calculate` raw-distance pre-check removed; reachability now decided solely by the
  pre-existing `cost <= movable.range` post-pathfind check) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass caught 3 real defects in the Verification section's own grep expectations:
  T02/C5's bare `current.distance(target)` search false-positived against the fix's own
  explanatory comment (reworded to search the full removed conditional); M1's `OutOfRange {`
  count was claimed 0 but is actually 1 (the enum's own still-present declaration); M2's `cost
  <= movable.range` count was claimed 1 but is actually 2 (code + a comment mention). All three
  corrected and re-verified live. Re-verified T01/T03 live post-fix (`cargo nextest run -p
  tiles_tools --all-features` via `longrun`, exit 0, 272/272 passed; `cargo check -p
  tiles_tools --all-features`, exit 0). `tsk .claim_verify 378` succeeded (❓→🔬, moved to
  `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified during BUG-343's own
  investigation (bug file History, 2026-08-18). This task's own contribution is the formal
  tracking registration and lifecycle walk, not the code change itself. `tsk .verify_pass 378`
  blocked by the same-actor guard (documented above) — task left at 🔬 Verifying per this
  sandbox's standing, previously documented limitation, not a quality defect in this task's own
  content.
| 2026-08-18 19:48:35 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 19:48:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
