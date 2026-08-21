# 506: Register morph_targets gui_setup initial weights display fix closes BUG-462

## Execution State

- **id:** 506
- **title:** Register morph_targets gui_setup initial weights display fix closes BUG-462
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-20 14:05:23
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/morph_targets
- **closes:** BUG-462
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-20 14:14:57
- **expires_at:** 2026-08-20 16:14:57
- **unverified_at:** 2026-08-20 14:14:55
- **unverified_by:** unknown
- **verifying_at:** 2026-08-20 14:14:57
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

BUG-462 (`task/bug/verified/462_morph_targets_gui_setup_wired_to_nan_sentinel_for_initial_display.md`, Medium
severity, 🎯 Verified) found `examples/minwebgl/morph_targets/src/gui_setup.rs`'s `setup` reading
NaN for every slider's initial displayed value -- downstream of BUG-330's fix, which introduced a
`gui_weights` override buffer pre-filled with `f32::NAN` sentinels, but `setup` still took a single
`weights` parameter that the caller filled with that same sentinel buffer, so `weight_settings_init`
had no non-NaN source to read a slider's starting value from. The fix -- splitting `setup`'s single
`weights` parameter into `initial_weights : &[ f32 ]` (the real, animation-driven weights, read once
for each slider's initial displayed value) and `gui_weights : &Rc< RefCell< Vec< f32 > > >` (the
unchanged NaN-sentinel override-tracking buffer slider drags write into) -- is already applied and
documented with a `Fix(BUG-462)`/`Root cause`/`Pitfall` 3-field source comment immediately above
`setup`. This task performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core
Procedures : Procedure - Promote Bug to Task` (PROC12) -- to formally register that already-complete,
already-verified fix as a tracked task, closing BUG-462.
Testable: `cd examples/minwebgl/morph_targets && cargo check --target wasm32-unknown-unknown` →
exit 0.

## In Scope

- `examples/minwebgl/morph_targets/src/gui_setup.rs`'s already-applied `setup` fix (split
  `initial_weights`/`gui_weights` parameters) and its `Fix(BUG-462)`/`Root cause`/`Pitfall` source
  comment -- verify present; no further edit expected.
- The already-updated call site in `src/main.rs` passing both the real weights buffer and the
  `gui_weights` sentinel buffer as two distinct arguments -- verify present; no further edit
  expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/462_morph_targets_gui_setup_wired_to_nan_sentinel_for_initial_display.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `examples/minwebgl/morph_targets` -- the fix is complete and
  independently verified by the bug's own VERIFY Gate.
- Re-running BUG-462's own VERIFY Gate -- already run and recorded in the bug file's Verification
  Record; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- Re-investigating BUG-330 or its own fix -- BUG-462 is a distinct downstream defect the BUG-330
  fix exposed, not a regression of BUG-330 itself; BUG-330's own resolution is out of scope here.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Fix already applied: `gui_setup::setup`'s single `weights` parameter split into
  `initial_weights : &[ f32 ]` and `gui_weights : &Rc< RefCell< Vec< f32 > > >`, with the required
  3-field source comment immediately above.
- Green state already confirmed, and re-confirmed live during this task's filing:
  `cargo check -p morph_targets --target wasm32-unknown-unknown` compiles clean.
- No refactor needed -- the fix is a parameter-list split plus an updated call site, no structural
  churn.
- Fix documentation already complete at the bug level: BUG-462 carries the full Root Cause/Why Not
  Caught/Fix Location/Prevention narrative in its own body -- this task does not duplicate it, only
  cross-links via `closes: BUG-462`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention -- document rather than force/spoof if
  so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -n "initial_weights : &\[ f32 \]" examples/minwebgl/morph_targets/src/gui_setup.rs` | Whole-file scan for the split parameter signature | present in `setup`'s parameter list |
| T02 | `grep -n "Fix(BUG-462)" examples/minwebgl/morph_targets/src/gui_setup.rs` | Whole-file scan for the fix comment | present immediately above `setup` |
| T03 | `cd examples/minwebgl/morph_targets && cargo check --target wasm32-unknown-unknown` | crate compiles for wasm32 | 0 errors |
| T04 | `grep -n "gui_setup::setup\|setup(" examples/minwebgl/morph_targets/src/main.rs` | call site passes two distinct buffers | both `initial_weights` and `gui_weights` arguments present, not the same single buffer |

## Acceptance Criteria

- `gui_setup::setup`'s parameter list carries both `initial_weights : &[ f32 ]` and
  `gui_weights : &Rc< RefCell< Vec< f32 > > >` as two distinct parameters
- The fix's source comment carries all 3 required fields: `Fix(BUG-462)`, `Root cause`, `Pitfall`
- `cargo check -p morph_targets --target wasm32-unknown-unknown` succeeds
- `main.rs`'s call site passes two distinct buffers, not the same buffer for both roles
- `task/bug/verified/462_morph_targets_gui_setup_wired_to_nan_sentinel_for_initial_display.md`'s header states
  `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify -- an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 -- Does `gui_setup::setup` take `initial_weights : &[ f32 ]` and
  `gui_weights : &Rc< RefCell< Vec< f32 > > >` as two distinct parameters?
- [ ] C2 -- Does the fix's source comment carry `Fix(BUG-462)`, `Root cause`, and `Pitfall`
  fields?
- [ ] C3 -- Does `cargo check -p morph_targets --target wasm32-unknown-unknown` succeed with 0
  errors?
- [ ] C4 -- Does `main.rs`'s call site pass two distinct buffers (not the same `gui_weights`
  sentinel buffer for both roles)?

**Registration correctness**
- [ ] C5 -- Does this task's `closes:` field name `BUG-462`?
- [ ] C6 -- Does BUG-462's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 -- No Edit/Write tool call in this task's own execution targeted
  `examples/minwebgl/morph_targets/src/gui_setup.rs` or `src/main.rs` (the fix content matches
  what BUG-462's own already-completed fix applied; this task made no further source edit to it --
  note this repo's working tree carries many pre-existing, unrelated uncommitted changes from
  other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 -- `grep -c "initial_weights\|gui_weights" examples/minwebgl/morph_targets/src/gui_setup.rs`
  → ≥ 5 (doc comment references plus the parameter declaration plus the two call sites inside
  `setup`)

### Invariants

- [ ] I1 -- `cargo check -p morph_targets --target wasm32-unknown-unknown` → 0 errors

### Anti-faking checks

- [ ] AF1 -- `weight_settings_init` is genuinely called with `initial_weights` (the real weights
  buffer), not `gui_weights` (the NaN-sentinel buffer) -- checked by reading the call site itself
  (`weight_settings_init( &mut settings, initial_weights )`), not just the parameter list's
  presence

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass: checked In Scope ("verify present, no further edit expected") against Out of Scope ("no further code change") for contradiction — none found, consistent; also caught a real defect — this file cited BUG-462's bug-file path as `462_morph_targets_gui_setup_initial_weights_nan_display.md` in 3 places, but the actual file on disk is `462_morph_targets_gui_setup_wired_to_nan_sentinel_for_initial_display.md` (`find` confirmed). Fixed via repo-wide replace before this record was written. | Corrected all 3 stale-filename references. |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass: confirmed the `Testable:` line's command (`cargo check --target wasm32-unknown-unknown` → exit 0) is a real, executable, falsifiable check, not a vacuous placeholder; also verified the goal's downstream-of-BUG-330 claim against `gui_weights`'s NaN-sentinel role, not asserted blind. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass: Null Hypothesis considered ("what if this task is never filed?") — BUG-462 stays permanently stuck at 🎯 Verified, unable to self-accept (same-actor guard), matching the 26-precedent pattern this sweep is following; not speculative. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass: T01-T04 re-run live this session (grep for split params/fix comment/call site, wasm32 `cargo check`) — all genuinely re-executable, not aspirational. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass: scanned every path cited in Delivery Requirements/Test Matrix for any escape outside repo root — none found. | — |
| D6 | Crate Scope Unity | — | 🟢 | Adversarial pass: the only path outside `examples/minwebgl/morph_targets` is the BUG-462 file link-back, which is task-system bookkeeping (PROC12 Step 4), not a code/test deliverable — matches task 379/504/505's precedent reasoning. | — |
| D7 | Crate Locality | — | 🟢 | Adversarial pass: confirmed `examples/minwebgl/morph_targets` is itself the leaf crate owning `gui_setup.rs`/`main.rs` — no deeper leaf exists to push this down to. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial pass: crate responsibility ("demonstrates glTF morph-target animation with a weight-override GUI") stays one sentence, no "and" — the fix is a parameter split within the crate's existing GUI-setup module, introduces no second responsibility domain. | — |

**Live re-verification (this session, 2026-08-20 ~14:15):**
- `grep -n "initial_weights : &\[ f32 \]" src/gui_setup.rs` → line 248 (parameter declaration); `Fix(BUG-462)` comment at line 229 (T01/T02/C1/C2 confirmed)
- `grep -n "gui_setup::setup\|weight_settings_init\|setup(" src/main.rs` → line 163: `gui_setup::setup( gltf.animations.clone(), &current_animation, &weights.borrow(), &gui_weights )` — two distinct buffer arguments, not the same buffer for both roles (T04/C4 confirmed; AF1 confirmed — `weight_settings_init` inside `setup` is called with `initial_weights`, not `gui_weights`, per the source read at gui_setup.rs:265)
- `grep -c "initial_weights\|gui_weights" src/gui_setup.rs` → 9 (M1's `≥5` satisfied)
- `cargo check -p morph_targets --target wasm32-unknown-unknown` via `longrun` (background task `biayj2ufy`) → exit 0, `Finished` clean, elapsed 120s (T03/C3/I1 confirmed)
- BUG-462's header confirmed to NOT yet carry `**Fix Task:**` prior to this task's own follow-up edit (C6 pending, applied immediately after this record)
- `tsk .verify_pass 506` → exit 0 with `self-verification forbidden (actor matches filed_by)` message (same-actor guard, expected per project convention)

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | FILED | Filed via PROC12 (bug_promote) to register BUG-462's already-complete `gui_setup::setup` NaN-display fix. |
| 2026-08-20 | READINESS_GATE_PASS | Tier 2 Dual-Role Self-Check, 8/8 🟢 — see Verification Record above. |
| 2026-08-20 | EXECUTED | Fix, call site, and wasm32 compile check all live-reconfirmed; BUG-462 header linked back via `**Fix Task:**`. |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-20 14:05:23 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FILED | task created |
| 2026-08-20 14:14:55 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-20 14:14:57 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
| 2026-08-20 14:16:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | Round 8 readiness gate 8/8 PASS; live-reconfirmed T01-T04; `tsk .verify_pass` expected to hit same-actor guard per project convention |
