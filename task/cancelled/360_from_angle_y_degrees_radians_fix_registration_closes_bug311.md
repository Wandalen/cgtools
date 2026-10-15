# Register `from_angle_y` degrees/radians fix (closes BUG-311)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🚫 (Cancelled)
- **closes:** BUG-311
- **unit_type:** repository
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** false
- **verifying_at:** 2026-08-18 16:05:59
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **foreign_ref:** null
- **relocation_reason:** null
- **cancelled_at:** 2026-08-18 19:15:56
- **cancelled_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/

## Goal

BUG-311 (`task/bug/verified/311_from_angle_y_called_with_raw_degrees_not_radians.md`, Medium
severity, 🎯 Verified) found `gl::Quat::from_angle_y( 90.0 )` called with a raw degree literal
(clearly intended as "90 degrees") at 3 byte-identical copy-pasted call sites across 3 sibling
`minwebgl` example crates (`curve_surface_rendering`, `lottie_surface_rendering`,
`animation_surface_rendering`), producing a ~116.62° "clouds" mesh rotation about Y instead of the
intended 90° -- `ndarray_cg::Quat::from_angle_y` takes radians (documented and implemented
correctly; the library API itself is not buggy, confirmed by the bug's own Root Cause H1). The fix
(`from_angle_y( 90.0 )` -> `from_angle_y( 90.0_f32.to_radians() )` at all 3 call sites, each with a
3-field `Fix(BUG-311)`/`Root cause`/`Pitfall` source comment) is already applied, and a new
`ndarray_cg` regression test (`test_from_angle_y_rejects_raw_degrees`, marked
`bug_reproducer(BUG-311)`) locking in the correct/incorrect boundary is already in place -- both
independently re-confirmed live during this task's own filing (2026-08-18: all 3 call sites read
`from_angle_y( 90.0_f32.to_radians() )` with the complete 3-field comment; the regression test
passes: 1 passed, 0 failed) on top of the bug's own VERIFY Gate (8/8 PASS, 2026-08-18). This task
performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core Procedures : Procedure -
Promote Bug to Task` (PROC12) -- to formally register that already-complete, already-verified fix
as a tracked task, closing BUG-311.
Testable: `cargo nextest run -p ndarray_cg -E 'test(test_from_angle_y_rejects_raw_degrees)'
--all-features` -> 1 passed, 0 failed.

## In Scope

- `examples/minwebgl/curve_surface_rendering/src/main.rs:173-179` -- the already-applied
  `from_angle_y( 90.0 )` -> `from_angle_y( 90.0_f32.to_radians() )` fix and its
  `Fix(BUG-311)`/`Root cause`/`Pitfall` source comment (verify present; no further edit expected).
- `examples/minwebgl/lottie_surface_rendering/src/main.rs:177-183` -- same fix, already applied.
- `examples/minwebgl/animation_surface_rendering/src/main.rs:234-240` -- same fix, already applied.
- `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` -- the already-added
  `test_from_angle_y_rejects_raw_degrees` test (marked `bug_reproducer(BUG-311)`); verify present
  and passing.
- Formal task registration and lifecycle walk (claim-verify, attempt `tsk .verify_pass`) for the
  already-complete fix.
- Linking `task/bug/verified/311_from_angle_y_called_with_raw_degrees_not_radians.md`'s header back
  to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to any of the 3 example crates, or to `ndarray_cg::Quat::from_angle_y`
  itself -- the library API is correct and documented as taking radians (BUG-311 Root Cause H1,
  confirmed via E1/E2); not touched by the original fix and not touched by this task.
- Any other `from_angle_[xyz]` call site in the workspace -- BUG-311's own Prevention section's
  repo-wide detection command (`grep -rn "from_angle_[xyz]( *[0-9]" examples/ --include=*.rs`),
  re-run post-fix, confirms only already-correct pre-existing sites remain outside these 3.
- BUG-312 (`character_control`'s own, distinct visible-mesh yaw-halving defect, filed immediately
  after BUG-311 during the same bug-hunt session) -- unrelated root cause per BUG-312's own History
  ("Distinct root cause"); not this task's concern.
- Re-deriving BUG-311's own MRE or re-running its VERIFY Gate -- already complete and recorded in
  the bug file's Verification Record (2026-08-18, 8/8 PASS); this task's own Readiness Verification
  Gate checks task-file quality, not the underlying fix.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-311's own MRE section shows the raw pre-fix
    expression `QuatF64::from_angle_y( 90.0 )` (the exact expression all 3 call sites used pre-fix)
    asserted via `assert_ne!` to diverge from the correct 90-degree rotation -- this task does not
    re-derive that evidence
-   Fix already applied at all 3 example call sites: `from_angle_y( 90.0 )` -> `from_angle_y(
    90.0_f32.to_radians() )`, each with the 3-field `Fix(BUG-311)`/`Root cause`/`Pitfall` source
    comment in place (independently re-confirmed live during this task's own filing, 2026-08-18)
-   Green state already confirmed: `test_from_angle_y_rejects_raw_degrees` passes (re-run live
    during this task's own filing via `longrun`: 1 passed, 0 failed, 36s); full `ndarray_cg` suite
    (282 tests) and clippy (native + wasm32 for the 3 example crates, `-D warnings`) clean per bug
    file History
-   No refactor needed -- each example call site changed only a single literal argument; no
    structural churn
-   Fix documentation already complete at the bug level: BUG-311 carries the 5-section fix
    documentation (Root Cause, Why Not Caught, Fix Location, Prevention) plus the 3-field source
    comment convention (`Fix`/`Root cause`/`Pitfall`) at all 3 call sites -- this task does not
    duplicate it, only cross-links via `closes: BUG-311`
-   Task state reaches 🎯 only if this task file's own Readiness Verification Gate genuinely passes
    all 8 dimensions -- including D6 (Crate Scope Unity): this task's own deliverable spans 4
    distinct Cargo crates (`curve_surface_rendering`, `lottie_surface_rendering`,
    `animation_surface_rendering`, `ndarray_cg`), a real multi-crate condition confirmed by
    `Cargo.toml` inspection, not merely a cross-reference; if D6 genuinely fails, `tsk.rulebook.md §
    Task File : Readiness Verification Gate`'s own D6 FAIL routing (`VERIFY_MIXED`) applies rather
    than being forced or worked around
-   Independent verification (the post-execution acceptance walk) must pass before this task's
    state advances to ✅ -- reaching 🎯 Verified via this task's own Readiness Verification Gate is
    not sufficient by itself for ✅
-   If the task reaches 🎯: `tsk .verify_pass` is then attempted per standard lifecycle (expected to
    hit this sandbox's known same-actor guard, per project convention -- document rather than
    force/spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo nextest run -p ndarray_cg -E 'test(test_from_angle_y_rejects_raw_degrees)' --all-features` | Regression test locking in the correct/incorrect `from_angle_y` boundary | 1 passed, 0 failed |
| T02 | `grep -c 'from_angle_y( 90.0_f32.to_radians() )' examples/minwebgl/curve_surface_rendering/src/main.rs` | Fixed call site (curve) | >=1 |
| T03 | `grep -c 'from_angle_y( 90.0_f32.to_radians() )' examples/minwebgl/lottie_surface_rendering/src/main.rs` | Fixed call site (lottie) | >=1 |
| T04 | `grep -c 'from_angle_y( 90.0_f32.to_radians() )' examples/minwebgl/animation_surface_rendering/src/main.rs` | Fixed call site (animation) | >=1 |
| T05 | `grep -rn "from_angle_[xyz]( *[0-9]" examples/ --include=*.rs` (BUG-311's own Prevention detection command) | Whole-workspace scan for the same bare-degree-literal pattern | Only already-correct pre-existing sites remain (no untouched bare-degree hit at any of the 3 former-bug sites) |
| T06 | `cargo check -p curve_surface_rendering -p lottie_surface_rendering -p animation_surface_rendering --target wasm32-unknown-unknown` | All 3 example crates compile for wasm32 | 0 errors |

## Acceptance Criteria

-   All 3 example call sites state `from_angle_y( 90.0_f32.to_radians() )`, not `from_angle_y(
    90.0 )`
-   All 3 example call sites' source comments carry all 3 required fields: `Fix(BUG-311)`, `Root
    cause`, `Pitfall`
-   `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` contains
    `test_from_angle_y_rejects_raw_degrees` marked `bug_reproducer(BUG-311)`, and it passes
-   `task/bug/verified/311_from_angle_y_called_with_raw_degrees_not_radians.md`'s header states
    `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify -- an independent verifier performs the
walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness (3 example crates)**
- [ ] C1 -- Does `examples/minwebgl/curve_surface_rendering/src/main.rs` state `from_angle_y(
      90.0_f32.to_radians() )` (not `from_angle_y( 90.0 )`)?
- [ ] C2 -- Does that same call site's source comment carry all 3 fields: `Fix(BUG-311)`, `Root
      cause`, `Pitfall`?
- [ ] C3 -- Does `examples/minwebgl/lottie_surface_rendering/src/main.rs` state the same fixed
      call, with the same 3-field comment?
- [ ] C4 -- Does `examples/minwebgl/animation_surface_rendering/src/main.rs` state the same fixed
      call, with the same 3-field comment?

**Regression test (`ndarray_cg`)**
- [ ] C5 -- Does `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` contain
      `test_from_angle_y_rejects_raw_degrees` marked `bug_reproducer(BUG-311)`?
- [ ] C6 -- Does `cargo nextest run -p ndarray_cg -E
      'test(test_from_angle_y_rejects_raw_degrees)' --all-features` pass?

**Registration correctness**
- [ ] C7 -- Does this task's `closes:` field name `BUG-311`?
- [ ] C8 -- Does BUG-311's own header carry a `**Fix Task:**` line pointing back at this task's ID?

**Out of Scope confirmation**
- [ ] C9 -- Is `module/math/ndarray_cg/src/quaternion/arithmetics.rs` (the library API itself)
      untouched by this task (`git diff --stat` empty for that path)?
- [ ] C10 -- Does a repo-wide grep for the bare-degree-literal pattern (`from_angle_[xyz]( *[0-9]`)
      return only already-correct (`.to_radians()`-wrapped or unambiguous `0.0`) sites outside
      these 3, with no untouched bare site introduced by this task?
- [ ] C11 -- Is `task/bug/verified/312_character_control_visible_mesh_yaw_halved_at_call_site.md`
      untouched by this task (`git diff --stat` empty for that path)?

### Measurements

- [ ] M1 -- `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0 ) )"
      examples/minwebgl/curve_surface_rendering/src/main.rs
      examples/minwebgl/lottie_surface_rendering/src/main.rs
      examples/minwebgl/animation_surface_rendering/src/main.rs` -> 0 each (was: 1 each, pre-fix;
      the naive pattern `from_angle_y( 90.0 )` without the `rotation_set(...)` call-site anchor was
      caught during this task's own Dual-Role Self-Check as a false-positive risk -- it also matches
      the `// Fix(BUG-311): \`from_angle_y( 90.0 )\` -> ...` documentation comment's "was" clause,
      not just live code, so a naive count would read 1 (not 0) at every already-fixed site)
- [ ] M2 -- `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0_f32.to_radians() ) )"
      examples/minwebgl/curve_surface_rendering/src/main.rs
      examples/minwebgl/lottie_surface_rendering/src/main.rs
      examples/minwebgl/animation_surface_rendering/src/main.rs` -> 1 each (3 total); same
      call-site-anchored pattern, avoids double-counting the comment's own "->" clause (which also
      contains the radians-wrapped text and would otherwise inflate the naive per-file count to 2)

### Invariants

- [ ] I1 -- `module/math/ndarray_cg/src/` unaffected: `git diff --stat -- module/math/ndarray_cg/src/` -> empty
- [ ] I2 -- workspace still builds: `cargo check -p curve_surface_rendering -p
      lottie_surface_rendering -p animation_surface_rendering --target wasm32-unknown-unknown` -> 0
      errors

### Anti-faking checks

- [ ] AF1 -- the fix changes only the argument expression (`90.0` -> `90.0_f32.to_radians()`), not
      `from_angle_y`'s own definition or signature -- checked by reading
      `module/math/ndarray_cg/src/quaternion/arithmetics.rs`'s literal diff (expected: empty), not
      just the absence of the old call-site value

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 16:05:59 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 19:15:56 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/ | CANCEL | task cancelled |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18 16:00:30]** `FILED` -- Task filed via `bug_promote` skill (PROC12) to formally
  register BUG-311's already-applied, already-verified fix (`from_angle_y( 90.0 )` ->
  `from_angle_y( 90.0_f32.to_radians() )` at 3 sibling example call sites, plus a new `ndarray_cg`
  regression test) as a tracked task, closing the bug. Live re-confirmed immediately before filing:
  all 3 call sites carry the fixed expression and complete 3-field comment;
  `test_from_angle_y_rejects_raw_degrees` passes (1 passed, 0 failed, `longrun` log
  `task/verified/-0030_longrun.log`); `git status --porcelain` on all 5 touched paths (3 example
  files, the `ndarray_cg` test file, the bug file itself) empty (no drift). ID 360 allocated after
  three live TOCTOU collisions with concurrent actors during filing: 356/357 (BUG-312, BUG-298
  promotions), then 358 (BUG-313 promotion), then 359 itself was independently claimed by a
  concurrent actor's `359_minwebgpu_texture_descriptor_default_format_fix_registration.md` in the
  same instant this file was first written there -- caught immediately after the fact (not
  pre-write) by a post-write `find task -iname "359_*"` sanity check, and corrected via `mv` to 360
  (confirmed free by a fresh `find` immediately beforehand) with this History note updated in
  place; no other repo state was touched by the misfire. Each earlier collision was caught by
  re-running the unbounded `find task -type f -name '*.md' | grep -oE '/[0-9]+_' | ...` highest-ID
  scan immediately before write, per this repo's own documented ID-collision history (see
  `task/readme.md`'s own `highest_id` note).
- **[2026-08-18 16:05:59]** `CLAIM` -- `tsk .claim_verify 360` succeeded; ❓ Unverified -> 🔬
  Verifying; `actor` set to `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/` (same identity as
  `filed_by` -- this session's own known same-actor condition, relevant only if D5/D6 had passed
  and `tsk .verify_pass` were later attempted; moot here since D6 fails first).
- **[2026-08-18 16:10:00]** `VERIFY_MIXED` -- Readiness Verification Gate (Tier 2 Dual-Role
  Self-Check, self-administered, Round 1) ran all 8 dimensions honestly. D1/D2/D3/D4/D5/D7/D8 all
  🟢 (both passes agree PASS). D6 (Crate Scope Unity) 🔴 (both passes agree FAIL, clean agreement --
  not a divergent 🟡/🟠): this task's own deliverable genuinely spans 4 distinct, independently
  Cargo.toml'd crates -- `curve_surface_rendering`, `lottie_surface_rendering`,
  `animation_surface_rendering`, `ndarray_cg` -- confirmed via direct `name =` field lookup in each
  crate's own `Cargo.toml`, not a spurious mis-attributed path or a shared/vendored crate-agnostic
  file. Per `tsk.rulebook.md § Task File : Readiness Verification Gate`'s D6 FAIL routing and
  `§ Task Lifecycle : Task State Machine - T2`'s guard (D5 PASS ∧ D6 PASS required for VERIFY_PASS
  or VERIFY_FAIL; D5 FAIL ∨ D6 FAIL forces VERIFY_MIXED ahead of and independent of any other FAIL
  branch), fired `VERIFY_MIXED` -- 🔬 Verifying -> 🌐 Mixed. No `tsk` CLI verb implements this
  transition (confirmed: only `.verify_pass`/`.verify_fail`/`.verify_reject`/`.verify_redraft` exist
  as `.verify_*` subcommands); applied by hand per the rulebook's own Effects column: cleared
  `actor`, `started_at`, `expires_at`, `foreign_ref`, `relocation_reason`; set `in_motion=false`;
  set `State: 🌐 (Mixed)`; moved file `task/verifying/` -> `task/mixed/` (new directory, mirroring
  the pre-existing empty `task/bug/mixed/` sibling -- this is this repo's first task to reach
  🌐 Mixed). Per `§ Task Lifecycle : Verify Mixed`'s own text, this is not a verification failure in
  the VERIFY_FAIL sense -- the task's own scope and quality are otherwise sound (7/8 dimensions
  clean PASS), only its crate/repo placement is wrong -- so no `## Verification Findings` section
  is appended and `round` is not incremented. Resolution requires an admin's `DECOMPOSE_SPLIT` (per
  `tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, PROC17) to split this into
  one per-crate task per affected crate, linked by `blocked_by`/`related_tasks` DAG edges per the
  rulebook's own Cross-Crate Deliverable Note -- explicitly not something this filing agent may
  self-apply ad hoc; PROC17's own Step 4 requires User authorization. BUG-311 itself remains 🎯
  Verified and its fix remains fully applied and passing regardless of this task's own routing --
  only the *registration bookkeeping task* is affected, not the underlying fix.
- **[2026-08-18]** `CANCELLED` — Reason: superseded by `DECOMPOSE_SPLIT` (PROC17,
  `tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`), authorized by the user
  ("Yes, proceed now") in response to this task's own `VERIFY_MIXED`/D6 FAIL routing above, and
  by a second explicit confirmation at PROC17's own Step 4 DAG-edge-proposal gate. Split into 4
  per-crate registration tasks, each independently filed, gated (Tier 2 Dual-Role Self-Check,
  8/8 PASS, no fixes needed), and lifecycle-walked through `tsk .claim_verify` before hitting
  the same documented same-actor `tsk .verify_pass` guard on each:
  [369](../verifying/369_register_curve_surface_rendering_from_angle_y_fix_closes_bug311_split_of_task_360.md)
  (`curve_surface_rendering`),
  [370](../verifying/370_register_lottie_surface_rendering_from_angle_y_fix_closes_bug311_split_of_task_360.md)
  (`lottie_surface_rendering`),
  [371](../verifying/371_register_animation_surface_rendering_from_angle_y_fix_closes_bug311_split_of_task_360.md)
  (`animation_surface_rendering`),
  [372](../verifying/372_register_ndarray_cg_from_angle_y_regression_test_closes_bug311_split_of_task_360.md)
  (`ndarray_cg`). DAG edge set between the 4 siblings: empty (confirmed via direct `Cargo.toml`
  inspection -- no crate declares any of the other 3 as a dependency), so all 4 are independent
  parallel tasks with no `blocked_by` edges between them. BUG-311 itself remains 🎯 Verified;
  only this registration task's own crate-scope routing was ever affected, never the underlying
  fix -- see BUG-311's own header for its updated `**Fix Task:**` backlink, repointed from this
  task to the 4 split tasks above. `tsk .cancel 360` moved this file 🌐 Mixed -> 🚫 Cancelled,
  `task/mixed/` -> `task/cancelled/`.
