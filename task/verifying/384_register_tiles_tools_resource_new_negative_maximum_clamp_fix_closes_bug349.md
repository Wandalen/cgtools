# 384: Register tiles_tools Resource new negative maximum clamp fix (closes BUG-349)

## Execution State

- **id:** 384
- **title:** Register tiles_tools Resource new negative maximum clamp fix (closes BUG-349)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:09:31
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-349
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **started_at:** 2026-08-18 20:10:19
- **expires_at:** 2026-08-18 22:10:19
- **unverified_at:** 2026-08-18 20:10:19
- **unverified_by:** unknown
- **verifying_at:** 2026-08-18 20:10:19
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/

## MOST Goal

BUG-349 (`task/bug/verified/349_resource_new_negative_maximum_panic.md`, Medium severity, 🎯
Verified) found `module/helper/tiles_tools/src/game_systems.rs`'s `Resource::new` and
`Resource::with_regeneration` storing a caller-supplied `maximum` directly with no clamp,
unlike the sibling setter `maximum_set` (which correctly clamps via `value.max(0.0)`). A
`Resource` constructed with a negative `maximum` succeeds silently, then panics on the very
next `modify`/`current_set` call — both call `.clamp(0.0, self.maximum)`, and `f32::clamp`
unconditionally asserts `min <= max`, so `0.0 <= negative_maximum` fails and crashes the
calling thread far from the actual construction site. The fix — clamping `maximum` via
`.max(0.0)` in both constructors, matching `maximum_set`'s existing invariant, with the
required `Fix(BUG-349)`/`Root cause`/`Pitfall` 3-field source comment on `new` (and a shorter
cross-referencing comment on `with_regeneration`) — is already applied and independently
confirmed via a new reproducer test
(`test_resource_new_with_negative_maximum_does_not_panic_on_modify`,
`tests/game_systems_test.rs:250`) proving `Resource::new(-5.0)` followed by `.modify(1.0)` no
longer panics and leaves both `maximum` and `current` non-negative — the bug file's own VERIFY
Gate, 8/8 PASS, 2026-08-18, plus a full-suite re-run (272/272 tests, re-confirmed live during
this task's own filing). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-349.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/game_systems.rs` — the already-applied `Resource::new` /
  `Resource::with_regeneration` `.max(0.0)` clamp fix and its `Fix(BUG-349)`/`Root cause`/
  `Pitfall` source comment — verify present; no further edit expected.
- The already-applied
  `tests/game_systems_test.rs::test_resource_new_with_negative_maximum_does_not_panic_on_modify`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/349_resource_new_negative_maximum_panic.md`'s header back to this
  task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and verified by
  the bug's own VERIFY Gate. Note: unlike sibling bugs 346/347/348, BUG-349's own VERIFY Gate
  ran a single pass, not two, and the bug's own source/test files carry only the
  `Fix(BUG-349)`/`test_kind: bug_reproducer(BUG-349)` markers, not the standalone
  `BUG-349 task/bug/...` backreference comment those siblings' second pass added — a minor
  documentation-completeness gap in the bug's own artifact, not a fix-correctness defect
  (independently confirmed: the clamp logic is live and the reproducer passes). Re-opening
  BUG-349's own VERIFY Gate to add that backreference is out of scope for this registration
  task (see next bullet) and is flagged in this task's own filing report instead.
- Re-running or amending BUG-349's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- `Resource::maximum_set` — confirmed by the bug file's own H1/E1/E2 to already clamp correctly;
  not touched by this fix or this task.
- Raw-struct-literal construction bypassing the constructors — the bug file's own D4 adversarial
  check confirmed this is a pre-existing, shared characteristic of every `pub`-field type in
  this crate, not a gap this fix could or should close; not re-derived here.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix panic (`min > max, or either was NaN. min = 0.0, max = -5.0`) via a permanent
  reproducer test run against the pre-fix source — this task does not re-derive that evidence.
- Fix already applied: `game_systems.rs`'s `Resource::new` and `Resource::with_regeneration`
  both clamp `maximum` via `.max(0.0)`, with the required 3-field source comment on `new`.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~1s warm build).
- No refactor needed — the fix is a single-line clamp added to each of 2 constructors, no
  structural churn.
- Fix documentation already complete at the bug level: BUG-349 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-349`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `game_systems_test::test_resource_new_with_negative_maximum_does_not_panic_on_modify` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | `Resource::new(-5.0)` then `.modify(1.0)` | fixed `new` | no panic; `resource.maximum >= 0.0`, `resource.current >= 0.0` |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -c "let maximum = maximum.max(0.0);"` | clamp present in both constructors | 2 matches |

## Acceptance Criteria

- `module/helper/tiles_tools/src/game_systems.rs`'s `Resource::new` and
  `Resource::with_regeneration` both clamp `maximum` via `.max(0.0)`
- `Resource::new`'s fix carries all 3 required source-comment fields: `Fix(BUG-349)`,
  `Root cause`, `Pitfall`
- `game_systems_test::test_resource_new_with_negative_maximum_does_not_panic_on_modify` exists
  and passes
- `Resource::maximum_set` remains unmodified
- `task/bug/verified/349_resource_new_negative_maximum_panic.md`'s header states
  `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Do both `Resource::new` and `Resource::with_regeneration` in `game_systems.rs`
  compute `let maximum = maximum.max(0.0);` before constructing `Self`?
- [ ] C2 — Does `Resource::new`'s fix source comment carry `Fix(BUG-349)`, `Root cause`, and
  `Pitfall` fields?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `game_systems_test::test_resource_new_with_negative_maximum_does_not_panic_on_modify`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -c "let maximum = maximum.max(0.0);"
  module/helper/tiles_tools/src/game_systems.rs` return exactly 2?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-349`?
- [ ] C7 — Does BUG-349's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/game_systems.rs` (the fix content matches what BUG-349's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "let maximum = maximum.max(0.0);"
  module/helper/tiles_tools/src/game_systems.rs` → 2
- [ ] M2 — `grep -c "Fix(BUG-349)" module/helper/tiles_tools/src/game_systems.rs` → 2

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually constructs `Resource::new(-5.0)` and calls the real
  `.modify(1.0)` method (not a hardcoded expected-value literal standing in for the call) —
  checked by reading the test body itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass re-checked the Tasks Index omission from In Scope (established precedent) and also verified the flagged BUG-349-own-gap note (missing standalone backreference comment, unlike siblings 346-348) is accurately described in Out of Scope, not silently omitted. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass re-ran the Testable line's command live: `cargo nextest run -p tiles_tools --all-features` via `longrun` → `272 tests run: 272 passed, 0 skipped`, exit 0 — claim holds exactly. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass scanned Delivery Requirements for scope creep, including whether this task should itself add the missing backreference comment to `game_systems.rs`/`game_systems_test.rs` — concluded no: that would be editing the bug's own verified fix artifact outside this registration task's remit, correctly left as an Out of Scope note instead. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass independently read both constructors' full live bodies and the reproducer test's full live body — the fix (`let maximum = maximum.max(0.0);` in both `new` and `with_regeneration`) and the test's exact scenario (`Resource::new(-5.0)`, `.modify(1.0)`, both fields asserted `>= 0.0`) both match this task's T02/AF1 claims exactly. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass confirmed `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/tiles_tools` matches the actual crate path and package name (`-p tiles_tools` ran successfully). | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`tiles_tools`) throughout In Scope/Out of Scope — no second-crate reference found. | — |
| D7 | Crate Locality | — | 🟢 | Confirmed via live read that `game_systems.rs` physically lives under `module/helper/tiles_tools/src/` — matches the `unit` field. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Confirmed `Resource::maximum_set` (the already-correct sibling setter) is untouched by this fix (bug file's own H1/E1/E2, re-confirmed by this task's Out of Scope) — no entanglement. | — |
| **Total** | | — | 🟢 | 0 open | — |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0, ~1s
(warm build). `grep -c "let maximum = maximum.max(0.0);" src/game_systems.rs` → 2. `grep -c
"Fix(BUG-349)"` → 2. Reproducer test body (`tests/game_systems_test.rs:250`) read live:
constructs the real `Resource::new(-5.0)`, calls the real `.modify(1.0)`, asserts both
`maximum` and `current` are `>= 0.0` — matches T02/AF1 exactly. Separately confirmed (adversarial
pass, informational only, not a task defect): `grep -n "BUG-349" src/game_systems.rs
tests/game_systems_test.rs` shows only `Fix(BUG-349)`/`test_kind: bug_reproducer(BUG-349)`
markers, no standalone `BUG-349 task/bug/...` backreference comment — unlike siblings
346/347/348, whose own bug files record a second VERIFY pass that added one. This is a
documentation-completeness gap in BUG-349's own artifact, correctly noted in this task's Out
of Scope rather than silently fixed or silently ignored. All Verification-section grep
patterns confirmed correct as originally written — no rewording needed this round.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:09:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | FILED | task created |
| 2026-08-18 20:10:19 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:10:19 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 384 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-349's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/game_systems.rs`'s
  `Resource::new` and `Resource::with_regeneration` now clamp `maximum` via `.max(0.0)`,
  matching `maximum_set`'s existing invariant and fixing a deferred panic on the next
  `modify`/`current_set` call) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass pre-verified every planned grep/measurement live before writing them into
  the Verification section, independently read both constructors' and the reproducer test's
  full live bodies to confirm the MOST Goal/T02/AF1 claims, and noted (informational, not a
  task defect) that BUG-349's own bug file lacks the standalone backreference comment its
  siblings 346-348 added in a second VERIFY pass — correctly left as an Out of Scope note
  rather than fixed by this registration task. Full crate suite re-run live via `longrun`
  (272/272 passed). `tsk .claim_verify 384` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and verified (bug file's own VERIFY Gate,
  2026-08-18) during BUG-349's own investigation. This task's own contribution is the formal
  tracking registration and lifecycle walk, not the code change itself. `tsk .verify_pass 384`
  blocked by the same-actor guard (documented above) — task left at 🔬 Verifying per this
  sandbox's standing, previously documented limitation, not a quality defect in this task's
  own content.
