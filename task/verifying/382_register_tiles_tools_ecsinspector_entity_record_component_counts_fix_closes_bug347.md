# 382: Register tiles_tools ECSInspector entity_record component_counts fix (closes BUG-347)

## Execution State

- **id:** 382
- **title:** Register tiles_tools ECSInspector entity_record component_counts fix (closes BUG-347)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:05:28
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-347
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:34
- **expires_at:** 2026-08-20 00:45:34
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system
- **verifying_at:** 2026-08-19 22:45:34
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-347 (`task/bug/verified/347_ecs_inspector_entity_record_inflates_component_counts.md`,
Medium severity, 🎯 Verified) found `module/helper/tiles_tools/src/debug.rs`'s
`ECSInspector::entity_record` incrementing `component_counts` unconditionally on every call,
with no matching decrement when an already-recorded `entity.id` is re-recorded —
`entity_data` self-corrects via `HashMap::insert` overwrite, but `component_counts` is purely
additive, so it silently and permanently inflates every time a live debug overlay refreshes an
entity's state (the intended, documented usage — there is no `entity_remove`/`unrecord`
workaround). The fix — decrementing the previous entry's per-component contributions (via
`saturating_sub(1)`, removing the key entirely at 0) before applying the new entity's counts,
with the required `Fix(BUG-347)`/`Root cause`/`Pitfall` 3-field source comment plus a
`BUG-347 task/bug/...` backreference — is already applied and independently confirmed via a
new reproducer test (`test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts`,
`tests/debug_test.rs:201`) proving that re-recording the same entity with a changed component
set leaves `component_counts` reflecting only the current state, not a sum across both calls —
the bug file's own VERIFY Gate, 8/8 PASS, 2026-08-18 (two independent passes, the second one
adding the missing backreference comment), plus a full-suite re-run (272/272 tests,
re-confirmed live during this task's own filing). This task performs the remaining lifecycle
bookkeeping — `tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) —
to formally register that already-complete, already-verified fix as a tracked task, closing
BUG-347.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/debug.rs` — the already-applied `ECSInspector::entity_record`
  decrement-previous-contribution fix and its `Fix(BUG-347)`/`Root cause`/`Pitfall` source
  comment plus `BUG-347` backreference — verify present; no further edit expected.
- The already-applied
  `tests/debug_test.rs::test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/347_ecs_inspector_entity_record_inflates_component_counts.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate (two independent passes).
- Re-running BUG-347's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- `entity_data`'s own overwrite-on-insert behavior — confirmed correct by the bug file's own
  H2/E2 (never accumulates stale entries); not touched by this fix or this task.
- `system_time_record` / `system_timing_record` — the bug file's own Generalized Version
  section confirmed this sibling `record`-named method appends to a `Vec` rather than
  maintaining a derived counter, so it does not share this failure mode; not re-derived here.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix overcount (`Position: 2 entities` after only 1 entity ever recorded) via a
  permanent reproducer test run against the pre-fix source — this task does not re-derive that
  evidence.
- Fix already applied: `debug.rs`'s `ECSInspector::entity_record` decrements the previous
  entry's component contributions (if `entity.id` was already recorded) before applying the new
  entity's counts, with the required 3-field source comment plus backreference.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~1s warm build).
- No refactor needed — the fix is contained entirely within `entity_record`'s own body, no
  signature or caller changes.
- Fix documentation already complete at the bug level: BUG-347 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-347`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `debug_test::test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | Record entity 1 with `[Position]`, then re-record entity 1 with `[Position, Health]` | fixed `entity_record` | `component_counts[Position] == 1`, `component_counts[Health] == 1`, `entity_count() == 1` |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -n "count.saturating_sub(1)\|count.saturating_sub( 1 )"` | decrement step present in `entity_record` | ≥1 match |

## Acceptance Criteria

- `module/helper/tiles_tools/src/debug.rs`'s `ECSInspector::entity_record` decrements the
  previous entry's component contributions before applying the new entity's counts
- The fix's source comment carries all 3 required fields: `Fix(BUG-347)`, `Root cause`,
  `Pitfall`, plus a `BUG-347` backreference
- `debug_test::test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts` exists
  and passes
- `entity_data`'s own overwrite-on-insert behavior remains unmodified
- `task/bug/verified/347_ecs_inspector_entity_record_inflates_component_counts.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `entity_record` in `debug.rs` decrement the previous entry's per-component
  counts (via `saturating_sub`) before applying the new entity's counts?
- [ ] C2 — Does the fix's source comment carry `Fix(BUG-347)`, `Root cause`, `Pitfall`, and a
  `BUG-347` backreference?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `debug_test::test_ecs_inspector_rerecording_entity_does_not_inflate_component_counts`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -n "if let Some(previous) = self.entity_data.get(&entity.id)"
  module/helper/tiles_tools/src/debug.rs` return a match?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-347`?
- [ ] C7 — Does BUG-347's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/debug.rs` (the fix content matches what BUG-347's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "if let Some(previous) = self.entity_data.get(&entity.id)"
  module/helper/tiles_tools/src/debug.rs` → 1
- [ ] M2 — `grep -c "count.saturating_sub(1)" module/helper/tiles_tools/src/debug.rs` → 1

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually calls `entity_record` twice with the same `entity.id`
  and different component sets, then reads `component_counts` via `report_generate` (not a
  hardcoded expected-value literal standing in for the call) — checked by reading the test body
  itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass re-checked the Tasks Index omission from In Scope (same recurring question) — consistent, established precedent across every registration task this batch, not a gap. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass re-ran the Testable line's command live: `cargo nextest run -p tiles_tools --all-features` via `longrun` → `272 tests run: 272 passed, 0 skipped`, exit 0 — claim holds exactly. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass scanned Delivery Requirements for scope creep — none found. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass independently read `entity_record`'s full live body and the reproducer test's full live body — the fix (decrement-before-insert block, `saturating_sub(1)`, remove-at-0) and the test's exact two-call scenario (`id: 1`, `[Position]` then `[Position, Health]`) both match this task's T02/AF1 claims exactly. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass confirmed `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/tiles_tools` matches the actual crate path and package name (`-p tiles_tools` ran successfully). | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`tiles_tools`) throughout In Scope/Out of Scope — no second-crate reference found. | — |
| D7 | Crate Locality | — | 🟢 | Confirmed via live read that `debug.rs` physically lives under `module/helper/tiles_tools/src/` — matches the `unit` field. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Confirmed `entity_data`'s overwrite behavior and the sibling `system_timing_record` method are both untouched by this fix (bug file's own H2/E2 and Generalized Version, re-confirmed by this task's Out of Scope) — no entanglement. | — |
| **Total** | | — | 🟢 | 0 open | — |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0, ~1s
(warm build). `grep -c "if let Some(previous) = self.entity_data.get(&entity.id)"
src/debug.rs` → 1. `grep -c "count.saturating_sub(1)"` → 1. Reproducer test body
(`tests/debug_test.rs:201`) read live: constructs two real `EntityDebugInfo` values sharing
`id: 1` with differing `components`, calls the real `entity_record` twice, asserts on real
`entity_count()`/`report_generate()` output — matches T02/AF1 exactly. All
Verification-section grep patterns confirmed correct as originally written — no rewording
needed this round.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:05:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | FILED | task created |
| 2026-08-18 20:06:14 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:06:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 382 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 382` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-347's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/debug.rs`'s
  `ECSInspector::entity_record` now decrements a re-recorded entity's previous component
  contributions before applying its new ones, fixing permanent `component_counts` inflation)
  as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass pre-verified every planned grep/measurement live before writing them into
  the Verification section, and independently read both the fix's and the reproducer test's
  full live bodies to confirm the MOST Goal/T02/AF1 claims — all confirmed accurate, no
  rewording needed. Full crate suite re-run live via `longrun` (272/272 passed). `tsk
  .claim_verify 382` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified (bug file's own
  VERIFY Gate, two independent passes, 2026-08-18) during BUG-347's own investigation. This
  task's own contribution is the formal tracking registration and lifecycle walk, not the code
  change itself. `tsk .verify_pass 382` blocked by the same-actor guard (documented above) —
  task left at 🔬 Verifying per this sandbox's standing, previously documented limitation, not a
  quality defect in this task's own content.
