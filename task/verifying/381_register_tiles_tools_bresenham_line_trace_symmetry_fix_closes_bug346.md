# 381: Register tiles_tools bresenham_line_trace symmetry fix (closes BUG-346)

## Execution State

- **id:** 381
- **title:** Register tiles_tools bresenham_line_trace symmetry fix (closes BUG-346)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:03:15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-346
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:34
- **expires_at:** 2026-08-20 00:45:34
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system
- **verifying_at:** 2026-08-19 22:45:34
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-346 (`task/bug/verified/346_bresenham_line_of_sight_asymmetric.md`, High severity, 🎯
Verified) found `module/helper/tiles_tools/src/field_of_view.rs`'s `bresenham_line_trace` —
the sole line-tracing primitive backing `FOVAlgorithm::Bresenham` — using a greedy "step to
whichever neighbor is closest to the fixed target" walk seeded at `from`, with no
canonicalization step. This walk is not path-reversible: tracing `A->B` and `B->A` could visit
different intermediate cells, so `line_of_sight(A, B)` and `line_of_sight(B, A)` could disagree
about the same wall configuration — a silent, direction-dependent logic error (stealth/AI
detection asymmetry, ranged-attack exploits), not a panic. The fix — canonicalizing walk
direction via a `Hash`-based comparison (walk from the hash-smaller endpoint to the
hash-larger, then reverse the result if the caller's `from`/`to` were the other way round),
with the required `Fix(BUG-346)`/`Root cause`/`Pitfall` 3-field source comment plus a
`BUG-346 task/bug/...` backreference — is already applied and independently confirmed via a
new permanent reproducer test (`test_bresenham_line_of_sight_is_symmetric_around_wall`,
`tests/integration/field_of_view_tests.rs:675`) asserting `line_of_sight(A,B) ==
line_of_sight(B,A)` for a wall cluster that previously produced opposite answers — the bug
file's own VERIFY Gate, 8/8 PASS, 2026-08-18 (two independent passes, the second one adding the
missing backreference comment), plus a full-suite re-run (272/272 tests, re-confirmed live
during this task's own filing). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-346.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/field_of_view.rs` — the already-applied `bresenham_line_trace`
  hash-based direction-canonicalization fix and its `Fix(BUG-346)`/`Root cause`/`Pitfall` source
  comment plus `BUG-346` backreference — verify present; no further edit expected.
- The already-applied `tests/integration/field_of_view_tests.rs::test_bresenham_line_of_sight_is_symmetric_around_wall`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/346_bresenham_line_of_sight_asymmetric.md`'s header back to this
  task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate (two independent passes).
- Re-running BUG-346's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- The other 3 `FOVAlgorithm` variants (`Shadowcasting`, `RayCasting`, `FloodFill`) — confirmed
  by the bug file's own H5/E5 to not call `bresenham_line_trace` and so not affected; not
  touched by this fix or this task.
- Any other directional/greedy line-tracing primitive in this crate — the bug file's own
  Generalized Version section confirmed `bresenham_line_trace` is the only "trace toward a
  fixed target" walk in `field_of_view.rs`; not re-derived by this task.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix asymmetry (`A->B = true, B->A = false`) via a permanent reproducer test run
  against the pre-fix source — this task does not re-derive that evidence.
- Fix already applied: `field_of_view.rs`'s `bresenham_line_trace` canonicalizes walk direction
  via `Hash` comparison before walking, reversing the result if the caller's `from`/`to` were
  the higher-hash endpoint first, with the required 3-field source comment plus backreference.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~1s warm build).
- No refactor needed — the fix is contained entirely within `bresenham_line_trace`'s own body,
  no signature or caller changes (confirmed by the bug file's own Fix Location: callers
  `bresenham_line_check` need no changes since `line_positions[0]` is still always `from`).
- Fix documentation already complete at the bug level: BUG-346 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-346`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | `FieldOfView::with_algorithm(FOVAlgorithm::Bresenham)`, walls at (2,1),(2,2),(3,2), A=(0,0), B=(5,3) | fixed `bresenham_line_trace` | `line_of_sight(A,B) == line_of_sight(B,A)` |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -n "let swapped = hash_of( from ) > hash_of( to )"` | canonicalization step present in `bresenham_line_trace` | 1 match |

## Acceptance Criteria

- `module/helper/tiles_tools/src/field_of_view.rs`'s `bresenham_line_trace` canonicalizes walk
  direction via `Hash` comparison and reverses the result when swapped
- The fix's source comment carries all 3 required fields: `Fix(BUG-346)`, `Root cause`,
  `Pitfall`, plus a `BUG-346` backreference
- `field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall` exists and passes
- The other 3 `FOVAlgorithm` variants remain unmodified (confirmed not to call
  `bresenham_line_trace`)
- `task/bug/verified/346_bresenham_line_of_sight_asymmetric.md`'s header states `**Fix Task:**`
  pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `bresenham_line_trace` in `field_of_view.rs` compute `swapped = hash_of( from )
  > hash_of( to )` and walk from the hash-smaller endpoint?
- [ ] C2 — Does the fix's source comment carry `Fix(BUG-346)`, `Root cause`, `Pitfall`, and a
  `BUG-346` backreference?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -c "if swapped" module/helper/tiles_tools/src/field_of_view.rs` return
  ≥1 (the result-reversal branch is present)?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-346`?
- [ ] C7 — Does BUG-346's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/field_of_view.rs` (the fix content matches what BUG-346's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "let swapped = hash_of( from ) > hash_of( to )"
  module/helper/tiles_tools/src/field_of_view.rs` → 1
- [ ] M2 — `grep -c "line_positions.reverse()"
  module/helper/tiles_tools/src/field_of_view.rs` → 1

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually constructs a `FieldOfView::with_algorithm(FOVAlgorithm::Bresenham)`,
  a wall set, and calls `line_of_sight` in both directions (not a hardcoded expected-value
  literal standing in for the call) — checked by reading the test body itself, not just its
  pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass re-checked the Tasks Index omission from In Scope (same question raised on prior siblings) — consistent, established precedent across every registration task this batch, not a gap. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass re-ran the Testable line's command live: `cargo nextest run -p tiles_tools --all-features` via `longrun` → `272 tests run: 272 passed, 0 skipped`, exit 0 — claim holds exactly. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass scanned Delivery Requirements for scope creep — none found. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass independently read `bresenham_line_trace`'s full live body and the reproducer test's full live body (not just its name) — the fix (`hash_of`/`swapped`/`line_positions.reverse()`) and the test's exact wall/coordinate scenario (`SquareCoord<EightConnected>`, walls (2,1)/(2,2)/(3,2), A=(0,0), B=(5,3)) both match this task's T02/AF1 claims exactly. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass confirmed `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/tiles_tools` matches the actual crate path and package name (`-p tiles_tools` ran successfully). | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`tiles_tools`) throughout In Scope/Out of Scope — no second-crate reference found. | — |
| D7 | Crate Locality | — | 🟢 | Confirmed via live read that `field_of_view.rs` physically lives under `module/helper/tiles_tools/src/` — matches the `unit` field. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Confirmed the other 3 `FOVAlgorithm` variants are untouched by this fix (bug file's own H5/E5, re-confirmed by this task's Out of Scope) — no entanglement. | — |
| **Total** | | — | 🟢 | 0 open | — |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0, ~1s
(warm build). `grep -c "let swapped = hash_of( from ) > hash_of( to )" src/field_of_view.rs` →
1. `grep -c "line_positions.reverse()"` → 1. `grep -c "if swapped"` → 2 (≥1, as claimed). All
Verification-section grep patterns and the T02 scenario/AF1 claim confirmed correct as
originally written — no rewording needed this round.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:03:15 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | FILED | task created |
| 2026-08-18 20:04:07 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:04:07 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 381 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 381` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-346's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/field_of_view.rs`'s
  `bresenham_line_trace` gains hash-based walk-direction canonicalization, fixing
  direction-dependent `line_of_sight` results around wall clusters) as a tracked task, closing
  the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass pre-verified every planned grep/measurement live before writing them into
  the Verification section, and independently re-read both the fix's and the reproducer test's
  full live bodies (not just names) to confirm the MOST Goal/T02/AF1 claims — all confirmed
  accurate, no rewording needed. Full crate suite re-run live via `longrun` (272/272 passed).
  `tsk .claim_verify 381` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified (bug file's own
  VERIFY Gate, two independent passes, 2026-08-18) during BUG-346's own investigation. This
  task's own contribution is the formal tracking registration and lifecycle walk, not the code
  change itself. `tsk .verify_pass 381` blocked by the same-actor guard (documented above) —
  task left at 🔬 Verifying per this sandbox's standing, previously documented limitation, not a
  quality defect in this task's own content.
