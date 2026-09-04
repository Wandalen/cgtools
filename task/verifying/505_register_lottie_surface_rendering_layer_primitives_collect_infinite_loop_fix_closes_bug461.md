# 505: Register lottie_surface_rendering layer_primitives_collect infinite loop fix closes BUG-461

## Execution State

- **id:** 505
- **title:** Register lottie_surface_rendering layer_primitives_collect infinite loop fix closes BUG-461
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-20 14:00:15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/lottie_surface_rendering
- **closes:** BUG-461
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-20 14:03:01
- **expires_at:** 2026-08-20 16:03:01
- **unverified_at:** 2026-08-20 14:03:00
- **unverified_by:** unknown
- **verifying_at:** 2026-08-20 14:03:01
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

BUG-461 (`task/bug/verified/461_lottie_layer_primitives_collect_infinite_loop.md`, High severity,
🎯 Verified) found `examples/minwebgl/lottie_surface_rendering/src/animation.rs`'s
`layer_primitives_collect` hanging forever on any Lottie layer whose `content` is
`velato::model::Content::None` or `Content::Instance` -- both legitimate, spec-valid content that
`layer_to_primitives` returns `None` for without ever advancing the manually-indexed `while` loop's
own `i`, so a `let-else` `continue` in that branch re-evaluates the identical loop condition
forever. The fix -- adding the missing `i += 1;` immediately before that branch's `continue;` --
is already applied and documented with a `Fix(BUG-461)`/`Root cause`/`Pitfall` 3-field source
comment, together with a genuine, timeout-bounded regression test
(`layer_primitives_collect_skips_non_shape_content_without_hanging`) that the bug file's own
Verification Record confirms was adversarially validated by temporarily reverting the fix and
observing the test actually fail via its 5-second `recv_timeout` panic path, not merely pass
vacuously. This task performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core
Procedures : Procedure - Promote Bug to Task` (PROC12) -- to formally register that
already-complete, already-verified fix as a tracked task, closing BUG-461.
Testable: `cd examples/minwebgl/lottie_surface_rendering && cargo test -p
lottie_surface_rendering 2>&1 | grep -q "test result: ok"` → PASS.

## In Scope

- `examples/minwebgl/lottie_surface_rendering/src/animation.rs`'s already-applied
  `layer_primitives_collect` fix (`i += 1;` before the non-`Shape`-content `continue;`) and its
  `Fix(BUG-461)`/`Root cause`/`Pitfall` source comment -- verify present; no further edit
  expected.
- The already-applied `#[ cfg( test ) ] mod tests`' `layer_primitives_collect_skips_non_shape_content_without_hanging`
  reproducer (thread + channel + 5s `recv_timeout` bound) -- verify present and passing; no
  further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/461_lottie_layer_primitives_collect_infinite_loop.md`'s header back
  to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `examples/minwebgl/lottie_surface_rendering` -- the fix is complete
  and independently verified by the bug's own VERIFY Gate, including a live adversarial
  fix-revert that confirmed the regression test genuinely catches the defect.
- Re-running BUG-461's own VERIFY Gate -- already run and recorded in the bug file's Verification
  Record (2026-08-20, 2/2 PASS); not re-litigated by this task's own Readiness Verification Gate,
  which checks task-file quality, not the underlying fix.
- Auditing other manually-indexed `while` loops in this crate or elsewhere for the same
  missing-index-advance shape -- not part of BUG-461's own Generalized Version/scope; a separate
  concern if raised.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own adversarial pass temporarily
  reverted the `i += 1;` fix and observed the regression test fail via its 5s `recv_timeout` panic
  ("did not return within 5s -- regressed to the BUG-461 infinite loop") -- this task does not
  re-derive that evidence.
- Fix already applied: `layer_primitives_collect`'s non-`Shape`-content branch advances `i` before
  `continue`, with the required 3-field source comment immediately above.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo test -p
  lottie_surface_rendering` (native, via `longrun`) passes, including the new regression test.
- No refactor needed -- the fix is a single added statement plus a comment, no structural churn.
- Fix documentation already complete at the bug level: BUG-461 carries the full Root Cause/Why Not
  Caught/Fix Location/Prevention narrative in its own body -- this task does not duplicate it,
  only cross-links via `closes: BUG-461`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention -- document rather than force/spoof if
  so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd examples/minwebgl/lottie_surface_rendering && cargo test -p lottie_surface_rendering` | `animation::tests::layer_primitives_collect_skips_non_shape_content_without_hanging` (bug_reproducer) | exit 0, test passes within the 5s timeout bound |
| T02 | 3-layer fixture (`None`, `Instance`, `Shape`) passed to `layer_primitives_collect` on a background thread | fixed function | returns within 5s via `mpsc::Receiver::recv_timeout`, no hang |
| T03 | `grep -n "i += 1;" examples/minwebgl/lottie_surface_rendering/src/animation.rs` | Whole-file scan for the added index-advance statement, immediately preceding the non-`Shape`-content `continue;` (line ~327) | present, immediately before that specific `continue;` |
| T04 | `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` | crate compiles for wasm32 | 0 errors |

## Acceptance Criteria

- `layer_primitives_collect`'s non-`Shape`-content branch (`let-else` `continue`) advances `i`
  before the `continue`, not after or never
- The fix's source comment carries all 3 required fields: `Fix(BUG-461)`, `Root cause`, `Pitfall`
- `layer_primitives_collect_skips_non_shape_content_without_hanging` exists and passes within its
  5s timeout bound
- `cargo test -p lottie_surface_rendering` (native) passes in full
- `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` succeeds
- `task/bug/verified/461_lottie_layer_primitives_collect_infinite_loop.md`'s header states
  `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify -- an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 -- Does `layer_primitives_collect`'s non-`Shape`-content branch advance `i` (`i += 1;`)
  immediately before its `continue;`?
- [ ] C2 -- Does the fix's source comment carry `Fix(BUG-461)`, `Root cause`, and `Pitfall`
  fields?
- [ ] C3 -- Does `cargo test -p lottie_surface_rendering` (via `longrun`) pass, including
  `layer_primitives_collect_skips_non_shape_content_without_hanging`?
- [ ] C4 -- Does `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown`
  succeed with 0 errors?

**Registration correctness**
- [ ] C5 -- Does this task's `closes:` field name `BUG-461`?
- [ ] C6 -- Does BUG-461's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 -- No Edit/Write tool call in this task's own execution targeted
  `examples/minwebgl/lottie_surface_rendering/src/animation.rs` (the fix content matches what
  BUG-461's own already-completed fix applied; this task made no further source edit to it --
  note this repo's working tree carries many pre-existing, unrelated uncommitted changes from
  other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 -- `grep -c "i += 1;" examples/minwebgl/lottie_surface_rendering/src/animation.rs` → ≥ 3
  (the pre-existing loop-tail advance, the fix's new advance, plus at least one other unrelated
  site)

### Invariants

- [ ] I1 -- `cargo test -p lottie_surface_rendering` → 0 failures
- [ ] I2 -- `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` → 0 errors

### Anti-faking checks

- [ ] AF1 -- the regression test genuinely constructs a `None`/`Instance`/`Shape` 3-layer fixture
  and calls the real `layer_primitives_collect` on a background thread bounded by
  `recv_timeout` (not a hardcoded pass, and not merely asserting on a mocked/stubbed function) --
  checked by reading the test body itself, not just its pass/fail result

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass: checked In Scope ("verify present, no further edit expected") against Out of Scope ("no further code change") for contradiction — none found, consistent. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass: confirmed the `Testable:` line's command (`cargo test ... \| grep -q "test result: ok"`) is a real, executable, falsifiable check, not a vacuous placeholder. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass: Null Hypothesis considered ("what if this task is never filed?") — BUG-461 stays permanently stuck at 🎯 Verified, unable to self-accept (same-actor guard), matching the 26-precedent pattern this sweep is following; not speculative. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass: T01-T04 re-run live this session (grep counts, native `cargo test` via longrun, wasm32 `cargo check`) — all genuinely re-executable, not aspirational. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass: scanned every path cited in Delivery Requirements/Test Matrix for any escape outside repo root — none found. | — |
| D6 | Crate Scope Unity | — | 🟢 | Adversarial pass: the only path outside `examples/minwebgl/lottie_surface_rendering` is the BUG-461 file link-back, which is task-system bookkeeping (PROC12 Step 4), not a code/test deliverable — matches task 379/504's precedent reasoning. | — |
| D7 | Crate Locality | — | 🟢 | Adversarial pass: confirmed `examples/minwebgl/lottie_surface_rendering` is itself the leaf crate owning `animation.rs` — no deeper leaf exists to push this down to. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial pass: crate responsibility ("renders Lottie animations to a surface via minwebgl") stays one sentence, no "and" — the added regression test lives inside the crate's existing test module, introduces no second responsibility domain. | — |

**Live re-verification (this session, 2026-08-20 ~14:12-14:13):**
- `grep -n "i += 1;" src/animation.rs` → lines 327, 333, 617 (3 matches, M1's `≥3` satisfied; line 327 is the fix site, immediately before the non-`Shape`-content `continue;`)
- `grep -n "Fix(BUG-461)\|Root cause\|Pitfall"` → present at both the fix site (312/315/319) and the regression test's own doc comment (700/703)
- `cargo test -p lottie_surface_rendering` via `longrun` → exit 0, `test animation::tests::layer_primitives_collect_skips_non_shape_content_without_hanging ... ok`, 1 passed; 0 failed (T01/T02/C3/I1 confirmed)
- `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` via `longrun` (background task `bwtcynmk9`) → exit 0, `Finished` clean (T04/C4/I2 confirmed)
- BUG-461's header confirmed to NOT yet carry `**Fix Task:**` prior to this task's own follow-up edit (C6 pending, applied immediately after this record)

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | FILED | Filed via PROC12 (bug_promote) to register BUG-461's already-complete `layer_primitives_collect` infinite-loop fix. |
| 2026-08-20 | READINESS_GATE_PASS | Tier 2 Dual-Role Self-Check, 8/8 🟢 — see Verification Record above. |
| 2026-08-20 | EXECUTED | Fix, test, and wasm32 compile check all live-reconfirmed; BUG-461 header linked back via `**Fix Task:**`. |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-20 14:00:15 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FILED | task created |
| 2026-08-20 14:03:00 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-20 14:03:01 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
| 2026-08-20 14:13:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | Round 8 readiness gate 8/8 PASS; live-reconfirmed T01-T04; `tsk .verify_pass` expected to hit same-actor guard per project convention |
