# Register character_control's yaw-halving fix (closes BUG-312)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 16:13:19
- **expires_at:** 2026-08-18 18:13:19
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-312
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-18 16:12:25
- **unverified_by:** unknown
- **in_motion:** true
- **verifying_at:** 2026-08-18 16:13:19
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

BUG-312 (`task/bug/verified/312_character_control_visible_mesh_yaw_halved_at_call_site.md`,
Medium severity, 🎯 Verified) found `examples/minwebgl/character_control/src/main.rs` (pre-fix
line 437) passing `character_controls.borrow().yaw() as f32 / 2.0` to `Quat::from_angle_y` when
orienting the visible character mesh -- a spurious `/ 2.0` with no basis anywhere in
`mingl::controls::CharacterControls`, whose own 4 internal call sites (`rotate()`,
`rotation_set()`, `forward_xz()`, `right_xz()`) all pass `self.yaw` to `from_angle_y` unmodified.
Since the camera orbits via the same controller's unhalved `forward()`, the visible mesh's
facing under-rotated to exactly half the camera's own rotation rate while walking with WASD --
after a 180° turn the camera faced fully backward but the mesh had only turned 90°. The fix
(removing the `/ 2.0`, with a `Fix(BUG-312)`/`Root cause`/`Pitfall` 3-field source comment) is
already applied, together with a reproducer test locking in the correct/incorrect boundary
(`module/min/mingl/tests/tests/character_controls.rs::test_yaw_passed_unhalved_to_from_angle_y_matches_rotation`),
independently re-confirmed this filing session via a fresh `cargo nextest` run (1 passed) and a
clean `cargo clippy -p mingl --all-targets --all-features -- -D warnings` (exit 0). This task
performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core Procedures : Procedure -
Promote Bug to Task` (PROC12) -- to formally register that already-complete, already-verified
fix as a tracked task, closing BUG-312.
Testable: `grep -qF 'Quat::from_angle_y( character_controls.borrow().yaw() as f32 )'
examples/minwebgl/character_control/src/main.rs && ! grep -qF 'yaw() as f32 / 2.0'
examples/minwebgl/character_control/src/main.rs && echo PASS || echo FAIL` → PASS.

## In Scope

- `examples/minwebgl/character_control/src/main.rs` (now lines 437-443) -- the already-applied
  removal of the stray `/ 2.0` on the yaw passed to `Quat::from_angle_y`, and its
  `Fix(BUG-312)`/`Root cause`/`Pitfall` source comment (verify both are present; no further edit
  expected).
- `module/min/mingl/tests/tests/character_controls.rs` -- the already-added
  `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation` reproducer (`test_kind:
  bug_reproducer(BUG-312)`), which locks in `CharacterControls`'s own correct (never-buggy)
  contract that the example call site had misused (verify present and passing; no further edit
  expected). Placed in `mingl` rather than `character_control` per BUG-312's own already-verified
  "Why Not Caught" rationale -- `character_control` is a `fn main()`-only WebGL demo binary with
  no test harness of its own, so the reproducer targets the library contract it misused instead
  (this repo's own `rulebook.md` § Test placement).
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/312_character_control_visible_mesh_yaw_halved_at_call_site.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `examples/minwebgl/character_control` or
  `module/min/mingl/src/controls/character_controls.rs` -- the fix is complete;
  `CharacterControls`'s own 4 internal call sites were confirmed correct by BUG-312's own
  investigation (Root Cause H1 confirmed via E1-E4), not touched by the fix.
- Re-running BUG-312's own Readiness Verification Gate -- already run and recorded in the bug
  file's own `## Verification Record` (8/8 PASS, 2026-08-18); not re-litigated by this task's own
  Readiness Verification Gate, which checks task-file quality, not the underlying fix's
  correctness.
- Any other `.yaw() as f32 / <number>` call site -- BUG-312's own Prevention section names the
  repo-wide detection command (`grep -rn "\.yaw()\s*as\s*f32\s*/" examples/ --include=*.rs`) and
  this task's own filing session re-confirmed it returns empty workspace-wide.
- BUG-311 (the sibling `from_angle_y` degrees/radians defect in 3 other example crates) -- a
  distinct root cause (unit-of-measurement confusion, not a spurious scaling factor) requiring
  its own separate promotion, not bundled into this one.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-312's own MRE -- `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation`'s
    `assert_ne!` half locks in exactly the wrong value the pre-fix call site computed
    (`from_angle_y( yaw / 2.0 )`)
-   Fix already applied: `examples/minwebgl/character_control/src/main.rs` states
    `Quat::from_angle_y( character_controls.borrow().yaw() as f32 )` (no `/ 2.0`), with the
    3-field `Fix(BUG-312)`/`Root cause`/`Pitfall` source comment in place
-   Green state already confirmed: this task's own filing session re-ran
    `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation` fresh
    (`cargo nextest run -p mingl -E 'test(...)' --all-features`, exit 0, 1 passed) and
    `cargo clippy -p mingl --all-targets --all-features -- -D warnings` (exit 0)
-   No refactor needed -- single-expression arithmetic simplification (`/ 2.0` removed), no
    structural churn
-   Fix documentation already complete at the bug level: BUG-312 carries the 5-section fix
    documentation (Root Cause, Why Not Caught, Fix Location, Prevention, Pitfall) in its own
    body; this task does not duplicate it, only cross-links via `closes: BUG-312`
-   Test Matrix populated before any test code was written -- satisfied historically: BUG-312's
    own MRE section documents `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation` was
    written before the fix landed (it is the bug's own reproducer); this task adds no new test
    code of its own
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
    before task state is updated to ✅
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate;
    `tsk .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit
    this sandbox's known same-actor guard, per project convention -- document rather than force/
    spoof if so); task state updated to ✅ only on independent verification pass, file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo nextest run -p mingl -E 'test(test_yaw_passed_unhalved_to_from_angle_y_matches_rotation)' --all-features` | Fixed `character_control` call site's regression coverage (test lives in `mingl`, the crate whose contract it protects) | exit 0, 1 passed |
| T02 | `grep -qF 'Quat::from_angle_y( character_controls.borrow().yaw() as f32 )' examples/minwebgl/character_control/src/main.rs` | Fixed call site, unhalved yaw | exit 0 (match found) |
| T03 | `grep -rn '\.yaw()\s*as\s*f32\s*/' examples/ --include=*.rs` (BUG-312's own repeat-defect detector) | Whole-workspace scan for the same spurious-scaling pattern | empty (no other site) |
| T04 | `cargo clippy -p mingl --all-targets --all-features -- -D warnings` | `mingl` crate (the fix's own reproducer test lives here) | exit 0, 0 warnings |

## Acceptance Criteria

-   `examples/minwebgl/character_control/src/main.rs` states
    `Quat::from_angle_y( character_controls.borrow().yaw() as f32 )`, with no `/ 2.0`
-   The same call site's source comment carries all 3 required fields: `Fix(BUG-312)`,
    `Root cause`, `Pitfall`
-   `module/min/mingl/tests/tests/character_controls.rs` contains
    `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation`, marked `test_kind:
    bug_reproducer(BUG-312)`, and it passes
-   No other file under `examples/` reproduces the same `.yaw() as f32 / <N>` spurious-scaling
    pattern
-   `task/bug/verified/312_character_control_visible_mesh_yaw_halved_at_call_site.md`'s header
    states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify -- an independent verifier performs the
walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `examples/minwebgl/character_control/src/main.rs` state `Quat::from_angle_y( character_controls.borrow().yaw() as f32 )` with no `/ 2.0`?
- [ ] C2 — Does the same call site's source comment carry `Fix(BUG-312)`, `Root cause`, and `Pitfall` fields?
- [ ] C3 — Does `module/min/mingl/tests/tests/character_controls.rs` contain `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation`, marked `test_kind: bug_reproducer(BUG-312)`?
- [ ] C4 — Does `cargo nextest run -p mingl -E 'test(test_yaw_passed_unhalved_to_from_angle_y_matches_rotation)' --all-features` pass?
- [ ] C5 — Does a repo-wide grep for the same spurious-scaling pattern (`\.yaw()\s*as\s*f32\s*/`) return empty outside this already-fixed site?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-312`?
- [ ] C7 — Does BUG-312's own header carry a `**Fix Task:**` line pointing back at this task's ID?

**Out of Scope confirmation**
- [ ] C8 — Is `module/min/mingl/src/controls/character_controls.rs` (the library source, not its test file) untouched by this task (`git diff --stat` empty for that path)?
- [ ] C9 — Is BUG-311 absent from this task's own scope (no code change to any of the 3 crates BUG-311 names)?
- [ ] C10 — Is `examples/minwebgl/character_control/src/main.rs`'s change scoped to exactly the documented fix (comment block + one-line arithmetic change), with no other modification in the file? The fix is already committed (`git log --oneline -1 -- examples/minwebgl/character_control/src/main.rs`), so `git diff -- examples/minwebgl/character_control/src/main.rs` is expected empty (clean working tree, not evidence of a missing fix) — scoping is instead confirmed by C1/C2 (grep) matching the committed content exactly.
- [ ] C11 — Is BUG-312's own `## Verification Record` section (`task/bug/verified/312_....md`) unmodified by this task's own filing (the bug's Readiness Verification Gate is not re-run or re-litigated here — only its header's `**Fix Task:**` line is added by PROC12 Step 4)?

### Measurements

- [ ] M1 — `grep -c 'yaw() as f32 / 2.0' examples/minwebgl/character_control/src/main.rs` → 0 (was: 1, pre-fix)
- [ ] M2 — `grep -c 'from_angle_y( character_controls.borrow().yaw() as f32 )' examples/minwebgl/character_control/src/main.rs` → 1

### Invariants

- [ ] I1 — `module/min/mingl/src/controls/character_controls.rs` unaffected: `git diff --stat -- module/min/mingl/src/controls/character_controls.rs` → empty
- [ ] I2 — `mingl` crate still green: `cargo nextest run -p mingl --all-features` → 0 failures
- [ ] I3 — `mingl` crate clippy clean: `cargo clippy -p mingl --all-targets --all-features -- -D warnings` → 0 warnings

### Anti-faking checks

- [ ] AF1 — the fix changes only the halving arithmetic (removes `/ 2.0`), not the angle source or attribute type -- checked by reading the literal diff at the call site, not just the absence of the old expression

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/)

Fresh evidence gathered this round (not carried over from filing): `cargo nextest run -p mingl -E 'test(test_yaw_passed_unhalved_to_from_angle_y_matches_rotation)' --all-features` → 1 passed (via `longrun`, pid 1578287, elapsed 203s); `cargo clippy -p mingl --all-targets --all-features -- -D warnings` → exit 0 (chained after nextest, same launch); `grep -c 'from_angle_y( character_controls.borrow().yaw() as f32 )' .../main.rs` → 1; `grep -c 'yaw() as f32 / 2.0' .../main.rs` → 0; repo-wide `grep -rn '\.yaw()\s*as\s*f32\s*/' examples/ --include=*.rs` → empty; `git diff --stat -- module/min/mingl/src/controls/character_controls.rs` → empty; `git log --oneline -1 -- examples/minwebgl/character_control/src/main.rs` → `254b7812` (fix already committed, working tree clean).

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass questioned whether a pure-registration task has a "meaningful observable outcome" (Scope Sizing Gate); confirmed against 5 identical sibling PROC12 promotions already accepted in this repo's task system (357/358/359/366/368), plus concrete testable outcomes (Fix Task backlink, `closes:` field) | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass checked Goal restates BUG-312's motivation without its own; confirmed lines 34-37 state the registration-specific motivation (closes the bug in the tracking system) distinct from the code fix's own | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass tried "fix already works, why track it" framing; Null Hypothesis holds — skipping leaves BUG-312 Verified-but-never-closed, inconsistent with this repo's now-established promotion convention | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass found C10's wording presumed an uncommitted diff ("shows only that hunk"), but the fix is already committed (`254b7812`) so `git diff` is empty by design, not evidence of absence | Reworded C10 to state the fix is already committed and scoping is confirmed via C1/C2 grep-matching, not a diff hunk |
| D5 | Execution Scope | — | 🟢 | Adversarial pass re-scanned every path in Goal/In Scope/AC; all resolve inside this repo (`examples/...`, `module/min/mingl/...`, `task/bug/verified/312_...`) | — |
| D6 | Crate Scope Unity | — | 🟢 | Adversarial pass found deliverable paths span 2 real Cargo crates (`character_control`, `mingl`) — a genuine structural fact, not spurious mis-attribution. Resolved via D6 = "D5 applied one level deeper": the `mingl` reference is explicitly read-only (`In Scope`: "verify present and passing; no further edit expected"; Delivery Requirements: "this task adds no new test code of its own") — same carve-out D5 grants for read-only cross-repo references, applied one level down to a cross-crate reference. Task's own declared scope broadened to `workspace` to be structurally honest about the 2-crate footprint (matching sibling task 366's precedent for an analogous tension), rather than a `module` unit_type that would falsely imply single-crate containment | Changed `unit_type: module` → `workspace`, `unit` → `lib/yrd_gamedev/cgtools` in Execution State and Tasks Index row |
| D7 | Crate Locality | — | 🟢 | Adversarial pass looked for any new artifact this task adds; Delivery Requirements add zero new code/test/doc artifacts (all pre-existing) — dimension has no object to apply to | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial pass questioned whether the BUG-312 header backlink (PROC12 Step 4) grafts a new concern onto `task/bug/`; confirmed this is `task/bug/`'s own existing, already-stated responsibility (bug-to-task traceability), not a second concern | — |
| **Total** | | — | 🟢 | 2 findings, both fixed in-pass | 2/2 |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 15:53:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created (as ID 356, later renumbered 360→363) |
| 2026-08-18 16:12:25 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 16:13:19 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 16:21:36 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 363` → blocked: "self-verification forbidden (actor matches filed_by)" — same-actor guard, not a defect; state remains 🔬 Verifying |

## History

- **[2026-08-18]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally register
  BUG-312's already-applied, already-verified fix
  (`examples/minwebgl/character_control/src/main.rs`, removed the stray `/ 2.0` on yaw passed to
  `Quat::from_angle_y`) as a tracked task, closing the bug. Filed as ID 356 via `tsk .create`,
  then renumbered 356→360→363 to resolve 3 successive same-day ID collisions with a concurrent
  session actor's own bug-promotion sweep (observed `task/unverified/356_pec_stitch_..._fix_registration.md`
  move to `.../360_pec_stitch_..._fix_registration.md` mid-filing; 357 and 359 also seen
  double-claimed) — see `task/readme.md`'s own "ID-namespace collision" note for the systemic
  root cause (documented prose-only fix landed upstream 2026-08-17; this repo's local `tsk`
  binary still races on allocation).
- **[2026-08-18]** `VERIFY` — Readiness Verification Gate run as Tier 2 Dual-Role Self-Check
  (confirming + adversarial passes) across all 8 dimensions: PASS 8/8. Adversarial pass found and
  fixed 2 in-pass defects: (1) D6 Crate Scope Unity — deliverable genuinely spans 2 Cargo crates
  (`character_control`, `mingl`); resolved by broadening `unit_type: module` → `workspace`
  (`unit` → `lib/yrd_gamedev/cgtools`), matching sibling task 366's precedent for the same
  tension, since the `mingl` reference is read-only (no new artifact added there) — the same
  carve-out D5 grants for read-only cross-repo references, applied one level down; (2) D4
  Implementation Readiness — Checklist item C10 presumed an uncommitted diff, but the fix is
  already committed (`254b7812`), so reworded C10 to rely on C1/C2's grep-matching instead of a
  diff hunk. All Test Matrix rows (T01-T04) and Checklist items independently re-confirmed fresh
  during this Gate run (not carried over from filing), including a live `longrun`-detached
  `cargo nextest`/`cargo clippy -p mingl` run (pid 1578287, exit 0). See task file's own
  `## Verification Record` for the full Gate Table. `tsk .verify_pass 363` then attempted;
  blocked by this sandbox's same-actor guard (`actor` == `filed_by`) — a known, documented,
  non-defect environment limitation (see `## Journal`'s `VERIFY_PASS_ATTEMPTED` entry and task
  254/358's own identical precedent). Task remains at 🔬 Verifying, not force-advanced to 🎯.

## Related Documentation

- `task/bug/verified/312_character_control_visible_mesh_yaw_halved_at_call_site.md` — the source
  bug this task promotes; carries the full Root Cause/MRE/Prevention/History detail this task
  does not duplicate
- `module/min/mingl/src/controls/character_controls.rs` — `CharacterControls`'s own
  rotation-building logic (confirmed correct at all 4 internal call sites, not modified by this
  task)
