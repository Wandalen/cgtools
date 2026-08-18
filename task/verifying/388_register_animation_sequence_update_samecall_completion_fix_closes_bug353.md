# 388: Register animation Sequence update same-call completion fix closes BUG-353

## Execution State

- **id:** 388
- **title:** Register animation Sequence update same-call completion fix closes BUG-353
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:57:44
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/animation/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/animation
- **closes:** BUG-353
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/animation/
- **started_at:** 2026-08-18 20:58:23
- **expires_at:** 2026-08-18 22:58:23
- **unverified_at:** 2026-08-18 20:58:23
- **unverified_by:** unknown
- **verifying_at:** 2026-08-18 20:58:23
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/animation/

## MOST Goal

BUG-353 (`task/bug/verified/353_sequence_update_misses_same_call_completion.md`, Medium
severity, 🎯 Verified) found `Sequence::update` (`src/sequencer.rs`) wrote its
`Pending -> Running` and `Running -> Completed` transitions as two mutually exclusive arms of one
`match self.state`, keyed on `self.state`'s value at match-entry — a `delta_time` large enough to
satisfy both conditions in the same call (leave `Pending` AND immediately finish the now-active
last player) only ever applied the first transition, leaving `is_completed()` reporting `false`
in the very call where `progress()` already reported `1.0`, self-correcting only on the next
`update()` call. The fix — replacing the single `match` with two sequential `if` checks, so the
`Running -> Completed` condition is re-evaluated against the possibly-just-updated `self.state`
within the same call — is already applied at `sequencer.rs:539-561` with a `Fix(BUG-353)` source
comment, and independently confirmed via a new reproducer test. Related to, but a distinct root
cause from, BUG-352 (registered as task 387) — same file, same `Sequence` struct, no shared fix.
This task performs the remaining lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures :
Procedure - Promote Bug to Task` (PROC12) — to formally register that already-complete,
already-verified fix as a tracked task, closing BUG-353. Testable (per the bug file's own
recorded evidence, 2026-08-18; this task's own filing-time attempt reuses task 387's
same-session, same-crate, same-command confirmation of an external workspace build blocker — see
Verification Record D2): `cd module/helper/animation && cargo nextest run -p animation
--all-features` → 46 tests run: 46 passed, 0 skipped.

## In Scope

- `module/helper/animation/src/sequencer.rs` — the already-applied `Sequence::update` fix
  (`:539-561`, two sequential `if` checks replacing the single `match self.state { ... }`), with
  its `Fix(BUG-353)` source comment — verify present via direct read; no further edit expected.
- The already-applied
  `tests/sequencer_test.rs::test_sequence_update_completes_in_same_call_that_leaves_pending`
  reproducer — verify present via direct read; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/353_sequence_update_misses_same_call_completion.md`'s header back to
  this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/animation` — the fix is complete and verified by the
  bug's own Verification Record (8/8 PASS, 2026-08-18).
- Re-running or amending BUG-353's own Verification Record — already run and recorded in the bug
  file; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- BUG-352's own fix (`Tween::pause`/`Sequence::pause`/`Tween::resume`, same file) — a distinct
  root cause, already registered separately as task 387; not duplicated here.
- `Sequencer::update` (the separate heterogeneous coordinator in the same file) — the bug file's
  own H3/E5 confirmed it has no `Pending` state or pre-roll-delay concept at all, so the
  same-call-double-transition shape this bug depends on cannot arise there; not touched by this
  fix or this task.
- Diagnosing or fixing the external `mdmath_core`/`ndarray_cg` workspace build blocker
  encountered during task 387's filing (see this task's own Verification Record D2, reusing that
  same-session finding) — a concurrent, unrelated in-flight refactor in a different crate family,
  entirely outside `animation`'s and this task's remit; documented for transparency only.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own Symptom/MRE sections directly
  captured the pre-fix inconsistency (`progress() == 1.0` while `is_completed() == false` in the
  same call).
- Fix already applied at the one site (`Sequence::update`), with the required 3-field source
  comment (`Fix(BUG-353)`/`Root cause`/`Pitfall`).
- Green state already confirmed by the bug file's own Verification Record (2026-08-18): `cargo
  nextest run -p animation --all-features` → 46/46 passed, including the reproducer, and
  explicitly confirmed no regression in BUG-138/139/140/142/143/147/148/149/231/232/233's own
  reproducer tests. This task's own filing-time environment hit the same external, unrelated
  blocker already confirmed minutes earlier in this same session on task 387 (same crate, same
  command) — the bug-level evidence stands independently.
- No refactor needed — the fix replaces 1 `match` with 2 sequential `if` checks, no structural
  churn, no new public surface.
- Fix documentation already complete at the bug level: BUG-353 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention/Pitfall/Generalized Version narrative — this task does not
  duplicate it, only cross-links via `closes: BUG-353`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|--------------------|
| T01 | `cargo nextest run -p animation --all-features` | full crate suite | 46/46 passed (per bug file's own recorded evidence, 2026-08-18) |
| T02 | 2-player Sequence `[delay 0.1, 0.6]`/duration 0.5, fresh `Pending`, `update(100.0)` | fixed `Sequence::update` | `progress() == 1.0` AND `is_completed() == true` in the same call |
| T03 | `grep -c "Fix(BUG-353)"` in `sequencer.rs` | fix comment present | 1 |

## Acceptance Criteria

- `Sequence::update` re-evaluates `Running -> Completed` against the possibly-just-updated
  `self.state`, in the same call as a `Pending -> Running` transition
- The reproducer test exists and passes
- `task/bug/verified/353_sequence_update_misses_same_call_completion.md`'s header states `**Fix
  Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row's claim holds against either a live run or the bug file's own recorded
  evidence (whichever this task's own filing-time environment allowed)

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `Sequence::update` contain two sequential `if` blocks (not one `match`) — the
  first checking `Pending -> Running`, the second re-checking `Running -> Completed` against the
  post-first-check `self.state`?
- [ ] C2 — Does the reproducer test construct a real, fresh `Sequence` and call real `.update()`,
  `.progress()`, `.is_completed()` (not hardcoded expected-value literals)?
- [ ] C3 — Does `cargo nextest run -p animation --all-features` (via `longrun`, when the
  workspace build is not externally blocked) pass 46/46?

**Registration correctness**
- [ ] C4 — Does this task's `closes:` field name `BUG-353`?
- [ ] C5 — Does BUG-353's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C6 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/animation/src/interpolation.rs`, `src/sequencer.rs`, either test file, or any
  `module/math/` file (the mdmath_core/ndarray_cg blocker is diagnosed, not fixed, by this task)
  — note this repo's working tree carries many pre-existing, unrelated uncommitted changes from
  other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here.

### Measurements

- [ ] M1 — `grep -c "Fix(BUG-353)"` in `src/sequencer.rs` → 1
- [ ] M2 — `grep -c "bug_reproducer(BUG-353)"` in `tests/sequencer_test.rs` → 1

### Invariants

- [ ] I1 — When the workspace build is not externally blocked: `cargo nextest run -p animation
  --all-features` → 0 failures

### Anti-faking checks

- [ ] AF1 — the reproducer test actually calls the real `.update()`/`.progress()`/
  `.is_completed()` methods on a real `Sequence` instance (not a hardcoded expected-state literal
  standing in for the calls) — checked by reading the test body itself, not just its pass/fail
  result

## Verification Record

**Gate Round 1** · Tier: 2 · Type: Full · Verdict: OPEN · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | MOST Goal Compliance | — | 🟢 | Confirming: goal states BUG-353, cites PROC12, gives a Testable line. Adversarial: tried to find an overstated claim (e.g. a fresh 46/46 this filing never actually ran) — the Testable line explicitly attributes the number to the bug file's own recorded evidence and names the reused blocker finding, no overstatement found. | — |
| D2 | Deliverable Verification Completeness | — | 🟢 | Confirming: Verification section carries C1-C6/M1-M2/I1/AF1, each independently checkable. Adversarial: rather than re-launching an identical `cargo nextest run -p animation --all-features` (same crate, same command, no source change since), reused task 387's own same-session live confirmation (pid 49133, `longrun` exit 101, `-0002_longrun.log`, timestamp 2026-08-18 20:56:05 — minutes earlier in this same continuous session) of `error[E0433]: cannot find general in mdmath_core` in `ndarray_cg`, itself first diagnosed on task 385's D2. Re-running the identical command against unchanged source would produce identical output, not new evidence — reuse here is direct same-session reuse, not blind trust of an older/different-crate result. C1-C2 and M1-M2 were instead verified by direct, character-for-character source read (gold standard, unaffected by build tooling): `grep -n "Fix(BUG-353)"` → `sequencer.rs:539` (1), matching M1 exactly, and the fix body read in full at `:539-561` confirms two sequential `if` blocks (Pending→Running at 551-554, Running→Completed re-checked against post-update `self.state` at 556-561), not a single `match`; `grep -n "bug_reproducer(BUG-353)"` → `tests/sequencer_test.rs:356` (1), matching M2 exactly, test body read in full (`:382-393`+) confirms real `Sequence::new`/`.update(100.0)` calls with assertions on real `.progress()`/`.is_completed()` return values, no hardcoded literal standing in for either call. C3/T01/I1 (the live 46/46 run) rest on the bug file's own recorded Verification Record (2026-08-18, predates the external breakage), per this task's own Delivery Requirements. | — |
| D3 | Anti-Cheating Readiness | — | 🟢 | Confirming: AF1 requires reading the actual test body, not trusting pass/fail alone. Adversarial: read `tests/sequencer_test.rs:382-393`+ directly this session (real `Sequence` construction, real `.update(100.0)` call, assertions on real `.progress()`/`.is_completed()` return values) — no hardcoded expected-state literal standing in for a method call found. | — |
| D4 | Execution Prerequisites | — | 🟢 | Confirming: `unit_type: module`, `unit: lib/yrd_gamedev/cgtools/module/helper/animation`, `closes: BUG-353` all set correctly in Execution State. Adversarial: checked for a mismatched unit path — matches the crate actually holding the fix site, identical in form to task 387's own `unit` field for the same crate. | — |
| D5 | Source-of-Truth Alignment | — | 🟢 | Confirming: no `docs/feature`/`docs/invariant`/`docs/api` instance exists for `Sequence::update`'s transition semantics to conflict with. Adversarial: searched for a doc instance that might contradict the same-call-reevaluation fix — none found; no BLOCKING spec.md/spec/ hygiene violation applies either (dev repo, no spec.md present). | — |
| D6 | Decomposition Fit | — | 🟢 | Confirming: fix spans exactly 1 crate (`animation`), 1 file, 1 method — no multi-crate split warranted. Adversarial: checked whether this shares a fix with BUG-352 (same file, same struct) that should collapse into one task — no: bug file's own Related Bugs section and this task's own Out of Scope both confirm distinct root causes (gating defect in `pause()` vs. same-call-transition gap in `update()`), no shared fix, correctly filed as separate tasks (387 vs. this one). | — |
| D7 | Rulebook Compliance | — | 🟢 | Confirming: no `cargo fmt` invoked, no git command run, Edit used exclusively (task file was `tsk .create`-generated then Edited, never Written after initial creation), no new temp artifacts created this task (reused task 387's existing hyphen-prefixed log). Adversarial: scanned this task's own tool-call history for a Write call against a pre-existing file, a non-whitelist git invocation, or a non-hyphenated temp file — none found. | — |
| D8 | Traceability | — | 🟢 | Confirming: `closes: BUG-353` set; bug file backlink to be added immediately after this gate (PROC12 Step 4). Adversarial: verified BUG-353's own file does NOT yet carry a `Fix Task` line (checked via prior Read before this edit) — confirming the backlink write is not a duplicate. | — |
| **Total** | | — | 🟢 | 1 non-blocking note (D2: live full-suite re-run substituted with same-session task-387 evidence + source-level cross-check, due to external blocker) | — |

Dual-Role Self-Check per `maav.rulebook.md § MAAV : Verification Tier Selection` — Tier 2 default, this session capped at Tier 2 per standing project convention (never escalate).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:57:44 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/animation/ | FILED | task created |
| 2026-08-18 20:58:23 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:58:23 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/animation/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 21:01:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/animation/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 388 "user1@w002/.../animation/"` → exit 1: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)`. Same-actor guard, documented sandbox constraint — not forced/spoofed. |

## History

- **FILED** (2026-08-18 20:57:44): Task created via `tsk .create`, registering the already-complete BUG-353 fix per PROC12.
- **READINESS_GATE_PASS** (2026-08-18 21:01:00): Tier 2 Dual-Role Self-Check, Gate Round 1, 8/8 dimensions PASS (see Verification Record above). D2 reuses task 387's same-session, same-crate, same-command confirmation of the external `mdmath_core`/`ndarray_cg` workspace build blocker (originally diagnosed on task 385) as a non-blocking note — substituted source-level + bug-file evidence for the live full-suite run this task's own environment could not obtain.
- **EXECUTED** (2026-08-18 21:01:00): `tsk .verify_pass` attempted and blocked by same-actor guard, per standard project convention for this sandbox — documented above, not circumvented.
