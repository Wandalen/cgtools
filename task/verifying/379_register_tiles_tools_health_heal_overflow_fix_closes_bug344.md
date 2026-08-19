# 379: Register tiles_tools Health heal overflow fix closes BUG-344

## Execution State

- **id:** 379
- **title:** Register tiles_tools Health heal overflow fix closes BUG-344
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 19:52:14
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-344
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 23:49:13
- **expires_at:** 2026-08-19 01:49:13
- **unverified_at:** 2026-08-18 23:47:42
- **unverified_by:** system
- **verifying_at:** 2026-08-18 23:49:13
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-344 (`task/bug/verified/344_health_heal_overflow_panic.md`, Medium severity, 🎯 Verified)
found `module/helper/tiles_tools/src/ecs/components.rs`'s `Health::heal` computing
`(self.current + amount).min(self.maximum)` — a raw `u32` addition that can overflow before the
`.min()` clamp ever runs, unlike its sibling `Health::damage`, which already uses the
overflow-safe `saturating_sub`. In a debug build this panics (`attempt to add with overflow`);
in a release build it silently wraps `current` to a small value, the exact opposite of `heal`'s
documented effect. The fix — switching the addition to `self.current.saturating_add(amount)`,
matching `damage()`'s existing convention, with a `Fix(BUG-344)`/`Root cause`/`Pitfall` 3-field
source comment — is already applied and independently confirmed via a new reproducer test
(`test_health_heal_saturates_instead_of_overflowing`, `tests/integration/ecs_tests.rs`) proving
`Health { current: u32::MAX - 5, maximum: u32::MAX }.heal(20)` now saturates to `u32::MAX`
instead of panicking — the bug file's own VERIFY Gate, 8/8 PASS, 2026-08-18 (two independent
passes), plus a full-suite re-run (272/272 tests, re-confirmed live during this task's own
filing). This task performs the remaining lifecycle bookkeeping — `tsk.rulebook.md § Core
Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally register that
already-complete, already-verified fix as a tracked task, closing BUG-344.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/ecs/components.rs` — the already-applied `Health::heal`
  `saturating_add` fix and its `Fix(BUG-344)`/`Root cause`/`Pitfall` source comment — verify
  present; no further edit expected.
- The already-applied `tests/integration/ecs_tests.rs::test_health_heal_saturates_instead_of_overflowing`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/344_health_heal_overflow_panic.md`'s header back to this task via
  PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate (two independent passes).
- Re-running BUG-344's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- Any other `(x + y).min(bound)`/`(x - y).max(bound)` unsafe-arithmetic-before-clamp shape in
  this crate — BUG-344's own Generalized Version section confirmed via a grep sweep of
  `src/ecs/components.rs` that `heal` was the only match (`damage` already used the saturating
  form); re-confirmed during this task's own filing (see Verification Record).

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix panic (`attempt to add with overflow`) via temporary revert-and-rerun — this task
  does not re-derive that evidence.
- Fix already applied: `components.rs`'s `Health::heal` computes
  `self.current.saturating_add(amount).min(self.maximum)`, with the required 3-field source
  comment immediately above.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~2s warm build).
- No refactor needed — the fix is a single-method arithmetic-operator change plus a comment, no
  structural churn.
- Fix documentation already complete at the bug level: BUG-344 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-344`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `ecs_tests::test_health_heal_saturates_instead_of_overflowing` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | `Health { current: u32::MAX - 5, maximum: u32::MAX }.heal(20)` | fixed `heal` | `current == u32::MAX`, no panic |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -n "self.current = ( self.current + amount )" module/helper/tiles_tools/src/ecs/components.rs` | Whole-file scan for the removed unchecked-addition *statement* (not the bare `self.current + amount` term, which the fix's own explanatory comment still mentions in backticks) | empty (raw `+` statement replaced by `saturating_add`) |

## Acceptance Criteria

- `module/helper/tiles_tools/src/ecs/components.rs`'s `Health::heal` uses `saturating_add`, not
  a raw `+`, before the `.min(self.maximum)` clamp
- The fix's source comment carries all 3 required fields: `Fix(BUG-344)`, `Root cause`,
  `Pitfall`
- `ecs_tests::test_health_heal_saturates_instead_of_overflowing` exists and passes
- No other `(x + y).min(bound)`/`(x - y).max(bound)` unsafe-arithmetic-before-clamp shape
  remains in `src/ecs/components.rs`
- `task/bug/verified/344_health_heal_overflow_panic.md`'s header states `**Fix Task:**`
  pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `Health::heal` in `components.rs` compute
  `self.current.saturating_add(amount).min(self.maximum)`?
- [ ] C2 — Does the fix's source comment carry `Fix(BUG-344)`, `Root cause`, and `Pitfall`
  fields?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `ecs_tests::test_health_heal_saturates_instead_of_overflowing`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -n "self.current = ( self.current + amount )"
  module/helper/tiles_tools/src/ecs/components.rs` return empty (the removed unchecked-addition
  *statement*, not merely the bare term the fix's own comment still mentions in backticks)?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-344`?
- [ ] C7 — Does BUG-344's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/ecs/components.rs` (the fix content matches what BUG-344's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "saturating_add( amount )" module/helper/tiles_tools/src/ecs/components.rs` → 1
- [ ] M2 — `grep -c "saturating_sub( amount )" module/helper/tiles_tools/src/ecs/components.rs` → 1

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually constructs a `Health` value near `u32::MAX` and calls
  the real `heal` method (not a hardcoded expected-value literal standing in for the call) —
  checked by reading the test body itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | Adversarial pass re-ran T04/C5's grep live: the bare `self.current + amount` search is not empty — it false-positives against the fix's own explanatory comment (`// Fix(BUG-344): \`self.current + amount\` could overflow...`), not real code | Reworded T04/C5 to search for the full removed statement (`self.current = ( self.current + amount )`), confirmed empty; M1/M2 (`saturating_add`/`saturating_sub` counts) verified correct as originally written (1 each) |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`module/helper/tiles_tools`); the BUG-344 link-back edit touches a tracking file outside `unit_type: module`'s crate boundary — same disposition as every other bug-promotion cross-link in this repo | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 fixed | 1/1 |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0, ~2s
(warm build). `grep -n "self.current = ( self.current + amount )" src/ecs/components.rs` →
empty. `grep -c "saturating_add( amount )"` → 1; `grep -c "saturating_sub( amount )"` → 1.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 19:52:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 19:53:02 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 19:53:02 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 379 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-344's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/ecs/components.rs`'s
  `Health::heal` switched from a raw `+` to `saturating_add`, matching the sibling `damage()`'s
  existing `saturating_sub` convention) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass caught the same class of defect as this batch's task 378: T04/C5's bare
  `self.current + amount` grep false-positived against the fix's own explanatory comment —
  reworded to search the full removed statement instead, confirmed empty. M1/M2 verified
  correct as originally written. Re-verified T01/T03 live post-fix (`cargo nextest run -p
  tiles_tools --all-features` via `longrun`, exit 0, 272/272 passed). `tsk .claim_verify 379`
  succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified (two independent
  Tier 2 passes) during BUG-344's own investigation (bug file History, 2026-08-18). This task's
  own contribution is the formal tracking registration and lifecycle walk, not the code change
  itself. `tsk .verify_pass 379` blocked by the same-actor guard (documented above) — task left
  at 🔬 Verifying per this sandbox's standing, previously documented limitation, not a quality
  defect in this task's own content.
