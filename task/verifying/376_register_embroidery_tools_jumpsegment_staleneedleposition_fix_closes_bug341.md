# 376: Register embroidery_tools jump-segment stale-needle-position fix (closes BUG-341)

## Execution State

- **id:** 376
- **title:** Register embroidery_tools jump-segment stale-needle-position fix (closes BUG-341)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 19:43:17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/embroidery_tools
- **closes:** BUG-341
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 23:49:13
- **expires_at:** 2026-08-19 01:49:13
- **unverified_at:** 2026-08-18 23:47:42
- **unverified_by:** system
- **verifying_at:** 2026-08-18 23:49:13
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-341 (`task/bug/verified/341_pes_writer_jump_segment_stale_needle_position.md`, Medium
severity, 🎯 Verified) found `module/helper/embroidery_tools/src/format/pes/writer.rs`'s
`as_segment_blocks`'s `Instruction::Jump` match arm reading the shared `stitched_x`/
`stitched_y` needle-position tracker to compute a jump segment's start point, but — unlike the
`Instruction::Stitch` arm, which writes both back after every stitch — never writing them back
itself. When two `Jump` command-blocks are separated only by a non-`Stitch` instruction
(`ColorChange`, `Trim`, or the catch-all, all of which fall through without touching the
tracker), the second jump's recorded start point is stale: it reads the position left over from
before the first jump fired, not where the first jump actually ended. The fix — two new
assignment lines (`stitched_x = last_instruction.x; stitched_y = last_instruction.y;`) added to
the `Jump` arm immediately after it reads `command_block.last()`, with a `Fix(BUG-341)`/
`Root cause`/`Pitfall` 3-field source comment — is already applied and independently confirmed
via a new byte-level reproducer test (`second_jump_after_colorchange_starts_where_first_jump_ended`,
`tests/pes_test.rs`) that writes the exact repro program to PES v6, decodes the actual CSewSeg
segment bytes, and asserts the second jump segment starts at `(15,15)` (where the first jump
ended) rather than the pre-fix stale `(10,10)` — the bug file's own VERIFY Gate, 8/8 PASS,
2026-08-18, additionally empirically proved via a revert/restore cycle (reverting the 2
write-back lines reproduces the exact documented failure; restoring returns 17/17 passed). This
task performs the remaining lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures :
Procedure - Promote Bug to Task` (PROC12) — to formally register that already-complete,
already-verified fix as a tracked task, closing BUG-341.
Testable: `cd module/helper/embroidery_tools && cargo nextest run -p embroidery_tools 2>&1 |
grep -q '17 tests run: 17 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/embroidery_tools/src/format/pes/writer.rs` — the already-applied `Jump` arm
  write-back (`stitched_x = last_instruction.x; stitched_y = last_instruction.y;`, immediately
  after the `command_block.last()` read) and its `Fix(BUG-341)`/`Root cause`/`Pitfall` source
  comment — verify present; no further edit expected.
- The already-applied `tests/pes_test.rs::second_jump_after_colorchange_starts_where_first_jump_ended`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/341_pes_writer_jump_segment_stale_needle_position.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/embroidery_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate (including an empirical revert/restore
  proof).
- Re-running BUG-341's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- Any other shared-tracker-across-match-arms pattern in this crate — BUG-341's own Generalized
  Version section confirmed via `grep -rn "let mut stitched_x\|let mut stitched_y"` that
  `as_segment_blocks` is the only function maintaining this kind of tracker; re-confirmed empty
  again during this task's own filing (see Verification Record).

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own VERIFY Gate empirically reverted
  the 2 write-back lines and observed the exact documented failure (`second_jump_...` FAILED,
  1 failed) before restoring them (17/17 passed) — this task does not re-derive that evidence.
- Fix already applied: `writer.rs`'s `Instruction::Jump` arm assigns
  `stitched_x = last_instruction.x; stitched_y = last_instruction.y;` immediately after reading
  `command_block.last()`, with the required 3-field source comment directly above.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo nextest
  run -p embroidery_tools` → 17 tests run: 17 passed, 0 skipped (via `longrun`, exit 0, ~33s).
- No refactor needed — the fix is a two-line addition plus a comment, no structural churn.
- Fix documentation already complete at the bug level: BUG-341 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-341`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/embroidery_tools && cargo nextest run -p embroidery_tools` | `pes_test::second_jump_after_colorchange_starts_where_first_jump_ended` (bug_reproducer) | exit 0, 17/17 passed |
| T02 | `grep -rn "let mut stitched_x\|let mut stitched_y" module/helper/embroidery_tools/src/` | Whole-crate scan for a sibling shared-tracker pattern | exactly 2 hits (the one `as_segment_blocks` declaration site, one line per variable), no sibling function |
| T03 | `cargo check -p embroidery_tools` | crate compiles | 0 errors |
| T04 | Reproducer's decoded CSewSeg bytes for the repro program (`stitch(0,0) stitch(10,10) jump(5,5) color_change(0,0) jump(5,5) stitch(0,0) stitch(1,1) end()`) | fixed `as_segment_blocks` output | second jump segment starts at `(15,15)`, not the pre-fix stale `(10,10)` |

## Acceptance Criteria

- `module/helper/embroidery_tools/src/format/pes/writer.rs`'s `Instruction::Jump` arm assigns
  `stitched_x`/`stitched_y` from `last_instruction` immediately after reading
  `command_block.last()`
- The fix's source comment carries all 3 required fields: `Fix(BUG-341)`, `Root cause`,
  `Pitfall`
- `pes_test::second_jump_after_colorchange_starts_where_first_jump_ended` exists and passes
- No other function in `module/helper/embroidery_tools/src/` maintains an analogous
  multi-arm-shared position tracker
- `task/bug/verified/341_pes_writer_jump_segment_stale_needle_position.md`'s header states
  `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does the `Instruction::Jump` arm in `writer.rs` assign `stitched_x`/`stitched_y`
  from `last_instruction` immediately after the `command_block.last()` read?
- [ ] C2 — Does the call site's source comment carry `Fix(BUG-341)`, `Root cause`, and
  `Pitfall` fields?
- [ ] C3 — Does `cargo nextest run -p embroidery_tools` (via `longrun`) pass
  `pes_test::second_jump_after_colorchange_starts_where_first_jump_ended`?
- [ ] C4 — Does `cargo check -p embroidery_tools` succeed with 0 errors?
- [ ] C5 — Does `grep -rn "let mut stitched_x\|let mut stitched_y"
  module/helper/embroidery_tools/src/` return exactly 2 hits (the one `as_segment_blocks`
  declaration site, one line per variable), with no sibling function declaring an analogous
  tracker?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-341`?
- [ ] C7 — Does BUG-341's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/embroidery_tools/src/format/pes/writer.rs` (the fix content matches what
  BUG-341's own already-completed fix applied; this task made no further source edit to it —
  note this repo's working tree carries many pre-existing, unrelated uncommitted changes from
  other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful
  signal here).

### Measurements

- [ ] M1 — `sed -n '468,489p' module/helper/embroidery_tools/src/format/pes/writer.rs | grep -c
  'stitched_x = last_instruction.x'` → 1
- [ ] M2 — `sed -n '468,489p' module/helper/embroidery_tools/src/format/pes/writer.rs | grep -c
  'stitched_y = last_instruction.y'` → 1

### Invariants

- [ ] I1 — `cargo nextest run -p embroidery_tools` → 0 failures
- [ ] I2 — `cargo check -p embroidery_tools` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually decodes real CSewSeg bytes written by `pes::write`
  (not a hardcoded expected-value literal standing in for the write path) — checked by reading
  the test body itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | Adversarial pass re-ran Test Matrix T02 and Checklist C5's grep live: `grep -rn "let mut stitched_x\|let mut stitched_y" module/helper/embroidery_tools/src/` returns 2 hits (one per variable name on the same declaration line pair), not the originally-claimed 1 | Reworded T02 and C5 to expect 2 hits (one per variable), no sibling declaration |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`module/helper/embroidery_tools`); the BUG-341 link-back edit touches a tracking file outside `unit_type: module`'s crate boundary — same disposition as every other bug-promotion cross-link in this repo (tracking-file edits are not crate-scope violations) | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 fixed | 1/1 |

**Reproduced live during this gate:** `cd module/helper/embroidery_tools && cargo nextest run -p
embroidery_tools` (via `longrun`) → 17 tests run: 17 passed, 0 skipped, exit 0, ~33s.
`sed -n '468,489p' src/format/pes/writer.rs | grep -c 'stitched_x = last_instruction.x'` → 1;
same for `stitched_y` → 1. `grep -rn "let mut stitched_x\|let mut stitched_y" src/` → 2 hits,
both at the single `as_segment_blocks` declaration (line 457-458), no sibling function.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 19:43:17 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 376 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-341's
  already-applied, already-verified fix (`module/helper/embroidery_tools/src/format/pes/writer.rs`
  `Instruction::Jump` arm gains a `stitched_x`/`stitched_y` write-back, matching the sibling
  `Stitch` arm's existing behavior) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass caught one real defect: Test Matrix T02 and Checklist C5 both claimed the
  sibling-tracker grep returns exactly 1 hit, but live re-run showed 2 (one per variable name
  at the single declaration site) — reworded both to the correct count. Re-verified T01/T03
  live post-fix (`cargo nextest run -p embroidery_tools` via `longrun`, exit 0, 17/17 passed).
  `tsk .claim_verify 376` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified (including an
  empirical revert/restore proof) during BUG-341's own investigation (bug file History,
  2026-08-18). This task's own contribution is the formal tracking registration and lifecycle
  walk, not the code change itself. `tsk .verify_pass 376` blocked by the same-actor guard
  (documented above) — task left at 🔬 Verifying per this sandbox's standing, previously
  documented limitation, not a quality defect in this task's own content.
| 2026-08-18 19:44:23 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 19:44:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
