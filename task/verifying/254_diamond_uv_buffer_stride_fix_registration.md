# Register diamond example's uv-buffer stride fix (closes BUG-114)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-17 13:23:30
- **expires_at:** 2026-08-17 15:23:30
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-114
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/diamond
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-17
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-17 13:23:30
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

BUG-114 (`task/bug/verified/114_diamond_uv_buffer_stride_mismatch.md`, High severity,
🎯 Verified) found `examples/minwebgl/diamond/src/main.rs:131` passing `.stride(3)` to
the uv attribute's `BufferDescriptor` — copy-pasted from the preceding `[f32;3]`
position/normal attribute lines without adjusting for uv's own 2-component `[f32;2]`
layout. Since `attribute_pointer` (`buffer.rs:200`) computes byte stride as
`self.stride * sz`, the wrong multiplier made every uv read after vertex 0 walk into
the next vertex's record, raising `GL_INVALID_OPERATION` at `drawElements` time — the
bug's own Root Cause section confirms the shared helper's stride math is correct
(H2 disproved) and the defect is entirely this caller's argument choice. The fix
(`.stride(3)` → `.stride(2)`, with a `Fix(BUG-114)`/`Root cause`/`Pitfall` 3-field
source comment) is already applied and independently confirmed via a Firefox
software-WebGL2 MRE reproducing the exact predicted symptom (bug file's VERIFY Gate,
8/8 PASS, 2026-08-17). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to
formally register that already-complete, already-verified fix as a tracked task,
closing BUG-114.
Testable: `grep -q '\[ f32; 2 \] >().stride( 2 )' examples/minwebgl/diamond/src/main.rs
&& echo PASS || echo FAIL` → PASS.

## In Scope

- `examples/minwebgl/diamond/src/main.rs` line 131 — the already-applied
  `.stride(3)` → `.stride(2)` fix and its `Fix(BUG-114)`/`Root cause`/`Pitfall`
  source comment (verify both are present; no further edit expected).
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/114_diamond_uv_buffer_stride_mismatch.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this
  file is filed).

## Out of Scope

- Any further code change to `examples/minwebgl/diamond` or
  `module/min/minwebgl/src/buffer.rs` — the fix is complete; `buffer.rs`'s
  `attribute_pointer` was confirmed correct by BUG-114's own investigation
  (Root Cause H2 disproved via E3-E5), not touched by the fix.
- Re-running BUG-114's MRE or its own VERIFY Gate — already run and recorded in
  the bug file's History (2026-08-17 Firefox round, 8/8 PASS); not re-litigated
  by this task's own Readiness Verification Gate, which checks task-file quality,
  not the underlying fix.
- Any other `BufferDescriptor::new::<[f32;N]>().stride(M)` call site — BUG-114's
  own Prevention section names the repo-wide detection command
  (`grep -rn 'BufferDescriptor::new::< \[ f32; 2 \] >().stride( 3 )' examples/
  module/`) and confirms it returns empty post-fix; no other site was found.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-114's MRE (byte-identical copy
    re-run at `~/mre114/repro.html`) reproduced `GL_INVALID_OPERATION` against the
    pre-fix `.stride(3)` value via Firefox's software WebGL2 fallback (bug file
    History, 2026-08-17) — this task does not re-derive that evidence
-   Fix already applied: `examples/minwebgl/diamond/src/main.rs:131` states
    `.stride(2)`, with the 3-field `Fix(BUG-114)`/`Root cause`/`Pitfall` source
    comment in place
-   Green state already confirmed: MRE's `fixed_stride` case returns `NO_ERROR`;
    `cargo check`/`clippy` clean per bug file History
-   No refactor needed — single scalar-literal change, no structural churn
-   Fix documentation already complete at the bug level: BUG-114 carries the
    5-section fix documentation (Root Cause, Why Not Caught, Fix Applied,
    Prevention, Pitfall) in `## Verification Findings`/body — this task does not
    duplicate it, only cross-links via `closes: BUG-114`
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate;
    `tsk .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle
    (expected to hit this sandbox's known same-actor guard, per project
    convention — document rather than force/spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -q '\[ f32; 2 \] >().stride( 2 )' examples/minwebgl/diamond/src/main.rs` | Fixed uv `BufferDescriptor` stride | exit 0 (match found) |
| T02 | `grep -rn 'BufferDescriptor::new::< \[ f32; 2 \] >().stride( 3 )' examples/ module/` (BUG-114's own repeat-defect detector) | Whole-workspace scan for the same copy-paste pattern | empty (no other site) |
| T03 | `cargo check -p minwebgl_diamond` | `diamond` crate compiles | 0 errors |
| T04 | Firefox MRE `fixed_stride` case (already run, bug file History 2026-08-17) | Corrected `byteStride=8` | `err_after_drawElements=0` (`NO_ERROR`) |

## Acceptance Criteria

-   `examples/minwebgl/diamond/src/main.rs:131` states `.stride(2)`, not `.stride(3)`
-   The line 131 source comment carries all 3 required fields: `Fix(BUG-114)`,
    `Root cause`, `Pitfall`
-   No other file under `module/` or `examples/` reproduces the same
    copy-pasted 3-stride-on-2-component defect
-   `task/bug/verified/114_diamond_uv_buffer_stride_mismatch.md`'s header states
    `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `examples/minwebgl/diamond/src/main.rs:131` state `.stride(2)`?
- [ ] C2 — Does the same line's source comment carry `Fix(BUG-114)`, `Root cause`, and `Pitfall` fields?
- [ ] C3 — Does `cargo check -p minwebgl_diamond` succeed with 0 errors?
- [ ] C4 — Does a repo-wide grep for the same copy-paste pattern (`stride( 3 )` on a `[f32;2]` `BufferDescriptor`) return empty outside this already-fixed site?

**Registration correctness**
- [ ] C5 — Does this task's `closes:` field name `BUG-114`?
- [ ] C6 — Does BUG-114's own header carry a `**Fix Task:**` line pointing back at this task's ID?

**Out of Scope confirmation**
- [ ] C7 — Is `module/min/minwebgl/src/buffer.rs` untouched by this task (`git diff --stat` empty for that path)?

### Measurements

- [ ] M1 — `grep -c '\[ f32; 2 \] >().stride( 3 )' examples/minwebgl/diamond/src/main.rs` → 0
- [ ] M2 — `grep -c '\[ f32; 2 \] >().stride( 2 )' examples/minwebgl/diamond/src/main.rs` → ≥1

### Invariants

- [ ] I1 — `module/min/minwebgl/src/buffer.rs` unaffected: `git diff --stat -- module/min/minwebgl/src/buffer.rs` → empty
- [ ] I2 — workspace still builds: `cargo check --workspace` → 0 errors (single-scalar change, unaffected)

### Anti-faking checks

- [ ] AF1 — the fix changes only the stride argument (`3`→`2`), not the element type or attribute count — checked by reading the literal diff at line 131, not just the absence of the old value

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | 🔴 | 🟢 | Adversarial pass caught every citation of `attribute_pointer`'s home file using a nonexistent path (`module/helper/renderer/src/buffer.rs`) — confirmed via `ls` that only `module/min/minwebgl/src/buffer.rs` exists; fixed across all 4 occurrences (Out of Scope, Acceptance Criteria C7, Invariants I1, Related Documentation), then re-verified I1/M1/M2/T01-T03 fresh against the corrected path (T03 `cargo check -p minwebgl_diamond` run live, exit 0, 17s) rather than trusting the earlier draft | Corrected path in 4 locations; re-ran verification commands live |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`examples/minwebgl/diamond`); the BUG-114 link-back edit touches `task/bug/verified/114_...md`, a tracking file outside `unit_type: module`'s crate boundary — same disposition as every other bug-promotion cross-link in this repo (tracking-file edits are not crate-scope violations) | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 fixed | 1/1 |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-17 13:23:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-17 13:24 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 243` → blocked: "self-verification forbidden (actor matches filed_by)" — same-actor guard, not a defect; state remains 🔬 Verifying |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-17]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally register BUG-114's already-applied, already-verified fix (`examples/minwebgl/diamond/src/main.rs:131` `.stride(3)`→`.stride(2)`) as a tracked task, closing the bug.
- **[2026-08-17]** `READINESS_GATE_PASS` — Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS. Adversarial pass caught and fixed a wrong `buffer.rs` path (`module/helper/renderer/...` → correct `module/min/minwebgl/...`) across 4 citations; re-verified T01-T03 live post-fix (`grep` M1/M2, `cargo check -p minwebgl_diamond` exit 0 in 17s via `longrun`, `git diff --stat -- module/min/minwebgl/src/buffer.rs` empty). State → 🎯 Verified.
- **[2026-08-17]** `EXECUTED` — No new edit performed: the described fix (`examples/minwebgl/diamond/src/main.rs:131` `.stride(3)`→`.stride(2)`, `Fix(BUG-114)`/`Root cause`/`Pitfall` comment) already existed on disk prior to this task's filing, applied during the original bug investigation (BUG-114's own History, 2026-08-16). This task's own contribution is the formal tracking registration and lifecycle walk, not the code change itself. `tsk .claim_verify` succeeded; `tsk .verify_pass` blocked by the same-actor guard (documented above) — task left at 🔬 Verifying per standing sandbox limitation, not a quality defect.
- **[2026-08-17]** `RENUMBERED` — 243 → 252, resolving a bug/task ID collision with `BUG-243` (`task/bug/completed/243_wide_outline_jfa_final_buffer_selection_inverted.md`), both filed independently under the shared tsk ID namespace. File, Tasks Index row, `health.md`, and `task/bug/verified/114_diamond_uv_buffer_stride_mismatch.md`'s Fix Task link all updated to 252. The `tsk .verify_pass 243` command transcript above is left verbatim as accurate historical fact (the task really was numbered 243 when that command ran).
- **[2026-08-17]** `RENUMBERED` — 252 → 254, a second hop within minutes of the first: a concurrent session actor (same sandbox identity, independent activity) filed `BUG-252` (`task/bug/completed/252_displacement_texture_size_zero_width_division_by_zero.md`) in the same race window this task's own 243→252 rename landed in — both sides independently computed "next free ID" from an on-disk scan and picked 252 within ~2 minutes of each other (confirmed via file mtimes: their `BUG-252` predates this file's own 243→252 rename by ~118s). Their bug/readme.md entry documents them dodging this session's already-visible 246-250 range but not this file's own just-landed 252, since it wasn't yet on disk at their scan time — a genuine TOCTOU race between two independent actors, not a defect in either side's renumbering logic. Since their `BUG-252` was already `task/bug/completed/` (terminal) and cross-referenced in `bug/readme.md` by the time this was discovered, this file moved again rather than displacing theirs — 254 confirmed free (254 also not claimed by their own `BUG-253`, `task/bug/completed/253_camera_projection_matrix_set_bypasses_bug174_validation.md`, filed immediately after their 252). File, Tasks Index row, `health.md`, and `task/bug/verified/114_diamond_uv_buffer_stride_mismatch.md`'s Fix Task link all updated to 254; `highest_id` bumped to 254.

## Related Documentation

- `task/bug/verified/114_diamond_uv_buffer_stride_mismatch.md` — the source bug this task promotes; carries the full Root Cause/MRE/Prevention/History detail this task does not duplicate
- `module/min/minwebgl/src/buffer.rs` — `attribute_pointer`'s stride computation (confirmed correct, not modified by this task)
