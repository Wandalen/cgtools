# 380: Register tiles_tools Animation update zero-frame-count underflow fix (closes BUG-345)

## Execution State

- **id:** 380
- **title:** Register tiles_tools Animation update zero-frame-count underflow fix (closes BUG-345)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 19:55:59
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-345
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:34
- **expires_at:** 2026-08-20 00:45:34
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system
- **verifying_at:** 2026-08-19 22:45:34
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-345 (`task/bug/verified/345_animation_update_zero_frame_count_underflow_panic.md`, Medium
severity, 🎯 Verified) found `module/helper/tiles_tools/src/ecs/components.rs`'s
`Animation::update` non-looping branch computing `self.current_frame = self.frame_count - 1;`
with no guard for `frame_count == 0` — a state trivially reachable via the public API since
every `Animation` field is `pub` and `Animation::new` performs no validation. Calling `update`
on such a zero-frame, non-looping animation underflows the `u32` subtraction and panics. The fix
— switching to `self.frame_count.saturating_sub(1)`, with a `Fix(BUG-345)`/`Root cause`/
`Pitfall` 3-field source comment — is already applied and independently confirmed via a new
reproducer test (`test_animation_zero_frame_count_non_looping_does_not_panic`,
`tests/integration/ecs_tests.rs`) proving `Animation::new(0, 0.1)` with `looping = false` and
`.update(0.2)` no longer panics and leaves `current_frame == 0` — the bug file's own VERIFY
Gate, 8/8 PASS, 2026-08-18 (two independent passes), plus a full-suite re-run (272/272 tests,
re-confirmed live during this task's own filing). The fix is nested in the same `while` loop as
the pre-existing BUG-132 fix but on a distinct line, addressing a distinct failure mode
(confirmed non-colliding in the bug file's own H3/E4). This task performs the remaining
lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task`
(PROC12) — to formally register that already-complete, already-verified fix as a tracked task,
closing BUG-345.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/ecs/components.rs` — the already-applied `Animation::update`
  non-looping-branch `saturating_sub(1)` fix and its `Fix(BUG-345)`/`Root cause`/`Pitfall`
  source comment — verify present; no further edit expected.
- The already-applied `tests/integration/ecs_tests.rs::test_animation_zero_frame_count_non_looping_does_not_panic`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/345_animation_update_zero_frame_count_underflow_panic.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate (two independent passes).
- Re-running BUG-345's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- The pre-existing, unrelated BUG-132 fix in the same `while` loop — confirmed non-colliding
  (different line, different failure mode) by the bug file's own H3/E4; not touched by this fix
  or this task.
- Any other unguarded `some_count - 1` shape in this crate — BUG-345's own Generalized Version
  section confirmed this is the only "last valid frame index" computation in
  `Animation::update`; re-confirmed during this task's own filing (see Verification Record).

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix panic (`attempt to subtract with overflow`) via temporary revert-and-rerun — this
  task does not re-derive that evidence.
- Fix already applied: `components.rs`'s `Animation::update` non-looping branch computes
  `self.frame_count.saturating_sub( 1 )`, with the required 3-field source comment immediately
  above.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~2s warm build).
- No refactor needed — the fix is a single-line arithmetic-operator change plus a comment, no
  structural churn.
- Fix documentation already complete at the bug level: BUG-345 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-345`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | `Animation::new(0, 0.1)` with `looping = false`, `.update(0.2)` | fixed non-looping branch | no panic; `current_frame == 0`, `playing == false` |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -n "self.frame_count = self.frame_count - 1\|self.current_frame = self.frame_count - 1"` | Whole-file scan for the removed unguarded-subtraction *statement* | empty (raw `- 1` statement replaced by `saturating_sub`) |

## Acceptance Criteria

- `module/helper/tiles_tools/src/ecs/components.rs`'s `Animation::update` non-looping branch
  uses `self.frame_count.saturating_sub( 1 )`, not a raw `- 1`
- The fix's source comment carries all 3 required fields: `Fix(BUG-345)`, `Root cause`,
  `Pitfall`
- `ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic` exists and passes
- The pre-existing BUG-132 fix in the same `while` loop remains unmodified
- `task/bug/verified/345_animation_update_zero_frame_count_underflow_panic.md`'s header states
  `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does the non-looping branch of `Animation::update` in `components.rs` compute
  `self.frame_count.saturating_sub( 1 )`?
- [ ] C2 — Does the fix's source comment carry `Fix(BUG-345)`, `Root cause`, and `Pitfall`
  fields?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `ecs_tests::test_animation_zero_frame_count_non_looping_does_not_panic`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -n "self.current_frame = self.frame_count - 1"
  module/helper/tiles_tools/src/ecs/components.rs` return empty?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-345`?
- [ ] C7 — Does BUG-345's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/ecs/components.rs` (the fix content matches what BUG-345's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "self.frame_count.saturating_sub( 1 )"
  module/helper/tiles_tools/src/ecs/components.rs` → 1
- [ ] M2 — `grep -c "self.current_frame = self.frame_count - 1"
  module/helper/tiles_tools/src/ecs/components.rs` → 0

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually constructs a zero-frame-count, non-looping `Animation`
  and calls the real `update` method (not a hardcoded expected-value literal standing in for
  the call) — checked by reading the test body itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass hunted for scope gaps: the task/readme.md Tasks Index row edit (PROC12 Step 9) is not named in In Scope. Cross-checked sibling task 379 — identical omission there too, consistent across every registration task in this batch (Tasks Index bookkeeping is treated as filing-process meta-work, not in-task scope, same as how the backlink is called out separately as a "follow-up edit"). Not a defect — matches established precedent. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass re-ran the Testable line's own command live rather than trusting the recalled figure: `cargo nextest run -p tiles_tools --all-features` via `longrun` → `272 tests run: 272 passed, 0 skipped`, exit 0 — claim holds exactly as stated. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass scanned Delivery Requirements for scope creep beyond the directive — none found; every bullet references either already-completed work or standard lifecycle steps. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass independently re-grepped `components.rs` and re-read the reproducer test body live before trusting the MOST Goal's specific claims: `saturating_sub( 1 )` at line 552, 3-field `Fix(BUG-345)`/`Root cause`/`Pitfall` comment at 539/543/548, test at lines 301-310 constructs a real zero-frame `Animation` and calls the real `update` method — all confirmed accurate on disk right now. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass confirmed `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/tiles_tools` matches the actual crate path and cargo package name (`-p tiles_tools` resolved and ran successfully). | — |
| D6 | Crate Scope Unity | — | 🟢 | Adversarial pass re-checked In Scope/Out of Scope for any second-crate reference (e.g. accidental carry-over from another sibling's template) — none found; single crate (`tiles_tools`) throughout. | — |
| D7 | Crate Locality | — | 🟢 | Adversarial pass confirmed via live `find`/`grep` that `components.rs` physically lives under `module/helper/tiles_tools/src/ecs/` — matches the `unit` field exactly, no path drift. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial pass checked for entanglement with the neighboring BUG-132 fix in the same `while` loop (lines ~517-522) — confirmed distinct lines, distinct failure mode, BUG-132's fix untouched by BUG-345's fix or by this task. | — |
| **Total** | | — | 🟢 | 0 open | — |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0, ~3s
(warm build). `grep -c "self.frame_count.saturating_sub( 1 )" src/ecs/components.rs` → 1.
`grep -c "self.current_frame = self.frame_count - 1"` → 0. `grep -c "self.frame_count =
self.frame_count - 1"` → 0. All Verification-section grep patterns confirmed correct as
originally written — no rewording needed this round (unlike tasks 376/378/379, where the
adversarial pass each found and fixed a comment-false-positive grep defect).

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 19:55:59 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 19:59:43 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:00:18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 380 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 380` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-345's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/ecs/components.rs`'s
  `Animation::update` non-looping branch switched from a raw `- 1` to `frame_count.saturating_sub( 1 )`)
  as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass pre-verified every planned grep/measurement live before writing them into
  the Verification section (rather than after), so all patterns (M1, M2, T04, C5) were already
  correct as filed — no rewording round needed this time, breaking the 3-for-3 defect streak
  from tasks 376/378/379. Adversarial pass also re-ran the full crate test suite live via
  `longrun` (272/272 passed) and cross-checked the D1 Tasks-Index-not-in-In-Scope question
  against sibling task 379, confirming it matches established precedent rather than being a
  gap. `tsk .claim_verify 380` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified (bug file's own
  VERIFY Gate, two independent passes, 2026-08-18) during BUG-345's own investigation. This
  task's own contribution is the formal tracking registration and lifecycle walk, not the code
  change itself. `tsk .verify_pass 380` blocked by the same-actor guard (documented above) —
  task left at 🔬 Verifying per this sandbox's standing, previously documented limitation, not a
  quality defect in this task's own content.
