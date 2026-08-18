# 372: Register ndarray_cg from_angle_y regression test (closes BUG-311, split of task 360)

## Execution State

- **id:** 372
- **title:** Register ndarray_cg from_angle_y regression test (closes BUG-311, split of task 360)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **closes:** BUG-311
- **filed:** 2026-08-18 17:48:35
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module (`scope` from the crate dir returns `SCOPE_LEVEL=package`, not in tsk.rulebook.md's 5-value enum `yard|repository|workspace|module|dir` -- mapped to the closest valid variant, a single crate/package within a workspace)
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 17:49:32
- **expires_at:** 2026-08-18 19:49:32
- **related_tasks:** 369 (curve_surface_rendering), 370 (lottie_surface_rendering), 371 (animation_surface_rendering) -- split siblings of cancelled task 360; supersedes task 360's portion for this crate
- **unverified_at:** 2026-08-18 17:49:26
- **unverified_by:** unknown
- **verifying_at:** 2026-08-18 17:49:32
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

Register `ndarray_cg`'s already-added, already-passing `test_from_angle_y_rejects_raw_degrees`
regression test -- the library-side reproducer locking in the correct/incorrect boundary that
BUG-311's 3 example call sites crossed (`task/bug/verified/311_from_angle_y_called_with_raw_degrees_not_radians.md`,
Medium severity, 🎯 Verified) -- as a tracked, crate-scoped task. **Motivated** by BUG-311 and by
task 360's own D6 (Crate Scope Unity) FAIL, which found the original multi-crate registration
task illegitimately spanned 4 crates and required an admin `DECOMPOSE_SPLIT` (PROC17) into one
task per crate -- this is that split's `ndarray_cg` slice, the only one of the 4 that adds a new
artifact (a test) rather than registering an example-crate call-site fix. **Observable**:
`module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs:175-177` carries `// test_kind:
bug_reproducer(BUG-311)` immediately above `fn test_from_angle_y_rejects_raw_degrees()` (verified
present, live, 2026-08-18). **Scoped**: exactly one crate, one new test function -- the library
API itself (`Quat::from_angle_y`) is unmodified. **Testable**: `cargo nextest run -p ndarray_cg -E
'test(test_from_angle_y_rejects_raw_degrees)' --all-features` -> 1 passed, 0 failed.

## In Scope

- `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` -- the already-added
  `test_from_angle_y_rejects_raw_degrees` test (marked `bug_reproducer(BUG-311)`); verify present
  and passing.
- Formal task registration and lifecycle walk (submit, claim-verify, attempt `tsk .verify_pass`)
  for this crate's already-complete regression test.

## Out of Scope

- `curve_surface_rendering`, `lottie_surface_rendering`, `animation_surface_rendering` (own
  sibling split tasks, each registering that crate's own call-site fix -- each crate's task
  registry is self-contained per `tsk.rulebook.md`'s Cross-Crate Deliverable Note; no dependency
  edge to any of the 3, since none of them declares `ndarray_cg` directly in its own `Cargo.toml`
  -- confirmed via direct inspection, not inferred).
- Any code change to `module/math/ndarray_cg/src/quaternion/arithmetics.rs` (the library API
  itself) -- `from_angle_y` is correct and documented as taking radians (BUG-311 Root Cause H1);
  not touched by the original fix and not touched by this task.
- BUG-312 (`character_control`'s own, distinct visible-mesh yaw-halving defect) -- unrelated root
  cause; not this task's concern.
- Re-deriving BUG-311's own MRE or re-running its VERIFY Gate -- already complete and recorded in
  the bug file's Verification Record (2026-08-18, 8/8 PASS).

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Test already added: `test_from_angle_y_rejects_raw_degrees`, marked `bug_reproducer(BUG-311)`,
  asserting `QuatF64::from_angle_y( 90.0_f64.to_radians() )` matches the closed-form 90-degree-
  about-Y quaternion, and separately that the raw literal `90.0` does NOT produce that same
  quaternion.
- Green state already confirmed: `cargo nextest run -p ndarray_cg -E
  'test(test_from_angle_y_rejects_raw_degrees)' --all-features` -> 1 passed, 0 failed (re-run live
  during this task's own filing via `longrun`, exit 0).
- No refactor needed -- one new test function added to an existing test file; the library API
  itself is unmodified.
- Fix documentation already complete at the bug level: BUG-311 carries the 5-section fix
  documentation plus the 3-field source comment convention at the 3 example call sites -- this
  task does not duplicate it, only cross-links via `closes: BUG-311`.
- Task state reaches 🎯 only if this task file's own Readiness Verification Gate genuinely passes
  all 8 dimensions -- D6 (Crate Scope Unity) is expected to PASS this time (exactly one crate,
  `ndarray_cg`, confirmed via `Cargo.toml` inspection), unlike source task 360.
- Independent verification (the post-execution acceptance walk) must pass before this task's
  state advances to ✅ -- reaching 🎯 Verified via this task's own Readiness Verification Gate is
  not sufficient by itself for ✅.
- If the task reaches 🎯: `tsk .verify_pass` is then attempted per standard lifecycle (expected to
  hit this sandbox's known same-actor guard, per project convention and per 7 other sibling
  registration tasks currently at 🔬 Verifying for the same reason -- document rather than
  force/spoof if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo nextest run -p ndarray_cg -E 'test(test_from_angle_y_rejects_raw_degrees)' --all-features` | Regression test locking in the correct/incorrect `from_angle_y` boundary | 1 passed, 0 failed |
| T02 | `grep -c "test_kind: bug_reproducer(BUG-311)" module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` | Bug-reproducer marker present | >=1 |
| T03 | `git diff --stat -- module/math/ndarray_cg/src/` | Library API itself untouched by this task | empty |

## Acceptance Criteria

- `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` contains
  `test_from_angle_y_rejects_raw_degrees` marked `bug_reproducer(BUG-311)`, and it passes.
- `module/math/ndarray_cg/src/quaternion/arithmetics.rs` (the library API itself) is untouched by
  this task.
- This task's `closes:` field names `BUG-311`.
- Every Test Matrix row passes.

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify -- an independent verifier performs the
walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

- [ ] C1 -- Does `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` contain
      `test_from_angle_y_rejects_raw_degrees` marked `bug_reproducer(BUG-311)`?
- [ ] C2 -- Does `cargo nextest run -p ndarray_cg -E
      'test(test_from_angle_y_rejects_raw_degrees)' --all-features` pass?
- [ ] C3 -- Does this task's `closes:` field name `BUG-311`?
- [ ] C4 -- Is `module/math/ndarray_cg/src/quaternion/arithmetics.rs` untouched by this task
      (`git diff --stat` empty for that path)?
- [ ] C5 -- Are the 3 example crates (`curve_surface_rendering`, `lottie_surface_rendering`,
      `animation_surface_rendering`) untouched by this task (`git diff --stat` empty for all 3
      paths)?

### Invariants

- [ ] I1 -- `module/math/ndarray_cg/src/` unaffected: `git diff --stat -- module/math/ndarray_cg/src/` -> empty.

### Anti-faking checks

- [ ] AF1 -- the new test exercises `from_angle_y` as a black box (via its public API), not by
      reimplementing or bypassing its half-angle formula -- checked by reading the test's own
      assertions, not just its pass/fail result.

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`ndarray_cg`, confirmed via `Cargo.toml` `name =` field) — the whole point of this split from task 360. None of the 3 example crates declares `ndarray_cg` directly in their own `Cargo.toml` (checked directly, not inferred), so no manifest-declared dependency edge exists to any of them | — |
| D7 | Crate Locality | — | 🟢 | Adversarial pass specifically re-checked this: the test targets the crate that actually owns `Quat::from_angle_y` (`ndarray_cg`), not one of the 3 untestable example binaries that merely call it — matches task 360's own Fix Location rationale ("added to the existing crate rather than to the 3 untestable example binaries") verbatim; this is the canonical correct-locality case, not a borderline one | — |
| D8 | Crate Single Responsibility | — | 🟢 | Zero code change to `ndarray_cg`'s own library API (only a new test function); crate's responsibility ("quaternion/vector math library") unaffected | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced live during this gate:** `cargo nextest run -p ndarray_cg -E
'test(test_from_angle_y_rejects_raw_degrees)' --all-features` (via `longrun`, 2026-08-18
17:46:16) → 1 passed, 0 failed. `grep -c "test_kind: bug_reproducer(BUG-311)"
module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` → 1.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 17:48:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 17:49:26 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 17:49:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 17:50 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 372 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC17 (`tsk.rulebook.md § Core Procedures :
  Procedure - Decompose by Crate`) to formally register task 360's `ndarray_cg` slice of
  BUG-311 -- the already-added, already-passing `test_from_angle_y_rejects_raw_degrees`
  regression test (`tests/inc/quat_test/arithmetic.rs:175-177`, marked
  `bug_reproducer(BUG-311)`) -- as a tracked, single-crate task -- one of 4 siblings
  (369/370/371/372) splitting task 360 after its own D6 (Crate Scope Unity) FAIL, per the
  user's explicit "Yes, proceed now" authorization to run DECOMPOSE_SPLIT. Unlike its 3
  siblings, this slice registers a library-level test, not an example call-site fix -- the
  library API (`Quat::from_angle_y`) itself is untouched.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS, no
  fixes needed. Adversarial pass specifically re-checked D7 (Crate Locality): confirmed the
  test targets the crate that actually owns `Quat::from_angle_y` (`ndarray_cg`), not one of
  the 3 untestable example binaries that merely call it -- matching task 360's own Fix
  Location rationale verbatim. Re-verified live: `cargo nextest run -p ndarray_cg -E
  'test(test_from_angle_y_rejects_raw_degrees)' --all-features` (via `longrun`, exit 0, 1
  passed, 0 failed); `grep -c "test_kind: bug_reproducer(BUG-311)"
  module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` → 1. `tsk .claim_verify 372`
  succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described test
  (`module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs:175-177`,
  `test_from_angle_y_rejects_raw_degrees`, marked `bug_reproducer(BUG-311)`) already existed
  on disk, added prior to this task's filing, and the library API itself
  (`module/math/ndarray_cg/src/quaternion/arithmetics.rs`) remains untouched. This task's own
  contribution is the formal per-crate tracking registration and lifecycle walk, not the test
  addition itself. `tsk .verify_pass 372` blocked by the same-actor guard (documented above) —
  task left at 🔬 Verifying per this sandbox's standing, previously-documented limitation
  (same guard that blocked task 254 and task 358's own `.verify_pass`), not a quality defect
  in this task's own content.
