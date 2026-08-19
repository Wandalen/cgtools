# Register sprite_animation example's frame-index modulus fix (closes BUG-313)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 23:49:12
- **expires_at:** 2026-08-19 01:49:12
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-313
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/sprite_animation
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-18 23:49:12
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-18 23:47:42
- **unverified_by:** system

## Goal

BUG-313 (`task/bug/verified/313_sprite_animation_modulus_skips_last_frame.md`, Medium
severity, 🎯 Verified) found `examples/minwebgl/sprite_animation/src/main.rs`'s per-frame
closure computing a single `amount` value (`sprite_sheet.amount as f32 - 1.0`, `63.0` for the
64-frame `rock.png` sheet) and using it for two unrelated purposes: the pacing divisor
(`step / amount`) and the wraparound modulus (`frame % amount`). Since `x % 63.0` is
mathematically confined to `[0, 63.0)`, frame index `63` (the sheet's 64th and last frame)
could never be produced, permanently skipping it every animation cycle. The fix -- splitting
`amount` into `hold_ticks` (unchanged pacing divisor, `main.rs:38`) and `sprite_count` (the
true wraparound range, `main.rs:39`), extracting the computation into a `sprite_frame_index()`
function (`main.rs:69-73`) whose modulus argument is `sprite_count`, with a
`Fix(BUG-313)`/`Root cause`/`Pitfall` 3-field source comment (`main.rs:43-48`) -- is already
applied and independently confirmed via an inline `#[cfg(test)] mod tests` reproducer
(`tests::test_sprite_frame_index_reaches_last_frame`, `main.rs:75-136`, closed-form arithmetic
picking the exact `step` where the pre-fix expression would wrap early) -- the bug file's own
VERIFY Gate, 8/8 PASS, 2026-08-18. This task performs the remaining lifecycle bookkeeping --
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) -- to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-313.
Testable: `cd examples/minwebgl/sprite_animation && cargo test --bins 2>&1 | grep -q
'test_sprite_frame_index_reaches_last_frame ... ok' && echo PASS || echo FAIL` → PASS.

## In Scope

- `examples/minwebgl/sprite_animation/src/main.rs` -- the already-applied `hold_ticks`
  (line 38) / `sprite_count` (line 39) split, extracted `sprite_frame_index()` function
  (lines 69-73), and its `Fix(BUG-313)`/`Root cause`/`Pitfall` source comment (lines 43-48)
  -- verify all are present; no further edit expected.
- The already-applied inline `#[cfg(test)] mod tests` reproducer
  (`tests::test_sprite_frame_index_reaches_last_frame`, lines 75-136) -- verify present and
  passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/313_sprite_animation_modulus_skips_last_frame.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `examples/minwebgl/sprite_animation` -- the fix is complete
  and independently verified by the bug's own VERIFY Gate.
- Re-running BUG-313's own VERIFY Gate -- already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own
  Readiness Verification Gate, which checks task-file quality, not the underlying fix.
- Any other sprite-sheet/frame-count call site using the same `count - 1`-as-modulus
  pattern -- BUG-313's own D6 gate confirmed via `grep -rn "amount.*- 1\.0\|\.amount as f32
  - 1" examples/` (excluding `sprite_animation`) that no other site exists; re-confirmed
  empty again during this task's own filing.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: the inline reproducer's `buggy` assertion
    (`sprite_frame_index( step, hold_ticks, hold_ticks )`, replicating the pre-fix
    single-`amount` expression) evaluates to `0` at the exact `step` where the fixed call
    reaches `63` -- both assertions run in the same test, empirically proving the pre-fix
    wraparound defect (bug file MRE section, 2026-08-18) -- this task does not re-derive
    that evidence
-   Fix already applied: `examples/minwebgl/sprite_animation/src/main.rs`'s
    `sprite_frame_index` (lines 69-73) computes `frame % sprite_count`, not
    `frame % hold_ticks`; the call site (lines 43-48) carries the 3-field
    `Fix(BUG-313)`/`Root cause`/`Pitfall` source comment
-   Green state already confirmed, and re-confirmed live during this task's filing:
    `cargo test --bins` in the crate → `test tests::test_sprite_frame_index_reaches_last_frame
    ... ok` (1 passed, 0 failed); `cargo check -p minwebgl_sprite_animation` → 0 errors
-   No refactor needed -- the fix is a single extracted-function change, no structural churn
-   Fix documentation already complete at the bug level: BUG-313 carries the full Root
    Cause/Why Not Caught/Fix Location/Prevention narrative in its own body -- this task does
    not duplicate it, only cross-links via `closes: BUG-313`
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate;
    `tsk .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to
    hit this sandbox's known same-actor guard, per project convention -- document rather
    than force/spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd examples/minwebgl/sprite_animation && cargo test --bins` | `tests::test_sprite_frame_index_reaches_last_frame` (bug_reproducer) | exit 0, `1 passed; 0 failed` |
| T02 | `grep -rn 'amount.*- 1\.0\|\.amount as f32 - 1' examples/` (excluding `sprite_animation`) | Whole-workspace scan for the same off-by-one modulus pattern | empty (no other site) |
| T03 | `cargo check -p minwebgl_sprite_animation` | crate compiles | 0 errors |
| T04 | Inline test's closed-form arithmetic (`step = 63.0 * hold_ticks`) | fixed vs. buggy `sprite_frame_index` calls at the same `step` | fixed → `63`; buggy (pre-fix replica) → `0` |

## Acceptance Criteria

-   `examples/minwebgl/sprite_animation/src/main.rs`'s `sprite_frame_index` function
    computes `frame % sprite_count`, not `frame % hold_ticks`
-   The fix's source comment (immediately above the call site) carries all 3 required
    fields: `Fix(BUG-313)`, `Root cause`, `Pitfall`
-   `tests::test_sprite_frame_index_reaches_last_frame` exists, is tagged
    `bug_reproducer(BUG-313)`, and passes
-   No other file under `examples/` reproduces the same `count - 1`-as-modulus pattern
-   `task/bug/verified/313_sprite_animation_modulus_skips_last_frame.md`'s header states
    `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify -- an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `examples/minwebgl/sprite_animation/src/main.rs`'s `sprite_frame_index`
  function compute `frame % sprite_count` (not `% hold_ticks`)?
- [ ] C2 — Does the call site's source comment carry `Fix(BUG-313)`, `Root cause`, and
  `Pitfall` fields?
- [ ] C3 — Does `cargo test --bins` (run from the crate directory) pass
  `tests::test_sprite_frame_index_reaches_last_frame`?
- [ ] C4 — Does `cargo check -p minwebgl_sprite_animation` succeed with 0 errors?
- [ ] C5 — Does a repo-wide grep for the same `count - 1`-as-modulus copy-paste pattern
  return empty outside this already-fixed site?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-313`?
- [ ] C7 — Does BUG-313's own header carry a `**Fix Task:**` line pointing back at this
  task's ID?

**Out of Scope confirmation**
- [ ] C8 — Was `examples/minwebgl/sprite_animation/src/main.rs` left unedited by this task
  itself (the fix content matches what BUG-313's own already-completed fix applied; this
  task made no further source edit to it -- note this repo's working tree carries many
  pre-existing, unrelated uncommitted changes from other concurrent activity, so a blanket
  repo-wide `git diff --stat` is not a meaningful signal here; the checkable fact is that no
  Edit/Write tool call in this task's own execution targeted `main.rs`)?

### Measurements

- [ ] M1 — `sed -n '69,73p' examples/minwebgl/sprite_animation/src/main.rs | grep -c 'frame % hold_ticks'` → 0 (unscoped `grep -c 'frame % hold_ticks'` over the whole file returns 1 -- a false positive matching the explanatory comment at line 130, not code; scoping to the `sprite_frame_index` function body itself is required for this measurement to be meaningful)
- [ ] M2 — `sed -n '69,73p' examples/minwebgl/sprite_animation/src/main.rs | grep -c 'frame % sprite_count'` → 1

### Invariants

- [ ] I1 — `cargo test --bins` in the crate → 0 failures
- [ ] I2 — `cargo check -p minwebgl_sprite_animation` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test's `buggy` assertion (`main.rs:132-134`) actually exercises
  the pre-fix expression (passes `hold_ticks` as the modulus, not a hardcoded
  expected-failure literal) -- checked by reading the test body itself, not just its
  pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | Adversarial pass re-ran the Goal's Testable line and both Measurements live: M1's original unscoped `grep -c 'frame % hold_ticks' main.rs` returned 1, not the claimed 0 — a false-positive match against an explanatory comment at line 130 (`frame % hold_ticks == 63.0 % 63.0 == 0.0`), not code; separately, Checklist item C8's blanket repo-wide `git diff --stat` claim was unverifiable as worded, since this repo's working tree carries many pre-existing, unrelated uncommitted files from other concurrent activity | Rescoped M1/M2 to the `sprite_frame_index` function body only (`sed -n '69,73p' main.rs \| grep -c ...`), re-verified live (0 and 1 respectively); reworded C8 to a directly checkable claim (no Edit/Write tool call touched `main.rs` during this task's own execution) rather than a blanket diff |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`examples/minwebgl/sprite_animation`); the BUG-313 link-back edit touches `task/bug/verified/313_sprite_animation_modulus_skips_last_frame.md`, a tracking file outside `unit_type: module`'s crate boundary — same disposition as every other bug-promotion cross-link in this repo (tracking-file edits are not crate-scope violations) | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 2 fixed | 2/2 |

**Reproduced live during this gate:** `cd examples/minwebgl/sprite_animation && cargo test --bins`
→ `test tests::test_sprite_frame_index_reaches_last_frame ... ok` (1 passed, 0 failed, via
`longrun`, exit 0, ~15s cold / ~0s warm); `cargo check -p minwebgl_sprite_animation` → exit 0,
0 errors (via `longrun`, ~63s cold). `grep -rn 'amount.*- 1\.0\|\.amount as f32 - 1' examples/`
excluding `sprite_animation` → empty (no sibling instance). Note: an unrelated, transient
workspace-manifest parse error (`examples/gpu_hal/triangle_vulkan_window/Cargo.toml`,
apparently a concurrent actor's in-flight edit — file is untracked/uncommitted and its
content changed between two consecutive reads a few seconds apart) caused the first
`cargo test` attempt to fail with an unrelated `default-features` inheritance error; the
retry succeeded cleanly once that concurrent edit had moved past its transient bad state.
Not a defect in this task's own deliverable -- `examples/gpu_hal/triangle_vulkan_window` is
untouched by, and unrelated to, this task's scope.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 16:00:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 16:03 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 358 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via `bug_promote` skill (PROC12) to formally
  register BUG-313's already-applied, already-verified fix
  (`examples/minwebgl/sprite_animation/src/main.rs` frame-index modulus `sprite_count - 1`
  → `sprite_count`) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass caught and fixed two Verification-section defects: M1's unscoped grep
  false-positived against an explanatory comment (rescoped to the `sprite_frame_index`
  function body) and Checklist C8's blanket repo-wide `git diff --stat` claim (reworded to
  a directly checkable no-edit-to-`main.rs` claim, since this repo's ambient working tree
  carries many unrelated pre-existing uncommitted files). Re-verified T01-T03 live post-fix
  (`cargo test --bins` via `longrun`, exit 0, 1 passed; `cargo check -p
  minwebgl_sprite_animation` via `longrun`, exit 0; sibling-pattern grep empty). `tsk
  .claim_verify 358` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix
  (`examples/minwebgl/sprite_animation/src/main.rs` `hold_ticks`/`sprite_count` split,
  extracted `sprite_frame_index()`, `Fix(BUG-313)`/`Root cause`/`Pitfall` comment, inline
  `bug_reproducer(BUG-313)` test) already existed on disk prior to this task's filing,
  applied during BUG-313's own investigation (bug file History, 2026-08-18). This task's own
  contribution is the formal tracking registration and lifecycle walk, not the code change
  itself. `tsk .verify_pass 358` blocked by the same-actor guard (documented above) — task
  left at 🔬 Verifying per this sandbox's standing, previously-documented limitation (same
  guard that blocked task 254's own `.verify_pass`), not a quality defect in this task's own
  content.

## Related Documentation

- `task/bug/verified/313_sprite_animation_modulus_skips_last_frame.md` -- the source bug
  this task promotes; carries the full Root Cause/MRE/Prevention/History detail this task
  does not duplicate
- `examples/minwebgl/sprite_animation/src/main.rs` -- the fixed frame-index computation
  (`sprite_frame_index()`) and its inline `bug_reproducer(BUG-313)` test
